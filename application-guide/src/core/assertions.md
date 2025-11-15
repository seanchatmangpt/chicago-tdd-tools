# Assertions & Verification

Assertions verify that code behaves correctly. Chicago TDD Tools provides helpers for clear, readable assertions.

## Basic Assertions

### Standard Assertions

```rust
use chicago_tdd_tools::prelude::*;

test!(test_basic_assertions, {
    let result: Result<u32, String> = Ok(42);

    // ✅ Chicago TDD style - clear intent
    assert_ok!(&result);
    assert_eq!(result.unwrap(), 42);

    // Error result
    let error: Result<u32, String> = Err("failed".to_string());
    assert_err!(&error);
});
```

### Common Assertion Helpers

```rust
// Success/Error checks
assert_ok!(&result);           // Verifies Ok(_)
assert_err!(&result);          // Verifies Err(_)

// Equality
assert_eq!(actual, expected);   // Equality check
assert_ne!(actual, expected);   // Inequality check

// Boolean
assert!(condition);             // Checks true
assert!(!condition);            // Checks false
```

## Numeric Assertions

### Range Checking

```rust
test!(test_numeric_assertions, {
    let value = 42;

    // Standard assertions
    assert_eq!(value, 42);
    assert!(value > 40);
    assert!(value < 50);

    // Chicago TDD helper
    assert_in_range!(value, 40, 50);  // Inclusive range
});
```

### Floating Point

```rust
test!(test_floating_point, {
    let value: f64 = 3.14159;
    let expected: f64 = 3.14;

    // ✅ Epsilon comparison (handles rounding errors)
    assert!((value - expected).abs() < 0.01);
});
```

## String Assertions

```rust
test!(test_string_assertions, {
    let text = "Hello, World!";

    // Equality
    assert_eq!(text, "Hello, World!");

    // Containment
    assert!(text.contains("World"));

    // Pattern matching
    assert!(text.starts_with("Hello"));
    assert!(text.ends_with("!"));
});
```

## Collection Assertions

```rust
test!(test_collection_assertions, {
    let vec = vec![1, 2, 3];

    // Length
    assert_eq!(vec.len(), 3);

    // Containment
    assert!(vec.contains(&2));

    // Empty check
    assert!(!vec.is_empty());
});
```

## Option/Result Assertions

```rust
test!(test_option_result_assertions, {
    let some_value: Option<i32> = Some(42);
    let none_value: Option<i32> = None;

    // Option checks
    assert!(some_value.is_some());
    assert!(none_value.is_none());

    // Result checks
    let ok_result: Result<i32, String> = Ok(42);
    let err_result: Result<i32, String> = Err("error".to_string());

    assert!(ok_result.is_ok());
    assert!(err_result.is_err());
});
```

## Custom Messages

Add context to assertions:

```rust
test!(test_with_messages, {
    let result = divide(10, 2);
    assert_eq!(result, 5, "Expected division to work correctly");

    let empty = vec![];
    assert!(!empty.is_empty(), "Vector should not be empty after initialization");
});
```

## Real-World Example: User Creation

```rust
test!(test_user_creation, {
    // Create user
    let user_data = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("email", "alice@example.com")
        .build_json()?;

    let result = create_user(&user_data)?;

    // Assert success
    assert_ok!(&result);
    let user = result.unwrap();

    // Assert properties
    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
    assert!(user.id > 0);
    assert!(user.created_at.len() > 0);
});
```

## Real-World Example: Collection Processing

```rust
test!(test_filter_operations, {
    let numbers = vec![1, 2, 3, 4, 5];

    // Filter evens
    let evens: Vec<_> = numbers
        .iter()
        .filter(|n| n % 2 == 0)
        .cloned()
        .collect();

    // Assertions
    assert_eq!(evens.len(), 2);
    assert!(evens.contains(&2));
    assert!(evens.contains(&4));
    assert!(!evens.contains(&1));  // 1 is odd
});
```

## Assert Patterns

### Pattern: Positive Case

```rust
test!(test_positive, {
    let result = parse_number("42");
    assert_ok!(&result);
    assert_eq!(result.unwrap(), 42);
});
```

### Pattern: Negative Case

```rust
test!(test_negative, {
    let result = parse_number("invalid");
    assert_err!(&result);
});
```

### Pattern: Boundary Case

```rust
test!(test_boundary, {
    // Zero
    assert_ok!(&parse_number("0"));

    // Large value
    assert_ok!(&parse_number("999999"));

    // Negative
    assert_ok!(&parse_number("-42"));
});
```

## Avoiding Common Pitfalls

### ❌ Don't: Use unwrap() in assertions

```rust
// Bad - panics if result is Err
let value = result.unwrap();
assert_eq!(value, 42);
```

### ✅ Do: Check first

```rust
// Good - checks properly
assert_ok!(&result);
if let Ok(value) = result {
    assert_eq!(value, 42);
}
```

### ❌ Don't: Assert implementation details

```rust
// Bad - depends on internal structure
assert_eq!(user.internal_id, 123);
```

### ✅ Do: Assert behavior

```rust
// Good - asserts external behavior
assert_eq!(user.name, "Alice");
assert_eq!(user.email, "alice@example.com");
```

## Assertion Order

Follow AAA pattern:

1. **Arrange** - Build data
2. **Act** - Execute code
3. **Assert** - Verify behavior

```rust
test!(test_order, {
    // Arrange
    let input = 5;

    // Act
    let result = multiply_by_two(input);

    // Assert (all together, at the end)
    assert_eq!(result, 10);
    assert!(result > 0);
    assert!(result < 100);
});
```

## Best Practices

✅ **Do:**
- Use helper macros (`assert_ok!`, `assert_err!`)
- Assert one behavior per test
- Add context with messages
- Check both success and error paths
- Use descriptive assertion messages

❌ **Don't:**
- Use `unwrap()` in assertions
- Mix act and assert
- Assert implementation details
- Skip error case assertions
- Use vague assertion messages

## Common Assertions Reference

| Pattern | Code |
|---------|------|
| Value matches | `assert_eq!(actual, expected)` |
| Value different | `assert_ne!(actual, expected)` |
| True condition | `assert!(condition)` |
| Ok result | `assert_ok!(&result)` |
| Err result | `assert_err!(&result)` |
| In range | `assert_in_range!(value, min, max)` |
| Contains text | `assert!(text.contains("substring"))` |
| Empty collection | `assert!(collection.is_empty())` |

## Next Steps

Learn about error paths: [Error Path Testing](error-paths.md)

---

## Summary

Chicago TDD emphasizes:
- Clear assertions with helper macros
- Both success and error case verification
- AAA pattern (Arrange-Act-Assert)
- Behavior verification, not implementation details

