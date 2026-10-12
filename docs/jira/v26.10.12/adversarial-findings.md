# Adversarial Skeptic Ensemble Findings Receipt (v26.10.12)

- **Subject under audit**: the Chatman Equation conservation claim (`A = μ(O*)`,
  `R = receipt(A)`; nothing is consumed that was not admitted, nothing admitted
  that is not conserved) as realized by the conformance kernel of
  `chicago-tdd-tools` v26.10.12.
- **Method**: 10-lane adversarial skeptic ensemble (SK1-SK10), 2026-10-11,
  fan-out on the canonical checkout `/Users/sac/chicago-tdd-tools` (main).
  Each lane was dispatched to falsify one surface; verdicts below are
  post-reproduction.
- **Subject SHA at dispatch**: 4d451c7 (main HEAD, v26.10.12 docs train).

## Per-lane verdict table

| Lane | Verdict | Finding (one line) | Disposition |
|------|---------|--------------------|-------------|
| SK1 | FALSIFIED | PN64 fold is non-associative and `fire` panics outside the 1-safe domain; conservation only holds on the restricted 1-safe sub-domain. | DOCUMENTED (domain precondition now stated at `src/conformance/pn64.rs:16-24`; conservation guard test landed) |
| SK2 | FALSIFIED | `wire_ctrt` is constant-*flow* (data-independent control), not constant-*time*; observable |t| spans 10-3842 flow steps. | QUEUED (doc correction: relabel claim as constant-flow) |
| SK3 | FALSIFIED | Raw-context collision: two raw contexts can collide and `SigningKey::from_seed` is publicly reachable (`src/ledger/signing/keys.rs:158`), bypassing key-id admission. | QUEUED |
| SK4 | FALSIFIED | Payload covert channel: payload bytes influence flow structure observable to a stream reader; BEAM and Rust runtimes diverge on the same fixture. | QUEUED |
| SK5 | FALSIFIED | Key rotation replay amnesty: `KeyStore::rotate` (`src/ledger/signing/keys.rs:254`) admits pre-rotation receipts (replay amnesty), and seed persistence is fail-open on write failure. | QUEUED |
| SK6 | FALSIFIED | Receipts are module-identity-blind (no module hash bound into `R.identity`), and out-pointers are trusted on read without re-verification. | QUEUED |
| SK7 | HELD | No falsification sustained on this lane's surface; original claim stands as stated. | NONE |
| SK8 | FALSIFIED | Word-identity sentinel misclassifies lawful self-loop firings (`W- == W+` leaves the marking bit-exact unchanged) as refusals; PN64 OR-coalescing makes this reachable on ordinary nets. | FIXED (post-tag, see below) |
| SK9 | PARTIAL | REA starvation is real but is a cost-elevation effect (starved transitions are delayed, not unschedulable); no correctness violation, bounded liveness preserved. | DOCUMENTED |
| SK10 | FALSIFIED | Open-texture predicates admit boolean-forcing; N19 receipt is dual-subject (binds source SHA and artifact set without a binding between them); DoS latch via unbounded replay of expensive checks. | PARTIALLY QUEUED (N19 dual-subject documented in `docs/releases/RELEASE_NOTES_v26.10.11.md:52` as 3-host NOT-RUN; boolean-forcing + DoS latch queued) |

Tally: 8 FALSIFIED, 1 PARTIAL, 1 HELD.

## Falsifications

### SK1 - PN64 fold non-associativity, panic, unsigned domain

- **Claim falsified**: that PN64 composition/fold is a free commutative monoid
  operation usable on arbitrary markings.
- **Repro**: `cargo test -p chicago-tdd-tools fire_token_conservation` /
  firing arithmetic in `fire` (`src/conformance/pn64.rs:95`): deposit uses
  bitwise OR, so a deposit into an occupied place coalesces two tokens into
  one - `|m'| != |m| - |W-| + |W+|` off the 1-safe domain; the validity
  predicates fold via `x | x.wrapping_neg()` (`src/conformance/pn64.rs:149`).
- **Disposition**: DOCUMENTED. The 1-safe domain precondition is now stated in
  the module doc (`src/conformance/pn64.rs:16-24`, working tree) and the
  conserved sub-domain is enumerated by the in-module conservation guard test.

### SK2 - wire_ctrt: constant-flow, not constant-time; |t| = 10-3842

- **Claim falsified**: that the wire control-flow transform is constant-time.
- **Repro**: timing-sensitive run over the wire surface
  (`src/ledger/dossier_wire.rs`); measured |t| range 10-3842 across inputs
  (constant-flow property: control flow independent of secrets, wall time not).
- **Disposition**: QUEUED - relabel the claim constant-flow-not-constant-time
  in `docs/features/CONFORMANCE.md` and the Diataxis attestation pages.

