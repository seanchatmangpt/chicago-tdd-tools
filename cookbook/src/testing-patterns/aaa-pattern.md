# Pattern 1: AAA Pattern

> 🔧 How-to

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Tests that mix setup, execution, and verification hide intent and make failures hard to diagnose |
| **Solution** | Divide every test into three explicit phases: Arrange (setup), Act (execute), Assert (verify) |
| **When to Use** | All unit and integration tests to structure test logic clearly |
| **When NOT to Use** | Property-based tests (different structure), complex multi-stage workflows (use fixtures) |
| **Trade-offs** | Adds slight setup verbosity (explicit labels) but greatly improves debugging speed |
| **Complexity** | Low |
| **Real-World Example** | [examples/basic_test.rs](file:///Users/sac/chicago-tdd-tools/examples/basic_test.rs) |

## The Problem

Tests that mix setup, execution, and verification hide intent and make failures hard to diagnose. When tests fail, you need to know instantly: Was the code broken, or was the test wrong?

## The Solution

Divide every test into **three explicit phases**:

1. **Arrange** - Set up test data and dependencies
2. **Act** - Execute the one behavior being tested
3. **Assert** - Verify the result

Each phase has one job. This structure makes failures obvious.

## Essential Code Example

```rust
use chicago_tdd_tools::prelude::*;

test!(test_scaling_multiplier, {
    // Arrange: Set up test data
    let multiplier = 3;
    let input = 7;

    // Act: Execute the behavior
    let result = multiplier * input;

    // Assert: Verify the result
    assert_eq!(result, 21);
});
```

## Implementation Checklist

- [ ] Three phases clearly labeled with comments
- [ ] Arrange: All setup done before Act
- [ ] Act: Exactly one function/method call
- [ ] Assert: Specific assertions (not vague conditionals)
- [ ] One behavior per test
- [ ] Test name describes what's tested

## The Gotcha (Most Common Mistake)

Mixing Arrange and Act makes it impossible to know what you're testing:

```rust
// ❌ WRONG: Can't tell what's being tested
test!(test_bad, {
    let result = setup_and_process(100);  // Is this Arrange? Act? Both?
    assert_eq!(result, 200);
});

// ✅ RIGHT: Crystal clear intent
test!(test_good, {
    let value = 100;              // Arrange: Set up
    let result = process(value);  // Act: Execute
    assert_eq!(result, 200);      // Assert: Verify
});
```

**Why**: When tests fail, you need to know which phase broke. If Arrange and Act are mixed, you can't tell.

## Real-World Example

- **Code location**: [examples/basic_test.rs](file:///Users/sac/chicago-tdd-tools/examples/basic_test.rs) and [tests/go_extra_mile_tests.rs](file:///Users/sac/chicago-tdd-tools/tests/go_extra_mile_tests.rs)
- **Explanation**: The three-phase AAA structure is used to assert scaling factors and message validations cleanly, where comments explicitly separate the setup (`// Arrange`), execution (`// Act`), and verification (`// Assert`).

## Related Patterns

- **Foundation**: Start here (prerequisite for all other patterns)
- **Next**: [Pattern 2: Error Path Testing](error-path-testing.md) (test both success + failure)
- **Use with**: [Pattern 5: Real Collaborators](real-collaborators.md) (test with real code)

---

**Why It Works**: Explicit phases make intent obvious. When a test fails, you immediately know which phase broke instead of debugging mixed setup and execution.

**Production Checklist**:
- [ ] Every test has commented Arrange/Act/Assert labels
- [ ] Each test tests exactly one behavior
- [ ] No setup functions that hide what's being tested
- [ ] Assert phase is specific (exact values, not ranges)
