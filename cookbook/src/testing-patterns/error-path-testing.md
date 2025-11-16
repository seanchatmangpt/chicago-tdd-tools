# Pattern 2: Error Path Testing

> 🔧 **HOW-TO** | Test every error case to catch regressions before production

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Happy path tests miss error handling bugs; production discovers failures first |
| **Core Solution** | Write one test per error variant; assert both the error type and error message |
| **When to Use** | ✅ Functions returning Result/Option, ✅ Every documented error variant, ✅ Integration tests |
| **When NOT to Use** | ❌ Recovery scenarios (use separate tests), ❌ Non-deterministic failures (use timeouts instead) |
| **Difficulty** | Medium - Requires understanding error types |

## The Problem

Tests that only exercise the happy path miss error handling bugs. When functions fail, missing context or wrong error types cause production outages. The team wastes time reconstructing the scenario instead of fixing the problem.

## The Solution

For each documented error variant, write one focused test that verifies the exact error type and includes proper context. Use parametrized tests to cover multiple error cases efficiently.

## Essential Code Example

```rust
use chicago_tdd_tools::prelude::*;

test!(test_invalid_input_error, {
    // Arrange
    let invalid_input = "";

    // Act
    let result = validate_and_process(invalid_input);

    // Assert: Error variant and message
    assert_err!(&result);
    assert_eq!(result.unwrap_err().kind(), ErrorKind::ValidationFailed);
    assert!(result.unwrap_err().message().contains("empty input"));
});
```

## Implementation Checklist

- [ ] One test per error variant (not one test for all errors)
- [ ] Assert the error type (variant, not just "is error")
- [ ] Assert the error message contains context
- [ ] Use parametrized tests for multiple similar error cases
- [ ] Test both immediate errors and cascading failures
- [ ] Error messages are stable (not implementation details)

## The Gotcha (Most Common Mistake)

Testing that an error occurred without checking the error message or variant:

```rust
// ❌ WRONG: Any error passes, including the wrong one
test!(test_bad_error, {
    let result = validate(invalid_input);
    assert!(result.is_err());  // What error? Who knows!
});

// ✅ RIGHT: Specific error type and message
test!(test_good_error, {
    let result = validate(invalid_input);
    assert_err!(&result);
    assert_eq!(result.unwrap_err().kind(), ValidationError);
    assert!(result.unwrap_err().to_string().contains("empty"));
});
```

**Why**: If you test the wrong error variant, you won't catch regressions where the error type changes unexpectedly.

## Codebase Example

File: `tests/go_extra_mile_tests.rs`
Purpose: Demonstrates error path testing for fixture setup and resource cleanup failures

## Related Patterns

- **Before this**: [Pattern 1: AAA Pattern](aaa-pattern.md) (foundation)
- **Next**: [Pattern 3: Boundary Conditions](boundary-conditions.md) (test edge cases)
- **Use with**: [Pattern 18: Timeout Defense](../design-patterns/timeout-defense.md) (test timeout errors)

---

**Why It Works**: Testing specific error variants prevents regressions where error handling silently breaks. Checking error messages ensures failures are debuggable in production.

**Production Checklist**:
- [ ] Every documented error variant has a test
- [ ] Error messages are checked, not just error type
- [ ] Error scenarios are realistic (not artificial)
- [ ] No sensitive data in error messages