### SK3 - Raw-context collision + public `from_seed`

- **Claim falsified**: that key construction is reachable only through the
  admitted keystore path.
- **Repro**: `grep -n "from_seed" src/ledger/signing/keys.rs` - public
  constructor at `src/ledger/signing/keys.rs:158`; two raw contexts can be
  constructed that collide on the admission predicate.
- **Disposition**: QUEUED - gate raw-context construction / make
  `from_seed` pub(crate) or seal behind the keystore.

### SK4 - Payload covert channel + BEAM/Rust divergence

- **Claim falsified**: that the wire encoding leaks nothing about payload
  structure to a stream reader across runtimes.
- **Repro**: property run over `src/ledger/dossier_wire.rs` encoders; payload
  bytes perturb the observable flow lattice; the same fixture yields different
  flow under the BEAM (wasmex/NIF) host vs the pure-Rust path.
- **Disposition**: QUEUED - normalize flow per payload or document the
  channel and the runtime divergence as out of threat model.

### SK5 - Rotation replay amnesty + fail-open persistence

- **Claim falsified**: that rotation invalidates all pre-rotation receipts.
- **Repro**: `KeyStore::rotate` (`src/ledger/signing/keys.rs:254`); a receipt
  sealed under the pre-rotation key id still verifies after rotation
  (replay amnesty), and a failed seed rewrite leaves the store in the old
  state with success reported (fail-open).
- **Disposition**: QUEUED - bind receipts to key epoch and make rotation
  persistence fail-closed.

### SK6 - Module-identity-blind receipts + out-pointer trust

- **Claim falsified**: that `R.identity` binds the executing module.
- **Repro**: inspect any sealed receipt: no module hash field; out-pointers
  (artifact refs) are dereferenced and trusted at replay without
  re-hashing.
- **Disposition**: QUEUED - add module hash to the receipt identity tuple;
  re-verify out-pointers at replay.

### SK8 - Self-loop sentinel + OR-coalescing domain

- **Claim falsified**: that a bit-exact unchanged word after
  `step_branchless` implies refusal.
- **Repro**: `cargo test -p chicago-tdd-tools self_loop_firing_completes_not_skipped`
  - a self-loop transition (`W- == W+`, word class `0x0000_0000_0001_0001`)
  leaves the word unchanged and was misread as `Skipped`. The fix replaces the
  word-identity sentinel with the verb-match + token-availability predicates
  (`src/conformance/session.rs`, `step` commit path; refusal predicate now
  computed from `pn64::tid`/`pn64::marking & pn64::pre`).
- **Disposition**: FIXED - regression tests in `src/conformance/session.rs`
  (in-module) and `tests/conformance_tests.rs` (public boundary:
  `unchanged_word_refusal_holds_at_public_boundary`,
  `trajectory_identity_is_model_relative_not_absolute`).

### SK9 - REA starvation = cost elevation

- **Claim falsified (partially)**: that REA scheduling can starve a
  transition into unschedulability, violating liveness.
- **Repro**: bounded-step runs under contention; the starved transition is
  delayed (cost rises) but remains schedulable; bounded liveness holds.
- **Disposition**: DOCUMENTED - starvation is a cost property, not a
  correctness violation.

### SK10 - Open texture, N19 dual-subject, DoS latch

- **Claim falsified**: that the evaluation vocabulary is closed under its own
  predicates.
- **Repro**: open-texture categories admit boolean-forcing (an evaluator can
  manufacture a pass by re-describing the predicate); `docs/receipts/N19.json`
  binds both a source SHA and an artifact set with no binding between them
  (dual-subject receipt); expensive re-verification checks are replayable
  without bound (DoS latch).
- **Disposition**: N19 dual-subject DOCUMENTED (release notes already record
  3-host NOT-RUN, `docs/releases/RELEASE_NOTES_v26.10.11.md:52`);
  boolean-forcing and DoS latch QUEUED.

## Landed post-tag (working tree, this session) vs queued

Landed at receipt time on top of 4d451c7 (SK8 fix-forward commit 4d26c84
plus the domain/identity doc edits in the shared working tree):

- SK1: 1-safe domain precondition doc + conservation guard (`src/conformance/pn64.rs`).
- SK6-adjacent / model-relative identity: trajectory identity documented as
  per-net positional (`src/conformance/session.rs` module doc).
- SK8: sentinel replacement + regression tests (`src/conformance/session.rs`,
  `tests/conformance_tests.rs`).

Queued (no code change in this session): SK2, SK3, SK4, SK5, SK6, SK10.

Note on fix IDs: the coordinator's R29-R34 batch corresponds to the landed
set above; per-R mapping is not reproduced in this receipt because the
R-numbered receipts were not visible to this lane at write time.
