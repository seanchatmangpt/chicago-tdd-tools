# User Guide

Complete guide to using Chicago TDD Tools, organized by user journey (beginner to advanced).

## Test Fixtures

Test fixtures: Reusable setup with state management. Test isolation. Metadata tracking.

**When to Use**: Isolated test state, test metadata, integration tests. **Avoid**: Simple unit tests, no state needed, one-off data.

**Basic Usage**: `TestFixture::new()?` for basic fixture, `TestFixture::with_data(data)` for custom data, `fixture.test_counter() -> u64` for unique counter, `fixture.set_metadata(key, value)` / `fixture.get_metadata(key)` for metadata.

**Automatic Setup**: `fixture_test!(name, fixture, { /* AAA */ })` for auto setup/teardown (1s timeout default). `fixture_test_with_timeout!(name, fixture, timeout_secs, { /* AAA */ })` for custom timeout. Prefer over manual creation.

**Patterns**: Shared setup (auto fixture), metadata tracking (test identification), isolation (unique counters).

**Anti-patterns**: Don't create fixtures unnecessarily for simple tests. Don't use fixtures when no state needed.

## Async Fixtures

Async fixtures: Async fixture creation with async traits (Rust 1.75+). Type-safe lifecycle management with GATs. Runtime lifecycle management.

**When to Use**: Async fixture creation (database connections, network resources), async setup/teardown, async resource management. **Requires**: `async` feature, Rust 1.75+. **Avoid**: Synchronous fixtures (use `TestFixture`), simple fixtures, no async needed.

**Basic Usage**: `AsyncFixtureProvider` trait with `async fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error>`, `AsyncFixtureManager::new(provider).setup().await?` for setup, `manager.teardown().await?` for teardown.

**Provider Pattern**: Implement `AsyncFixtureProvider` trait with sealed trait pattern. Define `Fixture<'a>` associated type (GAT), `Error` type. Implement `create_fixture()` async method. Must implement `chicago_tdd_tools::core::async_fixture::private::Sealed` trait.

**Manager Pattern**: `AsyncFixtureManager::new(provider)` creates manager, `manager.setup().await?` creates fixture, `manager.teardown().await?` cleans up. Use in async tests with proper error handling.

**Patterns**: Async resource creation (database connections), async setup/teardown (network resources), type-safe lifecycle (GATs), sealed trait pattern (API safety).

**Anti-patterns**: Don't use for synchronous fixtures. Don't skip error handling. Don't forget to implement sealed trait. Don't use without `async` feature enabled.

## Test Data Builders

Fluent builders for test data. JSON/HashMap output. Domain-specific helpers.

**When to Use**: Complex test data, many fields, fluent readable creation, reusable patterns. **Avoid**: Simple data (use literals), one-off structures, no JSON needed.

**Basic Builder**: `TestDataBuilder::new().with_var(key, value).build_json()` or `.build()` for HashMap.

**Business Helpers**: `with_order_data(order_id, amount)`, `with_customer_data(customer_id)`, `with_approval_data(request_id, amount)`. Create domain-specific extensions.

**Fake Data** (requires `fake-data` feature): `with_fake_email()`, `with_fake_name()`, `with_fake_uuid()` for realistic test data generation.

**Patterns**: Reusable test data (helper functions), JSON for APIs, HashMap for internal use.

**Anti-patterns**: Don't use builders for simple literals. Don't convert to JSON unnecessarily.

## Macros

Test macros: AAA pattern enforcement. Zero-boilerplate tests. Assertion macros. Alert macros.

**When to Use**: All tests (enforces AAA), async tests, fixture tests, performance tests. **Always use**: Reduces boilerplate, enforces patterns.

### Test Macros

- **`test!(name, { /* AAA */ })`**: Synchronous tests with AAA pattern enforcement
- **`async_test!(name, { /* AAA */ })`**: Async tests with AAA pattern (1s timeout default)
- **`async_test_with_timeout!(name, timeout_secs, { /* AAA */ })`**: Async tests with custom timeout (e.g., 30s for integration tests)
- **`fixture_test!(name, fixture, { /* AAA */ })`**: Async tests with automatic fixture setup/teardown (1s timeout default)
- **`fixture_test_with_timeout!(name, fixture, timeout_secs, { /* AAA */ })`**: Async tests with fixture and custom timeout
- **`performance_test!(name, { /* AAA */ })`**: Performance tests with tick budget validation
- **`param_test! { #[case(...)] fn test(...) { ... } }`**: Parameterized tests with rstest (requires `parameterized-testing` feature)
- **`otel_test!(name, { /* AAA */ })`**: OTEL span/metric validation tests (requires `otel` feature)
- **`weaver_test!(name, { /* AAA */ })`**: Weaver live validation tests (1s timeout default, requires `weaver` feature)
- **`weaver_test_with_timeout!(name, timeout_secs, { /* AAA */ })`**: Weaver tests with custom timeout

