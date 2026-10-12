# Explanation: The Chatman Equation as Conservation

## The claim, stated honestly

The Chatman Equation — `A = μ(O*)`, `R = receipt(A)`, with
`R = {identity, authority, consequence, replay, standing}` — is often glossed
as a theory of autonomy. In the settlement domain it is better read as a
**conservation law**: value and obligation admitted into a system must be
conserved across every transformation, and a receipt is the ledger entry that
makes the conservation checkable. `Σ positions == 0` in the netting kernel and
`A = μ(O*)` are the same shape of claim: nothing is created or destroyed in a
lawful transition, only rearranged under an admissible rule.

The honest question is what this buys over a classical audit trail. This
essay answers from landed code, not aspiration.

## What the landed chain actually is

The settlement dossier chain as landed in this repo:

1. **Court receipts** — each verification run produces a `CourtReceipt`
   (`src/nato/receipt.rs`) decoded through a typed receipt-class refusal
   (`CourtReceipt::from_json` refuses a malformed receipt rather than failing
   the whole document).
2. **Merkle commitment** — `src/nato/dossier_court.rs` computes a P13 Merkle
   root over canonical digests, leaves sorted byte-lexicographically by court
   identity, then seals the root with ML-DSA-65
   (`src/ledger/signing/signer.rs`; signature length 3309 bytes, wire form
   6618 lowercase hex chars per `dossier_wire.rs`).
3. **Wire surface** — `CTT-DOSSIER-v1` (`src/ledger/dossier_wire.rs`): format
   discriminator, embedded receipt objects, 64-char hex Merkle root. The hash
   law is never reimplemented; chain replay goes through
   `Blake3ChainValidator` over `Blake3ReceiptEntry` entries.
4. **Host-side seal** — the phase-10 sealing activities are marked
   CTT-HOST-SIDE in the model itself (`phase10_sealing.powl`); the master seal
   executes via `just emit-dossier`, outside the wasm guest boundary.

Every element of that chain exists and is tested. That is the ground truth
this essay stands on.

## What self-attesting receipts buy over a classical audit trail

A classical audit trail is a sequence of records plus a promise that some
trusted party wrote each record. Its weakness is not forgery of a single
record; it is the **gap between the record and the thing recorded**. Nothing
in a classical log proves the log entry corresponds to an execution that
actually produced the stated consequence.

The landed chain attacks exactly that gap, in three specific ways:

**1. Replay replaces testimony.** A receipt carries a replay path — the exact
subject, the court that admitted it, and the input identity — so a third party
re-derives the verdict instead of trusting it. The merger-guests ABI is the
same idea at the compute layer: each `mgi_*_compute` call is a pure function
of its input buffer, so byte-for-byte replay of any call is available to any
host. Conformance replay is the same idea at the process layer: the OCEL trace
replays through the POWL net and the session either reproduces the verdict or
latches a typed fault. Nothing rests on the writer's word.

**2. Post-quantum signature anchors identity now, not later.** Sealing the
Merkle root with ML-DSA-65 means the dossier's identity is anchored against an
adversary who can break classical signatures later ("harvest now, decrypt
later" applied to audit). A classical audit trail signed with ECDSA has a
forward-dated trust problem; the landed chain does not.

**3. Fail-closed conservation replaces sampled reconciliation.** The netting
kernel refuses (`ERR_OVERFLOW`) if `Σ positions != 0`, even though the
refusal is unreachable-by-construction — the check is retained because the
invariant is what makes the kernel a *ledger* and not just a calculator. A
classical audit reconciles after the fact, on samples; the conservation check
here runs on every call, fail-closed.

## What it does NOT buy — the honest ledger

- **Self-attestation is still attestation by the executing party.** The
  receipt chain proves internal consistency: this dossier commits to these
  court outputs, sealed under this key. It does not prove the *courts
  themselves* were correct — that burden stays on the court design (the
  falsifier corpora, the mutation checks). Conservation of a dossier is not
  correctness of its contents; it is non-malleability of its contents.
- **The key ceremony is an unconserved root of trust.** ML-DSA-65 anchors
  identity to a key; who holds the key, and how rotation is witnessed, is
  outside the conservation equation as landed (`src/ledger/signing/keys.rs`
  is where that law lives).
- **Attributes are UNSUPPORTED at the packed tier.** The packed OCEL64 tier
  roundtrips count, timeline, and objects losslessly but drops attributes —
  the phase-8 count law (zero over-issuance) therefore cannot ride the packed
  session tier at all; it is pinned on the rich JSON. Conservation claims must
  be scoped to the tier that can carry them.

## Status of the conservation equation: design invariant with landed approximations

The full conservation equation — *every unit of value entering the system
admits a receipt, and every receipt replays to the same consequence on any
host, forever* — is a **design invariant, not a landed theorem**. State it
that way. What IS landed, each independently tested:

- Per-call conservation in the netting kernel (fail-closed `ERR_OVERFLOW`).
- Per-document conservation in the dossier (Blake3 chain validation; a broken
  chain refuses; the hash law is never reimplemented at call sites).
- Per-settlement conservation in the POWL structure (the DvP forbidden marking
  is unreachable by construction and checked by `forbidden_reachable`; the
  2PC commit is one atomic transition so the two legs cannot diverge).
- Replay determinism end to end: pure guest kernels with stable sorts,
  deterministic `(timestamp_ns, event_id)` fixture ordering, frozen
  fault-latch semantics so a replayed verdict is the verdict.

The gap between the design invariant and the landed approximations is
precisely the list above: court correctness, key custody, and the packed-tier
attribute boundary. A reader who needs the strong claim should fund those
three lanes; a reader who needs the honest claim already has it — **the
landed chain makes tampering detectable and verdicts replayable, which is
what a settlement system actually requires, and it does so with mechanisms
that exist and pass their courts today.**

## See also

- `01-tutorial.md` — executing the Phase 7 court.
- `02-how-to.md` — the typed refusal surface and invariant masks.
- `03-reference.md` — the merger-guests ABI as landed.
- `crates/merger-guests/src/netting_engine.rs` — the conservation kernel.
- `src/nato/dossier_court.rs`, `src/ledger/dossier_wire.rs` — the seal chain.
