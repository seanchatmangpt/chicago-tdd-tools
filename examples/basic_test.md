# Basic Test Example

**Category:** Tutorial
**Level:** Beginner
**Prerequisites:** None
**Features Required:** None

---

## Overview

This example demonstrates the fundamental patterns of Chicago TDD Tools. Start here if you're new to the framework.

**What you'll learn:**
- Creating test fixtures for isolated test state
- Building test data with fluent builders
- Handling errors properly in tests
- Following the AAA (Arrange-Act-Assert) pattern

---

## Quick Start

```bash
cargo run --example basic_test
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- No additional features required

---

## Code Overview

The example demonstrates three core patterns:

### 1. Test Fixture Creation

```rust
use chicago_tdd_tools::prelude::*;

fn example_fixture_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Create fixture with proper error handling
    let fixture = match TestFixture::new() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create fixture: {e}");
            return Err(e.into());
        }
    };

    // Use fixture
    let counter = fixture.test_counter();
    Ok(())
}
```

**Key Points:**
- `TestFixture::new()` returns `Result` - handle errors properly
- Fixtures provide isolated test state
- Automatic cleanup on drop

### 2. Test Data Building

```rust
fn example_data_building() -> Result<(), Box<dyn std::error::Error>> {
    // Build test data with fluent API
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_order_data("ORD-001", "100.00")
        .build_json()?;

    Ok(())
}
```

**Key Points:**
- Fluent builder API for readable test data
- Build JSON or HashMap structures
- Proper error handling with `?` operator

### 3. Error Handling

```rust
fn example_error_handling() {
    let result: Result<(), String> = Ok(());

    // Demonstrate success path
    if result.is_ok() {
        println!("✓ Success case handled");
    }

    // Demonstrate error path
    let error_result: Result<(), String> = Err("example error".to_string());
    match error_result {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error handled: {e}"),
    }
}
```

**Key Points:**
- Use `match` for exhaustive error handling
- Test both success and error paths
- Never use `unwrap()` or `expect()` in production code

---

## Key Concepts

### AAA Pattern

All tests follow **Arrange-Act-Assert** structure:

```rust
test!(my_test, {
    // Arrange: Set up test data and fixtures
    let input = 5;

    // Act: Execute code under test
    let result = input * 2;

    // Assert: Verify expected behavior
    assert_eq!(result, 10);
});
```

### Test Isolation

Each test gets a fresh fixture:
- No state leakage between tests
- Automatic cleanup
- Predictable test environment

### Error Handling Best Practices

1. **Use `?` operator** to propagate errors
2. **Use `match`** for explicit handling
3. **Never use `unwrap()`** in production code
4. **Handle both success and error paths**

---

## Usage Examples

### In Your Tests

```rust
use chicago_tdd_tools::prelude::*;

test!(test_with_fixture, {
    // Arrange
    let fixture = TestFixture::new()?;

    // Act
    let counter = fixture.test_counter();

    // Assert
    assert!(counter >= 0);
});
```

### Building Complex Test Data

```rust
let test_data = TestDataBuilder::new()
    .with_var("user_id", "123")
    .with_var("email", "test@example.com")
    .with_order_data("ORD-001", "99.99")
    .build_json()?;
```

---

## Common Patterns

### Pattern 1: Fixture-Based Testing

```rust
test!(test_with_fixture, {
    let fixture = TestFixture::new()?;
    let data = fixture.test_data();
    // ... test code ...
});
```

### Pattern 2: Data Builder

```rust
let data = TestDataBuilder::new()
    .with_var("key", "value")
    .build_json()?;
```

### Pattern 3: Error Path Testing

```rust
let result: Result<(), String> = Err("error".to_string());
match result {
    Ok(_) => panic!("Expected error"),
    Err(e) => assert_eq!(e, "error"),
}
```

---

## Troubleshooting

### Error: "cannot find crate 'chicago_tdd_tools'"

**Cause:** Dependency not added or path incorrect

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
```

### Error: "cannot find macro 'test!'"

**Cause:** Missing prelude import

**Fix:**
```rust
use chicago_tdd_tools::prelude::*;
```

### Error: "TestFixture::new() failed"

**Cause:** Environment configuration issue

**Fix:**
- Check file system permissions
- Verify temp directory is accessible
- Check error message for specific issue

---

## Next Steps

After mastering basic patterns, explore:

1. **[Macro Examples](macro_examples.md)** - Learn test and assertion macros
2. **[Property Testing](property_testing.md)** - Random test data generation
3. **[Mutation Testing](mutation_testing.md)** - Test quality validation

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [Getting Started Guide](../docs/getting-started/GETTING_STARTED.md) - Setup and installation
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation
- [Pattern Cookbook](../cookbook/src/README.md) - Design patterns

---

## Reference

### Key Types

- `TestFixture` - Test state management and isolation
- `TestDataBuilder` - Fluent builder for test data
- `Result<T, E>` - Standard Rust error handling

### Key Functions

- `TestFixture::new() -> Result<TestFixture, FixtureError>`
- `TestDataBuilder::new() -> TestDataBuilder`
- `TestDataBuilder::build_json() -> Result<Value, String>`

### Key Macros

- `test!` - Synchronous test
- `async_test!` - Async test
- `fixture_test!` - Test with fixture

---

**Quality is the default. Prevention beats detection.**

*Example: basic_test.rs | Version: 1.2.0 | Updated: 2025-11-15*