### Assertion Macros

- **`assert_ok!(result)`** / **`assert_ok!(result, message)`**: Assert Result is Ok with detailed error messages
- **`assert_err!(result)`** / **`assert_err!(result, message)`**: Assert Result is Err with detailed error messages
- **`assert_within_tick_budget!(ticks)`** / **`assert_within_tick_budget!(ticks, message)`**: Validate performance constraints (≤8 ticks)
- **`assert_in_range!(value, min, max)`** / **`assert_in_range!(value, min, max, message)`**: Assert value is within range with detailed messages
- **`assert_eq_msg!(actual, expected, message)`**: Assert equality with custom message
- **`assert_eq_enhanced!(actual, expected)`** / **`assert_eq_enhanced!(actual, expected, message)`**: Enhanced equality assertion with automatic type inference
- **`assert_guard_constraint!(condition, constraint_name)`**: Validate guard constraints

### Alert Macros

- **`alert_critical!(message)`** / **`alert_critical!(message, fix)`**: Emit critical alert (🚨) - must stop immediately
- **`alert_warning!(message)`** / **`alert_warning!(message, fix)`**: Emit warning alert (⚠️) - should stop
- **`alert_info!(message)`**: Emit info alert (ℹ️) - informational
- **`alert_success!(message)`**: Emit success alert (✅) - operation completed
- **`alert_debug!(message)`**: Emit debug alert (🔍) - detailed diagnostics
- **`alert!(severity, message)`** / **`alert!(severity, message, stop, fix)`**: Emit custom alert with user-defined severity

### Procedural Macros

- **`#[tdd_test]`**: Procedural macro for zero-boilerplate tests with AAA validation
  - Import: `use chicago_tdd_tools::tdd_test;`
  - Works with both sync and async functions
- **`#[fixture]`**: Procedural macro for automatic fixture setup/teardown
  - Import: `use chicago_tdd_tools::fixture;`
  - Automatically creates `fixture` variable in test body
- **`#[derive(TestBuilder)]`**: Derive macro for fluent builder generation
  - Generates `{StructName}Builder` with `with_*` methods and `build()` method

**Patterns**: AAA structure (Arrange-Act-Assert), macro usage (all tests), assertion macros (better messages), timeout macros (integration tests).

**Anti-patterns**: Don't skip AAA comments. Don't use raw `#[test]` when macros available. Don't skip assertion macros. Don't forget timeout for integration tests.

## Assertions

Assertion helpers: Result assertions, predicates with HRTB, range assertions.

**When to Use**: Result validation, predicate testing, range validation, custom messages. **Always use**: Better error messages than standard assertions.

**Result Assertions**: `assert_success<T, E>(&Result<T, E>)`, `assert_error<T, E>(&Result<T, E>)`.

**Predicate Assertions**: `assert_that<T, F>(value, predicate)` (HRTB), `assert_that_with_msg<T, F>(value, predicate, msg)`.

**Range Assertions**: `assert_in_range<T>(value, min, max, msg)`, `assert_eq_with_msg<T>(actual, expected, msg)`.

**Patterns**: Result validation (assert_success/assert_error), predicate testing (assert_that), range validation (assert_in_range).

## Property-Based Testing

Property-based testing: Const generics. Reproducible with seeds. Invariant testing.

**When to Use**: Invariant testing, edge case discovery, random test data generation. **Requires**: `property-testing` feature. **Avoid**: Deterministic tests, simple unit tests.

**Basic Usage**: `PropertyTestGenerator::<MAX_ITEMS, MAX_DEPTH>::new().with_seed(seed).generate_test_data()`. Use `property_all_data_valid(generator, num_tests)` for validation.

**Custom Properties**: Create custom property functions for domain invariants. Use fixed seeds for reproducibility. Increase iterations for thorough testing.

**Patterns**: Invariant testing (reverse twice is identity), edge case discovery (random data), reproducible tests (fixed seeds).

**Anti-patterns**: Don't use for deterministic tests. Don't skip seed setting. Don't use insufficient iterations.

