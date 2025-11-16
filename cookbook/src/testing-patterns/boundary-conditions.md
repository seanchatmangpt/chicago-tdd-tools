# Pattern 3: Boundary Conditions

> 🔧 **HOW-TO** | Test limits to catch off-by-one errors and guardrail regressions

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Off-by-one errors, buffer overflows, missed guardrails silently fail in production |
| **Core Solution** | Test three cases per boundary: below, at, and above the limit |
| **When to Use** | ✅ Array sizes, ✅ Range checks, ✅ Min/max values, ✅ Exclusive boundaries |
| **When NOT to Use** | ❌ Floating-point ranges (use tolerance instead), ❌ Estimated quantities (use ranges) |
| **Difficulty** | Medium - Requires identifying all boundaries first |

## The Problem

Happy-path tests miss off-by-one errors and guardrail regressions. A buffer accepts 100 items but fails at 101. Tests only checked 50 and 100. In production, the 101st request crashes silently.

## The Solution

For each boundary, test three cases: **below** (n-1), **at** (n), and **above** (n+1). Use parametrized tests to keep the grid concise. Each case should have a clear name showing which boundary is being tested.

## Essential Code Example

```rust
use chicago_tdd_tools::prelude::*;

param_test! {
    #[case(9, true, "below max")]
    #[case(10, true, "at max")]
    #[case(11, false, "above max")]
    fn test_message_length_boundary(length: usize, expected_ok: bool, label: &str) {
        let message = "x".repeat(length);
        let result = validate_message(&message);

        match expected_ok {
            true => assert_ok!(&result, "{label}"),
            false => assert_err!(&result, "{label}"),
        }
    }
}
```

## Implementation Checklist

- [ ] Identify all boundary values in the specification
- [ ] For each boundary, test below/at/above cases
- [ ] Use parametrized tests (param_test!) to keep grid concise
- [ ] Each test case has a descriptive label (shows which boundary)
- [ ] Boundary constants are defined once (not duplicated)
- [ ] Include both inclusive and exclusive boundary tests

## The Gotcha (Most Common Mistake)

Testing only the happy path at a round number, missing boundaries entirely:

```rust
// ❌ WRONG: No boundary testing
test!(test_max_items, {
    let result = process_items(vec![1, 2, 3, 4, 5]);  // Safe middle ground
    assert_ok!(&result);
});

// ✅ RIGHT: Test below, at, and above
param_test! {
    #[case(9)]
    #[case(10)]  // MAX
    #[case(11)]
    fn test_max_items_boundary(count: usize) {
        let items = vec![0; count];
        let result = process_items(items);
        match count {
            0..=10 => assert_ok!(&result),
            _ => assert_err!(&result),
        }
    }
}
```

**Why**: Boundary bugs are common (off-by-one) and invisible in happy-path tests. Testing them explicitly prevents production outages.

## Codebase Example

File: `tests/go_extra_mile_tests.rs`
Purpose: Parametrized boundary tests for validation guards and constraints

## Related Patterns

- **Before this**: [Pattern 2: Error Paths](error-path-testing.md) (error cases)
- **Next**: [Pattern 4: Resource Cleanup](resource-cleanup.md) (cleanup after boundary tests)
- **Use with**: [Pattern 20: Macro Enforcement](../design-patterns/macro-enforcement.md) (compile-time boundaries)

---

**Why It Works**: Off-by-one errors are the most common boundary bug. Testing below/at/above catches them before production.

**Production Checklist**:
- [ ] Every numeric limit has below/at/above tests
- [ ] String length limits are tested
- [ ] Array capacity limits are tested
- [ ] Exclusive ranges are clearly tested
- [ ] Boundary violations produce clear error messages
