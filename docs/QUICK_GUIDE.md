# Quick Guide

Essential patterns for 80% of common use cases. See [User Guide](USER_GUIDE.md) for comprehensive coverage.

**Quick Navigation**:
- **New user?** Start with [Getting Started](GETTING_STARTED.md)
- **Need API details?** See [API Reference](API_REFERENCE.md)
- **Want architecture?** See [Architecture](ARCHITECTURE.md)

## Test Macros

### Synchronous Test

**Requires**: No feature flags. Core functionality.

**File**: `tests/my_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

test!(test_sync, {
    // Arrange
    let input = 5;
    // Act
    let result = input * 2;
    // Assert
    assert_eq!(result, 10);
});
```

### Async Test

**Requires**: Tokio runtime (included in dev-dependencies). No feature flags needed.

**File**: `tests/my_async_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

async_test!(test_async_operation, {
    // Arrange
    let expected = 10;
    // Act
    let result = async {
        // Your async operation here
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        5 * 2
    }.await;
    // Assert
    assert_eq!(result, expected);
});
```

### Async Test with Custom Timeout

```rust
use chicago_tdd_tools::prelude::*;

async_test_with_timeout!(test_async_integration, 30, {
    // Arrange
    let expected = 10;
    // Act: Slow async operation (30s timeout)
    let result = async { 5 * 2 }.await;
    // Assert
    assert_eq!(result, expected);
});
```

### Fixture Test

```rust
use chicago_tdd_tools::prelude::*;

fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    // Act
    let result = counter + 1;
    // Assert
    assert!(result > 0);
});
```

### Fixture Test with Custom Timeout

```rust
use chicago_tdd_tools::prelude::*;

fixture_test_with_timeout!(test_fixture_integration, fixture, 30, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    // Act: Slow operation (30s timeout)
    let result = counter + 1;
    // Assert
    assert!(result > 0);
});
```

### Performance Test

```rust
use chicago_tdd_tools::prelude::*;

performance_test!(test_hot_path, {
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

### Parameterized Test

**Requires**: `parameterized-testing` feature flag.

**Enable in `Cargo.toml`**:
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["parameterized-testing"]
}
```

**File**: `tests/my_param_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

param_test! {
    #[case(1, 2, 3)]
    #[case(2, 3, 5)]
    #[case(3, 4, 7)]
    fn test_addition(a: i32, b: i32, expected: i32) {
        assert_eq!(a + b, expected);
    }
}
```

### OTEL Test

**Requires**: `otel` feature flag.

**Enable in `Cargo.toml`**:
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["otel"]
}
```

**File**: `tests/my_otel_test.rs`:
```rust
use chicago_tdd_tools::{otel_test, prelude::*};

otel_test!(test_otel_span_validation, {
    // Arrange: Create test span
    let span = chicago_tdd_tools::otel::test_helpers::create_test_span("test.operation");
    // Act: Validate span
    let helper = chicago_tdd_tools::otel::OtelTestHelper::new();
    helper.assert_spans_valid(&[span.clone()]);
    // Assert: Verify span is valid
    assert_eq!(span.name, "test.operation");
});
```

### Weaver Test

**Requires**: `weaver` feature flag (automatically enables `otel`).

**Bootstrap once**:
```bash
cargo make weaver-bootstrap
```

**Daily guardrail**:
```bash
cargo make weaver-smoke    # version check + telemetry span
```

**Enable in `Cargo.toml`**:
```toml
[dev-dependencies]
chicago_tdd_tools = { 
    path = "../chicago-tdd-tools",
    features = ["weaver"]  # Automatically enables otel
}
```

**File**: `tests/my_weaver_test.rs`:
```rust
use chicago_tdd_tools::{weaver_test, prelude::*};
use std::path::PathBuf;

weaver_test!(test_weaver_validation, {
    // Arrange: Create validator
    let registry_path = PathBuf::from("registry/");
    let mut validator = chicago_tdd_tools::weaver::WeaverValidator::new(registry_path);
    // Act: Start Weaver (if available)
    let start_result = validator.start();
    // Assert: Verify Weaver started or handle unavailable case
    if start_result.is_ok() {
        assert!(validator.is_running());
        validator.stop().unwrap();
    }
});
```

### Weaver Test with Custom Timeout (requires `weaver` feature)

```rust
use chicago_tdd_tools::{weaver_test_with_timeout, prelude::*};
use std::path::PathBuf;

weaver_test_with_timeout!(test_weaver_integration, 30, {
    // Arrange: Create validator
    let registry_path = PathBuf::from("registry/");
    let mut validator = chicago_tdd_tools::weaver::WeaverValidator::new(registry_path);
    // Act: Start Weaver (30s timeout)
    let start_result = validator.start();
    // Assert: Verify Weaver started or handle unavailable case
    if start_result.is_ok() {
        assert!(validator.is_running());
        validator.stop().unwrap();
    }
});
```

## Assertion Macros

### Result Assertions

```rust
use chicago_tdd_tools::prelude::*;

test!(test_result_assertions, {
    // Arrange: Create Result
    let result: Result<u32, String> = Ok(42);
    // Assert: Use assertion macros for better error messages
    assert_ok!(&result);
    assert_ok!(&result, "Operation should succeed");
    
    // Error case
    let error_result: Result<u32, String> = Err("failed".to_string());
    assert_err!(&error_result);
    assert_err!(&error_result, "Expected error case");
});
```

### Range Assertions

```rust
use chicago_tdd_tools::prelude::*;

test!(test_range_assertions, {
    // Arrange
    let value = 5;
    // Assert: Verify value in range
    assert_in_range!(value, 0, 10);
    assert_in_range!(value, 0, 10, "Value should be valid");
});
```

### Equality Assertions

```rust
use chicago_tdd_tools::prelude::*;

test!(test_equality_assertions, {
    // Arrange
    let actual = 42;
    let expected = 42;
    // Assert: Enhanced equality assertions
    assert_eq_msg!(actual, expected, "Values should match");
    assert_eq_enhanced!(actual, expected);
    assert_eq_enhanced!(actual, expected, "Custom message");
});
```

### Performance Assertions

```rust
use chicago_tdd_tools::prelude::*;

test!(test_performance_assertions, {
    // Arrange
    let ticks = 5;
    // Assert: Verify tick budget (≤8 ticks)
    assert_within_tick_budget!(ticks);
    assert_within_tick_budget!(ticks, "Hot path operation");
});
```

### Guard Constraint Assertions

```rust
use chicago_tdd_tools::prelude::*;

test!(test_guard_constraints, {
    // Arrange
    let max_run_len = 5;
    // Assert: Verify guard constraint
    assert_guard_constraint!(max_run_len <= 8, "max_run_len");
});
```

## Alert Macros

### Alert Levels

```rust
use chicago_tdd_tools::prelude::*;

test!(test_alerts, {
    // Critical alert - must stop immediately
    alert_critical!("Docker daemon is not running", "Start Docker Desktop");
    
    // Warning alert - should stop
    alert_warning!("Container operation failed", "Check container state");
    
    // Info alert - informational
    alert_info!("Container started successfully");
    
    // Success alert - operation completed
    alert_success!("Test completed successfully");
    
    // Debug alert - detailed diagnostics
    alert_debug!("Container state: {:?}", "running");
    
    // Custom alert
    alert!("🚨", "Custom critical error", "STOP: Cannot proceed", "FIX: Resolve issue");
});
```

## Procedural Macros

### `#[tdd_test]` Attribute

```rust
use chicago_tdd_tools::tdd_test;

#[tdd_test]
fn my_test() {
    // Arrange
    let input = 5;
    // Act
    let result = input * 2;
    // Assert
    assert_eq!(result, 10);
}
```

### `#[fixture]` Attribute

```rust
use chicago_tdd_tools::fixture;

#[fixture]
fn my_test() {
    // Arrange: Fixture automatically created as `fixture`
    let counter = fixture.test_counter();
    // Act
    let result = counter + 1;
    // Assert
    assert!(result > 0);
}
```

### `#[derive(TestBuilder)]`

```rust
use chicago_tdd_tools::TestBuilder;

#[derive(TestBuilder)]
struct MyStruct {
    field1: String,
    field2: i32,
}

test!(test_builder, {
    // Arrange: Use generated builder
    let instance = MyStructBuilder::new()
        .with_field1("value".to_string())
        .with_field2(42)
        .build()
        .unwrap();
    // Act & Assert
    assert_eq!(instance.field1, "value");
    assert_eq!(instance.field2, 42);
});
```

## Data Builders

### Test Data Builder

```rust
use chicago_tdd_tools::prelude::*;

test!(test_data_builder, {
    // Arrange: Create test data
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_order_data("ORD-001", "100.00")
        .build_json()
        .expect("Failed to build JSON");
    // Assert
    assert_eq!(data["key1"], "value1");
});
```

## Async Fixtures (requires `async` feature, Rust 1.75+)

```rust
#[cfg(feature = "async")]
use chicago_tdd_tools::core::async_fixture::{AsyncFixtureManager, AsyncFixtureProvider};
use chicago_tdd_tools::core::fixture::FixtureError;

#[cfg(feature = "async")]
struct DatabaseFixture {
    connection: String,
}

#[cfg(feature = "async")]
struct DatabaseProvider;

#[cfg(feature = "async")]
impl chicago_tdd_tools::core::async_fixture::private::Sealed for DatabaseProvider {}

#[cfg(feature = "async")]
impl AsyncFixtureProvider for DatabaseProvider {
    type Fixture<'a> = DatabaseFixture;
    type Error = FixtureError;

    async fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error> {
        Ok(DatabaseFixture { connection: "connected".to_string() })
    }
}

#[cfg(feature = "async")]
async_test!(test_async_fixture, {
    // Arrange: Create async fixture manager
    let provider = DatabaseProvider;
    let manager = AsyncFixtureManager::new(provider);
    // Act: Setup async fixture
    let fixture = manager.setup().await?;
    // Assert: Verify fixture created
    assert_eq!(fixture.connection, "connected");
    Ok::<(), FixtureError>(())
});
```

## Next Steps

- **[User Guide](USER_GUIDE.md)** - Complete usage guide
- **[API Reference](API_REFERENCE.md)** - Full API documentation
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples

> **Short-term skip?** Export `WEAVER_ALLOW_SKIP=1` to bypass Weaver tests explicitly. Without it, missing prerequisites cause a panic to enforce dogfooding.
