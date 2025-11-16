# Pattern 1: AAA Pattern

> 🔧 **HOW-TO** | Structure every test with Arrange-Act-Assert clarity

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Tests that intermingle setup, behavior, and assertions become hard to read and debug |
| **Core Solution** | Divide test into three explicit phases: Arrange, Act, Assert |
| **When to Use** | ✅ All unit tests, ✅ Integration tests, ✅ Even simple assertions |
| **When NOT to Use** | ❌ Property-based tests (different structure), ❌ Complex multi-stage workflows (use fixtures) |
| **Difficulty** | Low - Easy to learn and apply immediately |
| **Trade-offs** | Slight verbosity for clarity |
| **Related Patterns** | Error Path Testing, Real Collaborators, Fixture Lifecycle |

## Context

You are writing or reviewing a test in Chicago TDD Tools. You want the test to communicate intent instantly and fail with a precise message when behavior regresses.

## Problem

Tests that intermingle setup, behavior, and assertions become hard to scan. When the failure occurs, teammates must untangle implicit state, which slows feedback and hides missing assertions.

## Solution

Structure every test body into three explicit phases – **Arrange**, **Act**, **Assert** – and let the framework enforce it. Use the `test!`, `async_test!`, or `fixture_test!` macros so that comments and code read in a top-to-bottom narrative.

## Forces

- Readability vs. flexibility: expressive labels without duplicating boilerplate
- Fast diagnosis vs. runtime overhead: compile-time enforcement should cost nothing
- Behavior proof vs. implementation detail: assertions must verify observable outcomes

## Examples

### Basic Example: Simple Calculation

```rust
use chicago_tdd_tools::prelude::*;

test!(test_scaling_multiplier, {
    // Arrange: Set up test data
    let multiplier = 3;
    let input = 7;

    // Act: Execute the behavior
    let result = multiplier * input;

    // Assert: Verify the result
    assert_eq!(result, 21, "multiplier should scale the input");
});
```

### Async Example: Fetching Data

```rust
use chicago_tdd_tools::prelude::*;

async_test!(test_fetch_customer, {
    // Arrange: Set up dependencies
    let client = FakeCrmClient::connected();

    // Act: Perform async operation
    let customer = client.fetch("cust-123").await?;

    // Assert: Verify outcome
    assert_eq!(customer.id, "cust-123");
    Ok(())
});
```

### With Fixtures: Complex Setup

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::fixture::*;

fixture_test!(test_user_creation, fixture, {
    // Arrange: Create fixture and prepare state
    fixture.set_metadata("user_type", "admin");
    let db = fixture.database();

    // Act: Execute business logic
    let user = db.create_user("alice", "admin")?;

    // Assert: Verify the result
    assert_eq!(user.name, "alice");
    assert_eq!(user.role, "admin");
});
```

## Implementation Checklist

- [ ] **Arrange phase** clearly sets up all necessary test data
- [ ] **Act phase** calls the function or method exactly once
- [ ] **Assert phase** verifies the result with specific assertions
- [ ] Comments label each phase (even if obvious)
- [ ] Test name describes what is being tested
- [ ] Each test tests one behavior
- [ ] No logic in assertions (e.g., no `if` statements)

## Common Mistakes & How to Avoid Them

### ❌ Mistake 1: Mixing Arrange and Act

```rust
// DON'T: Hidden state between phases
test!(test_bad_mixing, {
    let result = setup_and_call();  // Mixes arrange + act
    assert_eq!(result, 42);
});
```

```rust
// DO: Clear separation
test!(test_good_mixing, {
    // Arrange
    let input = 42;

    // Act
    let result = calculate(input);

    // Assert
    assert_eq!(result, 42);
});
```

### ❌ Mistake 2: Multiple Assertions Without Context

```rust
// DON'T: No comment explaining what each asserts
test!(test_bad_assertions, {
    // ... arrange and act ...
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice");
    assert!(user.is_active);
});
```

```rust
// DO: Clear what each assertion verifies
test!(test_good_assertions, {
    // ... arrange and act ...

    // Assert: User created with correct ID
    assert_eq!(user.id, 1);

    // Assert: User name matches input
    assert_eq!(user.name, "Alice");

    // Assert: User is active by default
    assert!(user.is_active);
});
```

### ❌ Mistake 3: Act Phase Doing Multiple Things

```rust
// DON'T: Multiple behaviors in Act
test!(test_bad_act, {
    let user = db.create_user("alice");
    user.send_welcome_email();  // Second behavior!
    assert!(user.email_sent);
});
```

```rust
// DO: One behavior per test
test!(test_user_creation, {
    let user = db.create_user("alice");
    assert_eq!(user.name, "alice");
});

test!(test_welcome_email, {
    let user = db.create_user("alice");
    user.send_welcome_email();
    assert!(user.email_sent);
});
```

## Real-World Example

The AAA pattern is foundational to Chicago TDD Tools. Every example test follows this structure, found throughout:
- `examples/basic_test.rs` - Simple unit tests
- `tests/common.rs` - Shared test utilities
- Test suites in the source code

## Advanced: Multiple Assertions in Assert Phase

✅ **Allowed**: Multiple assertions that verify the same behavior:

```rust
test!(test_user_properties, {
    // Arrange
    let user_data = vec!["alice", "25", "alice@example.com"];

    // Act
    let user = parse_user_csv(&user_data)?;

    // Assert: All properties of the user
    assert_eq!(user.name, "alice");
    assert_eq!(user.age, 25);
    assert_eq!(user.email, "alice@example.com");
    assert!(user.is_valid());
});
```

This is acceptable because all assertions verify related properties of the same object.

## Related Patterns

- **Pattern 2: Error Path Testing** - Same AAA structure for error cases
- **Pattern 3: Boundary Conditions** - AAA structure for edge cases
- **Pattern 5: Real Collaborators** - AAA with actual dependencies
- **Pattern 16: Fixture Lifecycle Management** - AAA with fixtures for complex setup
- **Pattern 17: Builder-Driven Test Data** - AAA with fluent builders for Arrange phase

## Next Steps

Learn these related patterns to master test structure:

1. **Next**: [Pattern 2: Error Path Testing](error-path-testing.md) - How to test failure cases
2. **Then**: [Pattern 5: Real Collaborators](real-collaborators.md) - Testing with actual dependencies
3. **Advanced**: [Pattern 16: Fixture Lifecycle](../design-patterns/fixture-lifecycle.md) - Complex multi-phase tests

---

## Summary

The AAA pattern is the foundation of readable, maintainable tests. By consistently separating **Arrange**, **Act**, and **Assert**, you create tests that:

✅ Are easy to read and understand
✅ Fail with clear messages about what went wrong
✅ Scale from simple to complex tests
✅ Can be easily maintained by teammates

**Pro tip**: Use comments to label each phase, even in simple tests. It takes one line and makes the intent crystal clear.
