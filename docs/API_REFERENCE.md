# API Reference - SPR

Complete API reference for Chicago TDD Tools, organized by module.

## fixture

Test fixtures with automatic cleanup. RAII patterns. Generic with type parameter.

**Types**: `TestFixture<T>` (generic fixture), `FixtureProvider` (GATs trait), `FixtureError` (CreationFailed, OperationFailed).

**Methods**: `new() -> FixtureResult<TestFixture<()>>`, `with_data(T) -> TestFixture<T>`, `inner() -> &T`, `inner_mut() -> &mut T`, `test_counter() -> u64`, `set_metadata(key, value)`, `get_metadata(key) -> Option<&String>`, `cleanup() -> FixtureResult<()>`.

**Pattern**: `TestFixture::new()?` for basic fixture, `TestFixture::with_data(data)` for custom data.

## async_fixture

Async fixture providers. Async traits (Rust 1.75+). Type-safe lifecycle management with GATs. Runtime lifecycle management.

**Types**: `AsyncFixtureProvider` (async trait with GATs), `AsyncFixtureManager<P>` (manager), `DefaultAsyncFixtureProvider` (default implementation).

**Trait Methods**: `create_fixture(&self) -> impl Future<Output = Result<Self::Fixture<'_>, Self::Error>> + Send` (async fixture creation).

**Manager Methods**: `new(provider: P) -> Self`, `setup(&self) -> impl Future<Output = Result<P::Fixture<'_>, P::Error>>` (async setup), `teardown(&self) -> impl Future<Output = FixtureResult<()>>` (async teardown).

**Associated Types**: `Fixture<'a>` (GAT fixture type), `Error` (error type).

**Pattern**: `AsyncFixtureManager::new(provider).setup().await?` for async setup, `manager.teardown().await?` for cleanup. Implement `AsyncFixtureProvider` with sealed trait pattern.

**Requirements**: `async` feature, Rust 1.75+.

## builders

Fluent builders for test data. JSON/HashMap output. Domain-specific helpers.

**Types**: `TestDataBuilder` (fluent builder).

**Methods**: `new() -> Self`, `with_var(key, value) -> Self`, `with_order_data(order_id, amount) -> Self`, `with_customer_data(customer_id) -> Self`, `with_approval_data(request_id, amount) -> Self`, `build_json() -> Value`, `build() -> HashMap<String, String>`.

**Pattern**: `TestDataBuilder::new().with_var("key", "value").build_json()`.

## assertions

Assertion helpers. Result assertions, predicates with HRTB, range assertions.

**Functions**: `assert_success<T, E>(&Result<T, E>)`, `assert_error<T, E>(&Result<T, E>)`, `assert_eq_with_msg<T>(actual, expected, msg)`, `assert_in_range<T>(value, min, max, msg)`, `assert_that<T, F>(value, predicate)` (HRTB), `assert_that_with_msg<T, F>(value, predicate, msg)`.

**Pattern**: `assert_success(&result)` for Result, `assert_that(&value, |v| *v > 0)` for predicates.

## macros

Test macros. AAA pattern enforcement. Zero-boilerplate tests.

**Test Macros**: `test!(name, body)` (synchronous), `async_test!(name, body)` (async), `fixture_test!(name, fixture_var, body)` (auto fixture), `performance_test!(name, body)` (tick budget).

**Assertion Macros**: `assert_ok!(result)` / `assert_ok!(result, message)`, `assert_err!(result)` / `assert_err!(result, message)`, `assert_within_tick_budget!(ticks)` / `assert_within_tick_budget!(ticks, message)`, `assert_in_range!(value, min, max)` / `assert_in_range!(value, min, max, message)`, `assert_eq_msg!(actual, expected, message)`, `assert_guard_constraint!(condition, constraint_name)`.

**Pattern**: `test!(test_name, { /* AAA */ })` for tests, `assert_ok!(&result)` for assertions.

## property

Property-based testing. Const generics. Reproducible with seeds.

