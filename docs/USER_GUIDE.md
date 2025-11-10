# User Guide

Complete guide to using Chicago TDD Tools for testing Rust applications. This guide is organized by user journey, from beginner to advanced patterns.

## Table of Contents

### Beginner Level
- [Test Fixtures](#test-fixtures)
- [Test Data Builders](#test-data-builders)
- [Macros](#macros)
- [Assertions](#assertions)

### Intermediate Level
- [Property-Based Testing](#property-based-testing)
- [Mutation Testing](#mutation-testing)
- [Performance Testing](#performance-testing)

### Advanced Level
- [Guards and Constraints](#guards-and-constraints)
- [JTBD Validation](#jtbd-validation)
- [Testcontainers Integration](#testcontainers-integration)
- [OTEL/Weaver Integration](#otelweaver-integration)

### Reference
- [Best Practices](#best-practices)
- [Common Patterns](#common-patterns)
- [Anti-patterns](#anti-patterns)
- [Troubleshooting](#troubleshooting)

---

## Test Fixtures

Test fixtures provide reusable test setup with automatic cleanup. Use fixtures when you need isolated test state or shared setup across multiple tests.

### When to Use

✅ **Use fixtures when:**
- You need isolated test state
- You want automatic cleanup
- You need test metadata
- You're writing integration tests

❌ **Avoid fixtures when:**
- Simple unit tests with no state
- Tests that don't need isolation
- One-off test data

### Basic Usage

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_with_fixture, {
    // Arrange: Create fixture
    let fixture = TestFixture::new().unwrap();
    
    // Act: Use fixture
    let counter = fixture.test_counter();
    
    // Assert: Verify state
    assert!(counter >= 0);
});
```

**Performance Note**: Fixtures use atomic counters for isolation, adding minimal overhead (~1-2 nanoseconds per test).

### Fixture with Metadata

Store test-specific metadata in fixtures:

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_fixture_metadata, {
    // Arrange: Create fixture and set metadata
    let mut fixture = TestFixture::new().unwrap();
    fixture.set_metadata("test_key".to_string(), "test_value".to_string());
    
    // Act: Retrieve metadata
    let value = fixture.get_metadata("test_key");
    
    // Assert: Verify metadata
    assert_eq!(value, Some(&"test_value".to_string()));
});
```

**Common Pattern**: Use metadata to track test context or pass data between test phases.

### Automatic Fixture Setup

Use `chicago_fixture_test!` macro for automatic fixture setup/teardown:

```rust
use chicago_tdd_tools::prelude::*;

chicago_fixture_test!(test_auto_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    
    // Act: Execute test
    let result = counter + 1;
    
    // Assert: Verify behavior
    assert!(result > 0);
    // Fixture automatically cleaned up on drop
});
```

**Best Practice**: Prefer `chicago_fixture_test!` over manual fixture creation for consistency.

### Common Patterns

**Pattern: Shared Setup**
```rust
chicago_fixture_test!(test_with_shared_setup, fixture, {
    // Fixture provides shared setup automatically
    let counter = fixture.test_counter();
    // Use counter for test isolation
});
```

**Pattern: Metadata Tracking**
```rust
chicago_test!(test_with_metadata, {
    let mut fixture = TestFixture::new().unwrap();
    fixture.set_metadata("test_id".to_string(), "test_001".to_string());
    // Use metadata for test identification
});
```

### Anti-patterns

❌ **Don't create fixtures unnecessarily:**
```rust
// Bad: Fixture not needed
chicago_test!(test_simple, {
    let fixture = TestFixture::new().unwrap(); // Unnecessary
    assert_eq!(2 + 2, 4);
});
```

✅ **Do use fixtures only when needed:**
```rust
// Good: No fixture needed
chicago_test!(test_simple, {
    assert_eq!(2 + 2, 4);
});
```

---

## Test Data Builders

Fluent builders for creating test data structures. Use builders when you need to construct complex test data with multiple fields.

### When to Use

✅ **Use builders when:**
- Creating complex test data
- Building data with many fields
- Need fluent, readable test data creation
- Want reusable data patterns

❌ **Avoid builders when:**
- Simple test data (use literals)
- One-off data structures
- Data doesn't need to be JSON

### Basic Builder

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_data_builder, {
    // Arrange: Create test data
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_var("key2", "value2")
        .build_json();
    
    // Assert: Verify data
    assert_eq!(data["key1"], "value1");
    assert_eq!(data["key2"], "value2");
});
```

**Performance Note**: Builders use `HashMap<String, String>` internally, converting to JSON only when `build_json()` is called.

### Business Data Helpers

Use business-specific helpers for common patterns:

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_business_data, {
    // Arrange: Use business-specific helpers
    let data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .with_approval_data("REQ-001", "50.00")
        .build_json();
    
    // Assert: Verify business data
    assert_eq!(data["order_id"], "ORD-001");
    assert_eq!(data["total_amount"], "100.00");
    assert_eq!(data["customer_id"], "CUST-001");
    assert_eq!(data["request_id"], "REQ-001");
});
```

**Best Practice**: Create domain-specific builder extensions for your project.

### Building HashMap

Build as `HashMap<String, String>` when you don't need JSON:

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_build_hashmap, {
    // Arrange: Build as HashMap
    let data = TestDataBuilder::new()
        .with_var("key", "value")
        .build(); // Returns HashMap<String, String>
    
    // Assert: Verify HashMap
    assert_eq!(data.get("key"), Some(&"value".to_string()));
});
```

**Performance Note**: `build()` is faster than `build_json()` since it skips JSON conversion.

### Common Patterns

**Pattern: Reusable Test Data**
```rust
fn create_test_order() -> serde_json::Value {
    TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json()
}

chicago_test!(test_order_processing, {
    let order = create_test_order();
    // Use order in multiple tests
});
```

**Pattern: Conditional Data**
```rust
chicago_test!(test_conditional_data, {
    let mut builder = TestDataBuilder::new();
    if some_condition {
        builder = builder.with_var("key", "value");
    }
    let data = builder.build_json();
});
```

### Anti-patterns

❌ **Don't build unnecessary JSON:**
```rust
// Bad: JSON conversion not needed
let data = TestDataBuilder::new()
    .with_var("key", "value")
    .build_json(); // Unnecessary if you only need HashMap
```

✅ **Do use appropriate build method:**
```rust
// Good: Use HashMap when JSON not needed
let data = TestDataBuilder::new()
    .with_var("key", "value")
    .build(); // Faster, no JSON conversion
```

---

## Macros

Macros reduce boilerplate and enforce Chicago TDD principles. Use macros for all tests to ensure consistency.

### When to Use

✅ **Always use macros for:**
- All test functions
- AAA pattern enforcement
- Consistent test structure
- Reduced boilerplate

### Test Macros

#### Synchronous Test

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_sync, {
    // Arrange: Set up test data
    let input = 5;
    
    // Act: Execute feature
    let result = input * 2;
    
    // Assert: Verify behavior
    assert_eq!(result, 10);
});
```

**Best Practice**: Always include AAA comments for clarity.

#### Async Test

```rust
use chicago_tdd_tools::prelude::*;

chicago_async_test!(test_async, {
    // Arrange: Set up test data
    let fixture = TestFixture::new().unwrap();
    
    // Act: Execute async operation
    let counter = fixture.test_counter();
    
    // Assert: Verify behavior
    assert!(counter >= 0);
    
    // Supports ? operator for error propagation
    // Ok::<(), MyError>(()) // Return Result - unwrapped automatically
});
```

**Performance Note**: Async tests have minimal overhead compared to standard `#[tokio::test]`.

#### Fixture Test

```rust
use chicago_tdd_tools::prelude::*;

chicago_fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    
    // Act: Execute test
    let result = counter + 1;
    
    // Assert: Verify behavior
    assert!(result > 0);
});
```

**Best Practice**: Use `chicago_fixture_test!` when you need fixtures - it's cleaner than manual setup.

#### Performance Test

```rust
use chicago_tdd_tools::prelude::*;

chicago_performance_test!(test_performance, {
    // Arrange: Set up test data
    let input = vec![1, 2, 3];
    
    // Act: Execute hot path and measure ticks
    let (result, ticks) = measure_ticks(|| {
        input.iter().sum::<i32>()
    });
    
    // Assert: Verify performance constraint (≤8 ticks)
    assert_within_tick_budget!(ticks, "Hot path operation");
    assert_eq!(result, 6);
});
```

**Performance Note**: RDTSC provides cycle-accurate measurement on x86_64; falls back to `std::time::Instant` on other platforms.

### Assertion Macros

#### Result Assertions

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_result_assertions, {
    // Arrange: Create results
    let ok_result: Result<u32, String> = Ok(42);
    let err_result: Result<u32, String> = Err("error".to_string());
    
    // Assert: Use assertion macros
    assert_ok!(&ok_result);
    assert_ok!(&ok_result, "Operation should succeed");
    
    assert_err!(&err_result);
    assert_err!(&err_result, "Operation should fail");
});
```

**Best Practice**: Always include custom messages for better error output.

#### Range Assertions

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_range_assertions, {
    // Arrange: Test value
    let value = 5;
    
    // Assert: Verify in range
    assert_in_range!(value, 0, 10);
    assert_in_range!(value, 0, 10, "Value should be valid");
});
```

#### Equality Assertions

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_equality_assertions, {
    // Arrange: Test values
    let actual = 42;
    let expected = 42;
    
    // Assert: Verify equality with message
    assert_eq_msg!(actual, expected, "Values should match");
});
```

#### Guard Constraint Assertions

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_guard_constraints, {
    // Arrange: Test constraint
    let max_run_len = 5;
    
    // Assert: Verify guard constraint (max_run_len ≤ 8)
    assert_guard_constraint!(max_run_len <= 8, "max_run_len");
});
```

#### Tick Budget Assertions

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_tick_budget, {
    // Arrange: Measure ticks
    let ticks = 5;
    
    // Assert: Verify tick budget (≤8 ticks)
    assert_within_tick_budget!(ticks);
    assert_within_tick_budget!(ticks, "Hot path operation");
});
```

### Common Patterns

**Pattern: Error Propagation**
```rust
chicago_async_test!(test_with_error_propagation, {
    let result = fallible_operation().await?;
    assert_eq!(result, expected);
    Ok::<(), MyError>(()) // Automatically unwrapped
});
```

**Pattern: Multiple Assertions**
```rust
chicago_test!(test_multiple_assertions, {
    let result = operation();
    assert_ok!(&result, "Operation should succeed");
    let value = result.unwrap();
    assert_eq!(value, expected, "Value should match");
    assert_in_range!(value, 0, 100, "Value should be in range");
});
```

### Anti-patterns

❌ **Don't skip AAA comments:**
```rust
// Bad: No AAA structure
chicago_test!(test_bad, {
    let x = 5;
    assert_eq!(x * 2, 10);
});
```

✅ **Do use AAA structure:**
```rust
// Good: Clear AAA structure
chicago_test!(test_good, {
    // Arrange
    let x = 5;
    
    // Act
    let result = x * 2;
    
    // Assert
    assert_eq!(result, 10);
});
```

---

## Assertions

Helper functions for common assertion patterns. Use when you need more flexibility than macros provide.

### When to Use

✅ **Use assertion helpers when:**
- You need custom predicate logic
- You want reusable assertion functions
- You need HRTB (Higher-Ranked Trait Bounds) flexibility

### Result Assertions

```rust
use chicago_tdd_tools::assertions::*;

chicago_test!(test_result_helpers, {
    // Arrange: Create results
    let ok_result: Result<u32, String> = Ok(42);
    let err_result: Result<u32, String> = Err("error".to_string());
    
    // Assert: Use helper functions
    assert_success(&ok_result);
    assert_error(&err_result);
});
```

### Predicate Assertions

Use HRTB for flexible predicates:

```rust
use chicago_tdd_tools::assertions::*;

chicago_test!(test_predicate_assertions, {
    // Arrange: Test value
    let value = 42;
    
    // Assert: Use predicate assertions
    assert_that(&value, |v| *v > 0);
    assert_that_with_msg(&value, |v| *v > 0, "Value should be positive");
});
```

**Best Practice**: Use `assert_that` when standard assertions don't fit your needs.

### Range Assertions

```rust
use chicago_tdd_tools::assertions::*;

chicago_test!(test_range_helpers, {
    // Arrange: Test value
    let value = 5;
    
    // Assert: Verify in range
    assert_in_range(&value, &0, &10, "Value should be in range");
});
```

---

## Property-Based Testing

Validate invariants with randomly generated test data. Use property-based testing to find edge cases automatically.

### When to Use

✅ **Use property-based testing when:**
- You need to find edge cases
- Testing mathematical properties
- Validating invariants
- Testing with random inputs

❌ **Avoid property-based testing when:**
- Testing specific scenarios
- Need deterministic test data
- Testing UI or external APIs

### Basic Property Testing

```rust
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "property-testing")]
chicago_test!(test_property_basic, {
    // Arrange: Create generator
    let mut generator = PropertyTestGenerator::<10, 3>::new()
        .with_seed(42);
    
    // Act & Assert: Test property
    assert!(
        property_all_data_valid(&mut generator, 100),
        "Property: All generated data is valid"
    );
});
```

**Performance Note**: Property tests run multiple iterations; use `with_seed()` for reproducibility.

### Custom Property Functions

Create custom property functions for your domain:

```rust
use chicago_tdd_tools::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "property-testing")]
fn property_all_keys_non_empty(
    generator: &mut PropertyTestGenerator<10, 3>,
    num_tests: usize,
) -> bool {
    for _ in 0..num_tests {
        let data = generator.generate_test_data();
        for key in data.keys() {
            if key.is_empty() {
                return false; // Property violated
            }
        }
    }
    true // Property holds
}

#[cfg(feature = "property-testing")]
chicago_test!(test_custom_property, {
    let mut generator = PropertyTestGenerator::<10, 3>::new();
    assert!(property_all_keys_non_empty(&mut generator, 100));
});
```

**Best Practice**: Use fixed seeds for reproducible tests; increase iterations for thorough testing.

### Common Patterns

**Pattern: Invariant Testing**
```rust
#[cfg(feature = "property-testing")]
fn property_reverse_twice_is_identity(
    generator: &mut PropertyTestGenerator<10, 3>,
    num_tests: usize,
) -> bool {
    for _ in 0..num_tests {
        let data = generator.generate_test_data();
        let reversed: HashMap<_, _> = data.iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();
        let double_reversed: HashMap<_, _> = reversed.iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();
        if data != double_reversed {
            return false;
        }
    }
    true
}
```

### Anti-patterns

❌ **Don't use property testing for specific scenarios:**
```rust
// Bad: Property testing for specific case
#[cfg(feature = "property-testing")]
chicago_test!(test_specific_case, {
    let mut generator = PropertyTestGenerator::<10, 3>::new();
    // Testing specific scenario - use regular test instead
});
```

✅ **Do use property testing for invariants:**
```rust
// Good: Property testing for invariant
#[cfg(feature = "property-testing")]
chicago_test!(test_invariant, {
    let mut generator = PropertyTestGenerator::<10, 3>::new();
    assert!(property_all_data_valid(&mut generator, 100));
});
```

---

## Mutation Testing

Validate test quality by introducing mutations. Use mutation testing to ensure your tests actually catch bugs.

### When to Use

✅ **Use mutation testing when:**
- Validating test quality
- Ensuring tests catch bugs
- Measuring test effectiveness
- CI/CD quality gates

❌ **Avoid mutation testing when:**
- Early development (too slow)
- Simple tests (overkill)
- Non-deterministic code

### Basic Mutation Testing

```rust
use chicago_tdd_tools::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "mutation-testing")]
chicago_test!(test_mutation_basic, {
    // Arrange: Create tester with original data
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    let mut tester = MutationTester::new(data);
    
    // Act: Apply mutations
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
    
    // Assert: Test mutation detection
    let caught = tester.test_mutation_detection(|data| {
        !data.is_empty() // Test should catch empty data
    });
    
    // Calculate mutation score
    let score = MutationScore::calculate(
        if caught { 1 } else { 0 },
        1
    );
    
    // Verify score is acceptable (≥80%)
    assert!(score.is_acceptable());
});
```

**Performance Note**: Mutation testing is slower than regular tests; use in CI/CD, not in development loop.

### Mutation Operators

```rust
use chicago_tdd_tools::mutation::*;
use std::collections::HashMap;

#[cfg(feature = "mutation-testing")]
chicago_test!(test_mutation_operators, {
    // Arrange: Create tester
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    let mut tester = MutationTester::new(data);
    
    // Act: Apply different mutations
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
    tester.apply_mutation(MutationOperator::AddKey("key2".to_string(), "value2".to_string()));
    tester.apply_mutation(MutationOperator::ChangeValue("key1".to_string(), "new_value".to_string()));
    
    // Assert: Verify mutations applied
    assert_eq!(tester.mutations.len(), 3);
});
```

**Best Practice**: Aim for ≥80% mutation score; improve tests if score is lower.

### Common Patterns

**Pattern: Comprehensive Mutation Testing**
```rust
#[cfg(feature = "mutation-testing")]
chicago_test!(test_comprehensive_mutations, {
    let mut data = create_test_data();
    let mut tester = MutationTester::new(data);
    
    // Apply all mutation types
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
    tester.apply_mutation(MutationOperator::AddKey("key2".to_string(), "value2".to_string()));
    tester.apply_mutation(MutationOperator::ChangeValue("key1".to_string(), "changed".to_string()));
    
    // Test detection
    let caught = tester.test_mutation_detection(|d| {
        d.contains_key("key1") && d.get("key1") != Some(&"changed".to_string())
    });
    
    let score = MutationScore::calculate(if caught { 3 } else { 0 }, 3);
    assert!(score.is_acceptable(), "Mutation score: {}%", score.score());
});
```

### Anti-patterns

❌ **Don't use mutation testing in fast feedback loops:**
```rust
// Bad: Mutation testing in development loop
#[cfg(feature = "mutation-testing")]
chicago_test!(test_dev_loop, {
    // Too slow for development
});
```

✅ **Do use mutation testing in CI/CD:**
```rust
// Good: Mutation testing in CI/CD
#[cfg(feature = "mutation-testing")]
#[cfg(test)]
mod mutation_tests {
    // Run in CI/CD, not in development
}
```

---

## Performance Testing

Validate hot path performance with tick measurement. Use performance testing to ensure operations meet the Chatman Constant (≤8 ticks).

### When to Use

✅ **Use performance testing when:**
- Validating hot path performance
- Ensuring operations meet tick budget
- Performance regression testing
- Critical path optimization

❌ **Avoid performance testing when:**
- Non-critical paths
- Operations that don't need to be fast
- External I/O operations

### Tick Measurement

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_tick_measurement, {
    // Arrange: Set up test data
    let input = vec![1, 2, 3, 4, 5];
    
    // Act: Measure ticks for hot path
    let (result, ticks) = measure_ticks(|| {
        input.iter().sum::<i32>()
    });
    
    // Assert: Verify performance constraint (≤8 ticks)
    assert_within_tick_budget!(ticks, "Hot path operation");
    assert_eq!(result, 15);
});
```

**Performance Note**: RDTSC provides cycle-accurate measurement on x86_64; falls back to `std::time::Instant` on other platforms.

### Performance Validation

```rust
use chicago_tdd_tools::performance::*;

chicago_test!(test_performance_validation, {
    // Arrange: Create tick counter
    let counter = TickCounter::start();
    
    // Act: Execute operation
    let _result = expensive_operation();
    
    // Assert: Verify tick budget
    let ticks = counter.elapsed_ticks();
    assert!(ticks <= HOT_PATH_TICK_BUDGET);
});
```

**Best Practice**: Use `HOT_PATH_TICK_BUDGET` constant (8 ticks) for consistency.

### Common Patterns

**Pattern: Hot Path Validation**
```rust
chicago_performance_test!(test_hot_path_validation, {
    let input = create_hot_path_input();
    let (result, ticks) = measure_ticks(|| hot_path_operation(&input));
    assert_within_tick_budget!(ticks, "Hot path must be fast");
    assert_ok!(&result, "Operation must succeed");
});
```

**Pattern: Performance Regression Testing**
```rust
chicago_performance_test!(test_performance_regression, {
    let input = create_test_input();
    let (_, ticks) = measure_ticks(|| operation(&input));
    // Fail if performance degrades
    assert!(ticks <= PREVIOUS_BEST_TICKS, "Performance regression detected");
});
```

### Anti-patterns

❌ **Don't test non-critical paths:**
```rust
// Bad: Performance testing non-critical path
chicago_performance_test!(test_slow_operation, {
    let (_, ticks) = measure_ticks(|| slow_io_operation());
    // IO operations don't need tick budget validation
});
```

✅ **Do test hot paths only:**
```rust
// Good: Performance testing hot path
chicago_performance_test!(test_hot_path, {
    let (_, ticks) = measure_ticks(|| fast_hot_path_operation());
    assert_within_tick_budget!(ticks, "Hot path must be fast");
});
```

---

## Guards and Constraints

Enforce guard constraints at ingress points. Use guards to prevent invalid data from entering your system.

### When to Use

✅ **Use guards when:**
- Validating input at boundaries
- Enforcing MAX_RUN_LEN ≤ 8 (Chatman Constant)
- Enforcing MAX_BATCH_SIZE constraints
- Input validation

### Guard Validation

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_guard_validation, {
    // Arrange: Create validator
    let validator = GuardValidator::new();
    
    // Act: Validate constraints
    let result = validator.validate_run_length(5);
    
    // Assert: Verify validation
    assert_ok!(&result, "Run length should be valid");
});
```

### Constraint Constants

```rust
use chicago_tdd_tools::guards::*;

chicago_test!(test_constraint_constants, {
    // Assert: Verify constants
    assert_eq!(MAX_RUN_LEN, 8); // Chatman Constant
    assert_eq!(MAX_BATCH_SIZE, 1000);
});
```

**Best Practice**: Use constants instead of magic numbers.

---

## JTBD Validation

Validate that code accomplishes its intended purpose. Use JTBD validation to ensure code does the job it's supposed to do.

### When to Use

✅ **Use JTBD validation when:**
- Validating end-to-end workflows
- Ensuring code accomplishes intended purpose
- Real-world scenario testing
- Business logic validation

### Basic JTBD Validation

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_jtbd_validation, {
    // Arrange: Create validator
    let mut validator = JtbdValidator::new();
    
    // Register scenario
    validator.register_scenario(JtbdScenario {
        name: "Order Processing".to_string(),
        setup_context: Box::new(|| {
            create_test_context()
        }),
        validate_result: Box::new(|ctx, result| {
            // Validate that order was actually processed
            result.success && result.variables.contains_key("order_id")
        }),
        expected_behavior: "Process order and update state".to_string(),
    });
    
    // Act: Validate all scenarios
    let results = validator.validate_all();
    
    // Assert: Verify all scenarios pass
    assert!(results.iter().all(|r| r.jtbd_success));
});
```

---

## Testcontainers Integration

Integration testing with Docker containers. Use testcontainers for real integration tests with actual services.

### When to Use

✅ **Use testcontainers when:**
- Integration testing with databases
- Testing with external services
- Real collaborator testing
- End-to-end testing

❌ **Avoid testcontainers when:**
- Unit tests
- Fast feedback loops
- Tests that don't need real services

### Basic Container Usage

```rust
use chicago_tdd_tools::testcontainers::*;

#[cfg(feature = "testcontainers")]
chicago_test!(test_with_container, {
    // Arrange: Create client and container
    let client = ContainerClient::new();
    let container = GenericContainer::new(
        client.client(),
        "alpine",
        "latest"
    ).unwrap();
    
    // Act: Use container
    let host_port = container.get_host_port(80).unwrap();
    
    // Assert: Verify port is valid
    assert!(host_port > 0);
    
    // Container automatically cleaned up on drop
});
```

**Performance Note**: Container startup adds overhead; use sparingly.

### Command Execution

```rust
use chicago_tdd_tools::testcontainers::*;

#[cfg(feature = "testcontainers")]
chicago_test!(test_container_exec, {
    // Arrange: Create container
    let client = ContainerClient::new();
    let container = GenericContainer::new(
        client.client(),
        "alpine",
        "latest"
    ).unwrap();
    
    // Act: Execute command
    let result = container.exec("echo", &["hello"]).unwrap();
    
    // Assert: Verify command output
    assert_eq!(result.stdout.trim(), "hello");
    assert_eq!(result.exit_code, 0);
});
```

---

## OTEL/Weaver Integration

Validate OpenTelemetry spans and metrics. Use OTEL/Weaver validation to ensure telemetry conforms to schema.

### When to Use

✅ **Use OTEL/Weaver validation when:**
- Validating telemetry schema
- Ensuring spans/metrics conform to conventions
- Live validation with Weaver
- Telemetry quality assurance

### OTEL Span Validation

```rust
use chicago_tdd_tools::otel::*;

#[cfg(feature = "otel")]
chicago_test!(test_otel_validation, {
    // Arrange: Create validator
    let validator = SpanValidator::new()
        .with_required_attributes(vec!["service.name".to_string()])
        .with_non_zero_id_validation(true);
    
    // Act: Validate span
    let span = create_test_span();
    let result = validator.validate(&span);
    
    // Assert: Verify validation
    assert_ok!(&result, "Span should be valid");
});
```

### Weaver Live Validation

```rust
use chicago_tdd_tools::weaver::*;

#[cfg(feature = "weaver")]
chicago_test!(test_weaver_validation, {
    // Arrange: Create validator
    let mut validator = WeaverValidator::new(
        PathBuf::from("./otel-registry")
    );
    
    // Act: Start Weaver and validate
    validator.start().unwrap();
    
    // Run tests that generate telemetry...
    
    // Stop Weaver
    validator.stop().unwrap();
    
    // Assert: Verify validation passed
    // (Weaver reports are generated automatically)
});
```

---

## Best Practices

### AAA Pattern

Always follow Arrange-Act-Assert pattern:

```rust
chicago_test!(test_aaa_pattern, {
    // Arrange: Set up test data
    let input = 5;
    let expected = 10;
    
    // Act: Execute feature
    let result = input * 2;
    
    // Assert: Verify behavior
    assert_eq!(result, expected);
});
```

### Use Macros

Prefer macros over manual test setup:

```rust
// Good: Use macro
chicago_fixture_test!(test_with_fixture, fixture, {
    // Test code
});

// Avoid: Manual setup
#[tokio::test]
async fn test_manual() {
    let fixture = TestFixture::new().unwrap();
    // Test code
}
```

### Real Collaborators

Use real dependencies, not mocks:

```rust
// Good: Real container
let container = GenericContainer::new(client.client(), "postgres", "latest").unwrap();

// Avoid: Mock
let mock_db = MockDatabase::new();
```

### State Verification

Verify outputs and state, not implementation:

```rust
// Good: Verify state
assert_eq!(result.order_id, "ORD-001");
assert_eq!(result.status, "processed");

// Avoid: Verify implementation
assert_eq!(result.internal_counter, 1);
```

---

## Common Patterns

### Pattern: Test Isolation

```rust
chicago_fixture_test!(test_isolation, fixture, {
    // Each test gets unique fixture
    let counter = fixture.test_counter();
    // Tests don't interfere with each other
});
```

### Pattern: Reusable Test Data

```rust
fn create_test_order() -> serde_json::Value {
    TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .build_json()
}
```

### Pattern: Error Handling

```rust
chicago_async_test!(test_error_handling, {
    let result = fallible_operation().await?;
    assert_ok!(&result, "Operation should succeed");
    Ok::<(), MyError>(())
});
```

---

## Anti-patterns

### ❌ Don't Skip AAA Comments

```rust
// Bad: No AAA structure
chicago_test!(test_bad, {
    let x = 5;
    assert_eq!(x * 2, 10);
});
```

### ❌ Don't Use Fixtures Unnecessarily

```rust
// Bad: Fixture not needed
chicago_test!(test_simple, {
    let fixture = TestFixture::new().unwrap(); // Unnecessary
    assert_eq!(2 + 2, 4);
});
```

### ❌ Don't Test Implementation Details

```rust
// Bad: Testing implementation
chicago_test!(test_implementation, {
    assert_eq!(internal_counter, 1); // Implementation detail
});
```

---

## Troubleshooting

### Common Issues

**Issue**: `TestFixture::new()` fails
- **Solution**: Ensure tokio runtime is available for async tests

**Issue**: Property-based tests don't compile
- **Solution**: Enable `property-testing` feature flag

**Issue**: Testcontainers tests fail
- **Solution**: Ensure Docker is running and `testcontainers` feature is enabled

**Issue**: Performance tests fail on non-x86_64
- **Solution**: RDTSC is x86_64-specific; tests fall back to `std::time::Instant` on other platforms

### Getting Help

- Check [API Reference](API_REFERENCE.md) for complete API documentation
- Review [Architecture](ARCHITECTURE.md) for design principles
- See `examples/` directory for working code examples

---

## Next Steps

- Read [API Reference](API_REFERENCE.md) for complete API documentation
- Review [Architecture](ARCHITECTURE.md) for design principles
- Check `examples/` directory for more examples
