# Pattern 3: Boundary Conditions

## Context

Logic depends on limits: minimum quantities, maximum run lengths, exclusive ranges. You must guarantee behavior at and around those boundaries.

## Problem

Happy-path tests routinely miss off-by-one errors, buffer limits, or guardrail regressions. Without explicit boundary coverage, downstream invariants fail silently.

## Solution

Adopt a deliberate boundary grid: **below**, **at**, and **above** each documented limit. Use `param_test!` to table-drive the grid and Chicago TDD Tools assertions such as `assert_in_range!` and `assert_guard_constraint!`. Include boundary-focused names so a failing case communicates which edge is breached.

## Forces

- Thoroughness vs. duplication: table-driven tests keep boundaries concise
- Performance vs. realism: small data sets reproduce the failure quickly
- Maintainability vs. specificity: boundary constants should live in one place

## Examples

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::validation::guards::{GuardValidator, MAX_RUN_LEN};

param_test! {
    #[case("below", MAX_RUN_LEN - 1, true)]
    #[case("at", MAX_RUN_LEN, true)]
    #[case("above", MAX_RUN_LEN + 1, false)]
    fn test_run_length_boundaries(label: &str, length: usize, expected_ok: bool) {
        let validator = GuardValidator::new();
        let result = validator.validate_run_length(length);
        match expected_ok {
            true => assert_ok!(&result, "{label} should be accepted"),
            false => assert_err!(&result, "{label} should be rejected"),
        }
    }
}
```

## Related Patterns

- Pattern 2: Error Path Testing
- Pattern 4: Resource Cleanup
- Pattern 20: Macro Pattern Enforcement