**Types**: `PropertyTestGenerator<const MAX_ITEMS: usize = 10, const MAX_DEPTH: usize = 3>` (const generics).

**Methods**: `new() -> Self`, `with_seed(seed: u64) -> Self`, `generate_test_data() -> HashMap<String, String>`, `max_items() -> usize`, `max_depth() -> usize`.

**Functions**: `property_all_data_valid<const MAX_ITEMS, const MAX_DEPTH>(generator, num_tests) -> bool`.

**Pattern**: `PropertyTestGenerator::<10, 3>::new().with_seed(42).generate_test_data()`.

## mutation

Mutation testing. Quality validation. Operators and scores.

**Types**: `MutationTester` (tester), `MutationOperator` (RemoveKey, AddKey, ChangeValue), `MutationScore` (total, caught, score).

**Methods**: `new(original) -> Self`, `apply_mutation(mutation) -> HashMap<String, String>`, `test_mutation_detection<F>(test_fn) -> bool`, `calculate(caught, total) -> Self`, `score() -> f64`, `is_acceptable() -> bool` (≥80%).

**Pattern**: `MutationTester::new(data).apply_mutation(MutationOperator::RemoveKey("key"))`.

## coverage

Test coverage analysis. Markdown reports. Percentage calculation.

**Types**: `CoverageReport` (total, covered, percentage, details).

**Methods**: `new() -> Self`, `add_item(name, covered)`, `generate_markdown() -> String`.

**Pattern**: `CoverageReport::new().add_item("function1", true).generate_markdown()`.

## generator

Test code generation. Compile-time arrays. Const validation.

**Types**: `TestGenerator` (code generator).

**Methods**: `new() -> Self`, `generate_test(name, spec) -> String`, `get_tests() -> &[String]`.

**Functions**: `generate_test_array<const N: usize>() -> [u8; N]`, `generate_test_array_pattern<const N: usize>(pattern) -> [u8; N]`, `const_assert(condition)` / `const_assert_msg(condition, msg)`.

**Pattern**: `const TEST_DATA: [u8; 10] = generate_test_array::<10>()`.

## performance

Performance validation. RDTSC benchmarking. Tick budget (≤8 ticks = 2ns).

**Types**: `TickCounter` (RDTSC counter), `PerformanceValidationError` (TickBudgetExceeded, InvalidMeasurement, MeasurementFailed).

**Methods**: `start() -> Self`, `elapsed_ticks() -> u64`.

**Functions**: `measure_ticks<F, R>(f) -> (R, u64)`.

**Constants**: `HOT_PATH_TICK_BUDGET: u64 = 8`.

**Pattern**: `let (result, ticks) = measure_ticks(|| operation())`.

## guards

Guard constraint enforcement. Chatman Constant (≤8). Batch size limits.

**Types**: `GuardValidator` (validator), `GuardConstraintError` (MaxRunLengthExceeded, MaxBatchSizeExceeded, InvalidConstraintValue).

**Methods**: `new() -> Self`, `validate_run_length(length) -> GuardConstraintResult<()>`, `validate_batch_size(size) -> GuardConstraintResult<()>`.

**Constants**: `MAX_RUN_LEN: usize = 8`, `MAX_BATCH_SIZE: usize = 1000`.

**Pattern**: `GuardValidator::new().validate_run_length(5)?`.

## jtbd

JTBD validation. Scenario validation. Real-world testing.

**Types**: `JtbdValidator` (validator), `JtbdScenario` (name, setup_context, validate_result, expected_behavior), `JtbdValidationResult` (scenario_name, jtbd_success, technical_success, error_message).

**Methods**: `new() -> Self`, `register_scenario(scenario)`, `validate_all() -> Vec<JtbdValidationResult>`.

**Pattern**: `JtbdValidator::new().register_scenario(scenario).validate_all()`.

## state

Type-level programming. AAA pattern enforcement. Compile-time phase tracking.

**Types**: `TestState<Phase>` (Arrange, Act, Assert phases).

**Methods (Arrange)**: `new() -> TestState<Arrange>`, `with_arrange_data(data) -> Self`, `act() -> TestState<Act>`.

**Methods (Act)**: `execute<F>(f) -> Self`, `assert() -> TestState<Assert>`.

**Methods (Assert)**: `act_result() -> Option<&Vec<u8>>`, `arrange_data() -> Option<&Vec<u8>>`, `assert_that<F>(predicate) -> bool`.

**Pattern**: `TestState::<Arrange>::new().with_arrange_data(data).act().execute(f).assert()`.

## testcontainers

Docker container support. Port mapping. Command execution. Auto-cleanup.

**Types**: `ContainerClient` (Docker client), `GenericContainer` (container wrapper), `ExecResult` (stdout, stderr, exit_code), `WaitCondition` (Http, Tcp, Log), `TestcontainersError` (CreationFailed, OperationFailed, InvalidConfig, CommandExecutionFailed, StdoutReadFailed, StderrReadFailed, ExitCodeFailed).

**Methods**: `new() -> Self`, `client() -> &Cli`, `new(client, image, tag) -> TestcontainersResult<Self>`, `get_host_port(container_port) -> TestcontainersResult<u16>`, `exec(cmd, args) -> TestcontainersResult<ExecResult>`, `set_env(key, value)`, `wait_for_ready(condition) -> TestcontainersResult<()>`.

**Pattern**: `GenericContainer::new(client.client(), "alpine", "latest")?`.

## otel

OTEL validation. Span/metric validation. Schema conformance.

**Types**: `SpanValidator` (span validator), `MetricValidator` (metric validator), `OtelValidationError` (SpanValidationFailed, MetricValidationFailed, MissingAttribute, InvalidAttributeType, InvalidSpanStatus, InvalidTraceId, InvalidSpanId).

**Methods**: `new() -> Self`, `with_required_attributes(attributes) -> Self`, `with_non_zero_id_validation(enabled) -> Self`, `validate(span) -> OtelValidationResult<()>`, `validate(metric) -> OtelValidationResult<()>`.

**Pattern**: `SpanValidator::new().with_required_attributes(vec!["service.name"]).validate(&span)?`.

## weaver

Weaver live validation. Registry integration. OTLP/Admin ports.

**Types**: `WeaverValidator` (live validator), `WeaverValidationError` (BinaryNotFound, ValidationFailed, RegistryNotFound, ProcessStartFailed, ProcessStopFailed, ProcessNotRunning).

**Methods**: `new(registry_path) -> Self`, `with_config(registry_path, otlp_grpc_port, admin_port) -> Self`, `check_weaver_available() -> WeaverValidationResult<()>`, `start() -> WeaverValidationResult<()>`, `stop() -> WeaverValidationResult<()>`.

**Pattern**: `WeaverValidator::new(path).start()?` / `stop()?`.

## prelude

Re-exports commonly used items. Feature-gated exports.

**Core**: `assertions::*`, `builders::*`, `fixture::*`, `guards::*`, `jtbd::*`, `performance::*`, `state::*`.

**Optional**: `otel::*` (otel feature), `property::*` (property-testing feature), `mutation::*` (mutation-testing feature), `weaver::*` (weaver feature), `testcontainers::*` (testcontainers feature).

**Usage**: `use chicago_tdd_tools::prelude::*;`.

## Summary

**Key Associations**: Fixtures = RAII = Auto-cleanup. Builders = Fluent = JSON/HashMap. Macros = AAA = Zero-boilerplate. Property = Const generics = Reproducible. Mutation = Quality = Operators. Performance = RDTSC = Tick budget. Guards = Constraints = Chatman Constant. State = Type-level = Compile-time. Testcontainers = Docker = Auto-cleanup. OTEL/Weaver = Validation = Schema.

**Pattern**: All modules follow consistent patterns: `new() -> Self`, `Result<T, E>` for fallible operations, feature-gated optional modules, zero-cost abstractions where possible.

## Next Steps

- **[Quick Guide](QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[User Guide](USER_GUIDE.md)** - Complete usage guide
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples
