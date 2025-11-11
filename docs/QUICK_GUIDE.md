# Quick Guide - SPR

Essential patterns for 80% of common use cases. See [User Guide](USER_GUIDE.md) for comprehensive coverage.

## Common Patterns

### Async Test

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

### Async Fixture

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

### Data Builder

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

### Performance Testing

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

## Next Steps

- **[User Guide](USER_GUIDE.md)** - Complete usage guide
- **[API Reference](API_REFERENCE.md)** - Full API documentation
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples

