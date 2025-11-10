# API Reference

Complete API reference for Chicago TDD Tools, organized by module.

## Table of Contents

- [fixture](#fixture)
- [builders](#builders)
- [assertions](#assertions)
- [macros](#macros)
- [property](#property)
- [mutation](#mutation)
- [coverage](#coverage)
- [generator](#generator)
- [performance](#performance)
- [guards](#guards)
- [jtbd](#jtbd)
- [state](#state)
- [testcontainers](#testcontainers)
- [otel](#otel)
- [weaver](#weaver)

## fixture

Test fixtures with automatic cleanup.

### Types

#### `TestFixture<T>`

Generic test fixture with type parameter.

```rust
pub struct TestFixture<T: ?Sized = ()> {
    // ...
}
```

**Methods:**

- `new() -> FixtureResult<TestFixture<()>>` - Create a new test fixture
- `with_data(data: T) -> TestFixture<T>` - Create fixture with custom data
- `inner() -> &T` - Get reference to inner data
- `inner_mut() -> &mut T` - Get mutable reference to inner data
- `test_counter() -> u64` - Get unique test counter
- `set_metadata(key: String, value: String)` - Set metadata
- `get_metadata(key: &str) -> Option<&String>` - Get metadata
- `cleanup() -> FixtureResult<()>` - Cleanup fixture resources

**Example:**

```rust
use chicago_tdd_tools::fixture::TestFixture;

let fixture = TestFixture::new().unwrap();
let counter = fixture.test_counter();
```

#### `FixtureProvider`

Trait for fixture creation with Generic Associated Types (GATs).

```rust
pub trait FixtureProvider {
    type Fixture<'a>: 'a where Self: 'a;
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error>;
}
```

#### `FixtureError`

Fixture error type.

```rust
#[derive(Error, Debug)]
pub enum FixtureError {
    CreationFailed(String),
    OperationFailed(String),
}
```

## builders

Fluent builders for test data.

### Types

#### `TestDataBuilder`

Builder for test data (case variables).

```rust
pub struct TestDataBuilder {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create a new builder
- `with_var(key: impl Into<String>, value: impl Into<String>) -> Self` - Add a variable
- `with_order_data(order_id: impl Into<String>, amount: impl Into<String>) -> Self` - Add order data
- `with_customer_data(customer_id: impl Into<String>) -> Self` - Add customer data
- `with_approval_data(request_id: impl Into<String>, amount: impl Into<String>) -> Self` - Add approval data
- `build_json(self) -> Value` - Build as JSON
- `build(self) -> HashMap<String, String>` - Build as HashMap

**Example:**

```rust
use chicago_tdd_tools::builders::TestDataBuilder;

let data = TestDataBuilder::new()
    .with_var("key", "value")
    .build_json();
```

## assertions

Assertion helpers for common patterns.

### Functions

#### `assert_success<T, E: Debug>(result: &Result<T, E>)`

Assert that a result is successful.

```rust
use chicago_tdd_tools::assertions::assert_success;

let result: Result<u32, String> = Ok(42);
assert_success(&result);
```

#### `assert_error<T: Debug, E>(result: &Result<T, E>)`

Assert that a result is an error.

```rust
use chicago_tdd_tools::assertions::assert_error;

let result: Result<u32, String> = Err("error".to_string());
assert_error(&result);
```

#### `assert_eq_with_msg<T: Debug + PartialEq>(actual: &T, expected: &T, msg: &str)`

Assert equality with custom message.

```rust
use chicago_tdd_tools::assertions::assert_eq_with_msg;

assert_eq_with_msg(&actual, &expected, "Values should match");
```

#### `assert_in_range<T: PartialOrd + Debug>(value: &T, min: &T, max: &T, msg: &str)`

Assert value is within range.

```rust
use chicago_tdd_tools::assertions::assert_in_range;

assert_in_range(&value, &0, &10, "Value should be valid");
```

#### `assert_that<T, F>(value: &T, predicate: F)`

Assert value satisfies predicate using HRTB.

```rust
use chicago_tdd_tools::assertions::assert_that;

assert_that(&value, |v| *v > 0);
```

#### `assert_that_with_msg<T, F>(value: &T, predicate: F, msg: &str)`

Assert value satisfies predicate with custom message.

```rust
use chicago_tdd_tools::assertions::assert_that_with_msg;

assert_that_with_msg(&value, |v| *v > 0, "Value should be positive");
```

## macros

Macros for reducing boilerplate and enforcing Chicago TDD principles.

### Test Macros

#### `chicago_test!(name, body)`

Synchronous test with AAA pattern enforcement.

```rust
use chicago_tdd_tools::chicago_test;

chicago_test!(test_name, {
    // Arrange
    // Act
    // Assert
});
```

#### `chicago_async_test!(name, body)`

Async test with AAA pattern enforcement.

```rust
use chicago_tdd_tools::chicago_async_test;

chicago_async_test!(test_name, {
    // Arrange
    // Act
    // Assert
});
```

#### `chicago_fixture_test!(name, fixture_var, body)`

Async test with automatic fixture setup/teardown.

```rust
use chicago_tdd_tools::chicago_fixture_test;

chicago_fixture_test!(test_name, fixture, {
    // Arrange: Use fixture
    // Act
    // Assert
});
```

#### `chicago_performance_test!(name, body)`

Performance test with tick budget validation.

```rust
use chicago_tdd_tools::chicago_performance_test;

chicago_performance_test!(test_name, {
    // Arrange
    // Act: Measure ticks
    // Assert: Verify tick budget
});
```

### Assertion Macros

#### `assert_ok!(result)` / `assert_ok!(result, message)`

Assert Result is Ok.

```rust
use chicago_tdd_tools::assert_ok;

assert_ok!(&result);
assert_ok!(&result, "Operation should succeed");
```

#### `assert_err!(result)` / `assert_err!(result, message)`

Assert Result is Err.

```rust
use chicago_tdd_tools::assert_err;

assert_err!(&result);
assert_err!(&result, "Operation should fail");
```

#### `assert_within_tick_budget!(ticks)` / `assert_within_tick_budget!(ticks, message)`

Assert ticks are within budget (≤8).

```rust
use chicago_tdd_tools::assert_within_tick_budget;

assert_within_tick_budget!(ticks);
assert_within_tick_budget!(ticks, "Hot path operation");
```

#### `assert_in_range!(value, min, max)` / `assert_in_range!(value, min, max, message)`

Assert value is within range.

```rust
use chicago_tdd_tools::assert_in_range;

assert_in_range!(value, 0, 10);
assert_in_range!(value, 0, 10, "Value should be valid");
```

#### `assert_eq_msg!(actual, expected, message)`

Assert equality with custom message.

```rust
use chicago_tdd_tools::assert_eq_msg;

assert_eq_msg!(actual, expected, "Values should match");
```

#### `assert_guard_constraint!(condition, constraint_name)`

Assert guard constraint is satisfied.

```rust
use chicago_tdd_tools::assert_guard_constraint;

assert_guard_constraint!(max_run_len <= 8, "max_run_len");
```

## property

Property-based testing framework.

### Types

#### `PropertyTestGenerator<const MAX_ITEMS: usize = 10, const MAX_DEPTH: usize = 3>`

Property test generator with const generics.

```rust
pub struct PropertyTestGenerator<const MAX_ITEMS: usize = 10, const MAX_DEPTH: usize = 3> {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create new generator
- `with_seed(seed: u64) -> Self` - Set random seed
- `generate_test_data(&mut self) -> HashMap<String, String>` - Generate test data
- `max_items() -> usize` - Get MAX_ITEMS constant
- `max_depth() -> usize` - Get MAX_DEPTH constant

**Example:**

```rust
use chicago_tdd_tools::property::PropertyTestGenerator;

let mut generator = PropertyTestGenerator::<10, 3>::new()
    .with_seed(42);
let data = generator.generate_test_data();
```

### Functions

#### `property_all_data_valid<const MAX_ITEMS: usize, const MAX_DEPTH: usize>(generator: &mut PropertyTestGenerator<MAX_ITEMS, MAX_DEPTH>, num_tests: usize) -> bool`

Property: All generated data is valid.

```rust
use chicago_tdd_tools::property::property_all_data_valid;

let mut generator = PropertyTestGenerator::<10, 3>::new();
assert!(property_all_data_valid(&mut generator, 100));
```

## mutation

Mutation testing framework.

### Types

#### `MutationTester`

Mutation tester for validating test quality.

```rust
pub struct MutationTester {
    // ...
}
```

**Methods:**

- `new(original: HashMap<String, String>) -> Self` - Create new tester
- `apply_mutation(&mut self, mutation: MutationOperator) -> HashMap<String, String>` - Apply mutation
- `test_mutation_detection<F>(&mut self, test_fn: F) -> bool` - Test if mutation is caught

**Example:**

```rust
use chicago_tdd_tools::mutation::MutationTester;

let mut tester = MutationTester::new(data);
tester.apply_mutation(MutationOperator::RemoveKey("key".to_string()));
let caught = tester.test_mutation_detection(|d| !d.is_empty());
```

#### `MutationOperator`

Mutation operator enum.

```rust
#[derive(Debug, Clone)]
pub enum MutationOperator {
    RemoveKey(String),
    AddKey(String, String),
    ChangeValue(String, String),
}
```

#### `MutationScore`

Mutation score (percentage of mutations caught).

```rust
pub struct MutationScore {
    pub total: usize,
    pub caught: usize,
    pub score: f64,
}
```

**Methods:**

- `calculate(caught: usize, total: usize) -> Self` - Calculate score
- `score(&self) -> f64` - Get score percentage
- `is_acceptable(&self) -> bool` - Is score acceptable (≥80%)

## coverage

Test coverage analysis.

### Types

#### `CoverageReport`

Coverage report.

```rust
pub struct CoverageReport {
    pub total: usize,
    pub covered: usize,
    pub percentage: f64,
    pub details: HashMap<String, bool>,
}
```

**Methods:**

- `new() -> Self` - Create new report
- `add_item(name: String, covered: bool)` - Add coverage item
- `generate_markdown(&self) -> String` - Generate markdown report

**Example:**

```rust
use chicago_tdd_tools::coverage::CoverageReport;

let mut report = CoverageReport::new();
report.add_item("function1".to_string(), true);
report.add_item("function2".to_string(), false);
let markdown = report.generate_markdown();
```

## generator

Test code generation.

### Types

#### `TestGenerator`

Test code generator.

```rust
pub struct TestGenerator {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create new generator
- `generate_test(&mut self, name: &str, spec: &str) -> String` - Generate test code
- `get_tests(&self) -> &[String]` - Get all generated tests

### Functions

#### `generate_test_array<const N: usize>() -> [u8; N]`

Generate test array at compile time.

```rust
use chicago_tdd_tools::generator::generate_test_array;

const TEST_DATA: [u8; 10] = generate_test_array::<10>();
```

#### `generate_test_array_pattern<const N: usize>(pattern: u8) -> [u8; N]`

Generate test array with pattern at compile time.

```rust
use chicago_tdd_tools::generator::generate_test_array_pattern;

const TEST_DATA: [u8; 10] = generate_test_array_pattern::<10>(42);
```

#### `const_assert(condition: bool)` / `const_assert_msg(condition: bool, msg: &'static str)`

Compile-time validation helpers.

```rust
use chicago_tdd_tools::generator::{const_assert, const_assert_msg};

const_assert(true);
const_assert_msg(true, "Condition should hold");
```

## performance

Performance validation with RDTSC benchmarking.

### Types

#### `TickCounter`

Tick counter using RDTSC.

```rust
pub struct TickCounter {
    // ...
}
```

**Methods:**

- `start() -> Self` - Create and start counter
- `elapsed_ticks(&self) -> u64` - Get elapsed ticks

**Example:**

```rust
use chicago_tdd_tools::performance::TickCounter;

let counter = TickCounter::start();
// ... execute operation ...
let ticks = counter.elapsed_ticks();
```

### Functions

#### `measure_ticks<F, R>(f: F) -> (R, u64)`

Measure ticks for a closure.

```rust
use chicago_tdd_tools::performance::measure_ticks;

let (result, ticks) = measure_ticks(|| expensive_operation());
```

### Constants

#### `HOT_PATH_TICK_BUDGET`

Tick budget for hot path operations (8 ticks = 2ns).

```rust
pub const HOT_PATH_TICK_BUDGET: u64 = 8;
```

### Types

#### `PerformanceValidationError`

Performance validation error.

```rust
#[derive(Error, Debug)]
pub enum PerformanceValidationError {
    TickBudgetExceeded(u64, u64),
    InvalidMeasurement(String),
    MeasurementFailed(String),
}
```

## guards

Guard constraint enforcement.

### Types

#### `GuardValidator`

Guard constraint validator.

```rust
pub struct GuardValidator {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create new validator
- `validate_run_length(&self, length: usize) -> GuardConstraintResult<()>` - Validate run length
- `validate_batch_size(&self, size: usize) -> GuardConstraintResult<()>` - Validate batch size

**Example:**

```rust
use chicago_tdd_tools::guards::GuardValidator;

let validator = GuardValidator::new();
validator.validate_run_length(5).unwrap();
```

### Constants

#### `MAX_RUN_LEN`

Maximum run length (Chatman Constant: ≤8).

```rust
pub const MAX_RUN_LEN: usize = 8;
```

#### `MAX_BATCH_SIZE`

Maximum batch size.

```rust
pub const MAX_BATCH_SIZE: usize = 1000;
```

### Types

#### `GuardConstraintError`

Guard constraint error.

```rust
#[derive(Error, Debug)]
pub enum GuardConstraintError {
    MaxRunLengthExceeded(usize, usize),
    MaxBatchSizeExceeded(usize, usize),
    InvalidConstraintValue(String),
}
```

## jtbd

JTBD (Jobs To Be Done) validation framework.

### Types

#### `JtbdValidator`

JTBD validator.

```rust
pub struct JtbdValidator {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create new validator
- `register_scenario(&mut self, scenario: JtbdScenario)` - Register scenario
- `validate_all(&self) -> Vec<JtbdValidationResult>` - Validate all scenarios

**Example:**

```rust
use chicago_tdd_tools::jtbd::*;

let mut validator = JtbdValidator::new();
validator.register_scenario(scenario);
let results = validator.validate_all();
```

#### `JtbdScenario`

JTBD scenario.

```rust
pub struct JtbdScenario {
    pub name: String,
    pub setup_context: Box<dyn Fn() -> TestContext>,
    pub validate_result: Box<dyn Fn(&TestContext, &TestResult) -> bool>,
    pub expected_behavior: String,
}
```

#### `JtbdValidationResult`

JTBD validation result.

```rust
#[derive(Debug, Clone)]
pub struct JtbdValidationResult {
    pub scenario_name: String,
    pub jtbd_success: bool,
    pub technical_success: bool,
    pub error_message: Option<String>,
}
```

## state

Type-level programming for test state.

### Types

#### `TestState<Phase>`

Test state with type-level phase tracking.

```rust
pub struct TestState<Phase> {
    // ...
}
```

**Phase Types:**

- `Arrange` - Arrange phase marker
- `Act` - Act phase marker
- `Assert` - Assert phase marker

**Methods (Arrange phase):**

- `new() -> TestState<Arrange>` - Create new test state
- `with_arrange_data(self, data: Vec<u8>) -> Self` - Add arrange data
- `act(self) -> TestState<Act>` - Transition to Act phase

**Methods (Act phase):**

- `execute<F>(self, f: F) -> Self` - Execute act operation
- `assert(self) -> TestState<Assert>` - Transition to Assert phase

**Methods (Assert phase):**

- `act_result(&self) -> Option<&Vec<u8>>` - Get act result
- `arrange_data(&self) -> Option<&Vec<u8>>` - Get arrange data
- `assert_that<F>(&self, predicate: F) -> bool` - Assert with predicate

**Example:**

```rust
use chicago_tdd_tools::state::*;

let arrange_state = TestState::<Arrange>::new()
    .with_arrange_data(vec![1, 2, 3]);
let act_state = arrange_state.act();
let act_state = act_state.execute(|data| {
    data.unwrap_or_default()
});
let assert_state = act_state.assert();
assert!(assert_state.assert_that(|result| result.is_some()));
```

## testcontainers

Testcontainers support for integration testing.

### Types

#### `ContainerClient`

Container client for Docker operations.

```rust
pub struct ContainerClient {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create new client
- `client(&self) -> &testcontainers::clients::Cli` - Get underlying client

#### `GenericContainer`

Generic container wrapper.

```rust
pub struct GenericContainer {
    // ...
}
```

**Methods:**

- `new(client: &Cli, image: &str, tag: &str) -> TestcontainersResult<Self>` - Create container
- `get_host_port(&self, container_port: u16) -> TestcontainersResult<u16>` - Get host port
- `exec(&self, cmd: &str, args: &[&str]) -> TestcontainersResult<ExecResult>` - Execute command
- `set_env(&mut self, key: &str, value: &str)` - Set environment variable
- `wait_for_ready(&self, condition: WaitCondition) -> TestcontainersResult<()>` - Wait for ready

**Example:**

```rust
use chicago_tdd_tools::testcontainers::*;

let client = ContainerClient::new();
let container = GenericContainer::new(
    client.client(),
    "alpine",
    "latest"
).unwrap();
let port = container.get_host_port(80).unwrap();
```

#### `ExecResult`

Result of executing a command in a container.

```rust
#[derive(Debug, Clone)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i64,
}
```

#### `WaitCondition`

Wait condition for containers.

```rust
pub enum WaitCondition {
    Http { port: u16, path: String },
    Tcp { port: u16 },
    Log { pattern: String },
}
```

### Types

#### `TestcontainersError`

Testcontainers error type.

```rust
#[derive(Error, Debug)]
pub enum TestcontainersError {
    CreationFailed(String),
    OperationFailed(String),
    InvalidConfig(String),
    CommandExecutionFailed(String),
    StdoutReadFailed(String),
    StderrReadFailed(String),
    ExitCodeFailed(String),
}
```

## otel

OTEL validation for spans and metrics.

### Types

#### `SpanValidator`

OTEL span validator.

```rust
pub struct SpanValidator {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create new validator
- `with_required_attributes(self, attributes: Vec<String>) -> Self` - Require attributes
- `with_non_zero_id_validation(self, enabled: bool) -> Self` - Enable/disable ID validation
- `validate(&self, span: &Span) -> OtelValidationResult<()>` - Validate span

**Example:**

```rust
use chicago_tdd_tools::otel::SpanValidator;

let validator = SpanValidator::new()
    .with_required_attributes(vec!["service.name".to_string()]);
validator.validate(&span).unwrap();
```

#### `MetricValidator`

OTEL metric validator.

```rust
pub struct MetricValidator {
    // ...
}
```

**Methods:**

- `new() -> Self` - Create new validator
- `validate(&self, metric: &Metric) -> OtelValidationResult<()>` - Validate metric

### Types

#### `OtelValidationError`

OTEL validation error.

```rust
#[derive(Error, Debug)]
pub enum OtelValidationError {
    SpanValidationFailed(String),
    MetricValidationFailed(String),
    MissingAttribute(String),
    InvalidAttributeType(String, String, String),
    InvalidSpanStatus(String),
    InvalidTraceId(String),
    InvalidSpanId(String),
}
```

## weaver

Weaver live validation integration.

### Types

#### `WeaverValidator`

Weaver live validation helper.

```rust
pub struct WeaverValidator {
    // ...
}
```

**Methods:**

- `new(registry_path: PathBuf) -> Self` - Create new validator
- `with_config(registry_path: PathBuf, otlp_grpc_port: u16, admin_port: u16) -> Self` - Create with config
- `check_weaver_available() -> WeaverValidationResult<()>` - Check if Weaver binary is available
- `start(&mut self) -> WeaverValidationResult<()>` - Start Weaver live-check
- `stop(&mut self) -> WeaverValidationResult<()>` - Stop Weaver live-check

**Example:**

```rust
use chicago_tdd_tools::weaver::WeaverValidator;
use std::path::PathBuf;

let mut validator = WeaverValidator::new(PathBuf::from("./otel-registry"));
validator.start().unwrap();
// Run tests...
validator.stop().unwrap();
```

### Types

#### `WeaverValidationError`

Weaver validation error.

```rust
#[derive(Error, Debug)]
pub enum WeaverValidationError {
    BinaryNotFound,
    ValidationFailed(String),
    RegistryNotFound(String),
    ProcessStartFailed(String),
    ProcessStopFailed(String),
    ProcessNotRunning,
}
```

## Prelude

The `prelude` module re-exports commonly used items:

```rust
pub mod prelude {
    pub use crate::assertions::*;
    pub use crate::builders::*;
    pub use crate::fixture::*;
    pub use crate::guards::*;
    pub use crate::jtbd::*;
    pub use crate::performance::*;
    pub use crate::state::*;

    #[cfg(feature = "otel")]
    pub use crate::otel::*;

    #[cfg(feature = "property-testing")]
    pub use crate::property::*;

    #[cfg(feature = "mutation-testing")]
    pub use crate::mutation::*;

    #[cfg(feature = "weaver")]
    pub use crate::weaver::*;

    #[cfg(feature = "testcontainers")]
    pub use crate::testcontainers::*;
}
```

**Usage:**

```rust
use chicago_tdd_tools::prelude::*;
```

This imports all commonly used types, functions, and macros.

