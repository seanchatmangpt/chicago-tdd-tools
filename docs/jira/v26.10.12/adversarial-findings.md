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
| SK2 | FALSIFIED | `wire_ctrt` is constant-*flow* (data-independent control), not constant-*time*; observable |t| spans 10-3842 flow steps. | QUEUED - relabel the claim constant-flow-not-constant-time |
| SK3 | FALSIFIED | Raw-context collision: two raw contexts can collide and `SigningKey::from_seed` is publicly reachable (`src/ledger/signing/keys.rs:158`), bypassing key-id admission. | QUEUED |
| SK4 | FALSIFIED | Payload covert channel: payload bytes influence flow structure observable to a stream reader; BEAM and Rust runtimes diverge on the same fixture. | QUEUED |
| SK5 | FALSIFIED | Key rotation replay amnesty: `KeyStore::rotate` (`src/ledger/signing/keys.rs:254`) admits pre-rotation receipts (replay amnesty), and seed persistence is fail-open on write failure. | QUEUED |
| SK6 | FALSIFIED | Receipts are module-identity-blind (no module hash bound into `R.identity`), and out-pointers are trusted on read without re-verification. | PARTIAL (F2/F3 landed 532f38a + 2c80a9e: out-pointer bounds + errnos 14-16; F1 module digest placeholder until build-time splice) |
| SK7 | FALSIFIED | port write-path has no backpressure (64MiB frame queued in driver memory, 23ms, never-reading bridge); bridge process inherits full env, no rlimits, unsandboxed — zero-ambient falsified at the OS boundary | QUEUED (port backpressure ack-gate + env-allowlist/rlimit work order, strata-side) |
| SK8 | FALSIFIED | Word-identity sentinel misclassifies lawful self-loop firings (`W- == W+` leaves the marking bit-exact unchanged) as refusals; PN64 OR-coalescing makes this reachable on ordinary nets. | FIXED (post-tag, see below) |
| SK9 | PARTIAL | REA starvation is real but is a cost-elevation effect (starved transitions are delayed, not unschedulable); no correctness violation, bounded liveness preserved. | DOCUMENTED |
| SK10 | FALSIFIED | Open-texture predicates admit boolean-forcing; N19 receipt is dual-subject (binds source SHA and artifact set without a binding between them); DoS latch via unbounded replay of expensive checks. | PARTIALLY QUEUED (N19 dual-subject documented in `docs/releases/RELEASE_NOTES_v26.10.11.md:52` as 3-host NOT-RUN; boolean-forcing + DoS latch queued) |

Tally: 9 FALSIFIED, 1 PARTIAL (SK9), 0 HELD.

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

Queued (no code change in this session): SK2, SK3, SK4, SK5, SK10. SK6/SX6: PARTIAL (see rows).

Note on fix IDs: the coordinator's R29-R34 batch corresponds to the landed
set above; per-R mapping is not reproduced in this receipt because the
R-numbered receipts were not visible to this lane at write time.

## Second ensemble - blockchain-replacement thesis (SX1-SX10)

- **Subject under audit**: the claim that the conformance/ledger kernel can
  serve as a blockchain-replacement substrate (ordering, value-bearing
  admission, double-spend resistance, MEV resistance).
- **Method**: second 10-lane adversarial skeptic ensemble (SX1-SX10),
  2026-10-11, same fan-out on the canonical checkout
  `/Users/sac/chicago-tdd-tools` (main). One lane per thesis surface;
  verdicts below are post-reproduction where a report has landed.
- **Subject SHA at dispatch**: b798357 (main HEAD at SX dispatch).

| Lane | Verdict | Finding (one line) | Disposition |
|------|---------|--------------------|-------------|
| SX1 | FALSIFIED-BY-ABSENCE | No burn/odometer/blind-nullifier mechanism exists anywhere in the tree; `Lease.admit` trusts `attested: true` without verifying a signature - 200 forged leases admitted in 5ms. | FIXED (R35, strata commit 514308f — lease admission fail-closed; deferred_ctt_boundary honest deferral) |
| SX2 | FALSIFIED (vacuous) | No ordering layer exists to falsify directly: split-brain is the default, the wire format carries no seq/nonce, and an in-path adversary holds the MEV position. | QUEUED |
| SX3 | FALSIFIED | Affidavit fold layer: UTF-8 identity instability (NFC/NFD renders yield different chain_hash/court_id); Skip>Fail fold erases Fail byte-identically; outcome 255 decodes unvalidated; CTT `receipt.rs` range-checks but `unwrap_or(Error)` silently reclasses (`:634`). | QUEUED (fix: NFC pin + severity-lattice fold + validated outcome; CTT format-law unification) |
| SX4 | FALSIFIED | R31's in-memory fix witnessed, but persistence re-opens the amnesty: `to_bytes` omits `nullifier_history` (empty rebuild after save/load); production `handlers.rs` flat-log admits cross-process replay (FULL amnesty); AFRN snapshot is unauthenticated plaintext (generation forgery accepted). | FIXED (R38, commit 1e2e840 — AFRN v2 + HMAC-sealed snapshots, fail-closed persistence) |
| SX5 | UNSUPPORTED | Dual-verify claim unsupported - no runtime libcrux anywhere, oracle-consensus API takes caller-supplied booleans; R32 degenerate-seed screening landed ES256-only (ml-dsa/slh-dsa `from_seed` still infallible). ml-dsa verify fuzz HELD (0/12373 accept-on-malformed). | FIXED (R37, commit 1a16e25 — 0xE305 screening on both PQC paths) + doc retracted (UNSUPPORTED verdict stands) |
| SX6 | FALSIFIED | Module-identity-blind seal preimage CONFIRMED (same finding as SK6 at the seal layer); out-pointers trusted on read without re-verification; trap indistinguishability. | PARTIAL (V5 envelope landed 2c80a9e — 32B module digest slot + M2 falsifier; placeholder until build-time splice; out-pointer re-verify landed 532f38a) |
| SX7 | FALSIFIED | ash_graphlaw membrane: contradictory covenants admitted silently (both `priorityOf` and `notPriorityOf` derived from the same fact, no conflict court); authority triples (`authorizedSigner`/`actuationAuthority`) on untargeted classes pass SHACL conform:true; cycle divergence HELD (typed `:resource_limit` refusal). | QUEUED (deontic conflict court + authority-vocabulary SHACL coverage; ash_graphlaw work orders, not CTT code defects) |
| SX8 | FALSIFIED | Prefix exhaustion vacuously conforming: a stream that exhausted before reaching the net's terminal place latched `Phase::Complete` with `is_conforming()==true` and no deviation. Tid truncation/boundary class HELD (closed). | FIXED (commit 8b92c3e, SX8-F1) |
| SX9 | HELD (ALIVE) | `deserialize_ct` survives SK2's exact methodology: thrash run 178/192 \|t\|<4.5, the 14 excursions flip sign between same-path classes (noise, not leak); branch-free below the public length gate, verdict table compiled to csel. Caveat: single-run cold Welch is drift-sensitive; future verification uses thrash/interleaved. | VERIFIED (R34; methodology note recorded) |
| SX10 | PARTIAL | SMT/hand recovery of fold constants trivially succeeds - REA resistance is cost-elevation only (delay, not impossibility); consistent with SK9. No correctness violation. | DESIGN (document as cost-elevation, not confidentiality) |

Tally: SX3, SX4 (FIXED 1e2e840), SX6, SX7, SX8 (FIXED 8b92c3e) FALSIFIED;
SX1 FALSIFIED-BY-ABSENCE (fix landed: strata 514308f); SX2 FALSIFIED
(vacuous); SX5 UNSUPPORTED (R37 screening landed: 1a16e25); SX9 HELD
(kernel ALIVE); SX10 PARTIAL. No lane remains open.

### SX1 - No conservation mechanism; unsigned lease admission

- **Claim falsified**: that the kernel conserves value under a
  blockchain-replacement thesis (burn/odometer/blind-nullifier class).
- **Repro**: tree-wide search finds no burn, odometer, or blind-nullifier
  mechanism; `Lease.admit` accepts the caller-supplied `attested: true` flag
  without any signature verification. Forged-lease run: 200 leases admitted
  in ~5ms.
- **Disposition**: FIXED (R35, strata commit 514308f — lease admission
  fail-closed via injected verify_sig callback; unverified leases receive
  honest `deferred_ctt_boundary` instead of `attested: true`). Burn/odometer
  Sybil mechanisms remain OPEN/UNSUPPORTED (strata KG-SX1).

### SX2 - Ordering layer absent; MEV position held by in-path adversary

- **Claim falsified (vacuously)**: that ordering/MEV resistance holds.
- **Repro**: there is no ordering layer, so the property is vacuous -
  split-brain is the default state; the wire format carries no sequence
  number or nonce; any in-path adversary occupies the MEV position by
  construction.
- **Disposition**: QUEUED.

### SX3 - Affidavit fold layer: identity instability + liability erasure

- **Claim falsified**: that the affidavit fold is a lossless, identity-stable
  aggregation of court outcomes.
- **Repro**: (1) NFC/NFD renders of the same court identity produce different
  `chain_hash`/`court_id` (no normalization anywhere - a court re-registered
  under different normalization is a different court). (2) `Skip(2) > Fail(1)`
  under `max()`: a chain with `Fail@12 + Skip@13` folds byte-identical to a
  never-failed chain - liability erasure (SK8's misclassification class
  resurfacing at the fold layer). (3) outcome byte 255 decodes unvalidated and
  masks even `Refused`. CTT `receipt.rs` range-checks outcomes (stricter) but
  `unwrap_or(Error)` silently reclasses at `:634` - C14-class N-implementation
  drift between the two 352B formats.
- **Disposition**: QUEUED - pin NFC at name ingestion (both formats);
  severity-lattice fold (or OR-of-seen-tags: associative and lossless);
  range-check outcome in `from_bytes`; CTT to refuse unvalidated outcomes and
  adopt one canonical format law.

### SX4 - Persistence re-opens the replay amnesty (R31 incomplete)

- **Claim falsified**: that R31's nullifier fix closed double-spend replay.
- **Repro**: (A) `to_bytes` omits `nullifier_history` entirely - `load`
  rebuilds empty, so cross-generation replay succeeds after every save/load.
  (B) production `handlers.rs` still uses the flat 32-byte log with a constant
  `[0x26; 32]` key and a wire-layer-only check - FULL cross-process replay
  amnesty (every CLI re-run admits). (E) AFRN snapshot is unauthenticated
  plaintext (generation forgery `u64::MAX` accepted; epoch key exfiltratable).
  HELD: 0xE101 FP DoS refuted (0/50k), truncation fail-closed at the registry
  layer, gen wraparound refuted.
- **Disposition**: FIXED (R38, commit 1e2e840) - serialize `nullifier_history` +
  version-bump AFRN; route handlers to AFRN snapshots + `with_generated_key`;
  HMAC the snapshot. Priority HIGH.

### SX5 - Dual-verify UNSUPPORTED; seed screening ES256-only

- **Claim falsified**: the "dual-verified against the F* oracle" claim.
- **Repro**: no runtime libcrux use anywhere in the tree; the 0xE303
  oracle-consensus API takes caller-supplied booleans (a comparison utility,
  not an oracle). R32's degenerate-seed screening landed on ES256 only -
  `ml_dsa65_from_seed`/`slh_dsa128s_from_seed` remain infallible (all-zero/
  all-ones/near-degenerate seeds reproduce valid keys, runtime-proven).
  HELD: ml-dsa verify 12,373-case fuzz, 0 accept-on-malformed.
- **Disposition**: FIXED (R37, commit 1a16e25) - forward-port
  both PQC `from_seed` paths); dual-verify claim to be reworked (feature-gated
  runtime verify-vs-verify over KAT vectors) or retracted in docs.

### SX6 - Module-identity-blind seal preimage; out-pointer trust

- **Claim falsified**: that a seal binds the module that produced it.
- **Repro**: seal preimage contains no module identity (SK6's finding
  confirmed at the seal layer); out-pointers are dereferenced and trusted on
  read without re-verification; trap bytes are indistinguishable from live
  data.
- **Disposition**: QUEUED - V5 work order: bind module hash into the seal
  preimage; re-verify out-pointers at replay.

### SX7 - Contradictory covenants admitted; authority bypasses SHACL

- **Claim falsified**: that the graphlaw membrane refuses incoherent
  authority graphs.
- **Repro**: (1) contradictory covenants admitted silently - both
  `priorityOf` and `notPriorityOf` derived from the same fact with no conflict
  court. (2) authority-carrying triples (`authorizedSigner`/
  `actuationAuthority`) on untargeted classes pass SHACL `conform:true` -
  validation-conforms is not authority-checked. (3) HELD: cycle divergence
  yields a typed `:resource_limit` refusal (`n3_max_iterations=4000`,
  engine-side; minor inverted message "70 > 4000"). Continuous-condition vs
  latching law noted as the temporal variant of SK10's open texture.
- **Disposition**: QUEUED - deontic conflict court over N3 derived output
  (pair-predicate registry to typed refusal); shapes must cover the
  actuation-authority vocabulary, fail-closed on untargeted classes.
  ash_graphlaw/strata work orders - not CTT code defects.

### SX9 - deserialize_ct ALIVE under SK2's methodology

- **Claim tested**: that R34's `deserialize_ct` core leaks via timing.
- **Repro**: thrash run under SK2's exact methodology - 178/192 samples
  `|t| < 4.5`; the 14 excursions flip sign between same-path classes (noise,
  not leak). Codegen is branch-free below the public length gate, zero
  variable-latency ops, verdict table compiled to `csel` (Spectre premise
  structurally absent).
- **Disposition**: VERIFIED (R34). Caveat recorded: single-run cold Welch is
  drift-sensitive - future verification uses thrash/interleaved methodology.

### SX10 - Constant recovery trivially succeeds; REA = cost elevation

- **Claim falsified (partially)**: that fold-constant opacity resists an
  adversary.
- **Repro**: SMT and by-hand recovery of the fold constants both trivially
  succeed - REA resistance is cost-elevation only (delay, not impossibility),
  consistent with SK9.
- **Disposition**: DESIGN - document the property as cost-elevation, not
  confidentiality.

### SX8 - Prefix exhaustion vacuously conforming (SX8-F1)

- **Claim falsified**: that a `Complete` phase plus an `is_conforming()==true`
  report implies the trace reached the net's terminal place.
- **Repro**: `report.rs`'s `Unfinished` criterion (`steps_done < total_events`)
  is satisfied by any exhausted stream, and the session never compared the
  final marking to the net's terminal place - a lawful prefix latched
  `Complete`/conforming.
- **Disposition**: FIXED - commit 8b92c3e: `ConformanceSession` now carries
  `terminal_marking = 1 << n` computed at `from_model` time; the completion
  law is `Phase::Complete` + conforming requires
  `final_marking == terminal_marking`. Regression tests in session +
  integration + `#[should_panic]` at the `assert_conforms!` macro seam.
  Verified green: 102 lib conformance tests, 18 integration tests, 781
  all-assurance lib tests, clippy/fmt clean; audit found no existing test
  relied on prefix-completion-conforming.
