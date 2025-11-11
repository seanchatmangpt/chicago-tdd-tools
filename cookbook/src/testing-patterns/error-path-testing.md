# Pattern 2: Error Path Testing

## Context

A function returns `Result` or `Option`, and the happy path already behaves as expected. You must guarantee that every failure mode is observable and carries actionable context.

## Problem

When tests only exercise the success path, regressions sneak in through unhandled errors, missing context, or broken guardrails. Production discovers the failure first, and the team spends time reconstructing the scenario.

## Solution

Enumerate every documented error variant and write a focused test for each. Use `test!` or `param_test!` to drive the error case, assert the specific variant, and check that error messages include the necessary context. Prefer direct construction or builders over mocks so that you validate the real failure.

## Forces

- Coverage vs. duplication: each error merits a separate assertion without creating noise
- Realistic behavior vs. test speed: use in-memory collaborators where possible, but ensure pathways stay intact
- Diagnostic clarity vs. maintenance: error text should be specific yet stable

## Examples

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::core::fixture::FixtureError;

param_test! {
    #[case("", FixtureError::CreationFailed("name required".into()))]
    #[case("db://unreachable", FixtureError::OperationFailed("reconnect failed".into()))]
    fn test_fixture_error_paths(input: &str, expected: FixtureError) {
        let result = TestFixture::with_data(input.to_string()).cleanup();
        assert_err!(&result);
        let error = result.unwrap_err();
        assert_eq!(format!("{}", error), format!("{}", expected));
    }
}
```

## Related Patterns

- Pattern 3: Boundary Conditions
- Pattern 18: Timeout Defense in Depth
- Pattern 19: Feature Gate Slices