## Mutation Testing

Mutation testing: Quality validation. Operators and scores. Test quality metrics.

**When to Use**: Test quality validation, CI/CD pipelines, finding weak tests. **Requires**: `mutation-testing` feature. **Avoid**: Development loop (too slow), simple tests.

**Basic Usage**: `MutationTester::new(data).apply_mutation(operator).test_mutation_detection(test_fn)`. Use `MutationScore::calculate(caught, total).is_acceptable()` (≥80%).

**Operators**: `MutationOperator::RemoveKey(key)`, `MutationOperator::AddKey(key, value)`, `MutationOperator::ChangeValue(key, value)`.

**Patterns**: Quality validation (CI/CD), score tracking (≥80% acceptable), operator testing (all operators).

**Anti-patterns**: Don't use in development loop. Don't accept scores <80%. Don't skip operator testing.

## Snapshot Testing

Snapshot testing: Output capture and comparison. Review workflow. Multiple formats (JSON/YAML/TOML).

**When to Use**: Complex data structures, output stability, regression testing, API response validation. **Requires**: `snapshot-testing` feature. **Avoid**: Simple unit tests, frequently changing outputs.

**Basic Usage**: `SnapshotAssert::assert_matches(&value, "snapshot_name")` for Display, `SnapshotAssert::assert_debug_matches(&value, "snapshot_name")` for Debug, `SnapshotAssert::assert_json_matches(&json_value, "snapshot_name")` for JSON.

**Review Workflow**: Run tests → `cargo make snapshot-review` → Accept/reject changes → Commit snapshots. Use `cargo make snapshot-accept` to accept all, `cargo make snapshot-reject` to reject all.

**Custom Settings**: `SnapshotAssert::with_settings(|settings| { settings.set_snapshot_path("custom"); }, || { /* test */ })` for custom paths.

**Patterns**: Output stability (snapshot comparison), regression testing (capture outputs), review workflow (cargo insta review), multiple formats (JSON/YAML/TOML).

**Anti-patterns**: Don't use for frequently changing outputs. Don't skip review workflow. Don't commit snapshots without review.

## Concurrency Testing

Concurrency testing: Thread model checking. Deterministic concurrency testing.

**When to Use**: Detecting concurrency bugs, verifying thread safety, deterministic concurrency testing. **Requires**: `concurrency-testing` feature. **Avoid**: Simple unit tests, no concurrency.

**Basic Usage**: `LoomModel::new().model(|| { /* concurrent code */ })` for deterministic concurrency testing.

**Patterns**: Thread model checking (loom), deterministic testing (exhaustive exploration), concurrency bug detection.

**Anti-patterns**: Don't use for simple sequential code. Don't skip thread safety verification.

## CLI Testing

CLI testing: Command-line tool testing. Golden files.

**When to Use**: Testing CLI tools, verifying command output, golden file testing. **Requires**: `cli-testing` feature. **Avoid**: Non-CLI code.

**Basic Usage**: `CliTest::new().run_command(cmd, args).assert_output(expected)` for CLI testing. Uses golden files (`.trycmd`) for output comparison.

**Patterns**: CLI output verification (golden files), command execution testing (trycmd), regression testing (output comparison).

**Anti-patterns**: Don't use for non-CLI code. Don't skip golden file updates.

## Performance Testing

Performance testing: RDTSC benchmarking. Tick budget (≤8 ticks = 2ns). Hot path validation.

**When to Use**: Hot path validation, performance-critical code, tick budget enforcement. **Avoid**: Non-critical paths, development loop.

**Tick Measurement**: `TickCounter::start().elapsed_ticks()`, `measure_ticks(|| operation()) -> (result, ticks)`. Use `HOT_PATH_TICK_BUDGET = 8`.

**Validation**: `assert_within_tick_budget!(ticks)` for hot path validation. RDTSC on x86_64, falls back to `std::time::Instant` on other platforms.

**Patterns**: Hot path validation (tick budget), performance measurement (measure_ticks), platform-specific (RDTSC fallback).

**Anti-patterns**: Don't use for non-critical paths. Don't ignore platform differences. Don't skip tick budget validation.

## Guards and Constraints

Guard constraints: Chatman Constant (≤8). Batch size limits. Input validation.

**When to Use**: Input validation, constraint enforcement, guard validation. **Always use**: Prevents invalid data at ingress.

