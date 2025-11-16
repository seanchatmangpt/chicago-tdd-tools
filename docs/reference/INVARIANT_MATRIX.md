# Invariant Failure Matrix: Chicago TDD Tools Fail-Fast Hardening

**Version:** 1.0.0 | **Date:** 2025-11-16 | **Scope:** Phases 1-12

## Philosophy

**Zero-tolerance policy:** Every internal invariant violation results in immediate test failure. No degradation, no warnings that are ignored, no "partial success." The framework recognizes exactly two states: `PASS` or `FAIL`.

---

## Matrix Format

| Phase | Failure Mode | Cause | Reaction | Exit Result |
|-------|-------------|-------|----------|------------|
| **Phase** | Specific invariant violation | What triggers it | Immediate action taken | User-visible outcome |

---

## Phase 1: Contract Definition

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 1.1 | **ContractMalformed** | Missing required fields at construction | Return `Err(InvariantViolation::Contract(...))` from smart constructor; contract not entered pipeline | ❌ FAIL |
| 1.2 | **DuplicateContractID** | Two contracts with identical IDs | Detect at registration; emit invariant violation | ❌ FAIL |
| 1.3 | **InvalidPhaseSequence** | Phases defined out of order or skipped | Validation check during parse; must be monotonic | ❌ FAIL |
| 1.4 | **ContractEscapedValidation** | A contract reached Phase 2+ without passing constructor | Runtime check at phase entry; abort with invariant violation | ❌ FAIL |
| 1.5 | **MalformedMetadata** | Contract metadata corrupted (NaN bounds, negative timeouts) | Reject at construction; return error | ❌ FAIL |

**Property Tests Required:**
- Generate random contract specs; verify all invalid combinations fail construction
- Generate valid contracts; verify all metadata invariants hold
- Verify ID uniqueness across generated contracts

---

## Phase 2: Thermal Testing (τ Measurement)

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 2.1 | **ClockBackward** | Measured τ decreases between samples | Emit `InvariantViolation::Thermal`; abort measurement | ❌ FAIL |
| 2.2 | **ClockMonsterJump** | τ increases by > sanity threshold (e.g., > 1e9 ticks) | Emit invariant violation; mark measurement invalid | ❌ FAIL |
| 2.3 | **ThermalBoundExceeded** | τ > `MAX_TICKS` configured | **Test-level SLA failure** (not invariant; expected for slow tests); record in receipt | ❌ FAIL (user-level) |
| 2.4 | **CannotMeasureThermal** | Measurement infrastructure broken (RDTSC unavailable, etc.) | Emit `InvariantViolation::Thermal`; no "degraded mode" | ❌ FAIL |
| 2.5 | **ThermalComputationCorrupted** | Arithmetic overflow/underflow during τ calculation | Return error; do not use computed value | ❌ FAIL |

**Property Tests Required:**
- Generate monotonically increasing τ sequences; verify computation is stable
- Generate adversarial sequences (backwards, jumps, NaNs); verify all trigger violations
- Verify τ bounds checking is consistent for all phase transitions

---

## Phase 3: Effects Tracking

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 3.1 | **UnobservedEffect** | Runtime observed an effect not declared in contract | Return `InvariantViolation::Effects`; fail test | ❌ FAIL |
| 3.2 | **EffectLost** | Declared effect not present in runtime observations (incomplete instrumentation) | If impossible to measure: invariant violation; if optional, still fail | ❌ FAIL |
| 3.3 | **EffectCompositionError** | Join/union/restrict operations fail or produce invalid sets | Return error; abort effect aggregation | ❌ FAIL |
| 3.4 | **ImplicitOtherEffect** | Attempt to use open "Other" variant in closed effect model | Compile-time error (sealed trait); or runtime rejection | ❌ FAIL |
| 3.5 | **EffectAlgebraViolation** | Operation violates expected laws (associativity, idempotence, etc.) | Return error immediately; emit violation | ❌ FAIL |

**Property Tests Required:**
- Generate random valid effect sets and compositions; verify laws hold
- Verify closed-world assumption: any "other" effect type is rejected
- Check composition operations commute/associate where specified

---

## Phase 4: State Machine Transitions

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 4.1 | **UnhandledStateEvent** | `(state, event)` pair has no defined transition | Compile-time exhaustiveness check; or runtime rejection with violation | ❌ FAIL |
| 4.2 | **DeadState** | State unreachable from initial state | Detect at harness construction via reachability analysis; fail build | ❌ FAIL |
| 4.3 | **InvalidStateTransition** | Transition violates contract invariants (e.g., skips required phase) | Guard on transition; return error; do not advance state | ❌ FAIL |
| 4.4 | **StateCorruption** | State field mutated outside transition function | Use sealed types/private constructors; panic if detected | ❌ FAIL |
| 4.5 | **CyclicPathViolation** | Transition creates invalid cycle (e.g., return to initial mid-pipeline) | Topology check; reject transition with violation | ❌ FAIL |

**Property Tests Required:**
- Generate random transitions; verify they reach all reachable states
- Verify exhaustiveness: no unhandled `(state, event)` pairs
- Detect cycles and verify no illegal cycles are traversed

---

## Phase 5: Receipt Generation & Validation

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 5.1 | **MissingReceipt** | Test executed; no receipt produced | Framework error; emit `InvariantViolation::Receipt` | ❌ FAIL |
| 5.2 | **CorruptedReceipt** | Receipt checksum/hash does not match content | Reject receipt on read; abort; emit violation | ❌ FAIL |
| 5.3 | **PartialReceipt** | Receipt missing required fields (e.g., end timestamp) | Constructor rejects; return error; do not create partial | ❌ FAIL |
| 5.4 | **ReceiptVersionMismatch** | Receipt schema version unsupported | Reject on parse; emit violation | ❌ FAIL |
| 5.5 | **ReceiptPersistenceFailure** | Cannot write receipt to storage | Abort test; emit violation (missing observability is critical) | ❌ FAIL |

**Property Tests Required:**
- Generate random receipts; verify checksums always validate
- Verify all partial constructions are rejected at compile-time
- Test serialization/deserialization round-trip integrity

---

## Phase 6: Swarm Orchestration

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 6.1 | **AbandonedTest** | Test scheduled but never executed or timed out (still pending) | Detect at run end; mark swarm as failed | ❌ FAIL |
| 6.2 | **OrchestratorTimeoutViolation** | Test exceeds configured timeout | Record as test failure; increment failure count; continue | ❌ FAIL |
| 6.3 | **FairnessViolation** | Some tests starved of execution time | Detect via fairness monitor; emit warning → failure | ❌ FAIL |
| 6.4 | **DuplicateExecution** | Same contract ID executed twice in same run | Detect; abort; emit invariant violation | ❌ FAIL |
| 6.5 | **OrchestrationQueueCorruption** | Queue structure corrupted (circular refs, lost items) | Consistency check; if violated, abort swarm | ❌ FAIL |

**Property Tests Required:**
- Generate random test schedules; verify all execute or timeout, none abandoned
- Verify fairness for mixed short/long tests
- Check for duplicate IDs in generated schedules

---

## Phase 7: Verification Pipeline

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 7.1 | **PartialPipelineSuccess** | Pipeline returns `OK` but not all configured phases ran | Should be impossible if earlier phases enforce; if detected → failure | ❌ FAIL |
| 7.2 | **PipelineConfigInvalid** | Attempted to use "relaxed" or unvalidated config | Removed from API; compile-time error if attempted | ❌ FAIL (compile-time) |
| 7.3 | **PipelinePhaseSkipped** | Contract phase 5 defined but pipeline skips it | Pipeline entry validation rejects; emit invariant violation | ❌ FAIL |
| 7.4 | **PipelineInternalStateCorruption** | Pipeline state machine reached invalid intermediate state | Sealed API prevents this; if detected, panic in debug | ❌ FAIL |
| 7.5 | **PipelineResultAmbiguous** | Multiple conflicting results (e.g., phase A passed, phase A failed) | Use Result-based composition; impossible by construction | ❌ FAIL |

**Property Tests Required:**
- Generate contracts with various phase combinations; run through pipeline
- Verify pipeline result is deterministic and consistent
- Verify no phase can be silently skipped

---

## Phase 8: Continuous Learning

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 8.1 | **InsufficientObservations** | Learner trained on < N samples; predictions unreliable | Disable learner predictions; return advisory-only state | ⚠️ DEGRADED → FAIL (no advisory override allowed) |
| 8.2 | **LearnerMathCorrupted** | Failure probability = NaN/Inf; confidence interval impossible | Mark learner state as corrupted; emit `InvariantViolation::Learning` | ❌ FAIL |
| 8.3 | **PredictionDivergesFromMeasurement** | Learner predicted pass; actual result is fail (or vice versa) | Record as misprediction; learner loses confidence; not a failure | ✓ OK (tracked, not fatal) |
| 8.4 | **LearnerLockout** | Learner has corrupted state; cannot be recovered | Mark learner offline; emit invariant violation for that contract | ❌ FAIL |
| 8.5 | **BadPredictionOrdering** | Learner-guided order violates phase dependencies | Reorder detection; emit invariant violation; use canonical order | ❌ FAIL |

**Property Tests Required:**
- Feed arbitrary receipt sequences; verify probabilities ∈ [0, 1]
- Verify learner gracefully disables on corruption
- Simulate mispredictions; verify learning mechanism adapts

---

## Phase 9: Distributed Consensus (2/3 BFT)

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 9.1 | **InvalidConsensusVote** | Vote signature fails verification | Abort voting round; emit `InvariantViolation::Consensus` | ❌ FAIL |
| 9.2 | **VoteIdentityCorruption** | Vote claims invalid voter identity | Reject vote; do not count toward quorum | ❌ FAIL |
| 9.3 | **InconclusiveConsensus** | Quorum not reached (< 2/3 approvals, but not 2/3 rejections) | Return `Indeterminate`; treat as **test failure** at system boundary | ❌ FAIL |
| 9.4 | **ConsensusDeadlock** | Voting round cannot progress (circular dependencies) | Timeout; return `Indeterminate`; fail test | ❌ FAIL |
| 9.5 | **UnauthorizedVoter** | Node not registered in validator set attempts to vote | Reject vote immediately; emit invariant violation | ❌ FAIL |

**Property Tests Required:**
- Generate random vote patterns for small clusters; verify consensus rules
- Verify only `Approved` results allow deploy; others fail
- Simulate Byzantine failures; verify 2/3 threshold holds

---

## Phase 10: Time-Travel Debugging (Snapshots & Replay)

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 10.1 | **SnapshotSchemaVersionMismatch** | Snapshot from version 1.0; replay engine is 1.1 | Reject replay; emit `InvariantViolation::Snapshot` | ❌ FAIL |
| 10.2 | **ReplayDiverges** | Replayed execution produces different state/result than original | Mark divergence; fail test with explicit "replay failed" message | ❌ FAIL |
| 10.3 | **SnapshotLost** | Configured to snapshot; snapshot write failed | Abort; emit `InvariantViolation::Snapshot` | ❌ FAIL |
| 10.4 | **SnapshotCorrupted** | Snapshot data corrupted (on disk or in memory) | Detect on read via checksum; reject; emit violation | ❌ FAIL |
| 10.5 | **NonDeterministicReplay** | Same snapshot, same inputs, different outputs twice | Indicates non-determinism in test; fail with invariant violation | ❌ FAIL |

**Property Tests Required:**
- Generate snapshots; verify round-trip serialization
- Replay snapshots; verify deterministic execution
- Corrupt snapshots; verify detection via checksums

---

## Phase 11: Performance Prophet

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 11.1 | **ProphetSelfCheckFailed** | Predicted τ is negative; confidence interval invalid | Mark prophet corrupted; emit `InvariantViolation::Prophet` | ❌ FAIL |
| 11.2 | **PredictionOverridesActual** | Attempted to use prediction to override measured τ for SLA | Architectural rule: actual τ always decides; reject override | ❌ FAIL (prevents this) |
| 11.3 | **PredictionMathOverflow** | Confidence calculation causes numeric overflow | Detect; return error; mark prophet offline for that contract | ❌ FAIL |
| 11.4 | **ProphetTrainingDataInvalid** | Historical receipt data used for training contains NaN/Inf | Reject training data; purge corrupted records; emit violation | ❌ FAIL |
| 11.5 | **ConflictingPredictions** | Two prophet instances give conflicting predictions for same contract | Both marked as unreliable; predictions disabled | ⚠️ Proceed without prophet (not fatal) |

**Property Tests Required:**
- Feed synthetic historical data; verify prophet trains stably
- Verify no negative/Inf predictions possible
- Simulate training data corruption; verify detection

---

## Phase 12: Quality Dashboard

| # | Failure Mode | Cause | Reaction | Exit |
|---|---|---|---|---|
| 12.1 | **DashboardInconsistency** | Sum of category counts ≠ total; percentages don't add to 100% | Pre-render validation fails; suppress report; emit violation | ❌ FAIL |
| 12.2 | **ApproximateMetrics** | Metrics computed via heuristic/sampling instead of precise receipt data | Reject; require exact computation or no report | ❌ FAIL |
| 12.3 | **DashboardRenderFailure** | Cannot serialize dashboard to JSON/HTML | Fail render; emit violation | ❌ FAIL |
| 12.4 | **MissingReceiptData** | Receipt referenced in dashboard does not exist | Consistency check fails; abort dashboard; emit violation | ❌ FAIL |
| 12.5 | **CorruptedAggregations** | Aggregated metrics have invalid values (negative counts, etc.) | Detect; reject aggregation; fail dashboard | ❌ FAIL |

**Property Tests Required:**
- Generate random receipt sets; compute dashboard; verify invariants
- Corrupt receipts; verify dashboard detection and failure
- Verify metrics always sum/percentage correctly

---

## Summary Table: Failure Mode Categories

| Category | Count | Examples | Reaction |
|----------|-------|----------|----------|
| **Construction/Validation Failures** | 5 | Contract malformed, invalid metadata, bad phase sequence | Always `FAIL` at construction |
| **Measurement/Observable Failures** | 8 | Clock anomalies, unobserved effects, missing receipts | Always `FAIL` (cannot degrade) |
| **State/Transition Failures** | 8 | Dead states, unhandled events, state corruption | Always `FAIL` (enforce invariants) |
| **Coordination Failures** | 7 | Abandoned tests, consensus invalid, vote corruption | Always `FAIL` (cannot degrade) |
| **Observability Failures** | 7 | Snapshot loss, replay divergence, checksum mismatch | Always `FAIL` (no blind spots) |
| **Math/Computation Failures** | 6 | Overflow, NaN/Inf, invalid bounds | Always `FAIL` (no silent errors) |
| **Consistency Failures** | 6 | Dashboard inconsistency, duplicate execution, circular refs | Always `FAIL` (enforce correctness) |

**Total:** 47 distinct failure modes across 12 phases. **Exit rule:** All result in test failure (exit code ≠ 0). No warnings are ignored. No graceful degradation in core.

---

## Implementation Checklist

- [ ] Implement `UnrecoverableInvariantViolation` enum with all 47 failure modes mapped
- [ ] Add `InvariantCheck` trait for phase invariants
- [ ] Create proptest property suites for each phase
- [ ] Update test macros to enforce fail-fast on invariant violations
- [ ] Add invariant assertions at all phase boundaries
- [ ] Implement receipt checksums and self-validation
- [ ] Create snapshot versioning and verification layer
- [ ] Add consensus vote verification (signatures, identity)
- [ ] Implement dashboard consistency validator
- [ ] Add integration tests verifying fail-fast for each mode
