# Explanation: Monoidal Trajectory Folding vs IVC/Nova — What This Substrate Actually Inherits

This page explains the folding structure behind the dossier ledger and states precisely which
ideas it shares with incremental verifiable computation (IVC) schemes such as Nova — and which
it does not. The honest one-line summary: **this is a monoidal fold with O(1) carrier size;
it is not succinct verification of arbitrary computation, and it does not claim to be.**

## The monoid

The carrier is a 32-byte BLAKE3 chain head. The fold operator, from the chain law in
`src/ledger/dossier.rs` (module header and `CourtLedgerBuilder::append`):

```text
head(0)     = [0u8; 32]                    (zeroed genesis)
head(n+1)   = BLAKE3(head(n) || receipt.stored_hash())
```

In monoid terms: carrier `H = [u8; 32]`, identity `e = [0u8; 32]`, and the fold of a receipt
with the running carrier is `h ⊕ r = BLAKE3(h || stored_hash(r))`. Each step consumes one
352-byte chained record (the CTRT wire format in `src/ledger/receipt.rs`: a 320-byte receipt
record + 32-byte chain hash; `COURT_RECORD_LEN = 352`), but the *carrier* stays 32 bytes no
matter how many receipts fold in — that is the O(1) property. The `dossier_wire::ChainRecord`
projection (`src/ledger/dossier_wire.rs`) names the same law at the wire level: RM1 offset law,
`content = record[..304]`, `stored = record[320..352]`, `prev = parent_receipt_hash`.

Associativity caveat, stated honestly: this is a **sequential chain, not a free-to-reassociate
monoid**. `prev_hash` threading is enforced (`LedgerError::ChainDiscontinuity` when a
receipt's parent does not equal the current head; byte-identical replay refused; cross-court
append refused with `CrossCourtAppend`), so `(a ⊕ b) ⊕ c` is the only admissible association.
The "monoid" is useful as a *carrier-size* statement — one accumulator, constant size — not as
a parallelism license. Verification of the fold is a linear replay of all 352-byte records.

## What the DossierBuilder adds

Per-court chains are aggregated into a Merkle dossier root (`DossierBuilder.seal`): leaves are
`(court_id, head)` pairs sorted byte-lexicographically, odd nodes are **promoted, never
duplicated**, and each court gets an inclusion proof of depth `ceil(lg2(N))` (~5 steps at 20
leaves). So the two-level structure is: hash chain *within* a court (temporal integrity of the
trajectory), Merkle root *across* courts (one 32-byte commitment to the whole dossier).

## What this inherits from folding-scheme theory

1. **The accumulator pattern.** One constant-size carrier (`h`) summarizes an unbounded
   sequence of steps, exactly as an IVC running accumulator summarizes an unbounded
   computation. Verifying "the trajectory happened" reduces to checking the final carrier
   plus replay material, not re-holding every intermediate.
2. **Incremental updates.** Each append is O(1) work over the previous carrier —
   `head || stored -> BLAKE3` — the same shape as Nova's fold of a new relaxed R1CS instance
   into the running accumulator.
3. **Domain separation and binding.** The chain binds content (`record[..304]`), parent
   (`parent_receipt_hash`), and identity (court `name` at body offset 204, absolute 220), so a
   folded head attests to a specific sequence — the role Nova's cross-term checks play.
4. **Fail-closed admission.** Refusals are typed and structural (wrong parent, replay,
   cross-court, empty chain), mirroring how a folding scheme refuses an instance that does not
   satisfy the accumulated constraint.

## What it does NOT inherit

This is the part that must not be overclaimed:

1. **No succinct verification.** Nova verifies an arbitrary (R1CS-expressible) computation with
   work polylogarithmic in the number of steps. Here, verifying the fold requires **hashing
   every 352-byte record in sequence** — O(N) work, linear in trajectory length. The carrier is
   O(1); the *verifier's effort is not*. Nothing is inherited from elliptic-curve folding, and
   the BLAKE3-based fold gives no pairing/SNARK-style proof object at all.
2. **No succinct argument of knowledge.** The chain proves that records were threaded in an
   order and were not altered after the fact (tamper-evidence). It does not prove that the
   *executions described by the records actually happened* — that burden is carried by the
   court system (N05/N06/W4 etc.: real timing, disassembly, and import-census evidence) that
   produced the receipts. A valid chain over fabricated receipts is a valid chain.
3. **No zero knowledge.** Everything in a receipt record is plaintext wire fields
   (subject hash, ticks, wall time, outcome, name). No hiding, no blinding.
4. **No folding of arbitrary relations.** Only one operator exists (`BLAKE3(head || stored)`).
   There is no constraint system, no witness, no relaxed-instance notion, and no cross-term
   absorption.
5. **No parallel/arbitrary-association folding.** As noted, parent threading fixes the
   association; Nova's folding is likewise sequential, but this scheme additionally lacks even
   the algebraic structure (group commitments) that makes Nova's accumulator updatable by
   short proofs from untrusted provers.

## Precise statement of what you get

Given the sequence of 352-byte CTRT records `r_1..r_N` and the final head `h_N`:

- **What a 32-byte head gives you:** a constant-size commitment such that any tampering with
  any record (or reordering, or splicing between courts) changes `h_N` with overwhelming
  probability, detectable on a linear replay.
- **What replay costs:** O(N) hashing of 352-byte records — the replay path runs through the
  domain `Blake3ChainValidator` (`dossier_wire.rs` explicitly refuses to reimplement the hash
  law: "Same bytes, same law, no second hash implementation").
- **What the Merkle root adds:** N-court aggregation with logarithmic inclusion proofs per
  court, sealed with ML-DSA-65 (`CTT-DOSSIER-v1`, `dossier_json`/`parse_*` in
  `dossier_wire.rs`).
- **What it does not give you:** a short proof that replaces replay. There is no object in this
  substrate whose verification cost is sublinear in N.

## Why this is still the right tool here

The trajectory is short, the verifier is local, and the evidence quality lives in the courts,
not in the fold. Folding-scheme machinery (Nova-class) would buy sublinear verification — at
the cost of a constraint system over the entire court pipeline, which this substrate does not
have and does not need at current scale. The monoidal carrier is the piece that was worth
taking: one 32-byte head per court, one root over all courts, typed structural refusals, and
a single hash law replayed through one implementation. Where an actual IVC requirement appears
(sublinear verification by an untrusting party), that is a different primitive, and this
substrate neither pretends to it nor blocks it — the chain law is one function, and the
receipts it folds are self-describing wire records.
