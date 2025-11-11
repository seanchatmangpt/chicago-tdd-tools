# API Reference

Complete API reference for Chicago TDD Tools, organized by module.

## Macros

### Test Macros

- **`test!(name, body)`**: Synchronous tests with AAA pattern enforcement
  - Signature: `test!($name:ident, $body:block)`
  - Expands to: `#[test] fn $name() { $body }`
  - Timeout: Relies on cargo-nextest profile timeout (1s for unit tests)

- **`async_test!(name, body)`**: Async tests with AAA pattern (1s timeout default)
  - Signature: `async_test!($name:ident, $body:block)`
  - Expands to: `#[tokio::test] async fn $name() { /* timeout wrapper */ }`
  - Timeout: 1s default (wrapped with `tokio::time::timeout`)

- **`async_test_with_timeout!(name, timeout_secs, body)`**: Async tests with custom timeout
  - Signature: `async_test_with_timeout!($name:ident, $timeout_secs:expr, $body:block)`
  - Expands to: `#[tokio::test] async fn $name() { /* timeout wrapper */ }`
  - Use for: Integration tests requiring longer timeouts (e.g., 30s)

- **`fixture_test!(name, fixture_var, body)`**: Async tests with automatic fixture setup/teardown (1s timeout default)
  - Signature: `fixture_test!($name:ident, $fixture_var:ident, $body:block)`
  - Expands to: `#[tokio::test] async fn $name() { let $fixture_var = TestFixture::new().unwrap_or_else(|e| panic!(...)); /* timeout wrapper */ }`
  - Timeout: 1s default
  - Note: Fixture creation panics on failure (appropriate for test macros)

- **`fixture_test_with_timeout!(name, fixture_var, timeout_secs, body)`**: Async tests with fixture and custom timeout
  - Signature: `fixture_test_with_timeout!($name:ident, $fixture_var:ident, $timeout_secs:expr, $body:block)`
  - Use for: Integration tests with fixtures requiring longer timeouts

- **`performance_test!(name, body)`**: Performance tests with tick budget validation
  - Signature: `performance_test!($name:ident, $body:block)`
  - Expands to: `#[test] fn $name() { $body }`

- **`param_test! { #[case(...)] fn test(...) { ... } }`**: Parameterized tests with rstest
  - Signature: `param_test! { $(#[$attr:meta])* fn $name:ident($($param:ident: $type:ty),*) $body:block }`
  - Requires: `parameterized-testing` feature
  - Expands to: `#[rstest::rstest] fn $name(...) { ... }`

- **`otel_test!(name, body)`**: OTEL span/metric validation tests
  - Signature: `otel_test!($name:ident, $body:block)`
  - Requires: `otel` feature
  - Expands to: `#[test] fn $name() { $body }`
  - Timeout: Relies on cargo-nextest profile timeout

- **`weaver_test!(name, body)`**: Weaver live validation tests (1s timeout default)
  - Signature: `weaver_test!($name:ident, $body:block)`
  - Requires: `weaver` feature (automatically enables `otel`)
  - Expands to: `#[tokio::test] async fn $name() { /* timeout wrapper */ }`
  - Timeout: 1s default

- **`weaver_test_with_timeout!(name, timeout_secs, body)`**: Weaver tests with custom timeout
  - Signature: `weaver_test_with_timeout!($name:ident, $timeout_secs:expr, $body:block)`
  - Use for: Integration tests requiring longer timeouts

### Assertion Macros

- **`assert_ok!(result)`** / **`assert_ok!(result, message)`**: Assert Result is Ok with detailed error messages
  - Signature: `assert_ok!($result:expr)` / `assert_ok!($result:expr, $msg:expr)`
  - Panics with: `"Expected Ok, but got Err: {:?}"` or `"{message}: Expected Ok, but got Err: {:?}"`

- **`assert_err!(result)`** / **`assert_err!(result, message)`**: Assert Result is Err with detailed error messages
  - Signature: `assert_err!($result:expr)` / `assert_err!($result:expr, $msg:expr)`
  - Panics with: `"Expected Err, but got Ok: {:?}"` or `"{message}: Expected Err, but got Ok: {:?}"`

- **`assert_within_tick_budget!(ticks)`** / **`assert_within_tick_budget!(ticks, message)`**: Validate performance constraints (≤8 ticks)
  - Signature: `assert_within_tick_budget!($ticks:expr)` / `assert_within_tick_budget!($ticks:expr, $msg:expr)`
  - Panics if: `ticks > 8` (Chatman Constant violation)

- **`assert_in_range!(value, min, max)`** / **`assert_in_range!(value, min, max, message)`**: Assert value is within range
  - Signature: `assert_in_range!($value:expr, $min:expr, $max:expr)` / `assert_in_range!($value:expr, $min:expr, $max:expr, $msg:expr)`
  - Panics if: Value not in `[min, max]` range

- **`assert_eq_msg!(actual, expected, message)`**: Assert equality with custom message
  - Signature: `assert_eq_msg!($actual:expr, $expected:expr, $msg:expr)`
  - Panics with: `"{message}: expected {:?}, got {:?}"`

- **`assert_eq_enhanced!(actual, expected)`** / **`assert_eq_enhanced!(actual, expected, message)`**: Enhanced equality assertion
  - Signature: `assert_eq_enhanced!($actual:expr, $expected:expr $(,)?)` / `assert_eq_enhanced!($actual:expr, $expected:expr, $($arg:tt)+)`
  - Panics with: `"assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`"`

- **`assert_guard_constraint!(condition, constraint_name)`**: Validate guard constraints
  - Signature: `assert_guard_constraint!($condition:expr, $constraint_name:expr)`
  - Panics with: `"Guard constraint violation: {constraint_name}"`

### Alert Macros

- **`alert_critical!(message)`** / **`alert_critical!(message, fix)`**: Emit critical alert (🚨)
  - Signature: `alert_critical!($message:expr)` / `alert_critical!($message:expr, $fix:expr)`
  - Output: `🚨 {message}\n   ⚠️  STOP: Cannot proceed\n   💡 FIX: {fix}`

- **`alert_warning!(message)`** / **`alert_warning!(message, fix)`**: Emit warning alert (⚠️)
  - Signature: `alert_warning!($message:expr)` / `alert_warning!($message:expr, $fix:expr)`
  - Output: `⚠️  {message}\n   ⚠️  WARNING: Investigate before proceeding\n   💡 FIX: {fix}`

- **`alert_info!(message)`**: Emit info alert (ℹ️)
  - Signature: `alert_info!($message:expr)`
  - Output: `ℹ️  {message}`

- **`alert_success!(message)`**: Emit success alert (✅)
  - Signature: `alert_success!($message:expr)`
  - Output: `✅ {message}`

- **`alert_debug!(message)`**: Emit debug alert (🔍)
  - Signature: `alert_debug!($message:expr)` / `alert_debug!($($arg:tt)*)`
  - Output: `🔍 {message}`

- **`alert!(severity, message)`** / **`alert!(severity, message, stop, fix)`**: Emit custom alert
  - Signature: `alert!($severity:expr, $message:expr)` / `alert!($severity:expr, $message:expr, $stop:expr, $fix:expr)`
  - Output: Custom format with severity emoji

### Procedural Macros

- **`#[tdd_test]`**: Procedural macro for zero-boilerplate tests with AAA validation
  - Import: `use chicago_tdd_tools::tdd_test;`
  - Works with: Both sync and async functions
  - Expands to: `#[test]` or `#[tokio::test]` based on function signature

- **`#[fixture]`**: Procedural macro for automatic fixture setup/teardown
  - Import: `use chicago_tdd_tools::fixture;`
  - Automatically creates: `fixture` variable in test body
  - Expands to: `#[test]` or `#[tokio::test]` with fixture setup

- **`#[derive(TestBuilder)]`**: Derive macro for fluent builder generation
  - Generates: `{StructName}Builder` struct with `with_*` methods and `build()` method
  - Requires: Struct with named fields
  - Pattern: `StructBuilder::new().with_field(value).build() -> Result<Struct, String>`

## core::fixture

Test fixtures with automatic cleanup. RAII patterns. Generic with type parameter.

**Types**: 
- `TestFixture<T>` (generic fixture, default `T = ()`)
- `FixtureProvider` (GATs trait)
- `FixtureError` (CreationFailed, OperationFailed)
- `FixtureResult<T>` (alias for `Result<T, FixtureError>`)

**Methods**: 
- `new() -> FixtureResult<TestFixture<()>>` - Create basic fixture
- `with_data(T) -> TestFixture<T>` - Create fixture with custom data
- `inner() -> &T` - Get reference to inner data
- `inner_mut() -> &mut T` - Get mutable reference to inner data
- `test_counter() -> u64` - Get unique test counter
- `set_metadata(key, value)` - Set metadata
- `get_metadata(key) -> Option<&String>` - Get metadata
- `cleanup() -> FixtureResult<()>` - Explicit cleanup (automatic via Drop)

**Pattern**: `TestFixture::new()?` for basic fixture, `TestFixture::with_data(data)` for custom data.

## core::async_fixture

Async fixture providers. Async traits (Rust 1.75+). Type-safe lifecycle management with GATs.

**Types**: 
- `AsyncFixtureProvider` (async trait with GATs)
- `AsyncFixtureManager<P>` (manager)
- `DefaultAsyncFixtureProvider` (default implementation)

**Trait Methods**: 
- `create_fixture(&self) -> impl Future<Output = Result<Self::Fixture<'_>, Self::Error>> + Send` (async fixture creation)

**Manager Methods**: 
- `new(provider: P) -> Self` - Create manager
- `setup(&self) -> impl Future<Output = Result<P::Fixture<'_>, P::Error>>` - Async setup
- `teardown(&self) -> impl Future<Output = FixtureResult<()>>` - Async teardown

**Associated Types**: 
- `Fixture<'a>` (GAT fixture type)
- `Error` (error type)

**Pattern**: `AsyncFixtureManager::new(provider).setup().await?` for async setup, `manager.teardown().await?` for cleanup. Implement `AsyncFixtureProvider` with sealed trait pattern.

**Requirements**: `async` feature, Rust 1.75+.

## core::builders

Fluent builders for test data. JSON/HashMap output. Domain-specific helpers.

**Types**: 
- `TestDataBuilder` (fluent builder)
- `GenericTestDataBuilder<K, V>` (generic builder, requires `K: Into<String>, V: Into<String>`)
- `ValidatedTestDataBuilder<T>` (type-level validated builder)

**Methods**: 
- `new() -> Self` - Create new builder
- `with_var(key, value) -> Self` - Add variable
- `with_order_data(order_id, amount) -> Self` - Add order data
- `with_customer_data(customer_id) -> Self` - Add customer data
- `with_approval_data(request_id, amount) -> Self` - Add approval data
- `with_fake_email() -> Self` - Add fake email (requires `fake-data` feature)
- `with_fake_name() -> Self` - Add fake name (requires `fake-data` feature)
- `with_fake_uuid() -> Self` - Add fake UUID (requires `fake-data` feature)
- `build_json() -> Result<Value, Error>` - Build as JSON
- `build() -> HashMap<String, String>` - Build as HashMap

**Pattern**: `TestDataBuilder::new().with_var("key", "value").build_json()`.

## core::assertions

Assertion helpers. Result assertions, predicates with HRTB, range assertions.

**Functions**: 
- `assert_success<T, E>(&Result<T, E>)` - Assert Result is Ok
- `assert_error<T, E>(&Result<T, E>)` - Assert Result is Err
- `assert_eq_with_msg<T>(actual, expected, msg)` - Assert equality with message
- `assert_in_range<T>(value, min, max, msg)` - Assert value in range
- `assert_that<T, F>(value, predicate)` (HRTB) - Assert predicate
- `assert_that_with_msg<T, F>(value, predicate, msg)` - Assert predicate with message

**Types**:
- `AssertionBuilder<T>` - Builder for assertions
- `ValidatedAssertion<T>` - Validated assertion result

**Pattern**: `assert_success(&result)` for Result, `assert_that(&value, |v| *v > 0)` for predicates.

## core::alert

Alert helpers. Visual problem indicators. Standard log crate integration.

**Types**:
- `AlertLogger` - Logger for standard log crate integration

**Functions**:
- `AlertLogger::init_default() -> Result<(), Error>` - Initialize with default level (Info)
- `AlertLogger::init(level: LevelFilter) -> Result<(), Error>` - Initialize with custom level
- `write_alert(writer, severity, message, stop, fix) -> Result<(), Error>` - Write alert to writer

**Pattern**: `AlertLogger::init_default().unwrap()` to enable log crate integration.

**Requirements**: `logging` feature (enabled by default).

## core::state

Type-level programming. AAA pattern enforcement. Compile-time phase tracking.

**Types**: `TestState<Phase>` (Arrange, Act, Assert phases).

**Methods (Arrange)**: 
- `new() -> TestState<Arrange>` - Create arrange state
- `with_arrange_data(data) -> Self` - Add arrange data
- `act() -> TestState<Act>` - Transition to act phase

**Methods (Act)**: 
- `execute<F>(f) -> Self` - Execute action
- `assert() -> TestState<Assert>` - Transition to assert phase

**Methods (Assert)**: 
- `act_result() -> Option<&Vec<u8>>` - Get act result
- `arrange_data() -> Option<&Vec<u8>>` - Get arrange data
- `assert_that<F>(predicate) -> bool` - Assert predicate

**Pattern**: `TestState::<Arrange>::new().with_arrange_data(data).act().execute(f).assert()`.

## core::const_assert

Compile-time assertions.

**Functions**: 
- `const_assert(condition)` - Compile-time assertion
- `const_assert_msg(condition, msg)` - Compile-time assertion with message

**Pattern**: `const_assert!(SIZE > 0)` for compile-time validation.

## core::type_level

Type-level arithmetic and compile-time validation.

**Types**: 
- `SizeValidatedArray<const SIZE: usize, const MAX_SIZE: usize>` - Size-validated array
- `ValidatedSize` - Marker type for validated size
- `ValidatedRange` - Marker type for validated range

**Pattern**: Use const generics for compile-time validation.

## testing::property

Property-based testing. Const generics. Reproducible with seeds.

**Types**: `PropertyTestGenerator<const MAX_ITEMS: usize = 10, const MAX_DEPTH: usize = 3>` (const generics).

**Methods**: 
- `new() -> Self` - Create generator
- `with_seed(seed: u64) -> Self` - Set seed for reproducibility
- `generate_test_data() -> HashMap<String, String>` - Generate test data
- `max_items() -> usize` - Get max items
- `max_depth() -> usize` - Get max depth

**Functions**: `property_all_data_valid<const MAX_ITEMS, const MAX_DEPTH>(generator, num_tests) -> bool`.

**Pattern**: `PropertyTestGenerator::<10, 3>::new().with_seed(42).generate_test_data()`.

**Requirements**: `property-testing` feature.

## testing::mutation

Mutation testing. Quality validation. Operators and scores.

**Types**: 
- `MutationTester` (tester)
- `MutationOperator` (RemoveKey, AddKey, ChangeValue)
- `MutationScore` (total, caught, score)

**Methods**: 
- `new(original) -> Self` - Create tester
- `apply_mutation(mutation) -> HashMap<String, String>` - Apply mutation
- `test_mutation_detection<F>(test_fn) -> bool` - Test if mutation detected
- `calculate(caught, total) -> Self` - Calculate score
- `score() -> f64` - Get score
- `is_acceptable() -> bool` (≥80%) - Check if score acceptable

**Pattern**: `MutationTester::new(data).apply_mutation(MutationOperator::RemoveKey("key"))`.

**Requirements**: `mutation-testing` feature.

## testing::snapshot

Snapshot testing. Output capture and comparison. Review workflow.

**Types**: `SnapshotAssert` (snapshot assertion helper).

**Methods**: 
- `assert_matches(&value, "snapshot_name")` - Assert Display matches snapshot
- `assert_debug_matches(&value, "snapshot_name")` - Assert Debug matches snapshot
- `assert_json_matches(&json_value, "snapshot_name")` - Assert JSON matches snapshot
- `with_settings(|settings| { ... }, || { ... })` - Custom settings

**Pattern**: `SnapshotAssert::assert_matches(&value, "snapshot_name")`.

**Requirements**: `snapshot-testing` feature.

## testing::concurrency

Concurrency testing. Thread model checking. Deterministic concurrency testing.

**Types**: `LoomModel` (loom model checker).

**Methods**: 
- `new() -> Self` - Create model
- `model(|| { /* concurrent code */ })` - Model concurrent code

**Pattern**: `LoomModel::new().model(|| { /* concurrent code */ })`.

**Requirements**: `concurrency-testing` feature.

## testing::cli

CLI testing. Command-line tool testing. Golden files.

**Types**: `CliTest` (CLI test helper).

**Methods**: 
- `new() -> Self` - Create CLI test
- `run_command(cmd, args) -> Self` - Run command
- `assert_output(expected)` - Assert output matches

**Pattern**: `CliTest::new().run_command("cmd", &["arg"]).assert_output("expected")`.

**Requirements**: `cli-testing` feature.

## testing::generator

Test code generation. Compile-time arrays. Const validation.

**Types**: `TestGenerator` (code generator).

**Methods**: 
- `new() -> Self` - Create generator
- `generate_test(name, spec) -> String` - Generate test code
- `get_tests() -> &[String]` - Get generated tests

**Functions**: 
- `generate_test_array<const N: usize>() -> [u8; N]` - Generate test array
- `generate_test_array_pattern<const N: usize>(pattern) -> [u8; N]` - Generate array with pattern

**Pattern**: `const TEST_DATA: [u8; 10] = generate_test_array::<10>()`.

## validation::coverage

Test coverage analysis. Markdown reports. Percentage calculation.

**Types**: 
- `CoverageReport` (total, covered, percentage, details)
- `CoveragePercentage` - Coverage percentage type
- `CoveredCount` - Covered count type
- `TotalCount` - Total count type

**Methods**: 
- `new() -> Self` - Create report
- `add_item(name, covered)` - Add coverage item
- `generate_markdown() -> String` - Generate markdown report

**Pattern**: `CoverageReport::new().add_item("function1", true).generate_markdown()`.

## validation::performance

Performance validation. RDTSC benchmarking. Tick budget (≤8 ticks = 2ns).

**Types**: 
- `TickCounter` (RDTSC counter)
- `PerformanceValidationError` (TickBudgetExceeded, InvalidMeasurement, MeasurementFailed)
- `ValidatedTickBudget` - Validated tick budget type

**Methods**: 
- `start() -> Self` - Start counter
- `elapsed_ticks() -> u64` - Get elapsed ticks

**Functions**: `measure_ticks<F, R>(f) -> (R, u64)` - Measure ticks for operation.

**Constants**: `HOT_PATH_TICK_BUDGET: u64 = 8`.

**Pattern**: `let (result, ticks) = measure_ticks(|| operation())`.

## validation::guards

Guard constraint enforcement. Chatman Constant (≤8). Batch size limits.

**Types**: 
- `GuardValidator` (validator)
- `GuardConstraintError` (MaxRunLengthExceeded, MaxBatchSizeExceeded, InvalidConstraintValue)

**Methods**: 
- `new() -> Self` - Create validator
- `validate_run_length(length) -> GuardConstraintResult<()>` - Validate run length
- `validate_batch_size(size) -> GuardConstraintResult<()>` - Validate batch size

**Constants**: `MAX_RUN_LEN: usize = 8`, `MAX_BATCH_SIZE: usize = 1000`.

**Pattern**: `GuardValidator::new().validate_run_length(5)?`.

## validation::jtbd

JTBD validation. Scenario validation. Real-world testing.

**Types**: 
- `JtbdValidator` (validator)
- `JtbdScenario` (name, setup_context, validate_result, expected_behavior)
- `JtbdValidationResult` (scenario_name, jtbd_success, technical_success, error_message)
- `ScenarioIndex` - Scenario index type

**Methods**: 
- `new() -> Self` - Create validator
- `register_scenario(scenario)` - Register scenario
- `validate_all() -> Vec<JtbdValidationResult>` - Validate all scenarios

**Pattern**: `JtbdValidator::new().register_scenario(scenario).validate_all()`.

## integration::testcontainers

Docker container support. Port mapping. Command execution. Auto-cleanup.

**Types**: 
- `ContainerClient` (Docker client)
- `GenericContainer` (container wrapper)
- `ExecResult` (stdout, stderr, exit_code)
- `WaitCondition` (Http, Tcp, Log)
- `TestcontainersError` (CreationFailed, OperationFailed, InvalidConfig, CommandExecutionFailed, StdoutReadFailed, StderrReadFailed, ExitCodeFailed)

**Methods**: 
- `new() -> Self` - Create client
- `client() -> &Cli` - Get Docker client
- `new(client, image, tag) -> TestcontainersResult<Self>` - Create container
- `get_host_port(container_port) -> TestcontainersResult<u16>` - Get host port
- `exec(cmd, args) -> TestcontainersResult<ExecResult>` - Execute command
- `set_env(key, value)` - Set environment variable
- `wait_for_ready(condition) -> TestcontainersResult<()>` - Wait for ready condition

**Pattern**: `GenericContainer::new(client.client(), "alpine", "latest")?`.

**Requirements**: `testcontainers` feature, Docker running.

## observability::otel

OTEL validation. Span/metric validation. Schema conformance.

**Types**: 
- `SpanValidator` (span validator)
- `MetricValidator` (metric validator)
- `OtelValidationError` (SpanValidationFailed, MetricValidationFailed, MissingAttribute, InvalidAttributeType, InvalidSpanStatus, InvalidTraceId, InvalidSpanId)
- `Span`, `Metric`, `SpanContext`, `TraceId`, `SpanId`, `SpanStatus` (OTEL types)

**Methods**: 
- `new() -> Self` - Create validator
- `with_required_attributes(attributes) -> Self` - Set required attributes
- `with_non_zero_id_validation(enabled) -> Self` - Enable/disable non-zero ID validation
- `validate(span) -> OtelValidationResult<()>` - Validate span
- `validate(metric) -> OtelValidationResult<()>` - Validate metric

**Functions**:
- `create_test_span(name) -> Span` - Create test span
- `OtelTestHelper::new() -> Self` - Create test helper
- `assert_spans_valid(spans)` - Assert spans are valid

**Pattern**: `SpanValidator::new().with_required_attributes(vec!["service.name"]).validate(&span)?`.

**Requirements**: `otel` feature.

## observability::weaver

Weaver live validation. Registry integration. OTLP/Admin ports.

**Types**: 
- `WeaverValidator` (live validator)
- `WeaverValidationError` (BinaryNotFound, ValidationFailed, RegistryNotFound, ProcessStartFailed, ProcessStopFailed, ProcessNotRunning)
- `WeaverValidationResult<T>` (alias for `Result<T, WeaverValidationError>`)
- `WeaverLiveCheck` - Live check type

**Methods**: 
- `new(registry_path) -> Self` - Create validator
- `with_config(registry_path, otlp_grpc_port, admin_port) -> Self` - Configure ports
- `check_weaver_available() -> WeaverValidationResult<()>` - Check if Weaver available
- `start() -> WeaverValidationResult<()>` - Start Weaver
- `stop() -> WeaverValidationResult<()>` - Stop Weaver
- `is_running() -> bool` - Check if running

**Pattern**: `WeaverValidator::new(path).start()?` / `stop()?`.

**Requirements**: `weaver` feature (automatically enables `otel`).

## prelude

Re-exports commonly used items. Feature-gated exports.

**Core**: `assertions::*`, `builders::*`, `fixture::*`, `guards::*`, `jtbd::*`, `performance::*`, `state::*`, `alert::*`.

**Optional**: 
- `otel::*` (otel feature)
- `property::*` (property-testing feature)
- `mutation::*` (mutation-testing feature)
- `snapshot::*` (snapshot-testing feature)
- `concurrency::*` (concurrency-testing feature)
- `cli::*` (cli-testing feature)
- `weaver::*` (weaver feature)
- `testcontainers::*` (testcontainers feature)

**Usage**: `use chicago_tdd_tools::prelude::*;`.

## Summary

**Key Associations**: Fixtures = RAII = Auto-cleanup. Builders = Fluent = JSON/HashMap. Macros = AAA = Zero-boilerplate. Property = Const generics = Reproducible. Mutation = Quality = Operators. Performance = RDTSC = Tick budget. Guards = Constraints = Chatman Constant. State = Type-level = Compile-time. Testcontainers = Docker = Auto-cleanup. OTEL/Weaver = Validation = Schema. Alerts = Visual Indicators = Log Integration.

**Pattern**: All modules follow consistent patterns: `new() -> Self`, `Result<T, E>` for fallible operations, feature-gated optional modules, zero-cost abstractions where possible.

## Next Steps

- **[Quick Guide](QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[User Guide](USER_GUIDE.md)** - Complete usage guide
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples
