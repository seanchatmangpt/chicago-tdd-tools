# Quick Guide - SPR

Essential patterns for 80% of common use cases. See [User Guide](USER_GUIDE.md) for comprehensive coverage.

## Common Patterns

### Async Test

```rust
use chicago_tdd_tools::prelude::*;

chicago_async_test!(test_async_operation, {
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

chicago_fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    // Act
    let result = counter + 1;
    // Assert
    assert!(result > 0);
});
```

### Data Builder

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_data_builder, {
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

## Next Steps

- **[User Guide](USER_GUIDE.md)** - Complete usage guide
- **[API Reference](API_REFERENCE.md)** - Full API documentation
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples

