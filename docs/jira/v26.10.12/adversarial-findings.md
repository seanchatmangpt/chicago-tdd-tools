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
| SK2 | FALSIFIED | `wire_ctrt` is constant-*flow* (data-independent control), not constant-*time*; observable |t| spans 10-3842 flow steps. | CLOSED - relabel landed (Diataxis reference pages now say constant-flow; `docs/features/CONFORMANCE.md` carries no CT claim) |
| SK3 | FALSIFIED | Raw-context collision: two raw contexts can collide on the admission predicate. The keys.rs raw `from_seed` surface was reworked in 1fc1c72; only `Signer::from_seed` remains publicly reachable (`src/ledger/signing/signer.rs:141`), bypassing key-id admission. | CLOSED (W26, e54b891: from_seed seed-screening sliver landed; keystore-sealing half remains open — see line-146 summary + closing register) |
| SK4 | FALSIFIED | Payload covert channel: payload bytes influence flow structure observable to a stream reader; BEAM and Rust runtimes diverge on the same fixture. | QUEUED |
| SK5 | FALSIFIED | Key rotation replay amnesty: `KeyStore::rotate` (`src/ledger/signing/keys.rs:264`) admits pre-rotation receipts (replay amnesty), and seed persistence is fail-open on write failure. | PARTIAL |
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
- **Disposition**: CLOSED. The relabel landed: Diataxis reference/attestation
  pages now say constant-flow, and `docs/features/CONFORMANCE.md` carries no
  constant-time claim.

### SK3 - Raw-context collision + public `from_seed`

- **Claim falsified**: that key construction is reachable only through the
  admitted keystore path.
- **Repro**: the raw `SigningKey::from_seed` surface cited in the original
  finding (`src/ledger/signing/keys.rs:158`) was reworked in 1fc1c72; the
  only remaining public raw entry point is `Signer::from_seed`
  (`src/ledger/signing/signer.rs:141`). Two raw contexts can still be
  constructed that collide on the admission predicate.
- **Disposition**: QUEUED (re-scoped) - seal `Signer::from_seed`
  (signer.rs:141) behind the keystore / make pub(crate); keys.rs surface
  already closed by 1fc1c72.

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
- **Repro**: `KeyStore::rotate` (`src/ledger/signing/keys.rs:264`); a receipt
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
- **Disposition**: PARTIAL - out-pointer re-verification landed `532f38a`;
  module-hash binding landed as the 32B digest slot in the v2 seal preimage
  `2c80a9e` (placeholder digest until the build-time splice, W-ABI2-2).

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

Queued: SK3 (CLOSED for the seed-screening sliver per W26; the keystore-sealing half of the original order remains open — see row), SK4, SK10, NATO-ENTROPY-SEAM-01 (entropy.rs replica-seam work order — self-attested in-source). Other statuses: SK5: PARTIAL (persistence half landed 1fc1c72; rotation key-epoch binding queued). SK6/SX6: PARTIAL (see rows). SK2: CLOSED (relabel landed).

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
| SX6 | FALSIFIED | Module-identity-blind seal preimage CONFIRMED (same finding as SK6 at the seal layer); out-pointers trusted on read without re-verification; trap indistinguishability. | PARTIAL (V5 envelope landed 2c80a9e — 32B module digest slot + M2 falsifier; placeholder until build-time splice; out-pointer bounds validation landed 532f38a) |
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
- **Disposition**: FIXED (R37, commit `1a16e25` in `/Users/sac/affidavit`) -
  forward-port degenerate-seed screening to both PQC `from_seed` paths;
  dual-verify claim to be reworked (feature-gated
  runtime verify-vs-verify over KAT vectors) or retracted in docs.

### SX6 - Module-identity-blind seal preimage; out-pointer trust

- **Claim falsified**: that a seal binds the module that produced it.
- **Repro**: seal preimage contains no module identity (SK6's finding
  confirmed at the seal layer); out-pointers are dereferenced and trusted on
  read without re-verification; trap bytes are indistinguishable from live
  data.
- **Disposition**: PARTIAL - V5 envelope landed `2c80a9e` (32B module-digest
  slot + M2 falsifier; placeholder until build-time splice); out-pointer
  re-verify landed `532f38a`;

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

### W13 REGISTER ADDITIONS (audit wave-14)

- **core::receipt TestReceipt** (PARTIAL-QUEUED — UPDATE: CLOSED per W26, e54b891 full-field signing; see closing register): signature covers
  identity fields only (receipt_id/contract_name/timestamp/result) —
  code_hash/invariants_checked/effects_exercised/environment/timing are
  NOT tamper-covered; signature is unkeyed SHA-256 (recomputable).
  Keyed verification exists only on `governance::TaskReceipt`. Docs
  now disclose; hardening (full-field signature or keyed verify) queued.
- **cli_proof TempWorkspace** (COVERAGE-QUEUED): drop-cleanup is
  tempfile-RAII-delegated and unwitnessed by any test.
- **N04 AUDITED_MODULES scope** (DISCLOSED W13; count refreshed W24):
  15/26 nato modules
  scanned; timing/env-gated courts exempt by design (disclosed
  in zero_ambient.rs); extending to eligible pure modules queued.
- **weaver_test_with_timeout! hallucinated cites** (FIXED W14): all
  live-doc cites purged (playground invocation included — W15).
- **core::type_level markers** (DOCS-GRADE, W15 audit): ValidatedSize/
  ValidatedRange/ConstSizeValid/ConstRangeValid have zero production
  consumers (only prelude-surface test + doctests); docs honestly say
  "documentation only, no actual validation" — ERRC candidate or
  explicit by-design note, queued.
- **core::receipt partial-signature** (SEE ABOVE) and **cli_proof
  TempWorkspace drop-cleanup** (unwitnessed, tempfile-RAII-delegated).
- **invariant_properties coverage** (W15 audit): 21 proptest properties
  cover ~8 of 12 phases; phases 1/4/7/10 have unit tests only.
- **nfr1_latency_slo_test** (strata, W14): timing-sensitive flake
  witnessed once on cold fail-fast run, passed on retries — same class
  as fr7_constant_time.

## W24 REGISTER ADDITIONS (audit wave-24)

- **core::verification_pipeline String error channel** (QUEUED):
  all `execute_*` fns in `src/core/verification_pipeline.rs` return
  `String` errors; refactor to the typed
  `UnrecoverableInvariantViolation` vocabulary used by the rest of the
  invariant surface. Phase values are output labels only (straight-line
  sequencing, no state machine) — the refactor is vocabulary-only, no
  control-flow change. Source: wave-24 audit.

## W26 REGISTER ADDITIONS (wave-25/26 audit sweep)

- QUEUED (latent, dead code): testcontainers wait.rs/exec.rs stub paths are
  unreachable under the parent `#[cfg(feature = "testcontainers")]` gate at
  src/integration/mod.rs:17 — and wait.rs's stub imports the gated
  `implementation::{ContainerClient, GenericContainer}` path, so it would NOT
  compile if the gate were lifted; stub signature `_wait_for: ()` also
  diverges from the real `WaitFor` parameter. Source: audit W26.
- QUEUED (doc-truth/law): thermal.rs hot tier gates wall-clock ticks against
  τ ≤ 8 while validation/performance.rs's own header rules the Chatman
  Constant a logical step count and prohibits timer-as-τ gates; thermal's only
  enforced check is the prohibited pattern (its own success test runs at
  50k ticks). Needs either a doc-scoped honest disclaimer or a re-scoped
  gate law. Source: audit W25.
- CLOSED (W26): TestReceipt partial-signature (UPDATE: CLOSED per W26 — e54b891 full-field signing) — sign/verify now cover all
  receipt fields; metadata-mutation court added (commit e54b891).
- CLOSED (W26): SK3 signer sliver — Signer::from_seed screens all-zero/
  all-ones seeds typed (commit e54b891).
- CLOSED (W26): master_trace.ocel.json placeholder atoms + stale intake_pacs008
  replaced with real phase-model atoms; anti-placeholder guard court added
  (merger_conformance_tests 26/0).
- CLOSED (W26): weaver_macro_tests wired into test-integration;
  teamwork_preview_integration.rs deleted (tautological self-test); 3 orphan
  scripts deleted; mcp stale pre-rename snapshot regenerated.

## W27 REGISTER ADDITIONS (wave-27)
- CLOSED (W27): net.rs packed pre/post window (16 places) now refused typed
  at compile — was a debug panic / release silent-wrap (d4ee667).
- CLOSED (W27): docs/README 6 mis-statuses; API_REFERENCE missing
  cli_proof/sector_stacks sections + 3 submodule entries (d4ee667).
- CLOSED (W27): strata-side Erlang N08/N12/N14/N15 claim corrected
  (19de2bc). CHANGELOG/RELEASE_NOTES/API_REFERENCE "diverges from the
  declared model" discovery overclaim still open (audit W27).
- CLOSED (W27): docs/OCEL.md ".ocel.jsonl export" claim removed — no such
  exporter exists (XML exporter lives in strata_protocol).
- NOTE (W27): commit d4ee667 additionally removed the tracked
  cookbook/book/ mdbook build artifacts (80 files, regenerable via
  `just cookbook-build`) — disclosed here; not in that commit's message.
- QUEUED (operator): docs/latex/chicago-tdd-tools-formalization.tex
  \includes 4 nonexistent chapters + a nonexistent appendix/ dir — the
  paper does not compile; regenerate-paper.yml is an admitted stub.
- PARTIAL (updated by W28 closing probe): the teamwork_preview_integration.rs
  clause is CLOSED — the test was deleted in 2c679bb and tests/README.md no
  longer lists it. STILL OPEN: ARCHITECTURE.md module tree omits ~15 modules
  (core/testing/validation/observability/swarm submodules). CLOSED: the
  find_best_member clause — the union-semantics doc correction landed in
  e5b3b88 (src/swarm/coordinator.rs:158) and a further probe found the doc
  makes no sector claim to overstate.

## W27 CONVERGENCE ADDITIONS
- QUEUED: HasEffect trait unsealed (src/testing/effects.rs — in-source audit
  note present; sealing or guarantee-rewording is the work order).
- QUEUED: invariant_properties "derived from the 47 failure modes" claim
  sources to HISTORICAL-bannered INVARIANT_MATRIX.md; no per-property
  traceability link.
- CLOSED: probe-B doc fixes (generator ident caveat, telemetry flush-on-drop)
  landed after being found missing from d03b4f7's diff.

## W27-CLOSING (wave-27 closing repairs)
- CLOSED: coverage.rs `add_item` duplicate-name drift — disclosed in-source
  (afb14e2); the behavior itself is kept and documented, not changed.
- CLOSED: continuous_learning modules-field semantics — AUDIT NOTE landed
  (5652fbb); a keyspace rename is the follow-up if ever productized.
- CLOSED: observability/receipt.rs tamper-law doc corrected (afb14e2 —
  third attempt, verified on disk).
- CLOSED: `#[fixture]` generated-path defect — the generated doctest path
  referenced the uncompilable `chicago_tdd_tools::fixture::` module; fixed
  to `core::fixture` (afb14e2).
- CLOSED: swarm_coordination.rs cryptographic-receipts residual.
- QUEUED (W27-CLOSING, registered late): validate.rs JSON-loader re-check -
  from_powl_json does NOT resolve activity references (runtime-verified: an
  undeclared activity in JSON loads and passes validate); the fail-closed
  layering holds only for the text entry point. In-source note at
  src/conformance/validate.rs:10-13 + walk body. Work order: re-check in
  validate() or resolve in the JSON loader.
- QUEUED (operator): README quick-start example missing_docs deny
  interaction — the rewritten assertion compiles, but a fresh-copy user may
  still hit `missing_docs` on the test file (probe finding; needs either a
  doc comment in the example or a documented allow).

## W28–W32 CLOSING REGISTER (2026-10-12, verification waves + same-tick repairs)

- W28: closing sweep at 049aa30; repairs 89fc3b4/25ff66f/1d2102f. QUICK_GUIDE
  note landed after being claimed-twice-missing; hamming 100%-admit claim
  falsified (measured ~0.13, subset condition); generator.rs false const
  allow removed; lint red (doc_markdown) fixed.
- W29: the W28 QUICK_GUIDE note itself falsified twice (StateMachine DOES
  exist at src/testing/state_machine.rs:82, un-prelude-exported; HTML
  comment invisible in mdbook) — rewritten as visible blockquote (96ea408).
  Extension-layer `?`-in-`-> Self` invalid-Rust snippets fixed (006e820).
  SK3 row annotation + TestReceipt cross-ref actually landed (006e820;
  first attempt was a string-replace-miss). Stale :396 row split (27c84f0).
- W30: 19-lane audit wave. 16 Quick Glance banners stripped; nato README
  matrix gained N08/N12/N14/N15 rows (16→20); phantom workflow-engine flag,
  stale 1.2.0/v1.3+ versions, README SLA 5s→30s, CODE_REVIEW court-offline/
  honesty-probe phantoms, DOG_FOODING nightly-clippy/audit-cron claims all
  corrected; tests/README inventory regenerated (2 phantoms out, 12 in);
  PERFECT_WEAVER timeout constant marked aspirational; USER_GUIDE impossible
  private::Sealed instruction replaced (7ccc021).
- W31: ARCHITECTURE.md submodule trees added (40 second-level omissions
  closed); master_trace.ocel.json committed-status disclosed; 12 generated
  .ggen/.ggen-v2 state files untracked + .ggen-v2/ ignored (no key material
  was ever tracked); docs/jira/** added to package exclude (774e628).
- W32: import_json.rs wire-format doc truth corrected (E2 correction that
  had never landed: published OcelLog is builder-based NON-serde; serde
  types are in-crate); import_xml.rs standard-surface line added (86aba86).
- CLOSED this window: guest_abi quota-0 RED (0ba7dae, test aligned to RS1
  law); prelude 29-symbol assurance re-exports verified LANDED; find_best_member
  doc clause (e5b3b88); teamwork README clause (2c679bb); ARCHITECTURE tree.
- STILL QUEUED (operator/next-session): nightly AFFIDAVIT_DIR env on
  assurance.yml:220; merger-guests 6 dead errno consts (0..13 range kept
  verbatim per do-not-renumber law — by-design, disclosed); nextest override
  terminate-after=2 on star_toml_e2e (only non-1 value); testcontainers
  dead-stub gate; thermal τ-law re-scope; W24 typed-channel refactor;
  validate.rs JSON re-check; crates.io publish order (proc_macros first);
  strata temprun_step ABI re-base; worker.ex backpressure/env allowlist.

## W34 REGISTER (2026-10-12, 20-lane finish wave)

- LANDED (fa6a633): validate.rs JSON activity-resolution law (W27 re-check
  order CLOSED); W24 typed PipelineError channel; N04 extension
  (AUDITED_MODULES 15→20 + 5 justified allowlist entries, exact pin);
  nightly AFFIDAVIT_DIR checkout + step env (pin = origin/main b5112f32 —
  the first pin was a local-only SHA and would have failed the runner);
  latex UNWRITTEN-STUB disclosures; tests/temp_workspace_tests.rs (9/0);
  nextest star_toml_e2e terminate-after=1; thermal τ budget disclosures.
- REPAIRED in-wave: QUICK_GUIDE note falsified a THIRD time ("NOT
  prelude-exported" was false — prelude glob src/lib.rs:383); 2 merger
  goldens gained is_conforming() asserts; stale merger header prose;
  tests/README inventory + temp_workspace_tests.rs.
- NEW QUEUED (guest ABI, W-ABI2 gaps): `guest-abi-v1` shim feature is
  doc-promised (src/conformance/guest.rs:540) but absent from both
  Cargo.tomls — no v1 gating exists; `CTT_MODULE_DIGEST` static is
  doc-promised (guest.rs:114) but defined nowhere in the workspace
  (module-digest build-time splicing never landed). Both are doc-vs-code
  gaps: the docs claim the v2 seal binds module identity; today the seal
  takes a module_digest ARGUMENT but nothing populates it from the
  artifact bytes. Priority with V5 (WASM-digest envelope binding).
- NOTED (commit-message accuracy): 27c84f0 claimed the prelude re-export
  closure landed in that commit; the register entry actually landed in
  W30/W33. Historical message-only overclaim, recorded here.
- Observed tag counts after W34: CLOSED 25 / QUEUED 24 / PARTIAL 14 /
  FALSIFIED 20 (substring counts).

## W36–W37 REGISTER (2026-10-12, 20-lane wave 2: W-ABI2 substrate + skeptic repairs)

- LANDED (70a42ef, plus 4c04eeb docs): guest-abi-v1 shim feature (ABI probe
  1/2 cfg-selected; 34-byte legacy seal-preimage body under v1, digest
  ignored + disclosed; errno remap NOT covered — scoped ~55-70 lines via
  boundary mapping, follow-up); CTT_MODULE_DIGEST 32-byte zero static
  (seal passes it by name; all-zero = "not yet attested", disclosed).
  W-ABI2 items 2-substrate and 5-shim are closed; item 2's build-time
  splice pipeline (extend guest_audit walk_sections; blake3 in lock;
  length-preserving in-place patch; splice==recompute verify) is DESIGNED
  and queued.
- MERGER REPAIRS: the W34 commit message claimed the merger golden
  is_conforming() repairs; they had NOT landed (script abort before those
  edits — the string-replace-miss/message-overclaim class again). Now
  actually landed: phase4/phase6 goldens + phase8 violation-skeleton
  Complete arm assert conformance; header names the composed bridge.
- QUICK_GUIDE mdbook claim corrected: 96ea408's "made mdbook-visible" was
  overstated — QUICK_GUIDE.md is not in either mdbook's SUMMARY; the note
  is accurate on disk and renders nowhere (book-exclusion is the residual,
  disclosed here rather than re-adding the file to a book).
- CI BLOCKER (operator): the nightly AFFIDAVIT_DIR fix is structurally
  correct, but origin/main of affidavit (b5112f32, the pin) predates the
  affi dossier verbs (local eeda22b, 22 commits unpushed) — nightly
  emit-dossier will fail at CLI dispatch until the affidavit main is
  pushed. Disclosed at assurance.yml:229.
- Registered follow-ups: guest errno remap (v1 collapse 14/15/16→11);
  ctt-splice-digest build pipeline; QUICK_GUIDE book placement decision.

## DoD CERTIFICATION REGISTER — SYSTEM SPECIFICATION v26.10.12 (2026-10-12, W39–W44)

Per-item verdicts from the 18-lane DoD certification wave (all repaired
same-tick where repairable; cert basis = HEAD d329535):

GREEN (certified): compile-time wire composition pin 352=320+32
  (receipt.rs:104) + torn/slot size_of/align_of proofs; zero-#\[feature\]
  stable const-generic surface + fresh /tmp consumer witness; just check +
  lint 0 warnings; MemBoundGuard capture/restore-on-drop w/ mutex (+
  EnvVarGuard/CliEnvironment corroborators); full suites 1150/0 lib,
  146/0 doctests, 1313-class workspace counts; merger 123+17+3 /0;
  20/20 COURT_REGISTRY sweep + falsifier no-persist; emitter bitwise
  parity receipted (5189603/e465c1a) + non-clobber falsifier; Merkle
  determinism 38/0 incl. zeroed-genesis + cross-court refusal; typed
  error vocabularies (ConformanceError 18, LedgerError 15 + exit 0/10-19,
  SealError x2, PipelineError 4, guest errno 0-16) with typed refusal
  spot-checks; anti-cheat GREEN (ggen standards exit 0, zero mocks,
  tautology kills receipted); Andon framing landed in CI; version
  lockstep GREEN 26.10.12; packaging excludes GREEN; bench/examples
  compile exit 0; tick budgets (hot 8, warm 500k≈125μs ≤ 1ms).
CLOSED THIS WINDOW (was gaps): standing 10k CTRT wire-mutation loop
  (wire_fuzz_10k..., W44); guest-abi-v1 errno remap (abi_map boundary);
  ctt-splice-digest tool + workspace wiring (splice==recompute,
  self-locating verify); CI docs-check backstop (D2) + BRE sync (D1);
  nato README pillar column (9 rows → in-code enum); CONFORMANCE.md
  4-step classification law + UnknownActivity{0,0} + ArityBelowMinimum
  fields; diataxis probe/Sealed/exit-class refresh (5 pages);
  gitignore/tracked-artifact rationalization (coverage.lcov,
  weaver-reports untracked; root lock tracked, child locks ignored);
  guest README probe qualification; CI pin blocker disclosure.
HOLDS-AS-DISCLOSED: constant-time — ct_eq fold on magic+digest is real;
  wire decode is constant-FLOW only (SK2 relabel); LEDGER.md:85 is a
  per-compare claim, never an end-to-end CT claim; N05 dudect |t|<4.5
  gates the kernel. DoD §2.4 certified with this scoping, not beyond it.
GAPS (disclosed, not certifiable at HEAD):
- DoD §2.3: ~30 public fns still return Result<_, String> (dossier_wire
  x7, swarm, builders, receipt json, governance, otel/weaver, testing,
  reproducible, loader) — typed-channel migration is a follow-up wave.
- DoD §3.4: anti-replay nullifier windows RED-NOT-LANDED (epoch-scoped
  keyed-BLAKE3 set + LedgerError::ReplayDetected = spec item V3,
  sequenced after RL2; closest analog = stored_hash replay refusal).
- DoD §4.3 (DESCOPED per lane design, variance REPORTED not GATED):
  criterion reports per-bench std-dev/CI (madr) in target/criterion; the
  ≤2.5% ceiling is a release-runbook comparison against
  benches/receipts/receipt_chain-v26.9.26.json on a quiet machine. The
  only automated bound is the deliberate ~100x-headroom tripwire
  (<2s debug, tests/dependency_lock_guard.rs:391) — a tick-variance gate
  would be the nfr1 flake class (aarch64 ticks ARE wall-clock,
  performance.rs:16-19). No standing variance court by design.
OPERATOR-GATED (cannot certify from this repo): final dossier re-seal
  (current NATO_STANAG_4107_DOSSIER.json sealed 06:44, predates HEAD —
  re-run just emit-dossier after affidavit main push); v26.10.12 seal
  tag (existing tag points at 4d451c7, NOT HEAD); crates.io publish
  (proc_macros first); affidavit main push (unblocks nightly CI).

## W46 REGISTER (2026-10-12, waves-38-45 window + typed-error wave)

- LANDED: CHANGELOG waves-38-45 addendum (with skeptic corrections: the W43
  message's ArityBelowMinimum-claim was pre-existing work — recorded here as
  the fourth instance of the claimed-vs-landed class; the d016764 gitignore
  rationalization was self-contradictory until W44); QUICK_GUIDE placed in
  the Application Guide (reference/quick-guide.md copy + SUMMARY entry,
  canonical source noted — drift check is a registered follow-up);
  cross-crate ABI pin cfg-mirrored (the one REAL DoD-2a defect; the other 11
  "failures" were by-design fixture-self-test oracles, disclosed at
  crates/chicago-claims/Cargo.toml:27-36); spec-harness/playground pins
  bumped to 26.10.12; guest README Build section gains the splice steps;
  CI guest job gains guest-splice + guest-verify before guest-audit (the
  uploaded artifact is now attested, and the audit inspects the spliced
  artifact); Cargo.toml guest-abi-v1 comment corrected (remap IS covered —
  the "no errno remap" text contradicted landed code).
- TYPED-ERROR WAVE (DoD §2.3): swarm done (SwarmError, 43/0);
  dossier_wire done (DossierWireError 14 variants, 9/0 + 14/0);
  receipt/builders lane in flight. Remaining Result<_,String> surface
  (governance, otel/weaver, testing, reproducible, loader) = next wave.
- Application-guide tracked book/ artifact deleted (29.5k lines of build
  output out of the repo).

- OPERATOR-QUEUE CONSOLIDATION (HEAD c44b7fb+): two queue items were
  tracked outside this register and are folded in here.
  (1) CTT main push — this repo is main...origin/main [ahead 220+];
      nothing downstream (spliced guests, pins, nightly AFFIDAVIT_DIR
      pin verification) sees any of it until pushed.
  (2) Strata push — fix/kg-42-status-code (incl. 59ed80c temprun_step
      ABI-v2 rebase) is rebase-landed but pending merge/push to strata
      main (docs/releases/CHANGELOG.md:46); the earlier "strata
      temprun_step ABI re-base" queue entry (W28-W32) is superseded by
      this named-branch item.
  Ordering constraint: CTT push + affidavit push precede dossier
  re-seal; re-seal precedes the v26.10.12 re-tag; crates.io publish
  (proc_macros first) follows the re-tag.
- TYPED-ERROR WAVE COMPLETION ADDENDUM: all lanes green — swarm 43/0,
  dossier_wire 9/0 + dossier_cli 14/0, receipt 93/0, builders 46/0,
  governance 11/0, weaver 29/0, otel 41/0. Crate-root re-exports added
  for BuilderError/ReceiptJsonError/DossierWireError (ledger-gated)/
  SwarmError. Remaining: DiagnosticSink::emit/close (blocks on a
  coordinated ocel-collector splice — registered).
- DoD §2.3 — FULLY CLOSED (0c2375d, W49f/g): zero `Result<_, String>`
  error slots in src/ production code at ALL visibilities (pub fn, trait
  method, type alias, closure bound). Final 3: builders ValidationFn
  alias + with_validation bound (BuilderError), effects
  RequiresEffect::execute family (BuilderError::BuildFailure), weaver
  download_weaver_runtime (WeaverError::Spawn, 10 sites). Prelude
  DossierWireError gate corrected ledger→nato (the ledger-only gate was
  an E0432 — `cargo check --features ledger` and bare
  `cargo test -p ctt-wasm-guest` were broken by it); prelude surface
  test extended with size_of nameability assertions for the four typed
  channels (62/0). GETTING_STARTED Transition snippet moved to the
  typed channel (W49e). The only remaining src/ grep hits are 3 historical doc comments
  (governance/otel/weaver error types).
