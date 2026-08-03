# DfLSS Capability Matrix — chicago-tdd-tools v26.7.9

Updated 2026-07-09. Zero-trust audit: every capability is presumed hallucinated
until proven by execution evidence gathered this session.

## Method

The defect this matrix measures is information loss between claimed capability
and actual capability. The Critical-to-Quality requirement (CTQ) is: every
claimed capability has executable proof. The evidence standard is strict —
a capability counts as real only if it was compiled and run this session
(consumer-mode crate, feature-scoped test suites, full `--all-features` lib
suite, or the targeted probe crate), or was affirmatively refuted by inspection
of source (HALLUCINATED). Anything else is UNPROVEN and is excluded from the
v26.7.9 requirement set by default.

The hallucinated/dead surface is largely deliberate explore/exploit output
(Gemini-generated exploration sweeping the design space via combinatorial
maximalism), not pure defect. The defect is not exploration garbage — it is
UNLABELED STRATA: exploration residue reporting as verified capability. This
matrix's job is stratification so exploit can select.

Session evidence base:

```text
1. Consumer-mode crate (nightly-2026-06-22): compiled + ran 3/3 —
   test!/assert_ok!, ReceiptChainBuilder + Blake3ChainValidator
   round-trip + tamper detection, proptest property.
2. Feature-scoped suite (property-testing, receipt-validation,
   ocel-generation, parameterized-testing, snapshot-testing):
   472 passed / 4 failed. All 4 failures are stale insta snapshots in
   testing::snapshot::tests (module works; its own tests are stale).
3. Full --all-features lib suite: 601 passed / 5 failed / 6 ignored
   (extra failure: environment-flaky Docker timeout test).
4. Playground crate DOES NOT COMPILE: missing [workspace] +
   TotalCount::new API drift (playground/src/validation/coverage.rs:27,39,85).
5. Probe crate (ctt-probe-results.md): 14 targeted probes, all compiled;
   9 WORKS / 5 RUNTIME-SURPRISE / 0 compile-fail.
6. Claims audit (ctt-claims-audit.md): 82 doc claims vs src — 61 exist,
   5 signature-drift, 4 stub, 2 renamed, 10 hallucinated.
7. Dark-surface audit (ctt-dark-surface.md): 437 exports, 144 dark,
   27 DEAD, 12 hidden gems, 49 INTERNAL-but-tested.
```

Chatman Constant reinterpretation (applies to all tick-budget rows): the
constant is 8 ticks of LOGIC — a deterministic operation count — not
wall-clock or CPU-cycle time, and the engine targets WASM where no timing
guarantees exist. It is a forcing function for decomposition, never a budget
to optimize under: a unit needing 9 logical steps is split-and-routed
(WarmPathRequired), never granted a raised limit. Ticks are per-atom and
never mixed or summed across units — aggregate budgets let complexity hide
inside seamless blobs, and seams are where route decisions, refusals, and
receipts attach. Any timing-based instrument claiming to enforce the constant
is therefore the wrong instrument by construction.

Verdict vocabulary: VERIFIED-WORKING, DRIFTED (exists, docs/signature/behavior
stale vs claim), STALE-TEST (works, own tests broken), STUB, DEAD,
HALLUCINATED (claimed but absent), UNPROVEN.

Disposition vocabulary: KEEP+DOCUMENT, KEEP (already sound), FIX (named fix),
DELETE (law-shaped or dead), ENGINE-OWNS (belongs to the Chatman Engine),
UNPROVEN-EXCLUDE, MINE (harvest the idea as a requirement and rebuild
clean-room; never port the explored source).

## Triage rule: mechanics vs law

CTT owns domain-agnostic test mechanics (harnesses, assertions, generators,
validators over caller-supplied bytes). The Chatman Engine owns receipt/refusal
law (what a crown receipt is, what an invariant refusal means). Any CTT surface
that encodes law — not merely validates data — is DELETE or ENGINE-OWNS even if
it compiles and its probes pass. Applied here: `assert_crown_receipt!` and
`ensure_invariant!`/`invariant_context!` are ENGINE-OWNS regardless of probe
outcome (probe 2 shows ensure_invariant works; it still moves). `cli_proof`
and `Blake3ChainValidator` are mechanics (replay/tamper checks over opaque
bytes) and are KEEP+DOCUMENT.

## The Matrix

Column key: Claimed In cites the doc file (or `dark` for undocumented
surface); CTQ Protected names the consumer defect class the capability would
catch; Evidence cites the session run or probe/audit item.

### Macros (test entry points)

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `test!` macro | API_REFERENCE.md:9 | untyped test boilerplate | consumer crate 3/3 ran | VERIFIED-WORKING | KEEP |
| `test!` "AAA enforcement" claim | API_REFERENCE.md:9, README.md:16 | none — checks nothing | claims audit: expansion wraps body, no AAA check | STUB | MINE (the claim is the spec: make the typestate API deliver enforced AAA; delete the false claim from `test!` docs) |
| `async_test!` / `async_test_with_timeout!` | API_REFERENCE.md:18-23 | hung async tests | probe 14: timeout fires at 1.00s and fails the test; within-budget passes | VERIFIED-WORKING | KEEP (document consumer needs tokio `time` feature) |
| `fixture_test!` / `fixture_test_with_timeout!` | API_REFERENCE.md:28,36 | fixture leakage | suite compile; macro variants absent from tests/ | UNPROVEN (run path) | FIX (add macro-expansion tests) |
| `performance_test!` | API_REFERENCE.md:40 | perf regression | suite compile only | UNPROVEN | UNPROVEN-EXCLUDE |
| `param_test!` | API_REFERENCE.md:44 | case-matrix gaps | probe 7: works WITH consumer-side rstest; without it, E0433 — feature alone insufficient, rstest not re-exported | DRIFTED (dependency footgun) | FIX (re-export rstest or document the required dev-dep) |
| `otel_test!` | API_REFERENCE.md:50 | untraced tests | compile only; untested variant | UNPROVEN | UNPROVEN-EXCLUDE |
| `weaver_test!` | API_REFERENCE.md:55-61 | schema-invalid telemetry | exists; expands to sync `#[test]` + block_on, not `#[tokio::test]` as documented | DRIFTED | FIX (correct expansion description) |
| `weaver_test_with_timeout!` | API_REFERENCE.md:63, TIMEOUT_ENFORCEMENT.md:26 | — | absent from src | HALLUCINATED | DELETE (doc rows) |
| `weaver_async_test!` | dark | schema-invalid telemetry | tested in repo suite, undocumented | VERIFIED-WORKING (suite) | KEEP+DOCUMENT |
| `#[tdd_test]` proc macro | API_REFERENCE.md:124 | lifecycle hooks | full suite compile+run | VERIFIED-WORKING (suite) | KEEP |
| `#[tdd_test]` "AAA validation" claim | API_REFERENCE.md:124 | none | claims audit: only OCEL hooks injected | STUB | MINE (same spec as `test!` AAA row; delete the false claim) |
| `#[fixture]` / `#[derive(TestBuilder)]` | API_REFERENCE.md:130-139 | builder boilerplate | probe 5: UserBuilder new/default/with_*/build all generated and ran | VERIFIED-WORKING | KEEP |
| `scaffold!` + `#[chicago_test]` | proc_macros docs, README | untracked pending work | probe 6: compile-gates ticket+test files, runtime panic, catch_scaffold turns pending into a PASSING test named `chicago_unknown_*` | VERIFIED-WORKING | KEEP+DOCUMENT (pending=green subtlety; ticket-ID extraction yields `unknown` for plain paths) |
| `test_builder_derive` (internal fn) | dark | — | DEAD list | DEAD | DELETE |

