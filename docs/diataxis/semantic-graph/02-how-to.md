# How To: Ground Workflow Receipts in Git Objects with gix

> Diataxis axis: **How-To** (task-oriented). Every function named here is
> transcribed from the real file
> /Users/sac/affidavit/src/git_merkle.rs — no invented APIs.

## Problem

You have a validated conformance receipt (or a 352-byte CTRT receipt) and you
want it content-addressed in real Git objects — blobs, trees, commits, notes
— without shelling out to the `git` binary. The substrate is `gix`
(pure-Rust Git): `gix-odb`, `gix-object`, `gix-hash`, `gix-actor`.

## The module

`/Users/sac/affidavit/src/git_merkle.rs` provides three things:

1. `compute_census_binding(git_tree_sha: &[u8], wasm_bytecode_hash: &[u8]) -> [u8; 32]`
   — `Digest = BLAKE3(GitTreeSHA || WasmBytecodeHash)`, deterministic,
   32 bytes.
2. `GitTreeBuilder` — in-process blob/tree/commit writes and tree traversal
   through a `gix_odb::loose::Store`.
3. `GitDossierNotes` — a receipts store in Git notes under the canonical ref
   `refs/notes/strata-dossier` (constant `STRATA_DOSSIER_REF`).

Zero shell commands: no `/bin/sh`, no `System::process`. Everything is loose
object reads/writes in `.git/objects`.

## Error model: machine codes 0xE401..=0xE409

`GitMerkleError` carries a `const fn machine_code(&self) -> u16` for
structured classification:

| code | variant | meaning |
|---|---|---|
| `0xE401` | `ObjectNotFound(String)` | object missing |
| `0xE402` | `TreeCorruptOrUnparseable(String)` | tree bytes unparseable |
| `0xE403` | `Io(std::io::Error)` | filesystem I/O |
| `0xE404` | `Odb(String)` | gix ODB write/read failure |
| `0xE405` | `Decode(String)` | object decode failure |
| `0xE406` | `Receipt(TrajectoryFoldError)` | receipt format/fold error |
| `0xE407` | `InvalidCommit(String)` | malformed commit |
| `0xE408` | `RefNotFound(String)` | ref missing |
| `0xE409` | `EntryNotFound(String)` | tree entry missing |

`0xE401` and `0xE402` are asserted directly in the module's test
`test_git_merkle_error_machine_codes`. Note honestly what the module does
today: `read_object` on a missing OID returns `GitMerkleError::Odb(...)`
(0xE404), not `ObjectNotFound` — the 0xE401 variant exists and is classified,
but current call sites mostly surface ODB-level errors. Code against the
`machine_code()` classification, not against which variant a given call site
happens to raise.

## Task 1: Build a tree from a directory

```rust
use affidavit::git_merkle::GitTreeBuilder;

let builder = GitTreeBuilder::new(".git/objects"); // loose store, Sha1
let root_tree = builder.write_dir_recursive(std::path::Path::new("workdir"))?;
```

What happens: entries are sorted by file name, `.git` is skipped, files
become blobs (`EntryKind::Blob`), subdirectories recurse into subtrees
(`EntryKind::Tree`), and the resulting tree is serialized and written.
Non-UTF-8 file names refuse with `GitMerkleError::Odb`. You can also write
objects individually:

- `write_blob(&self, data: &[u8]) -> Result<ObjectId, GitMerkleError>`
- `write_tree(&self, entries: Vec<gix_object::tree::Entry>) -> Result<ObjectId, GitMerkleError>`
  (entries are sorted for you; git tree entry ordering is handled by the
  `Ord` impl on `gix_object::tree::Entry`)
- `read_object(&self, oid: &ObjectId) -> Result<Vec<u8>, GitMerkleError>`
- `traverse_tree(&self, tree_oid: &ObjectId) -> Result<Vec<GitTreeNode>, GitMerkleError>`
  — returns flat `GitTreeNode { mode, path, oid }` records with slash-joined
  paths, depth-first.

## Task 2: Create a commit without the git CLI

```rust
use gix_actor::Signature;
use gix_object::date::Time;

let sig = Signature {
    name: "Strata Bot".into(),
    email: "bot@strata.org".into(),
    time: Time::new(1_700_000_000, 0),
};

let commit_oid = builder.create_signed_commit(
    tree_oid,
    vec![parent_oid],          // empty vec = root commit
    sig.clone(),
    sig,
    "attach conformance receipt for run 42",
    vec![("strata-dossier-receipt", "352B-CTRT")], // extra headers
)?;
```

Signature is `gix_actor::Signature { name, email, time }` with
`BString` name/email and `gix_object::date::Time`. Note the name: despite
"signed" in the name, what `create_signed_commit` does today is write a
Commit object with arbitrary extra headers — there is no cryptographic
signature applied in this function; header-based provenance only. If you need
real signing, that happens outside this module (e.g. ML-DSA in the wider
pipeline).

## Task 3: Attach and read a receipt in Git notes

The notes model: a note on target commit `C` is a blob named `<hex C>` in the
notes tree, reachable from the commit that `refs/notes/strata-dossier` points
at. `attach_receipt` handles the whole chain — write receipt blob, merge with
the existing notes tree, create a notes commit whose parent is the previous
notes commit, bump the ref file.

```rust
use affidavit::git_merkle::{GitDossierNotes, STRATA_DOSSIER_REF};

let notes = GitDossierNotes::new(repo_root); // repo root, not .git
let notes_commit = notes.attach_receipt(
    &target_commit_oid,
    &ctrt_receipt,             // 352-byte CTRT receipt
    committer_signature,
)?;
let recovered = notes.read_receipt(&target_commit_oid)?; // Option<CtrtReceipt>
```

Contract details that will bite you if ignored:

- The receipt must encode to exactly `COURT_RECORD_LEN` (352) bytes;
  anything else refuses via `TrajectoryFoldError::InvalidLength` (0xE406).
  `attach_receipt` does not hash-chain for you: the caller is responsible for
  filling `chain_hash` (in the module's own test, `chain_hash` is copied from
  `encoded[320..352]` before attach).
- `get_notes_ref()` reads `.git/refs/notes/strata-dossier` as a plain file
  and returns `Ok(None)` when absent — a fresh repo has no notes ref.
- `update_notes_ref` writes `<hex>\n` with `fs::write`; the notes ref is a
  flat loose ref file, not packed. Refs are read/written via the filesystem,
  not via gix's refdb — fine for this substrate, a divergence from full gix
  ref handling worth knowing.
- Notes commits carry the extra header `strata-dossier-receipt: 352B-CTRT`,
  so a notes commit is identifiable by header without decoding the tree.

## Tying it to the conformance flow

The census binding is the join key between the workflow world and the Git
world: take the tree SHA of your case working directory
(`write_dir_recursive` on the case dir gives you the tree; its OID is the
`git_tree_sha` bytes), take the WASM bytecode hash of the conformance
engine guest, and `compute_census_binding` yields the 32-byte digest that the
receipt and the dossier note both carry. Replay of a receipt then means:
recompute the binding from the same tree + bytecode, compare, and read the
note for the full 352-byte CTRT record.

## Boundaries (what this module does NOT do)

- No packfile access — `gix_odb::loose::Store` only; packed objects are
  invisible to `read_object`.
- No worktree/index operations; it does not check anything out.
- No packed-refs reading; `get_notes_ref` reads only the loose ref file.
- No signature verification.

## Source of truth

/Users/sac/affidavit/src/git_merkle.rs (all signatures above are from this
file, lines as cited in the code: `compute_census_binding` at line 63,
`GitTreeBuilder` at 79, `create_signed_commit` at 211, `GitDossierNotes` at
248, `attach_receipt` at 291, `read_receipt` at 370).
