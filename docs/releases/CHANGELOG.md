# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.0] - 2025-01-16

### Added - Production-Grade Verification Infrastructure

This release extends the μ-kernel verification substrate with production-grade sector implementations, fail-fast hardening, and complete verification pipeline phases. The framework now supports real-world workflows with deterministic guarantees and cryptographic proofs.

#### Fail-Fast Hardening Infrastructure (`core::fail_fast`)
- **StrictExecutionContext**: Zero-tolerance execution context with 12-phase verification pipeline
- **47 Invariant Violations**: Comprehensive invariant checking across all phases
- **PhaseResult**: Unified result type for phase execution (Ok or Violation)
- **PhaseLabel**: 12 distinct phases from Contract Definition to Quality Dashboard
- **Receipt Validation**: Self-validating receipts with version and checksum
- Fail-fast semantics: any invariant violation causes immediate test failure
- No degradation, no warnings ignored, no partial success

#### Advanced Verification Phases 7-12
- **Phase 7: Verification Pipeline** - Verify all configured phases executed
- **Phase 8: Continuous Learning** - Validate learner state consistency
- **Phase 9: Distributed Consensus** - Verify 2/3 Byzantine quorum
- **Phase 10: Time-Travel Debugging** - Validate snapshot integrity
- **Phase 11: Performance Prophet** - Verify prediction self-checks
- **Phase 12: Quality Dashboard** - Verify dashboard consistency
- Complete end-to-end verification from contract definition to quality metrics

#### Sector-Grade Reference Stacks (`sector_stacks`)
- **Academic Publishing Workflow** (`sector_stacks::academic`): Complete paper review lifecycle
  - 6 workflow stages: Submission → Desk Review → Reviewer Assignment → Review Collection → Decision → Notification
  - Deterministic decision algorithm (3.5+ avg review score = Accept)
  - All-or-nothing decision logic (any rejection → Rejected)
  - 5+ knowledge hooks per major operation
  - 3 theorems mapped to property-based tests
- **Enterprise Claims Processing** (`sector_stacks::claims`): Insurance claims workflow
  - 6 workflow stages: Validation → Fraud Detection → Entitlements → Settlement → Payment → Receipt
  - 7 knowledge hooks per operation
  - 5 guard types: Legality, Budget, Chronology, Causality, Recursion
  - 100+ synthetic test claims configuration
  - Deterministic fraud detection and settlement calculation
- **OperationReceipt**: Generic receipt structure with cryptographic merkle roots
- **SectorOperation Trait**: Extensible trait for sector-specific operations
- Property-based tests proving determinism and reproducibility

#### RDF Integration (`sector_stacks::rdf`)
- **SectorOntology**: Core RDF ontology data structures
- **WorkflowStage**: RDF-driven workflow stage definitions
- **RdfOperationValidator**: Runtime validation against RDF ontologies
- RDF ontologies as single source of truth for workflow definitions
- Oxigraph integration moved to playground (optional tooling, not core dependency)
- 13 new RDF tests (6 ontology + 7 validation)

#### Core Ontology & Operator Registry
- **Operator Registry**: Global singleton for pattern registration
- **Guard System**: 5 guard types with property tracking
- **12 YAWL Patterns**: Registered patterns with guard definitions
- **ggen Templates**: Rust code and LaTeX documentation generation
- Complete class hierarchy with comprehensive properties
- Single source of truth for workflow patterns

#### Spec Harness (`spec-harness`)
- **Theorem Registry**: Executable theorem-to-test mapping
- **Receipt Generation**: Merkle-rooted proofs for theorem conformance
- **23 Passing Tests**: 100% theorem coverage (17/17 theorems)
- Machine-checkable specification with cryptographic receipts

#### Paper as Self-Hosting RDF Instance
- **Auto-Regeneration**: LaTeX documentation generated from RDF
- **CI Pipeline**: Automated paper regeneration and verification
- **RDF Instance**: Paper represented as RDF for semantic querying
- Complete auditability and reproducibility

#### Swarm Protocol (`swarm`)
- **Distributed Multi-Sector Coordination**: Agent-driven task coordination
- **Task Receipt System**: Cryptographic task receipts
- **Knowledge Hooks**: Composition for multi-sector orchestration
- **Public Task Ledger**: Transparent task tracking

#### Snapshot Testing Improvements (`testing::snapshot`)
- **Enhanced Test Fixtures**: Reusable test data structures
- **Improved Test Organization**: Better AAA pattern alignment
- **Complex Structure Support**: Better handling of nested JSON, enums, and maps
- **Sensitive Data Redaction**: Enhanced redaction capabilities for testing

### Added - Documentation & Examples
- **Phase Summaries**: Comprehensive documentation for Phases 1-4
- **RDF Integration Guide**: Complete guide for RDF-driven validation
- **Sector Stack Examples**: Production-grade workflow implementations
- **Cookbook Updates**: Enhanced patterns and troubleshooting guides
- **Application Guide**: Complete mdbook-based application guide

### Changed
- **RDF Architecture**: Oxigraph moved to playground, core library remains lightweight
- **Module Organization**: Sector stacks integrated into main library
- **Test Organization**: Enhanced snapshot testing with better fixtures
- **Documentation Structure**: Improved organization with phase-based summaries

### Performance
- **Sector Operations**: Deterministic execution with <1ms overhead per operation
- **Receipt Generation**: ~100 μs per receipt (SHA-256 signing)
- **RDF Validation**: Minimal overhead for runtime validation
- **Fail-Fast Pipeline**: <1% overhead for typical test suites

### Migration
- All changes are backward compatible
- Existing tests continue working without modification
- Sector stacks are opt-in (use `sector_stacks` module)
- RDF integration is optional (use `rdf` feature or playground)

### See Also
- [Phase 1 Summary](../../PHASE_1_SUMMARY.md)
- [Phase 2 Summary](../../PHASE_2_SUMMARY.md)
- [Phase 3 Summary](../../PHASE_3_SUMMARY.md)
- [Phase 4 Summary](../../PHASE_4_SUMMARY.md)
- [RDF Integration Summary](../../RDF_INTEGRATION_SUMMARY.md)
- [Release Notes](RELEASE_NOTES_v1.4.0.md)

## [1.3.0] - 2025-11-16

### Added - Hyper-Advanced μ-Kernel Verification Substrate

This release transforms Chicago TDD Tools from a "great testing framework" into a **canonical verification substrate for A = μ(O)** (Artifacts equal micro-operator of Observations). Six hyper-advanced tracks provide compile-time contracts, timing discipline, effect typing, state machine validation, cryptographic proofs, and agent-driven orchestration.

#### Track 1: Test Contracts as First-Class Types (`core::contract`)
- **TestContract**: Const-evaluable test descriptors with coverage/invariant declarations
- **TestContractRegistry**: Registry with gap analysis (`uncovered_modules`, `uncovered_invariants`)
- **ResourceEnvelope**: Memory/CPU/network resource constraints
- **Thermal Classification**: Hot/Warm/Cold path designation for timing budgets
- Compile-time coverage analysis prevents gaps in test coverage
- Zero overhead (const evaluation)

#### Track 2: τ-Aware Test Harness (`validation::thermal`)
- **HotPathTest**: Enforces Chatman Constant (τ ≤ 8 ticks) for critical paths
- **WarmPathTest**: Standard paths with heap allocation (τ ≤ 100 ticks)
- **ColdPathTest**: Integration tests without timing constraints
- **HotPathConfig**: Configurable constraints (`max_ticks`, `enforce_no_alloc`, `enforce_no_syscall`)
- **ThermalTestError**: Budget violation errors with detailed diagnostics
- RDTSC/CNTVCT cycle-accurate timing integration
- Relaxed configs for test environments (strict enforcement in production)
- ~10 cycle measurement overhead

#### Track 3: Effect-Typed Tests (`testing::effects`)
- **EffectTest<E>**: Type-safe effect constraints via phantom types
- **Effect Markers**: `Pure`, `NetworkRead`, `NetworkWrite`, `StorageRead`, `StorageWrite`, `Privileged`
- **EffectCoverageRegistry**: Track which effects are tested across test suite
- **RequiresEffect<E>**: Compile-time enforcement that operations match declared effects
- Zero overhead (phantom types)

#### Track 4: Type-Directed State Machine Testing (`testing::state_machine`)
- **StateMachine<S>**: Phantom-typed states with compile-time valid transition enforcement
- **Transition<From, To>**: Trait defining valid state transitions
- **Schedule**: Concurrent state exploration schedules
- **ModelChecker**: Deterministic concurrent state machine testing
- Invalid transitions caught at compile time
- Zero overhead (phantom types)

#### Track 5: Proof-Carrying Test Receipts (`core::receipt`)
- **TestReceipt**: Cryptographic provenance with SHA-256 signatures (placeholder for Ed25519)
- **TestReceiptRegistry**: Governance query API (Γₜ) for deployment decisions
- **EnvironmentFingerprint**: Captured execution environment (OS, arch, Rust version)
- **TimingMeasurement**: τ compliance tracking
- **TestOutcome**: Pass/Fail/Skip with metadata
- Serialization support (JSON) for storage/audit trails
- Governance queries: `tau_violations()`, `failed_receipts()`, `query_by_metadata()`
- ~100 μs per receipt (SHA-256 signing)
- ~2 KB per receipt

#### Track 6: Swarm-Native Test Orchestrator (`swarm::test_orchestrator`)
- **TestOrchestrator**: Agent-driven test scheduling with priority/QoS
- **TestPlan**: Test execution plans with resource budgets
- **QoSClass**: Priority levels (Premium/Standard/BestEffort)
- **ResourceBudget**: CPU/memory/network constraints
- **TestPlanningAPI**: Coverage gap analysis and thermal filtering
- **ΔΣ Analysis**: `suggest_tests_for_change()` suggests minimal test sets for code changes
- ~1 ms per plan
- ~1 KB per plan

### Added - Testing & Documentation
- **Integration Tests** (`tests/hyper_advanced_integration.rs`): 19 comprehensive test cases covering all 6 tracks
- **Complete Workflow Test**: End-to-end demonstration of Contract → Test → Receipt → Orchestration → Governance
- **Comprehensive Documentation** (`docs/features/HYPER_ADVANCED_MICROKERNEL.md`): 400+ line guide with theory, examples, and migration path
- **Example Program** (`examples/hyper_advanced_microkernel.rs`): Runnable demonstration of all 6 tracks
- **Example Documentation** (`examples/hyper_advanced_microkernel.md`): Tutorial with expected output

### Changed
- **Prelude Exports**: Added hyper-advanced types to prelude for easy access
  - `testing::effects::*`
  - `testing::state_machine::*`
  - `swarm::test_orchestrator::*`
  - (Track 1, 2, 5 already exported via `core::*` and `validation::*`)
- **Module Organization**: New modules integrated into capability groups
  - `core::contract` (Track 1)
  - `core::receipt` (Track 5)
  - `validation::thermal` (Track 2)
  - `testing::effects` (Track 3)
  - `testing::state_machine` (Track 4)
  - `swarm::test_orchestrator` (Track 6)

### Performance
- **Total Overhead**: <1% for typical test suites
- **Compile-time Features**: Zero overhead (contracts, effects, state machines)
- **Runtime Features**: Minimal overhead (thermal: ~10 cycles, receipts: ~100 μs, orchestrator: ~1 ms)

### Theory
- **A = μ(O)**: Canonical equation for verification substrate
  - **A (Artifacts)**: Test receipts, contracts, proofs
  - **μ (Micro-operator)**: μ-kernel substrate transforming observations
  - **O (Observations)**: Timing, effects, state transitions
- **Chatman Constant (τ ≤ 8)**: Maximum sustainable cognitive load for hot paths, derived from μ-kernel timing discipline
- **Poka-Yoke Design**: Compile-time prevention of invalid states via phantom types
- **80/20 Thinking**: Second idea = sweet spot (demonstrated in 19 integration tests)

### Migration
- All changes are backward compatible
- Existing tests continue working without modification
- Gradual adoption path: start with Track 2 (thermal testing) for immediate value
- Relaxed configs available for test environments while maintaining strict production enforcement

### See Also
- [Feature Documentation](../docs/features/HYPER_ADVANCED_MICROKERNEL.md)
- [Integration Tests](../../tests/hyper_advanced_integration.rs)
- [Example Program](../../examples/hyper_advanced_microkernel.rs)
- [Release Notes](RELEASE_NOTES_v1.3.0.md)

## [1.2.0] - 2025-11-14

### Added
- **Coverage Enforcement**: Mandatory 85% line coverage enforcement (up from 70% warning in v1.1.0)
  - Production code target: 90%+ coverage
  - Test utilities target: 80%+ coverage
  - CI/CD enforcement as hard requirement (blocking merges)
  - Coverage strategy documented in `docs/coverage/v1.2.0-coverage-strategy.md`

### Changed
- **Coverage Threshold**: Increased from 70% (warning) to 85% (enforced)
  - 85% coverage catches 95% of bugs (Codecov data)
  - Aligns with industry best practices for testing frameworks
  - Achievable without excessive test maintenance burden
- **CI/CD**: Coverage checks now block merges if threshold not met
  - Hard requirement instead of warning-only
  - Supports Poka-Yoke design (error-proofing) through comprehensive error path testing

### Documentation
- Added coverage strategy documentation (`docs/coverage/v1.2.0-coverage-strategy.md`)
- Updated coverage enforcement guidelines
- Documented rationale for 85% threshold (coverage science, Chicago TDD context)

## [1.1.2] - 2025-11-14

### Fixed
- **Testcontainers**: Fixed compilation errors in testcontainers module
  - Added `Debug` derive for error types
  - Fixed string comparison logic in port tests
  - Improved Docker availability checking with timeout protection and retry logic
  - Enhanced error messages for Docker unavailability scenarios
- **Config Module**: Fixed config module test isolation
  - Changed working directory to isolate config tests
  - Added `tempfile` as dev-dependency for config module tests
  - Declared config module to resolve dead code warnings
- **Test Assertions**: Fixed `assert_that` usage to `assert_that_with_msg` for message arguments
  - Updated all test files to use correct assertion macro
  - Improved test error messages with descriptive assertions

### Enhanced
- **CI/CD Pipeline**: Comprehensive improvements to GitHub Actions workflows
  - FMEA improvements: OIDC authentication, path filters, artifact validation
  - Andon signals implementation for quality monitoring
  - Enhanced error handling and reporting
- **Testcontainers**: Improved Docker availability checking
  - Added timeout protection (5s) to prevent hanging when Docker is unavailable
  - Implemented retry logic with exponential backoff for parallel test execution
  - Better error messages with actionable guidance for Docker setup
- **Documentation**: Added comprehensive guides
  - Timeout enforcement documentation (`docs/features/TIMEOUT_ENFORCEMENT.md`)
  - Observability testing guide (`docs/observability/observability-testing-guide.md`)
  - OTEL/Weaver integration guide (`docs/observability/otel-weaver-guide.md`)
  - CLI testing guide (`docs/testing/cli-testing-guide.md`)
  - Coverage strategy documentation (`docs/coverage/v1.2.0-coverage-strategy.md`)

### Changed
- **Build System**: Improved build reliability
  - Enhanced timeout handling in testcontainers Docker checks
  - Better error messages for common failure scenarios
- **Code Quality**: Applied kaizen improvements
  - Extracted magic numbers to named constants (Docker check timeout, retry counts)
  - Improved error message consistency and clarity
  - Enhanced code documentation with kaizen improvement notes

### Documentation
- Added timeout enforcement guide explaining timeout SLAs and patterns
- Added comprehensive observability testing guides (OTEL and Weaver)
- Added CLI testing guide with examples
- Added coverage strategy documentation
- Updated code comments with kaizen improvement documentation

## [1.1.1] - 2025-11-14

### Enhanced
- **`test!` Macro**: Added `Result<(), E>` return type support
  - Tests can now return `Result<(), E>` and use the `?` operator for error propagation
  - Backward compatible: existing tests returning `()` continue to work unchanged
  - Automatic conversion between `()` and `Result<(), E>` return types
  - Improved ergonomics for testing fallible operations
- **`assert_fail!` Macro**: New convenience macro for testing error paths
  - Calls a function and asserts it returns `Err`, then returns the error value
  - Enables concise error path testing without intermediate variables
  - Works seamlessly with `test!` macro's new `Result` return type support
  - Example: `let error = assert_fail!(fallible_function());`

### Documentation
- Added examples demonstrating `Result` return type usage in `test!` macro
- Updated `async_test!` macro documentation to clarify Result return type support
- Added `assert_fail!` macro documentation and examples

## [1.1.0] - 2025-11-10

### Added
- **Weaver Integration**: OpenTelemetry live validation with Weaver (`weaver` feature)
  - `WeaverValidator` for lifecycle management (start/stop)
  - `send_test_span_to_weaver()` helper function for testing
  - Static schema validation via `validate_schema_static()`
  - Automatic Weaver binary download during build (when `weaver` feature enabled)
- **OTEL Validation**: OpenTelemetry span/metric validation (`otel` feature)
  - `SpanValidator` for span validation
  - `MetricValidator` for metric validation
  - Type-safe OTEL types (TraceId, SpanId, SpanContext, etc.)
- **Testcontainers Support**: Docker container integration testing (`testcontainers` feature)
  - Generic container support
  - Port mapping, environment variables, command execution
  - Wait conditions (HTTP health checks, log messages)
  - Automatic cleanup via `Drop` trait
- **Module Reorganization**: Modules organized into capability groups
  - `core/`: Core testing infrastructure
  - `testing/`: Advanced testing techniques
  - `validation/`: Quality & validation
  - `observability/`: Telemetry & observability
  - `integration/`: Integration testing
  - Backward compatibility maintained (all modules re-exported at crate root)
- **Dog Fooding**: Framework tests itself using its own tools
  - All framework tests use `chicago_test!` macros
  - Framework validates its own ergonomics through self-testing

### Changed
- Module organization: Modules moved into capability groups for better discoverability
- Build system: All commands use `cargo make` with timeout protection
- Documentation: Comprehensive documentation updates (README, guides, architecture)
- Test framework: All tests migrated to use `chicago_test!` macro
- Code quality: Extracted magic numbers to named constants for improved readability and maintainability
- Tooling: Retired the custom git hook installer; `cargo make` tasks (`pre-commit`, `check`, `lint`, `audit`) are now the canonical workflow

### Fixed
- Documentation: Updated outdated reports to reflect actual implementation status
- Test framework: Fixed test organization and consistency
- Dead code: Removed duplicate `andon.rs` module (393 lines) - `alert.rs` is the correct implementation

### Documentation
- Added comprehensive README with Chicago TDD principles and dog fooding
- Added architecture documentation
- Added user guides and quick start
- Added SPR (Sparse Priming Representation) methodology guide
- Added dog fooding documentation

## [1.0.0] - 2024-01-01

### Added
- Initial release
- Core testing framework
- Fixtures, builders, assertions
- Test macros
- Property-based testing
- Mutation testing


