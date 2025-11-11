# Pattern 14: Compile-Time Validation

## Context

Invariants (tick budgets, guard limits, feature combinations) should fail fast during compilation.

## Problem

Runtime checks add overhead and can be bypassed in rarely executed code paths. Missing an invariant leads to subtle production bugs.

## Solution

Push validation to compile time with const generics, type-level markers, and `const_assert!`. When runtime validation is unavoidable, encapsulate it in constructors that return `Result`, making misuse impossible through types.

## Forces

- Safety vs. flexibility: some values remain runtime, but defaults should be encoded in types
- Compile time vs. ergonomics: const generics expose parameters without macros
- Diagnostics vs. noise: compile errors must explain the invariant succinctly

## Examples

```rust
pub const fn assert_tick_budget(ticks: u64) {
    const_assert!(ticks <= 8);
}

pub struct SizeValidatedArray<const SIZE: usize, const MAX: usize> {
    data: [u8; SIZE],
    _marker: PhantomData<[u8; MAX]>,
}
```

## Related Patterns

- Pattern 11: Zero-Cost Abstractions
- Pattern 15: Type State Enforcement
- Pattern 18: Timeout Defense in Depth
