# Architecture

## Overview

Chicago TDD Tools: Generic testing framework base layer. Extensible for domain-specific needs. Reusable across projects. Maintains consistency, avoids duplication.

> See the [Pattern Cookbook](../../cookbook/src/README.md) for Alexander-style documentation of the architectural and design patterns summarized here.

## Architecture

**Base Layer** (chicago-tdd-tools): Generic components (Fixtures, Async Fixtures, Builders, Assertions, Macros, State, Const Assert, Type Level, Alert, Property, Mutation, Snapshot, Concurrency, CLI, Generator, Coverage, Guards, JTBD, Performance, Testcontainers, OTEL/Weaver).

**Extension Layer** (domain-specific): Extends base components (Workflow Fixture, Workflow Builder, Domain-Specific).

**Relationship**: Extensions use base layer. Base layer has no domain dependencies.

## Design Principles

**Generic Base**: No domain dependencies. Extensible. Composable. Enables reuse across projects.

**Composition Over Duplication**: Extend TestFixture. Use or extend TestDataBuilder. Compose generic components. Avoids duplication, maintains flexibility.

**Single Source of Truth**: Generic components in one place. Domain projects import, don't duplicate. Ensures consistency, reduces maintenance.

**Zero-Cost Abstractions**: Macros expand efficiently. Compile-time validation. Efficient data structures. Performance critical, no overhead.

## Module Organization

Modules are organized into capability groups for better discoverability and maintainability:

**Core Testing Infrastructure** (`core/`): Foundational primitives that all tests use.
- `fixture`: Test fixtures (GATs, RAII, automatic cleanup)
- `async_fixture`: Async fixture providers (async traits, Rust 1.75+, requires `async` feature)
- `builders`: Fluent builders for test data (JSON/HashMap, GenericTestDataBuilder, ValidatedTestDataBuilder)
- `assertions`: Assertion helpers (Result, predicate, range, AssertionBuilder, ValidatedAssertion)
- `macros`: Test macros (AAA pattern, async, fixture, performance, parameterized, OTEL, Weaver)
- `state`: Type-level AAA enforcement (sealed traits, zero-sized types)
- `const_assert`: Compile-time assertions
- `type_level`: Type-level arithmetic and compile-time validation (const generics, type-level arithmetic)
- `alert`: Alert helpers for visual problem indicators (with optional `log` crate integration, requires `logging` feature)
- `governance`: Agent governance loop — diagnostic/severity types, global diagnostic channel, admission laws, sector stacks (`Diagnostic`, `DiagnosticSink`, `SectorStack`, `MergeStrategy`)
- `poka_yoke`: Poka-Yoke error-prevention patterns — type-level error prevention making invalid states unrepresentable (compile-time prevention; not glob re-exported to avoid conflicts with feature-gated poka_yoke modules)
- `config`: Configuration loading and type-safe configuration types (`loading`, `poka_yoke`)
- `contract`: Test contracts as first-class types — declared covered modules/capabilities/invariants, resource envelope (time/memory/network), required environment, Q constraints (τ ≤ 8, no panics), and test ontology Σₜ queries without running tests
- `invariants`: Fail-fast hardening core type system — 52 unrecoverable invariant violation types across the 12 framework phases plus per-phase validators
- `fail_fast`: Strict verification pipeline with zero tolerance — orchestrates all 12 phases with fail-fast semantics; every invariant violation is immediate test failure
- `invariant_properties`: Property-based invariant validation — proptest properties exhaustively verifying that invariant violations are always detected (derived from the 47 failure modes in INVARIANT_MATRIX.md)
- `receipt`: Proof-carrying test receipts — cryptographically verifiable receipts (contract ID, code hash, environment fingerprint, invariants checked, timing windows, SHA-256 signature) making tests observable operations
- `verification_pipeline`: Phase 7 unified verification pipeline — straight-line Contract → Thermal → Effects → State Machine → Receipt → Orchestration sequencing for the A = μ(O) transformation (typed `PipelineError` channel — the wave-24 String channel was replaced in fa6a633)
- `test_utils`: Community-requested test utilities — retry configuration for flaky tests, timing and path helpers

**Advanced Testing Techniques** (`testing/`): Specialized testing methodologies.
- `property`: Property-based testing (const generics, reproducible, requires `property-testing` feature)
- `mutation`: Mutation testing (quality validation, operators, scores, requires `mutation-testing` feature)
- `snapshot`: Snapshot testing (requires `snapshot-testing` feature)
- `concurrency`: Concurrency testing (requires `concurrency-testing` feature)
- `cli`: CLI testing (requires `cli-testing` feature)
- `generator`: Test code generation
- `effects`: Effect-typed tests — type-level effect system declaring/constraining test capabilities (network I/O, storage I/O, privileged ops, pure), enabling static pure-test verification and CI routing by effect
- `state_machine`: Type-directed state machine testing — phantom-typed states, transition traits preventing illegal transitions, automatic schedule generation for concurrent actors (loom integration lives in `concurrency`)
- `continuous_learning`: Phase 8 continuous learning — ML-inspired test optimization/prediction from historical receipts: failure-likelihood prediction, execution-order optimization, τ-pattern learning, preventive-test suggestions

**Quality & Validation** (`validation/`): Quality assurance and constraint validation.
- `coverage`: Test coverage analysis (tracking, reports, CoveragePercentage, CoveredCount, TotalCount)
- `guards`: Guard constraints (input validation, Chatman module tree, batch size limits)
  - validated: typestate guard module (src/validation/guards/validated.rs)
- `jtbd`: Jobs To Be Done validation (scenario validation, real-world testing, ScenarioIndex)
- `performance`: Performance validation (RDTSC, tick measurement, hot path budget, ValidatedTickBudget)
- `advanced_phases`: Advanced phases 9–12 — distributed consensus, time-travel debugging, performance prophet, quality metrics dashboard
- `thermal`: Thermal test harness — τ-aware hot/warm/cold test classification binding the suite to the Chatman Constant τ ≤ 8 and μ-kernel timing discipline

**Telemetry & Observability** (`observability/`): Telemetry validation.
- `otel`: OTEL validation (span/metric validation, schema conformance, requires `otel` feature)
- `weaver`: Weaver live validation (live validation, registry, OTLP, requires `weaver` feature, automatically enables `otel`)
- `ocel`: OCEL 2.0 process mining (Object-Centric Event Logs from test runs, `Raw → Admitted → Receipted` evidence lifecycle on `wasm4pm-compat`, requires `ocel-generation`; process discovery via `ocel-generation-discovery`)
- `unified`: Unified observability testing API — single ergonomic API over OTEL + Weaver testing with RAII lifecycle management, auto-detection of Weaver binary/registry, and invalid-states-unrepresentable typing (re-exported as the main observability API)
- `receipt`: BLAKE3 receipt chain validation (requires `receipt-validation`) — generic replay and tamper-evidence utilities for any project claiming BLAKE3 receipt chaining
  - ocel: collector, discovery, idmap_bridge, import_json, import_xml, ocel64_bridge, powl_fixture_bridge, projections, types, wasm4pm
  - fixtures: assertions, telemetry_capture, validation_results, weaver_fixture
- `fixtures`: Weaver fixture and telemetry capture utilities (requires `weaver` + `otel`) — Chicago-aligned fixtures, telemetry capture helpers, and assertion adapters wiring live-check results into the assertion pipeline

**Integration Testing** (`integration/`): External system integration.
- `testcontainers`: Docker support (port mapping, exec, auto-cleanup, requires `testcontainers` feature)

**Orchestration** (`swarm/`): Distributed multi-sector coordination.
- `wave`: N-phase sequential waves with M parallel tasks, wave-state observability, `ResidualClass` failure classification
- `composition`: Knowledge hook composition — ordered operation chains across sectors (`OperationChain` with order-sorted steps and cursor; bookkeeping only, no step execution or data piping)
- `coordinator`: Swarm coordinator — central orchestration of members, task distribution, and deterministic swarm consensus
- `member`: Swarm member — individual agent record with capabilities (sectors), lifecycle state, task capacity, heartbeat; passive, all coordination in `SwarmCoordinator`
- `task`: Task receipt system — proof-of-work receipts for swarm agents (what was done, by whom, when, result), `TaskQueue`/`TaskRequest`/`TaskReceipt`
- `test_orchestrator`: Swarm-native test orchestrator — agent-driven test scheduling at scale: tenant/org/priority/QoS scheduling, hardware-aware μ-kernel placement, minimal sufficient test-set suggestions

**Workflow Patterns** (crate root):
- `operator_registry`: All 43 YAWL workflow control patterns (`OperatorRegistry`, `GuardType`, `global_registry()`)

**Court & Evidence** (top-level, feature-gated):
- `nato`: NATO court framework — pillar-classified adversarial courts with mandatory positive controls, receipted outcomes, and scoping disclosure (requires `nato` feature)
  - Submodules: bell_lapadula, cache, disasm, dossier_court, entropy, guest_audit, hamming, kat, liveness, mils, orthogonality, rng, seu, stackbound, thrash, torn, transition_tables, wcet, zero_ambient
- `ledger`: P15 ledger domain surface for court-receipt sealing and verification (requires `ledger` or `nato` feature)
  - Submodules: dossier_wire, proof, receipt, dossier, signing (keys/sealer/signer/subject)
- `conformance`: POWL conformance subsystem — DSL loading, model validation, PN64 replay, structured verdict reporting, and reachability state-space analysis (requires `conformance` feature)
  - Submodules: bcinr, guest_seal, guest_telemetry, guest_yield, loader, net, report, session

**Proof & Domains** (top-level):
- `cli_proof`: CLI proof testing — real boundary-crossing harness primitives (requires `cli-proof` feature)
- `sector_stacks`: Sector reference stacks — demo-grade Chatman Equation worked examples per sector (synthetic data)
- `bin/`: Standalone binaries (weaver_smoke, otel_production_run, otel_negative_run, ctdd_combinatorial_plan — the retired git_hook_installer/check_dog_fooding/verify_lint_config stubs were removed in the wave-8 ERRC pass; their duties live in scripts/)

**Companion crates**: `crates/chicago-tdd-lsp/` (editor guard, lsp-max, emits
`CTDD-DEV-001`; excluded from the workspace build) and the assurance workspace
members — `crates/ctt-wasm-guest/` (zero-import pn64 guest kernel), 
`crates/merger-guests/` (8 zero-dep financial leaf kernels), 
`crates/chicago-tdd-mcp/` + `crates/chicago-claims/` (internal tools,
`publish = false`). The assurance chain is conformance (pn64/POWL replay) →
ledger (CTRT wire + ML-DSA-65 sealing) → nato (20-court matrix, N01–N20, 
sealed STANAG-4107 dossier at `docs/nato/NATO_STANAG_4107_DOSSIER.json`).

**Backward Compatibility**: All modules are re-exported at the crate root. Existing code using `chicago_tdd_tools::fixture::*` continues to work. New code is encouraged to use capability group paths: `chicago_tdd_tools::core::fixture::*`

## Module Dependencies

Most modules have no dependencies (zero-cost). Optional features are feature-gated. Internal types avoid external dependencies.

**Dependency Graph**: lib.rs → core (fixture, async_fixture, builders, assertions, macros, state, const_assert, type_level, alert, config, contract, receipt, invariants, fail_fast, verification_pipeline), testing (property, mutation, snapshot, concurrency, cli, generator, effects, state_machine, continuous_learning), validation (coverage, guards, jtbd, performance, thermal, advanced_phases), observability (otel, weaver, unified, fixtures, receipt), integration (testcontainers), swarm (wave, composition, coordinator, member, task, test_orchestrator). Most modules have no dependencies (zero-cost). Optional features are feature-gated. Internal types avoid external dependencies.

**Assurance feature edges** (per `Cargo.toml` feature declarations):
- `conformance`: zero-dependency feature (`conformance = []`) — POWL/PN64 kernel
- `ledger` → `dep:blake3` + `receipt-validation` (which itself pulls `dep:blake3`) — CTRT wire + sealing
- `nato` → `ledger` + `conformance` + `dep:ml-dsa` (+ `zeroize`, `getrandom`) — 20-court matrix with ML-DSA-65 dossier sealing
- `ocel-generation` → `dep:wasm4pm-compat` + `dep:blake3` (+ `dashmap`, `tracing`); `ocel-generation-discovery` → `ocel-generation` + `wasm4pm-compat/wasm4pm`

## Feature Flags

**default**: Core framework with `logging` feature enabled. **Core features**: `mutation-testing`, `async`, `benchmarking`. **Testing features**: `property-testing`, `snapshot-testing`, `fake-data`, `concurrency-testing`, `parameterized-testing`, `cli-testing`. **Observability features**: `otel`, `weaver` (requires otel). **Integration features**: `testcontainers`. **Feature groups**: `testing-extras` (property-testing, snapshot-testing, fake-data), `testing-full` (all testing features), `observability-full` (otel, weaver), `integration-full` (testcontainers, weaver).

**Rationale**: Users include only what they need. Reduces compile time, binary size.

## Extension Patterns

**Extend TestFixture**: Compose base fixture with domain fields. Delegate common operations. Pattern: `WorkflowTestFixture { base: TestFixture<()>, engine: WorkflowEngine }`.

**Use TestDataBuilder**: Use directly or extend with domain helpers. Pattern: `TestDataBuilder::new().with_var().build_json()` or extend with `with_workflow_data()`.

**Compose Components**: Use fixture + builder + assertions together. Pattern: `fixture_test!` with `TestDataBuilder` and `assert_ok!`.

**Extend Async Fixtures**: Implement `AsyncFixtureProvider` trait with sealed trait pattern. Use `AsyncFixtureManager` for lifecycle management.

## Type Safety

**GATs**: Flexible fixture creation with type-safe lifetimes. Pattern: `trait FixtureProvider { type Fixture<'a>; }`.

**Async Traits**: Native async trait support (Rust 1.75+). Pattern: `trait AsyncFixtureProvider { async fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error>; }`. Use `AsyncFixtureManager` for lifecycle management. Requires `async` feature and Rust 1.75+. Sealed trait pattern prevents external implementations. GATs provide type-safe lifetime management. Runtime lifecycle (not compile-time). Use for async resource creation (database connections, network resources).

**Const Generics**: Compile-time configuration, zero runtime cost. Pattern: `PropertyTestGenerator<const MAX_ITEMS: usize>`.

**Type State**: Compile-time AAA enforcement. Pattern: `TestState<Phase>` with `PhantomData<Phase>`. Prevents wrong method order.

**Sealed Traits**: API safety and extensibility control. Pattern: `mod private { pub trait Sealed {} }` prevents external implementations. Used in `AsyncFixtureProvider` to prevent external implementations.

**Type-Level Arithmetic**: Runtime size/range validation with const generics. Pattern: `SizeValidatedArray<const SIZE: usize, const MAX_SIZE: usize>` (runtime validation, future: compile-time with Rust 1.79+). Marker types: `ValidatedSize`, `ValidatedRange` (documentation only, no validation).

**HRTB**: Flexible predicates with any lifetime. Pattern: `F: for<'a> Fn(&'a T) -> bool`.

## Macros

**Test Macros**: `test!`, `async_test!`, `async_test_with_timeout!`, `fixture_test!`, `fixture_test_with_timeout!`, `performance_test!`, `param_test!` (requires `parameterized-testing`), `otel_test!` (requires `otel`), `weaver_test!` (requires `weaver`; skip-gated, no timeout variant).

**Assertion Macros**: `assert_ok!`, `assert_err!`, `assert_within_tick_budget!`, `assert_in_range!`, `assert_eq_msg!`, `assert_guard_constraint!`.

**Alert Macros**: `alert_critical!`, `alert_warning!`, `alert_info!`, `alert_success!`, `alert_debug!`, `alert!`.

**Procedural Macros**: `#[tdd_test]`, `#[fixture]`, `#[derive(TestBuilder)]`.

**Timeout Management**: Async macros (`async_test!`, `fixture_test!`, `weaver_test!`) use 1s timeout by default. Use `*_with_timeout!` variants for custom timeouts (e.g., 30s for integration tests). Synchronous macros (`test!`, `otel_test!`) rely on cargo-nextest profile timeouts.

## Error Handling

All fallible operations return `Result<T, E>`. No `unwrap()` in production. Use `thiserror` for error types. Errors include context.

**Pattern**: `pub enum FixtureError { CreationFailed(String), OperationFailed(String) }`.

**Error Types**: `FixtureError`, `FixtureResult<T>`, `TestcontainersError`, `OtelValidationError`, `WeaverValidationError`, `WeaverValidationResult<T>`, `GovernanceError` (governance), `WeaverError` (weaver typed channel, distinct from `WeaverValidationError`), `OtelError`, `SwarmError`, `StateMachineError`, `BuilderError`, `ReceiptJsonError`, `DossierWireError` (ledger-gated), `PipelineError`, `GuardConstraintError`, `PerformanceValidationError`, `JtbdValidationResult`.

## Performance

**Zero-Cost**: Macros expand efficiently. Const generics = compile-time. Type state = zero-sized types.

**Efficient**: Atomic counters for fixtures (~1-2ns). Builders use HashMap, convert to JSON when needed. LCG for property testing.

**Hot Path**: RDTSC on x86_64. Tick budget enforced at compile time. Minimal allocations, reuse structures.

**Timeout Enforcement**: Multiple layers (test-level timeouts via `tokio::time::timeout`, cargo-nextest profile timeouts, process-level timeouts via Makefile.toml). Defense in depth ensures tests don't hang.

## Security

**No Unsafe**: Core framework has no unsafe (except RDTSC on x86_64, platform-specific). All operations memory-safe. Proper error handling prevents panics.

**Input Validation**: Public APIs validate inputs. Guard constraints prevent invalid data. Error messages don't leak sensitive info.

**Resource Cleanup**: RAII patterns. Drop traits handle cleanup. No leaks in error paths.

## Chicago TDD Alignment

State-based testing (verify outputs, not implementation). Real collaborators (actual dependencies, e.g., Docker). Behavior verification (verify what code does, not how). AAA pattern (enforced by macros and type state).

## Procedural Macros

**`#[tdd_test]`**: Zero-boilerplate tests with AAA validation. Works with both sync and async functions. **`#[fixture]`**: Automatic fixture setup/teardown. Automatically creates `fixture` variable. **`#[derive(TestBuilder)]`**: Derive macro for fluent builders. Generates `{StructName}Builder` with `with_*` methods.

**Rationale**: Compile-time validation, reduce boilerplate.

## Thread Safety

Fixtures: Atomic counters, thread-safe. Builders: Not thread-safe (single-threaded tests). Property generators: Not thread-safe (single-threaded). Performance counters: Thread-safe (atomic operations).

**Note**: Test execution typically single-threaded, so thread safety less critical than production.

## Platform Support

x86_64: Full support including RDTSC. ARM/Other: RDTSC falls back to `std::time::Instant`. All platforms: Core functionality works.

## Summary

Generic, extensible testing framework base. Chicago TDD principles. Type-safe with zero-cost abstractions. Extensible for domain needs. Performance-optimized. Security-conscious.

**Key Associations**: Generic = Reusable = Extensible. Type-safe = Zero-cost = Performance. Composition = Flexibility = Consistency. Base layer = 80% value = Domain extensions = 20% effort.

**Design Philosophy**: 80% value with 20% complexity. Users extend for specific needs.

## See Also

- **[API Reference](API_REFERENCE.md)** - Complete API documentation
- **[SLA Reference](SLA_REFERENCE.md)** - Service level agreements and quality standards
- **[Getting Started](../getting-started/GETTING_STARTED.md)** - Quick start guide
- **[User Guide](../getting-started/USER_GUIDE.md)** - Comprehensive usage guide
- **[Pattern Cookbook](../../cookbook/src/README.md)** - Alexander-style pattern language