**Guard Validation**: `GuardValidator::new().validate_run_length(length)?`, `validate_batch_size(size)?`. Use `MAX_RUN_LEN = 8`, `MAX_BATCH_SIZE = 1000`.

**Patterns**: Input validation (guard constraints), constraint enforcement (MAX_RUN_LEN), guard validation (all ingress points).

## JTBD Validation

JTBD validation: Scenario validation. Real-world testing. Purpose validation.

**When to Use**: Real-world scenario testing, purpose validation, end-to-end validation. **Avoid**: Simple unit tests.

**Basic Usage**: `JtbdValidator::new().register_scenario(scenario).validate_all()`. Use `JtbdScenario` with setup_context, validate_result, expected_behavior.

**Patterns**: Scenario validation (real-world), purpose validation (JTBD), end-to-end validation (complete workflows).

## Testcontainers Integration

Testcontainers: Docker container support. Port mapping. Command execution. Auto-cleanup.

**When to Use**: Integration testing, Docker services, real dependencies. **Requires**: `testcontainers` feature, Docker running. **Avoid**: Unit tests, mocked dependencies.

**Basic Usage**: `GenericContainer::new(client.client(), image, tag)?`, `container.get_host_port(port)?`, `container.exec(cmd, args)?`. Use `ContainerClient::new()` for client.

**Command Execution**: `container.exec(cmd, args) -> ExecResult` (stdout, stderr, exit_code). Use for service containers that stay running.

**Wait Conditions**: `GenericContainer::with_wait_for(client, image, tag, WaitFor::http(path, port))?` for HTTP services. Verify observable behavior (HTTP responses).

**Patterns**: Integration testing (real containers), port mapping (get_host_port), command execution (exec), wait conditions (HTTP services).

**Anti-patterns**: Don't use for unit tests. Don't skip cleanup. Don't use containers that exit immediately.

## OTEL/Weaver Integration

OTEL/Weaver: Span/metric validation. Schema conformance. Live validation.

**When to Use**: OTEL validation, schema conformance, live validation. **Requires**: `otel` feature, `weaver` feature (requires otel).

**OTEL Validation**: `SpanValidator::new().with_required_attributes(attrs).validate(span)?`, `MetricValidator::new().validate(metric)?`.

**Weaver Validation**: `WeaverValidator::new(registry_path).start()?` / `stop()?`. Use `with_config(registry_path, otlp_grpc_port, admin_port)` for custom config.

**Macros**: `otel_test!(name, { /* AAA */ })` for OTEL tests, `weaver_test!(name, { /* AAA */ })` for Weaver tests (1s timeout), `weaver_test_with_timeout!(name, timeout_secs, { /* AAA */ })` for custom timeout.

**Patterns**: Span validation (required attributes), metric validation (schema conformance), live validation (weaver).

## Alert Helpers

Alert helpers: Visual problem indicators. Standard log crate integration.

**When to Use**: Visual problem indicators, actionable guidance, structured logging. **Requires**: `logging` feature (enabled by default).

**Basic Usage**: `alert_critical!(message, fix)`, `alert_warning!(message, fix)`, `alert_info!(message)`, `alert_success!(message)`, `alert_debug!(message)`, `alert!(severity, message, stop, fix)`.

**Log Integration**: Initialize `AlertLogger::init_default().unwrap()` to use standard `log` macros (`log::error!`, `log::warn!`, etc.) with alert format.

**Patterns**: Visual indicators (emoji alerts), actionable guidance (fix suggestions), structured logging (log crate integration).

**Anti-patterns**: Don't use for normal logging. Don't skip AlertLogger initialization when using log crate.

## Best Practices

**AAA Pattern**: Arrange-Act-Assert structure required. Use AAA comments. Verify observable outputs.

**Use Macros**: Always use `test!`, `async_test!`, `fixture_test!`, `async_test_with_timeout!`, `fixture_test_with_timeout!`. Never use raw `#[test]`.

**Real Collaborators**: Use real objects, minimize mocks. Use testcontainers for integration tests.

**State Verification**: Verify outputs and state, not implementation. Verify observable behavior.

**Behavior Verification**: Tests verify what code does, not how. Verify state changes, outputs, execution order.

**Timeout Management**: Use `async_test_with_timeout!`, `fixture_test_with_timeout!`, `weaver_test_with_timeout!` with 30s timeout for integration tests.

## Common Patterns

**Test Isolation**: `fixture_test!` provides unique fixtures. Tests don't interfere.

**Reusable Test Data**: Helper functions for common test data. `fn create_test_order() -> Value`.

**Error Handling**: Use `?` operator in async tests. Use `assert_ok!` / `assert_err!` for Result validation.

**Poka-Yoke Patterns**: Use `TestResult<T, E>` wrapper that doesn't provide `unwrap()` method. Forces use of `assert_ok!()` or `assert_err!()` macros with better error messages.

**Patterns**: Result validation (assert_ok/assert_err), error propagation (? operator), poka-yoke error prevention (TestResult wrapper).

**Anti-patterns**: Don't use `unwrap()` in tests. Don't skip error handling. Don't use `expect()` without good reason.

## Poka-Yoke (Error Prevention) Design

**Poka-Yoke**: Type-level error prevention to make invalid states unrepresentable. Uses Rust's type system to prevent errors at compile time.

**When to Use**: Prevent common test errors, enforce behavior verification, prevent invalid operations. **Always use**: Makes errors impossible through design.

**Behavior Verification Enforcement**: Use `BehaviorVerification<AssertOkCalled, T>` to enforce that tests verify observable outputs after calling `assert_ok!()`. Type system prevents accessing value without verification.

**Error Handling Enforcement**: Use `TestResult<T, E>` wrapper that doesn't provide `unwrap()` method. Forces use of `assert_ok!()` or `assert_err!()` macros.

**Basic Usage**: 
```rust
use chicago_tdd_tools::core::poka_yoke::*;

// Behavior verification enforcement
let result: Result<i32, String> = Ok(42);
let verification = BehaviorVerification::<AssertOkCalled, i32>::new(result.unwrap());
let verified = verification.verify_behavior(|v| *v == 42);
assert_eq!(verified.value(), 42);

// Error handling enforcement
let result: Result<i32, String> = Ok(42);
let test_result = TestResult::new(result);
let value = test_result.assert_ok("Operation should succeed");
assert_eq!(value, 42);
```

**Patterns**: Type-level prevention (make invalid states unrepresentable), behavior verification enforcement (require output verification), error handling enforcement (prevent unwrap).

**Anti-patterns**: Don't skip behavior verification. Don't use `unwrap()` in tests. Don't ignore type-level safety.

**Integration Test Timeouts**: Use `*_with_timeout!` macros with 30s timeout for Docker operations.

## Anti-patterns

**Don't Skip AAA Comments**: Always use Arrange-Act-Assert structure with comments.

**Don't Use Fixtures Unnecessarily**: Only use fixtures when state/isolation needed.

**Don't Test Implementation Details**: Verify outputs and state, not internal counters/state.

**Don't Use Mocks When Real Available**: Use real collaborators (testcontainers, real dependencies).

**Don't Skip Behavior Verification**: Tests must verify observable outputs, not just function existence.

**Don't Forget Timeouts**: Use `*_with_timeout!` macros for integration tests that require longer timeouts.

## Troubleshooting

**TestFixture::new() fails**: Ensure tokio runtime available for async tests.

**Property-based tests don't compile**: Enable `property-testing` feature flag.

**Parameterized tests don't compile**: Enable `parameterized-testing` feature flag.

**OTEL tests don't compile**: Enable `otel` feature flag.

**Weaver tests don't compile**: Enable `weaver` feature flag (automatically enables `otel`).

**Testcontainers tests fail**: Ensure Docker running and `testcontainers` feature enabled.

**Performance tests fail on non-x86_64**: RDTSC is x86_64-specific; falls back to `std::time::Instant`.

**Async fixture tests fail**: Ensure Rust 1.75+ and `async` feature enabled.

**Alert macros don't work with log crate**: Initialize `AlertLogger::init_default().unwrap()`.

## Summary

**Key Associations**: Fixtures = State Management = Test Isolation. Builders = Fluent = JSON/HashMap. Macros = AAA = Zero-boilerplate. Property = Const generics = Reproducible. Mutation = Quality = Operators. Performance = RDTSC = Tick budget. Guards = Constraints = Chatman Constant. Testcontainers = Docker = Auto-cleanup. OTEL/Weaver = Validation = Schema. Alerts = Visual Indicators = Log Integration.

**Pattern**: All features follow consistent patterns: when to use, basic usage, common patterns, anti-patterns. Use macros for all tests. Verify observable outputs. Use real collaborators. Use timeout macros for integration tests.

## Next Steps

- **[Quick Guide](QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[API Reference](API_REFERENCE.md)** - Complete API documentation
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples
