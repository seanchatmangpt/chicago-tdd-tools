# Pattern 11: Zero-Cost Abstractions

## Context

You are designing APIs or extensions and need expressive abstractions without runtime overhead.

## Problem

Runtime polymorphism or heap allocations can slow hot paths. Manual inlining or duplicated code sacrifices readability.

## Solution

Lean on generics, const generics, and macros to express behavior that compiles down to the same machine code as bespoke implementations. Favor references over owned values and prefer stack allocation. When dynamic dispatch is required, isolate it behind narrow traits.

## Forces

- Expressiveness vs. performance: abstractions must be ergonomic without cost
- Safety vs. control: compile-time guarantees should not block optimization
- Maintainability vs. specialization: macros and generics prevent copy-paste

## Examples

```rust
pub fn measure_ticks<F, T>(operation: F) -> (T, u64)
where
    F: FnOnce() -> T,
{
    // Generic function specialized per call site; no dynamic dispatch
    chicago_tdd_tools::validation::performance::measure_ticks(operation)
}
```

## Related Patterns

- Pattern 12: Type Safety with GATs
- Pattern 14: Compile-Time Validation
- Pattern 20: Macro Pattern Enforcement
