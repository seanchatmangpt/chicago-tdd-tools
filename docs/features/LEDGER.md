# LEDGER

Evidence lifecycle for court receipts: raw adjudication output becomes a fixed-layout
wire record, records chain into per-court BLAKE3 chains, court heads seal into a Merkle
dossier root, and the root seals under an ML-DSA-65 (FIPS 204) detached signature.

Lifecycle: **Raw → Admitted → Receipted → Sealed dossier.**

- A court run produces a `CourtReceiptV1` (typed, in-memory).
- `ChainedCourtReceipt` encodes it to the fixed 320-byte record and appends a 32-byte
  BLAKE3 chain hash (352-byte chained record).
- `VerifiedCourtReceipt::from_bytes_verified` is the only way to obtain a verified
  receipt; it enforces every structural, range, name, flag, and chain-hash law at
  decode time.
- `CourtLedgerBuilder` maintains one court's chain; `DossierBuilder` seals the Merkle
  root over all court heads and mints per-court inclusion proofs (`CourtProof`).
- `signing::sealer::seal` binds the root to a domain-separated subject and signs it
  with an ML-DSA-65 `Signer`; `verify_sealed` is fail-closed.

All code cited below is in `src/ledger/` (`receipt.rs`, `dossier.rs`, `proof.rs`,
`error.rs`, `mod.rs`, `dossier_wire.rs`,
`signing/{mod,signer,subject,sealer,keys}.rs`). The `ctdd_ledger` CLI binary
(`src/bin/ctdd_ledger.rs`) and the `ledger-cli` feature were excised in the V2
cleanup; the CLI verbs landed in the external `affi` binary (Phase D M2, `~/affidavit`: receipt validate/chain-verify, dossier seal/verify behind the `ctdd` feature).

## Receipt wire format

Feature: `ledger`. Module: `src/ledger/receipt.rs`. Little-endian, serde-free.

Constants: `MAGIC = b"CTRT"`, `VERSION = 1`, `COURT_RECEIPT_V1_LEN = 320`,
`COURT_RECORD_LEN = 352` (receipt + 32-byte chain hash).

### 352-byte chained record layout

Header (record-relative):

| offset | size | field | constraint |
|---|---|---|---|
| 0..4 | 4 | magic | `b"CTRT"` (constant-time compare) |
| 4..6 | 2 | version (u16le) | must be 1 |
| 6 | 1 | flags | bit0 = positive_control_present, bit1 = fail_closed; unknown bits rejected |
| 7 | 1 | reserved | must be 0 |
| 8..16 | 8 | body_len (u64le) | must be 304 |
| 16..304 | 288 | body prefix | see below |
| 304..352 | 32 | chain_hash | `BLAKE3(parent_receipt_hash ‖ record[0..304])`, constant-time compare |

Body (body-relative offsets, body starts at record offset 16):

| offset | size | field | constraint |
|---|---|---|---|
| 0..8 | 8 | reserved | must be 0 |
| 8..40 | 32 | subject_hash | |
| 40..72 | 32 | parent_receipt_hash | zeros = genesis |
| 72..80 | 8 | timestamp_ms (u64le) | |
| 80..88 | 8 | run_id (u64le) | |
| 88..96 | 8 | total_ticks (u64le) | |
| 96..104 | 8 | wall_ms (u64le) | |
| 104 | 1 | thermal tag | 0 = Hot, 1 = Warm, 2 = Cold |
| 105 | 1 | outcome tag | 0 = Pass, 1 = Fail, 2 = Skip, 3 = Error, 4 = Refused |
| 106 | 1 | budget_met | 0 or 1 |
| 107 | 1 | padding | must be 0 |
| 108..140 | 32 | positive_control | all-zero ⇔ flag bit0 clear |
| 140..172 | 32 | metadata_digest | |
| 172..204 | 32 | prev_self_hash | chain hash of the previous receipt about the same subject |
| 204..304 | 100 | name | NUL-padded UTF-8, no interior NUL |

### NAME_MAX resolution

The 100-byte `name` field ends with a 16-byte zero-only trailer
(record offsets 288..304, `TRAILER_LEN = 16`), capping usable name bytes at
`NAME_MAX = 100 - 16 = 84`. The trailer must be all zero on decode
(`ZeroTrailerViolation` otherwise). The flag byte is never stored in the struct:
bit0 is derived from the control bytes and bit1 from `fail_closed`, so an
`encode()` output always satisfies the flag/control consistency check
(`positive_control != [0;32]` ⇔ flag bit0 set) enforced at decode.

### Verified decode

`VerifiedCourtReceipt` has exactly one constructor:
`from_bytes_verified(&[u8]) -> Result<Self, LedgerError>`. It checks, in order:
length (352 exactly), magic, version, flag-bit mask, reserved bytes (header byte 7,
body prefix, body padding, zero trailer), `body_len = 304`, enum ranges
(thermal/outcome/budget), name (≤ 84 bytes, valid UTF-8, no interior NUL),
flag/control consistency, then recomputes `chain_hash_for(parent, record[0..304])`
and constant-time compares it to the stored hash. A 352-case single-byte-flip
tamper matrix test plus a bit-flip matrix over the first and last 32 bytes pins
full-record tamper evidence.

`as_bytes()` returns the verified 352-byte record; `receipt()` returns the typed
`CourtReceiptV1`. `ChainedCourtReceipt` implements
`crate::observability::receipt::Blake3ReceiptEntry` (`prev_hash` /
`content_bytes` / `stored_hash`), so `Blake3ChainValidator` applies unchanged.

## Chain and Merkle dossier

Feature: `ledger`. Modules: `src/ledger/dossier.rs`, `src/ledger/proof.rs`.

### Per-court chain

- `CourtLedgerBuilder::new(court_id)` starts with zeroed genesis head `[0u8; 32]`.
- `append(receipt)` advances
  **`head(n+1) = BLAKE3(head(n) ‖ receipt.stored_hash())`**.
- A receipt whose validated `name` (court id) differs from the builder's court is
  refused (`LedgerError::CrossCourtAppend`); the head does not advance on refusal.
- `build()` refuses an empty chain (`LedgerError::EmptyChain`).
- `court_id_of(receipt)` decodes the court id from the name field at absolute
  record offset 220 (16-byte header + body offset 204).

### Merkle dossier root

`DossierBuilder::add_court(court_id, head)` collects leaves; duplicate court ids
(`DossierError::DuplicateCourt`) and empty dossiers (`DossierError::EmptyDossier`)
are refused. `seal()` / `seal_with_proofs()`:

- Leaves are sorted byte-lexicographically by `court_id` — insertion order never
  affects the root.
- **Leaf law**: `leaf = BLAKE3(0x00 ‖ "P13-LEAF" ‖ 0x00 ‖ u32le(len(court_id)) ‖ court_id ‖ head)`
- **Node law**: `node = BLAKE3(0x00 ‖ "P13-NODE" ‖ 0x00 ‖ left ‖ right)`
- **Empty-root law**: `empty_root() = BLAKE3("P13-EMPTY")`; no proof verifies
  against it.
- Odd nodes are **PROMOTED** (carried to the next level, appended last), never
  duplicated — pinned against the duplicated-variant reference at 3 and 5 leaves.
- `seal_with_proofs` mints one `CourtProof` per court; at N leaves the deepest path
  is `ceil(lg2(N))` steps (5 at 20 leaves, pinned by test). `CourtProof::verify`
  replays leaf→root with side flags and refuses paths over `MAX_PROOF_STEPS = 64`
  (`DossierError::MalformedProof`); a root mismatch is
  `DossierError::VerificationFailed(court_id)`.

`ledger::chain_verify` (mod.rs) replays any `Blake3ReceiptEntry` chain through the
crate's single validator, `Blake3ChainValidator`; an empty chain is refused
(`LedgerError::TooShort`), not laundered, and a broken link is
`LedgerError::ChainLinkMismatch { index }` carrying the failing record index.

## ML-DSA-65 sealing

Feature: `nato`. Modules: `src/ledger/signing/`. Ported from strata-protocol
`src/signer.rs @ 0de13a3`.

### Wire constants (FIPS 204)

| constant | value | meaning |
|---|---|---|
| `SIGNATURE_LEN` | 3309 | detached ML-DSA-65 signature (FIPS 204 §7.2, lambda=3, cat. 3) |
| `PUBLIC_KEY_LEN` | 1952 | encoded verifying key |
| `SEED_LEN` | 32 | generation seed (xi) |
| `SIGN_CONTEXT` | `b"chicago-ledger-dossier-v1"` | fixed ML-DSA context string (FIPS 204 §5.4) |

`assert_wire_sizes()` checks the ml-dsa crate's actual encoded sizes against these
constants at runtime. Signing uses the FIPS 204 deterministic variant, so the same
seed + subject reproduces byte-identical signatures.

### Subject domain separation

`subject.rs`: **`subject = BLAKE3(u32le(len(SUBJECT_DOMAIN)) ‖ SUBJECT_DOMAIN ‖ root)`**
with `SUBJECT_DOMAIN = b"chicago-ledger-dossier-subject-v1"`. The `u32le` length
prefix makes the domain/root boundary unambiguous; a foreign domain tag or an
unframed hash produces a different subject.

### Seal envelope and fail-closed verification

`sealer.rs`: `seal(root, signer) -> SealedDossier` builds a `SealEnvelope`
{ `key_id` = first 8 bytes of BLAKE3(public_key), `alg_marker` = `"ML-DSA-65"`
NUL-padded to 16 bytes, `subject`, `format_version = FORMAT_VERSION = 1` } and signs
the subject. `verify_sealed(&SealedDossier, &[u8; 1952]) -> SealVerdict` is
two-valued (`Valid` / `Invalid`) and evaluates in this exact order:

1. `format_version` supported.
2. **Subject binding**: `dossier_subject(&sealed.root)` must equal
   `envelope.subject` — deliberately **before** any signature work; a valid
   signature over a mismatched subject must never read as Valid (pinned by
   `subject_before_signature_with_garbage_sig`).
3. Signature over the subject against the claiming public key (plus alg-marker and
   key-id recheck).

### Wire encode/decode

`sealer.rs` gives the seal a deterministic wire form (Gap G-1): fixed
`WIRE_LEN = 3399` bytes — `root[32] || key_id[8] || alg_marker[16] ||
subject[32] || format_version[2 le] || signature[3309]` — so the CLI can
persist seal output and read it back. `encode(&SealedDossier) -> [u8; 3399]`
(sealer.rs:120) is pure structural serialization; `decode(&[u8]) ->
Result<SealedDossier, SealWireError>` (sealer.rs:165) is a pure structural
decode — no signature check, no subject binding; decoded bytes are
**unverified** until passed through `verify_sealed`. A wrong-length input is
`SealWireError { expected, actual }`, a distinct type from `SealError`
because a truncated file is a transport problem, not a sealing-law refusal.

`SealedDossier` is currently an in-memory struct (`root`, `envelope`, `signature`);
`sealer::encode` / `sealer::decode` give it a deterministic 3399-byte wire form
(see below). Both `key_id` laws — the sealer's envelope law and
`keys::key_id` (keys.rs:287) — are unified on BLAKE3: first 8 bytes of
BLAKE3 over the encoded public key (lane R8 reconcile).

### Key custody (`keys.rs`)

`keys` is the ONLY module in `crate::ledger` permitted to touch `std::fs`
(enforced by module law; everything else is pure in-memory computation).

- Seed file: `ml_dsa_65.seed` under `DEFAULT_KEY_DIR = .ggen/keys/ledger`
  (relative to the process working directory); `KeyStore::with_dir` overrides for
  tests, which must use a tempdir and never the real dir.
- Generate-on-first-use: seed present and exactly 32 bytes → deterministic reload
  (same key id every time); absent → fresh OS entropy (`getrandom::fill`), then
  atomic persist (temp file + rename, **0600 file / 0700 dir** on unix; best-effort
  no-op elsewhere, documented limitation).
- Wrong-length or unreadable existing seed → `SealError::KeyCustody` refusal. The
  seed is NEVER silently regenerated: a regenerated key would mint seals the old
  public key cannot verify — indistinguishable from tampering. A refused custody
  check leaves the corrupt seed untouched on disk.
- `rotate()` atomically rewrites the seed with fresh entropy and returns the new
  key id. Forward-only: old seals stay verifiable (each seal carries its key id and
  prior public keys resolve from earlier ledger records); history is never rewritten.
- Entropy failure is a typed `SealError::EntropyRefused(RefusalDigest)` carrying an
  opaque 32-byte digest — never a panic, never a partial key.

## CLI (`affi`, Phase D M2 — landed in `~/affidavit`)

The `ctdd_ledger` binary and the `ledger-cli` feature were removed in the V2
cleanup. The former subcommand surface is LANDED as `affi` verbs (Phase D M2,
`~/affidavit`): `affi receipt validate`, `affi receipt chain-verify`,
`affi dossier seal`, `affi dossier verify`, `affi dossier emit`,
`affi dossier inspect`. The domain logic lives in the library (`src/ledger/`,
notably `dossier_wire.rs`); the Justfile recipes `verify-ledger`, `emit-dossier`,
`dossier-verify`, and `dossier-inspect` invoke the `affi` binary via
`cargo run --manifest-path {{AFFIDAVIT_DIR}}/Cargo.toml` (default
`/Users/sac/affidavit`).

Former subcommands → `affi` verb mapping (all landed):

| subcommand | args | behavior |
|---|---|---|
| `validate` | `[path]` (`-` or omitted = stdin) | validate one receipt |
| `chain-verify` | `<dir>` | verify the receipt chain in a directory of ordered receipt files |
| `seal` | `<dir> --output <file>` | seal a dossier directory into an output file |
| `verify` | `<file>` | verify a previously sealed dossier file |
| `dossier-emit` | `--receipts-dir <dir> [--out <file>]` | emit a `CTT-DOSSIER-v1` JSON dossier from the 20 court receipts `N01.json`..`N20.json` in the directory (default `--out docs/nato/NATO_STANAG_4107_DOSSIER.json`); refuses with a typed error unless all 20 receipts exist and every one is `Pass` + `MutantCaught` |
| `dossier-inspect` | `<file> [--verify-pqc-signature ML-DSA-65] [--assert-merkle-root]` | recompute the Merkle root from the embedded receipts (always runs, fail-closed), and optionally verify the ML-DSA-65 signature; only `ML-DSA-65` is accepted for `--verify-pqc-signature` |

### Dossier JSON schema (`CTT-DOSSIER-v1`)

Emitted by `affi dossier emit` (Phase D M2, landed in `~/affidavit`; formerly the
`ctdd_ledger` bin's `dossier_json`/`DossierJson` — the field set now lives in
`src/ledger/dossier_wire.rs`):

| field | type | content |
|---|---|---|
| `format` | string | always `"CTT-DOSSIER-v1"` (`DOSSIER_FORMAT`) |
| `court_receipts` | array | the 20 embedded court receipt JSON objects (each decoded through `CourtReceipt::from_json` on inspect) |
| `merkle_root` | string | 64-char lowercase hex of the 32-byte Merkle dossier root |
| `key_id` | string | 16-char hex of the 8-byte ML-DSA-65 key id |
| `signature` | string | hex of the 3309-byte ML-DSA-65 signature over the root (`SIGNATURE_LEN`) |
| `sealed_at` | integer | Unix seconds at emission |

### Dossier emit/verify via just

```bash
just emit-dossier     # run nato::dossier_court courts, then affi dossier emit --receipts-dir docs/receipts --out docs/nato/NATO_STANAG_4107_DOSSIER.json --json
just dossier-verify   # affi dossier inspect docs/nato/NATO_STANAG_4107_DOSSIER.json --verify-pqc-signature ML-DSA-65 --assert-merkle-root --json
```

`docs/nato/NATO_STANAG_4107_DOSSIER.json` is a generated artifact (regenerate
with `just emit-dossier`); the emitted copy from the last gated run is checked
in. `emit-dossier` is gated on the
`nato::dossier_court` courts passing first, and `dossier-emit` refuses with a
typed error when `docs/receipts/N01..N20.json` are absent — court-receipt
persistence to `docs/receipts` is wired via the opt-in court test
(`emit_dossier_receipts_when_opted_in`, gated on `CTT_EMIT_RECEIPTS=1`;
permission errors skip typed, never panic) — it is a prerequisite and
`src/nato/dossier_court.rs` (see the honesty note in the Justfile and
`src/nato/README.md` "Receipt contract").

Exit codes — `LedgerExitCode` (`src/ledger/mod.rs`); 0 = success, one failure class
per code:

| code | variant | class |
|---|---|---|
| 0 | `Ok` | chain (or other operation) verified |
| 10 | `MalformedReceipt` | receipt bytes malformed / undecodable |
| 11 | `SchemaVersionMismatch` | schema version not supported by this reader |
| 12 | `ChainBroken` | BLAKE3 chain failed to replay |
| 13 | `Tampered` | stored material disagrees with recomputed material |
| 14 | `AdmissionRefused` | admission refused (custody / entropy / policy) |
| 15 | `DanglingObject` | referenced object dangling / unknown |
| 16 | `NonMonotonic` | sequence ordering regressed |
| 17 | `EmptyDossier` | dossier contained no proofs / no courts |
| 18 | `DigestMismatch` | stored digest did not match recomputed digest |
| 19 | `DossierInvalid` | sealed dossier structurally invalid (format tag / receipt count / recomputed Merkle root mismatch) |

`exit_code(&LedgerError)` (mod.rs:245-258) is the error→exit-code mapping:
`UnsupportedVersion` → 11, `ChainLinkMismatch` → 12, `DigestMismatch` → 13,
`CrossCourtAppend` / `EmptyChain` → 16, everything else → 10.

End-to-end one-shot verification is wired in the Justfile:

```bash
just verify-ledger SOURCE
```

which runs the `affi` binary (`cargo run --manifest-path /Users/sac/affidavit/Cargo.toml
--features ctdd --bin affi -- --format json receipt validate "<SOURCE>"`,
matching the Justfile recipe at Justfile:199-200).

### In-flight seams (honesty notes)

- The `ctdd_ledger` bin was excised (V2 cleanup); the CLI verbs are re-landed in affi
  affi (Phase D M2). The domain contract it consumed —
  `ledger::{validate_receipt, chain_verify, seal_dossier, verify_sealed_file,
  exit_code}` — remains landed in `src/ledger/mod.rs` (`verify_sealed_file` at
  mod.rs:184).
- The error→exit-code table in `mod.rs` and the decode paths in `receipt.rs`
  use the same variant spellings (`ControlFlagMismatch`, `ReservedNonzero`);
  the earlier spelling mismatch has been reconciled.

## See Also

- `docs/features/NATO_COURTS.md`
- `docs/features/CONFORMANCE.md` (the conformance runs whose receipts the ledger seals)
- `src/ledger/receipt.rs` (wire-format module doc reproduces the full layout)
- `Justfile` recipes: `test-ledger`, `verify-ledger`
