# Macro Examples

**Category:** Tutorial
**Level:** Beginner
**Prerequisites:** Basic Rust knowledge
**Features Required:** None

---

## Overview

This example demonstrates test and assertion macros provided by Chicago TDD Tools. Macros expand to test functions and enforce AAA pattern.

**What you'll learn:**
- Using `test!` macro for synchronous tests
- Using `assert_ok!` and `assert_err!` for Result handling
- Adding custom messages to assertions
- Available test and assertion macros

---

## Quick Start

```bash
cargo test --example macro_examples
```

**Note:** This example contains tests in `#[cfg(test)]` modules, so use `cargo test` not `cargo run`.

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- No additional features required

---

## Available Macros

### Test Macros

| Macro | Purpose | Usage |
|-------|---------|-------|
| `test!` | Synchronous test | `test!(name, { body })` |
| `async_test!` | Async test | `async_test!(name, { body })` |
| `fixture_test!` | Test with fixture | `fixture_test!(name, fixture, { body })` |
| `performance_test!` | Performance test | `performance_test!(name, { body })` |

### Assertion Macros

| Macro | Purpose | Usage |
|-------|---------|-------|
| `assert_ok!` | Assert Result is Ok | `assert_ok!(&result, "message")` |
| `assert_err!` | Assert Result is Err | `assert_err!(&result, "message")` |
| `assert_in_range!` | Assert value in range | `assert_in_range!(value, min, max)` |
| `assert_eq_msg!` | Assert equality with message | `assert_eq_msg!(left, right, "message")` |
| `assert_within_tick_budget!` | Validate tick budget | `assert_within_tick_budget!(ticks, max)` |
| `assert_guard_constraint!` | Validate guard constraints | `assert_guard_constraint!(guard, constraint)` |

---

## Code Examples

### Example 1: Basic AAA Pattern

```rust
use chicago_tdd_tools::test;

test!(test_basic_aaa_pattern, {
    // Arrange: Set up test data
    let input = 5;
    let expected = 10;

    // Act: Execute feature under test
    let result = input * 2;

    // Assert: Verify behavior
    assert_eq!(result, expected);
});
```

**Key Points:**
- `test!` macro enforces AAA pattern
- Expands to standard `#[test]` function
- Readable, maintainable test structure

### Example 2: Result Handling

```rust
use chicago_tdd_tools::test;

test!(test_result_handling, {
    // Arrange: Create a Result
    let result: Result<u32, String> = Ok(42);

    // Act & Assert: Verify Result is Ok and check value
    chicago_tdd_tools::assert_ok!(&result, "Result should be Ok");
    if let Ok(value) = result {
        assert_eq!(value, 42, "Value should be 42");
    }
});
```

**Key Points:**
- `assert_ok!` provides better error messages than manual checks
- Use reference `&result` to avoid moving the value
- Optional custom message parameter

### Example 3: Error Handling

```rust
use chicago_tdd_tools::test;

test!(test_error_handling, {
    // Arrange: Create an error Result
    let result: Result<u32, String> = Err("test error".to_string());

    // Act & Assert: Verify Result is Err
    chicago_tdd_tools::assert_err!(&result, "Result should be Err");
});
```

### Example 4: Custom Messages

```rust
use chicago_tdd_tools::test;

test!(test_with_custom_message, {
    // Arrange: Set up test data
    let value = 42;
    let expected = 42;

    // Act & Assert: Verify with custom message
    assert_eq!(value, expected, "Value should equal expected");
});
```

---

## Macro Details

### test! Macro

**Syntax:**
```rust
test!(test_name, {
    // Test body
});
```

**Expands to:**
```rust
#[test]
fn test_name() {
    // Test body
}
```

### assert_ok! Macro

**Syntax:**
```rust
assert_ok!(&result);
assert_ok!(&result, "custom message");
```

**Behavior:**
- Panics if result is `Err`
- Provides detailed error message
- Takes reference to avoid moving value

### assert_err! Macro

**Syntax:**
```rust
assert_err!(&result);
assert_err!(&result, "custom message");
```

**Behavior:**
- Panics if result is `Ok`
- Provides detailed error message
- Takes reference to avoid moving value

---

## Common Patterns

### Pattern 1: Simple Test

```rust
test!(test_addition, {
    assert_eq!(2 + 2, 4);
});
```

### Pattern 2: Result Validation

```rust
test!(test_with_result, {
    let result: Result<i32, String> = Ok(42);
    chicago_tdd_tools::assert_ok!(&result);
    if let Ok(value) = result {
        assert_eq!(value, 42);
    }
});
```

### Pattern 3: Error Path Testing

```rust
test!(test_error_path, {
    let result: Result<(), String> = Err("error".to_string());
    chicago_tdd_tools::assert_err!(&result, "Should fail");
});
```

---

## Macro Usage Notes

### Important: Macro Exports

Macros are exported with `#[macro_export]`, so they're available at crate root automatically.

```rust
// ✓ Correct: Use macro directly
assert_ok!(result);
test!(my_test, { /* body */ });

// ✗ Incorrect: Don't import macros (causes unused import error)
// use chicago_tdd_tools::assert_ok;  // Unused import
```

### Macro Availability

All macros are available without explicit import when using prelude:

```rust
use chicago_tdd_tools::prelude::*;

// All macros available:
test!(...);
async_test!(...);
assert_ok!(...);
assert_err!(...);
```

---

## Troubleshooting

### Error: "cannot find macro `test!`"

**Cause:** Missing prelude import

**Fix:**
```rust
use chicago_tdd_tools::prelude::*;
```

### Error: "unused import: `chicago_tdd_tools::assert_ok`"

**Cause:** Macros don't need imports (exported with `#[macro_export]`)

**Fix:** Remove the import, use macro directly:
```rust
// Remove this:
// use chicago_tdd_tools::assert_ok;

// Just use:
chicago_tdd_tools::assert_ok!(&result);
```

### Error: "expected expression, found `test`"

**Cause:** Wrong syntax for test macro

**Fix:**
```rust
// ✓ Correct
test!(test_name, {
    // body
});

// ✗ Wrong
test! test_name {
    // body
}
```

---

## Next Steps

After mastering macros, explore:

1. **[Basic Test](basic_test.md)** - Apply macros to practical testing
2. **[Property Testing](property_testing.md)** - Use macros with property-based tests
3. **[Fixture Testing](../docs/getting-started/USER_GUIDE.md)** - Advanced fixture patterns

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete macro documentation
- [Quick Guide](../docs/getting-started/QUICK_GUIDE.md) - Essential patterns

---

## Reference

### Test Macros

- `test!(name, { body })` - Synchronous test
- `async_test!(name, { body })` - Async test
- `fixture_test!(name, fixture, { body })` - Test with fixture
- `performance_test!(name, { body })` - Performance test

### Assertion Macros

- `assert_ok!(&result, message?)` - Assert Result is Ok
- `assert_err!(&result, message?)` - Assert Result is Err
- `assert_in_range!(value, min, max)` - Assert value in range
- `assert_eq_msg!(left, right, message)` - Assert equality with message
- `assert_within_tick_budget!(ticks, max)` - Validate tick budget
- `assert_guard_constraint!(guard, constraint)` - Validate guards

---

**Quality is the default. Prevention beats detection.**

*Example: macro_examples.rs | Version: 1.2.0 | Updated: 2025-11-15*
