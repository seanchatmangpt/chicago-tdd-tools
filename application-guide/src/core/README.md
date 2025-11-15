# Core Testing Patterns

Welcome to the core testing patterns section! Here you'll learn the everyday patterns you'll use in almost every test.

## Overview

Core patterns include:

1. **Fixtures** - Isolated test state and setup
2. **Data Builders** - Fluent API for constructing test data
3. **Assertions** - Clear, readable verification of results
4. **Error Paths** - Testing failure scenarios

These patterns form the foundation of all testing in Chicago TDD Tools.

## Quick Start

Here's a complete test using core patterns:

```rust
use chicago_tdd_tools::prelude::*;

test!(test_user_creation, {
    // Arrange: Set up with fixtures and builders
    let fixture = TestFixture::new()?;
    let user_data = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("email", "alice@example.com")
        .build_json()?;

    // Act: Execute the code under test
    let result = create_user(&user_data)?;

    // Assert: Verify with assertions
    assert_ok!(&result);
    assert_eq!(result.unwrap().name, "Alice");
});
```

## Core Concepts

### Test Isolation

Each test must be independent:

```rust
test!(test1, {
    let fixture = TestFixture::new()?;  // Fresh fixture
    // test1 uses fixture1
});

test!(test2, {
    let fixture = TestFixture::new()?;  // Different fixture
    // test2 uses fixture2
    // Both tests can run in parallel with no interference
});
```

### Data Construction

Build complex test data with fluent builders:

```rust
let order = TestDataBuilder::new()
    .with_var("order_id", "ORD-001")
    .with_order_data("ORD-001", "100.50")
    .build_json()?;
```

### Clear Assertions

Use assertion helpers for readability:

```rust
assert_ok!(&result);           // Checks is_ok()
assert_err!(&result);          // Checks is_err()
assert_eq!(value, expected);   // Equality check
```

## Sections

- [Getting Started with Fixtures](fixtures.md) - Fixture lifecycle and setup
- [Building Test Data](data-builders.md) - Data builders and fluent API
- [Assertions & Verification](assertions.md) - Assertion helpers and patterns
- [Error Path Testing](error-paths.md) - Testing failure scenarios

## Common Patterns

### Pattern: Arrange-Act-Assert

Every test follows this structure:

```rust
test!(test_example, {
    // Arrange: Set up test state
    let fixture = TestFixture::new()?;
    let data = TestDataBuilder::new()...build_json()?;

    // Act: Execute code under test
    let result = function_under_test(&data)?;

    // Assert: Verify behavior
    assert_ok!(&result);
    assert_eq!(result.unwrap().field, expected_value);
});
```

### Pattern: Error Path Testing

Always test both success and failure:

```rust
test!(test_with_error_path, {
    // Success path
    let ok_result = parse_number("42");
    assert_ok!(&ok_result);

    // Error path
    let err_result = parse_number("not_a_number");
    assert_err!(&err_result);
});
```

### Pattern: Boundary Conditions

Test edge cases:

```rust
test!(test_boundaries, {
    // Minimum value
    assert_ok!(&function(0));

    // Maximum value
    assert_ok!(&function(u32::MAX));

    // Off-by-one
    assert_ok!(&function(1));
    assert_ok!(&function(u32::MAX - 1));
});
```

## When to Use Core Patterns

Use core patterns for:
- ✅ Testing individual functions
- ✅ Testing pure logic
- ✅ Testing with simple setup
- ✅ Most of your test suite (80%+)

For complex scenarios, see:
- [Property-Based Testing](../advanced/property-testing.md) - Random test data
- [Snapshot Testing](../advanced/snapshot-testing.md) - Golden files
- [CLI Testing](../advanced/cli-testing.md) - Command-line interfaces

## Next Steps

👉 **Start with [Fixtures](fixtures.md)**

Then learn:
1. [Data Builders](data-builders.md) - Construct test data
2. [Assertions](assertions.md) - Verify results
3. [Error Paths](error-paths.md) - Test failures
