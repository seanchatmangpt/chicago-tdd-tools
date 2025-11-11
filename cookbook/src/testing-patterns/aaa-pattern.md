# Pattern 1: AAA Pattern

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

```rust
use chicago_tdd_tools::prelude::*;

test!(test_scaling_multiplier, {
    // Arrange
    let multiplier = 3;
    let input = 7;

    // Act
    let result = multiplier * input;

    // Assert
    assert_eq!(result, 21, "multiplier should scale the input");
});
```

Async tests follow the same skeleton:

```rust
use chicago_tdd_tools::prelude::*;

async_test!(test_fetch_customer, {
    // Arrange
    let client = FakeCrmClient::connected();

    // Act
    let customer = client.fetch("cust-123").await?;

    // Assert
    assert_eq!(customer.id, "cust-123");
    Ok(())
});
```

## Related Patterns

- Pattern 2: Error Path Testing
- Pattern 5: Real Collaborators
- Pattern 11: Zero-Cost Abstractions