### Assertions

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `assert_ok!`/`assert_err!`/`assert_fail!` | API_REFERENCE.md:69-84 | silent Result misuse | consumer crate ran assert_ok! | VERIFIED-WORKING | KEEP |
| `assert_matches!` + collection asserts | cookbook quick-reference | pattern/collection drift | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `assert_eq_msg!`/`assert_eq_enhanced!`/`assert_approx_eq!` | API_REFERENCE.md:95-102 | equality noise | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `assert_in_range!` / `assert_guard_constraint!` | API_REFERENCE.md:91,104 | bound violations | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `assert_within_tick_budget!` "panics if ticks > 8" | API_REFERENCE.md:87 | hot-path budget blowout | probe 9: debug counter overhead (~350 ticks) fails ANY op; release trivial op reads 0; ticks are ~0.96ns ARM timer units, not cycles — and the Chatman Constant is logical steps, not time | DRIFTED (wrong instrument for the claimed purpose) | FIX (rebuild on the logical step counter, per-atom; see TickCounter row) |
| `assert_json_eq!` | cookbook | JSON shape drift | probe 13: order-insensitive equality ran | VERIFIED-WORKING | KEEP |
| `AssertionBuilder` / `ValidatedAssertion` | API_REFERENCE.md:229-231 | ad-hoc assertion soup | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `assert_violation_count` x2 | dark | — | DEAD list | DEAD | DELETE |

### Governance / law-shaped (triage rule applies)

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `assert_crown_receipt!` | governance docs | receipt law (Engine domain) | probe 1: runs on violating metadata WITHOUT failing the test — diagnostics queued only | DRIFTED (assert that never asserts) | ENGINE-OWNS (remove from CTT regardless) |
| `assert_admitted!` | governance docs | admission gating | probe 1: same silent-queue behavior; failed admission swallowed unless a sink inspects severity | DRIFTED | FIX (fail the test on violation, or rename to `emit_admission_diagnostic!`) |
| `ensure_invariant!` / `invariant_context!` | docs | refusal law (Engine domain) | probe 2: early-return Err and context conversion work exactly as documented | VERIFIED-WORKING | ENGINE-OWNS (law-shaped; moves despite working) |
| `assert_invariant` macro (fail_fast.rs:441) | dark | — | DEAD list | DEAD | DELETE |
| Governance diagnostic channel (register_sink, emit_diagnostic, close_channel, on_test_started/completed) | API_REFERENCE.md:567-583 + dark | lost test-run telemetry | emit path exercised by probe 1 (queue push observed); close_channel/sink path never run this session | UNPROVEN (full loop) | FIX (add channel round-trip tests) if kept — mechanics, not law |
| `register_sector_stack` | dark | — | DEAD list | DEAD | DELETE |

### Typestate / compile-time

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `TestState<Phase>` AAA typestate | API_REFERENCE.md:250-266 | out-of-order test phases | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `const_assert!` / `const_assert_msg!` | API_REFERENCE.md:269-275 | invalid constants | full suite compile (fires at compile time) | VERIFIED-WORKING | KEEP |
| `SizeValidatedArray` / `Validated<T>` | API_REFERENCE.md:278-287 | size/range bugs | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| Type-level `Add/Sub/Mul/Div`, `ValidatedSize`/`ValidatedRange` | dark | — | zero users (built and abandoned) | DEAD | DELETE |
| `ConstSizeValid`/`ConstRangeValid` traits | dark | — | DEAD list | DEAD | DELETE |
| `ValidatedTickBudget<8>` | API_REFERENCE.md:441 | budget as type | probe 9: compiles, but delegates to the same wall-time runtime check; "compile-time validation" is only a const generic; debug=Err, release=Ok for identical code | DRIFTED (wrong instrument) | FIX (retarget the const generic at the logical step count per atom; timing delegate removed) |

### Fixtures / builders

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `TestFixture` / `FixtureProvider` / `ScopedMetadata` | API_REFERENCE.md:142-163 | shared-state bleed | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `AsyncFixtureProvider` (GAT) / `AsyncFixtureManager` | API_REFERENCE.md:166-187 | async fixture races | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `TestDataBuilder` family + fake-data | API_REFERENCE.md:190-212 | hand-rolled test data | full suite pass; fake-data examples shallow | VERIFIED-WORKING (suite) | KEEP (deepen examples) |
| `FakeDataGenerator` | dark | — | compile only | UNPROVEN | UNPROVEN-EXCLUDE |
| `core::test_utils` `RetryConfig`/`TestTimer` | dark (gem #12) | flaky-test retries | used by repo tests; suite pass | VERIFIED-WORKING (suite) | KEEP+DOCUMENT |

### Generative / property / mutation

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `PropertyTestGenerator` + with_seed | API_REFERENCE.md:293-303 | unexplored input space | probe 8: equal seeds → identical 5-batch sequences; suite + consumer proptest also ran | VERIFIED-WORKING | KEEP (note: MAX_DEPTH unused by generate_test_data — FIX doc) |
| `property_test!` macro | cookbook real-world-scenarios.md:211 | — | absent from src | HALLUCINATED | DELETE (doc row) |
| `MutationTester`/`MutationOperator`/`MutationScore` | API_REFERENCE.md:307-320 | weak assertions | probe 10: apply/detect/score(100, acceptable) ran; scope is test-DATA mutation, not source mutation | VERIFIED-WORKING | KEEP (document data-mutation scope vs cargo-mutants) |
| `NegateCondition` mutation variant | README.md:420 | — | variant absent; real `StringCase` undocumented | HALLUCINATED (variant) | FIX (README variant list) |
| `TestGenerator` (string-template test gen) | API_REFERENCE.md:378-390 | — | 258-line String template; no test/example | STUB | DELETE |
| `generate_test_array<const N>` | API_REFERENCE.md:378 | array fixtures | compile only | UNPROVEN | UNPROVEN-EXCLUDE |
| `continuous_learning` (`AdaptiveTestSelector` etc.) | dark | — | untested, uncalled | DEAD | DELETE |

### Snapshot

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `SnapshotAssert` (assert_matches/debug/json) | API_REFERENCE.md:337-347 | output regression | feature suite: module works; 4 own tests fail on stale insta snapshots; probe 13 insta interplay green after snapshot acceptance | STALE-TEST | FIX (`cargo insta accept` on testing::snapshot::tests; note consumer must add insta with `json` feature) |
| `insta::cleanup_unused_snapshots!()` | application-guide snapshot-testing.md:193 | — | no such macro in insta 1.x | HALLUCINATED | DELETE (doc row) |

### Concurrency / state machines / effects

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `ConcurrencyTest::run` | src docs | interleaving bugs | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `LoomModel` API | API_REFERENCE.md:352-361 | — | renamed to ConcurrencyTest; docs describe nonexistent API | DRIFTED | FIX (rewrite doc section) |
| `ConcurrencyTest::run_with_config` | src docs | — | both config params ignored; body identical to `run` | STUB | FIX (implement or delete params) |
| Type-level `StateMachine`/`Transition` | API_REFERENCE + dark | illegal state transitions | probe 3: 3-state machine compiles+transitions; illegal transitions are compile errors | VERIFIED-WORKING | KEEP+DOCUMENT |
| `ModelChecker` / `ScheduleGenerator` | API_REFERENCE + dark (gem #8) | — | probe 3: generator returns only [empty, hardcoded Lock/Unlock] regardless of input; no API to feed your machine — vacuous "model checking" | STUB | FIX (wire generator to caller's machine) or DELETE |
| `testing::effects` (`EffectTest`/`EffectCoverage`) | dark (gem #9) | untracked side effects | probe 4: Pure/NetworkRead ran; markers are phantom-type convention only, no runtime enforcement | VERIFIED-WORKING | KEEP+DOCUMENT (state the convention-only limit) |

### CLI / cli-proof

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `CliTest` (`run_tests`, `command`) | src | CLI regressions | compile only | UNPROVEN | UNPROVEN-EXCLUDE |
| `CliTest::new().run_command().assert_output()` fluent API | API_REFERENCE.md:365-374 | — | docs describe nonexistent fluent API | DRIFTED | FIX (rewrite doc to real API) |
| `cli_proof` module (`CliHarness`, `TempWorkspace`, `ReceiptAssertions`, `SabotageFixture`) | dark (gem #1) | non-replayable receipts, silent tampering | compiles under --all-features; zero tests/examples; not probed | UNPROVEN (run) | KEEP+DOCUMENT (mechanics gem; proof obligation below) |
| `CliHarnessError` | dark | — | DEAD list | DEAD | FIX (wire into CliHarness errors or delete) |

### Performance

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `TickCounter`/`measure_ticks`/`HOT_PATH_TICK_BUDGET` as Chatman gate | API_REFERENCE.md:434-449 | hot-path complexity creep | probe 9: reads `cntvct_el0` (ARM fixed-freq wall timer, ~1.04GHz), NOT a cycle counter; "8 ticks = 2ns" comment wrong (8 ticks ≈ 7.7ns); constant is logical steps, engine targets WASM (no timing guarantees) | DRIFTED (measures time; the constant is logical steps) | FIX: replace with a deterministic logical step counter (increment per table access/mask op, asserted == expected; WASM-portable, replay-compatible, receipt material) + structural verification (single `[_;256]` indexed load on u8 = 1 logical tick, branchless by construction); per-unit assertion only, never summed across units; add a structural scan flagging hot-path fns with unbounded loops or multi-unit work, remedy "decompose and route" (WarmPathRequired), never "raise budget"; wall-clock demotes to informational criterion bench on native, never a gate |
| `TickMeasurer`/`AsyncTickMeasurer`/`BenchmarkResult` | dark (gem #10) | tick measurement | compile only | UNPROVEN | UNPROVEN-EXCLUDE (or prove with probe) |
| `validation::thermal` Warm/ColdPathConfig | dark (gem #11) | thermal misclassification | compile only | UNPROVEN | UNPROVEN-EXCLUDE |
| "Zero-cost abstractions" claim | README.md:18 | — | no benchmark tied to claim; test!/#[tdd_test] inject runtime work | STUB | FIX (qualify claim or add benchmark) |

### Receipts / OCEL

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `Blake3ChainValidator`/`ReceiptChainBuilder`/`RawReceiptEntry` | README, gem #2 | broken/tampered receipt chains | consumer crate ran round-trip + tamper; probe 11: single-bit flip → HashMismatch{index:1} | VERIFIED-WORKING | KEEP+DOCUMENT (zero examples today) |
| `TestReceipt`/`EnvironmentFingerprint`/`TimingMeasurement` | dark (gem #3) | unreproducible evidence | receipt-validation suite passed | VERIFIED-WORKING (suite) | KEEP+DOCUMENT |
| `OcelCollector` + `seal_run` digest | API_REFERENCE.md:551-565, README.md:641 | lost event lineage; unstable receipts | probe 12: 6 identical runs produced TWO distinct digests — hasher iterates unordered HashMap (wasm4pm.rs:71-81); also `run_id` param unused, README `new()` snippet doesn't compile | DRIFTED (nondeterministic "seal") | FIX (sort events before hashing; use or remove run_id; fix README snippet) |
| `project_admission_events`/`graduate_for_discovery`/`ProcessModelStore` | API_REFERENCE.md:558-563 | undiscoverable process models | compile only | UNPROVEN | UNPROVEN-EXCLUDE |

### Observability (OTel / Weaver / alerts)

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `SpanValidator` | API_REFERENCE.md:504-520 | wrong span attributes | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| `OtelTestValidator` + assert_span_exists etc. | cookbook observability-instrumentation.md:49-92 | — | no such type/methods anywhere | HALLUCINATED | MINE (fictional API is the ergonomic target for span validation — rebuild clean-room on SpanValidator; delete the doc rows) |
| `WeaverTestFixture::collect_spans()` | cookbook:54-64 | — | method absent; real API is new/tracer/finish | HALLUCINATED (method) | FIX (doc to real API) |
| `WeaverValidator` lifecycle | API_REFERENCE.md:524-543 | invalid semconv registries | compile only; weaver runs env-dependent | UNPROVEN | UNPROVEN-EXCLUDE |
| Weaver env vars `WEAVER_STARTUP_TIMEOUT`, `WEAVER_REGISTRY_VERSION`, `WEAVER_SKIP_REGISTRY_VALIDATION`, const `DEFAULT_WEAVER_STARTUP_TIMEOUT_SECONDS` | PERFECT_WEAVER_LIVE_CHECK.md, REGISTRY_VERSION_PINNING.md | — | absent from code; only `WEAVER_ALLOW_SKIP` exists | HALLUCINATED (4 items) | MINE (harvest as configurability requirements — startup timeout, registry pinning, validation skip; delete the doc rows) |
| Alert macros + `AlertLogger` | API_REFERENCE.md:106-121 | swallowed failures | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| OTel/testcontainers poka-yoke types (`ValidAttribute*`, `ValidContainerConfig`) | dark | — | DEAD list | DEAD | DELETE |
| `LiveCheckAdvice`/`LiveCheckStatistics` | dark | — | DEAD list | DEAD | DELETE |
| `weaver_*_wait_milliseconds` config fns | dark | — | DEAD list | DEAD | DELETE |

### Validation / verification pipeline / swarm

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `CoverageReport`/`JtbdValidator`/`GuardValidator` | API_REFERENCE.md:418-478 | untested modules shipping | full suite pass; but playground consumer broken on `TotalCount::new` drift (coverage.rs:27,39,85) | DRIFTED (consumer API) | FIX (playground repair in progress this session; stabilize TotalCount API) |
| `assert_guard_run_len`/`assert_guard_batch_size` | dark | — | dead per dark-surface | DEAD | DELETE |
| `VerificationPipeline` + `DeploymentDecision`/`PipelineMetrics`/`PipelineResult` | dark (gem #4) | shipping unverified changes | pipeline used by examples; output types unconsumed anywhere | UNPROVEN (outputs) | FIX (document consuming outputs) or UNPROVEN-EXCLUDE |
| `StrictExecutionContext`/`PhaseLabel` (fail_fast) | dark (gem #5) | late failure detection | top-level example exists; per-phase API dark | UNPROVEN (phase API) | KEEP+DOCUMENT (phase API) with proof obligation |
| `TestOrchestrator`/`TestPlan`/`QoSClass`/`ResourceBudget` | dark (gem #6) | unbudgeted test runs | tested in repo suite (pass); result types unsurfaced | VERIFIED-WORKING (suite) | KEEP+DOCUMENT |
| `Wave` + `WaveReceipt`/`ResidualClass` | API_REFERENCE.md:585-597 | unstructured multi-phase runs | full suite pass | VERIFIED-WORKING (suite) | KEEP |
| Teamwork "Integrity Modes: Permissive/Strict/Cryptographic-Audit" | README.md:684-689 | — | no such enum/mode anywhere | HALLUCINATED | MINE (Permissive/Strict/Cryptographic-Audit is a real requirement — rebuild clean-room; delete the doc rows) |
| Teamwork orchestration (prompt_draft.md, .agents/, heartbeats, handoff) | README.md:678-700 | — | only a self-contained mock-file simulation test | STUB | FIX (relabel as preview/simulation) or DELETE claim |
| Operator registry (43 YAWL patterns + query API) | API_REFERENCE.md:600-615 | workflow-pattern gaps | full suite pass (registry tests) | VERIFIED-WORKING (suite) | KEEP |

### Integration (testcontainers)

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `ContainerClient`/`GenericContainer`/exec/wait_for_ready | API_REFERENCE.md:482-500 | broken integration envs | full suite: 1 Docker-timeout failure, environment-flaky, not code-refuted | STALE-TEST (env-flaky) | FIX (skip/ignore when Docker daemon absent) |
| `config/loading.rs` accessors (INTERNAL, summarized) | dark | config typos | inline `#[cfg(test)]` pass under suite | VERIFIED-WORKING (suite) | KEEP (internal by design) |

### MCP / A2A

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| `McpAgentBridge` / `A2aTaskHarness` | README.md:704-707 | agent-protocol drift | exists, compiles; no run evidence this session | UNPROVEN | UNPROVEN-EXCLUDE |
| 8 A2A DEAD items (`assert_task_state`, `assert_task_failed`, `DataPart`, `FilePart`, `JsonRpcResponse`, `TaskError`, `TaskGetParams`, `TaskCancelParams`, `RecordedRequest`) | dark | — | DEAD list (caveat: possible serde-only construction) | DEAD | DELETE (after serde-construction check) |

### Dark-surface summary rows (INTERNAL plumbing)

| Capability | Claimed In | CTQ Protected | Evidence | Verdict | Disposition |
|---|---|---|---|---|---|
| 49 INTERNAL dark items (poka-yoke newtypes, config accessors, A2A wire types) | dark | internal correctness | inline tests pass under full suite | VERIFIED-WORKING (suite) | KEEP (unadvertised by design) |
| Remaining ~90 dark exports not enumerated above | dark | — | compile-only under --all-features | UNPROVEN | UNPROVEN-EXCLUDE |

## Sigma summary

Row basis: 94 capability rows (the weaver env-var line counts as 4 items; the
27 DEAD exports collapse into 13 rows; 49 INTERNAL items are one summary row).
Rows cover all doc-claimed capabilities from the claims audit, all 12 hidden
gems, and all 27 DEAD exports; residual dark plumbing is summarized.

Verdict counts:

| Verdict | Count |
|---|---|
| VERIFIED-WORKING | 34 |
| DRIFTED | 11 |
| STALE-TEST | 2 |
| STUB | 7 |
| DEAD | 13 rows (27 exports) |
| HALLUCINATED | 11 items (8 rows) |
| UNPROVEN | 16 |
| Total | 94 |

Disposition counts: KEEP 25, KEEP+DOCUMENT 10, FIX 21, DELETE 19 rows,
MINE 5 (distinct from DELETE: the artifact goes, the idea becomes a
requirement), ENGINE-OWNS 2, UNPROVEN-EXCLUDE 12.

Defect rate (non-VERIFIED-WORKING / total rows): 60/94 ≈ 64%. Restricted to
doc-claimed surfaces only (excluding dark/DEAD/internal rows): roughly half of
claimed capabilities lack clean execution proof — 10+ items are outright
hallucinated in docs, and 5 probe surprises (silent governance asserts,
vacuous ModelChecker, param_test dependency footgun, tick semantics,
nondeterministic OCEL seal) pass compilation while violating their own claims.

Features with ZERO example/playground coverage (dark-surface report):
`cli-proof`, `receipt-validation`, `workflow-engine`, `benchmarking`,
`ocel-generation`, `ocel-generation-discovery`, `git-hooks`,
`governance-tests`.

## v26.7.9 requirement extract

### Requirement set (KEEP / KEEP+DOCUMENT / FIX)

Proven core (KEEP): `test!`, async/timeout macros, `param_test!` (after dep
fix), `#[tdd_test]`, `#[fixture]`/`TestBuilder`, `scaffold!`/`#[chicago_test]`,
the assertion family, `TestState` typestate, const asserts, fixtures
(sync+async GAT), builders, `PropertyTestGenerator`, `MutationTester`,
`ConcurrencyTest::run`, type-level `StateMachine`, `SpanValidator`, alerts,
operator registry, `Wave`, `TestOrchestrator`, receipts
(`Blake3ChainValidator`, `TestReceipt`).

Surface-and-document (KEEP+DOCUMENT): `cli_proof`, `Blake3ChainValidator`
examples, `TestReceipt`/`EnvironmentFingerprint`, `weaver_async_test!`,
`testing::effects`, type-level state machine, `RetryConfig`/`TestTimer`,
fail_fast phase API, `TestOrchestrator` result types, scaffold pending=green
semantics.

Named fixes (FIX):

1. Sort OCEL events before hashing in `seal_run` (wasm4pm.rs:71-81) —
   nondeterministic seal digest is the worst live defect found.
2. Make `assert_admitted!` fail the test on violation (or rename it).
3. Re-export or document the `rstest` dev-dep for `param_test!`.
4. Rebuild the Chatman gate as a deterministic logical step counter,
   asserted per-atom (never summed across units); add the structural scan
   (unbounded loops / multi-unit work in hot paths → "decompose and route",
   WarmPathRequired); demote wall-clock ticks to a native-only informational
   bench, never a gate.
5. Refresh 4 stale insta snapshots in `testing::snapshot::tests`.
6. Repair playground (`[workspace]` + `TotalCount::new` drift) — in progress.
7. Doc rewrites: LoomModel→ConcurrencyTest, CliTest real API, weaver_test
   expansion, MutationOperator variant list, README OcelCollector snippet,
   delete AAA-enforcement claims, qualify zero-cost claim.
8. Gate the Docker testcontainers test on daemon availability.
9. Implement or drop `ConcurrencyTest::run_with_config` params; wire
   `ScheduleGenerator` to caller machines or delete `ModelChecker`.

### DELETE list

All 27 DEAD exports (dark-surface list), `TestGenerator` string-template stub,
`continuous_learning`, type-level arithmetic traits, plus remaining
HALLUCINATED doc rows (`weaver_test_with_timeout!`, `property_test!`,
`NegateCondition`, `collect_spans`, `insta::cleanup_unused_snapshots!`).
ENGINE-OWNS moves: `assert_crown_receipt!`, `ensure_invariant!`/
`invariant_context!` — law, not mechanics, per the triage rule.

### MINE list (harvest as requirements; rebuild clean-room, never port)

1. `OtelTestValidator` fictional API — the ergonomic target for span
   validation ergonomics, built on the real `SpanValidator`.
2. Weaver env vars (startup timeout, registry version pinning, validation
   skip) — configurability requirements for the weaver harness.
3. Teamwork Integrity Modes (Permissive/Strict/Cryptographic-Audit) — a real
   requirement for swarm-run integrity levels.
4. AAA enforcement (`test!` + `#[tdd_test]` claims) — the claim is the spec
   the typestate API should be made to deliver.

### Proof obligations for UNPROVEN rows someone wants to keep

- `cli_proof`: one example + one test per assertion (`assert_chain_linked`,
  sabotage bit_flip round-trip) under `--features cli-proof`.
- Governance channel: register_sink → emit → close_channel round-trip test
  asserting `RunSummary` contents.
- `fixture_test!`/`performance_test!`/`otel_test!` macro variants: one
  expansion-and-run test each.
- `VerificationPipeline` outputs: an example consuming `DeploymentDecision`.
- `TickMeasurer`/thermal configs, `WeaverValidator`, OCEL discovery,
  `McpAgentBridge`/`A2aTaskHarness`, `CliTest`, `FakeDataGenerator`,
  `generate_test_array`: compiled-and-run probe each, or stay excluded.

## See Also

- `docs/reference/API_REFERENCE.md` — primary claimed surface
- `ctt-claims-audit.md`, `ctt-dark-surface.md`, `ctt-probe-results.md`
  (session scratchpad) — evidence sources for this matrix
- `docs/features/NOMRG_DFLSS_CHARTER.md` — DfLSS charter context
