# Pattern 15: Type State Enforcement

## Context

An API has a prescribed call order (Arrange → Act → Assert) or requires configuration before use.

## Problem

Runtime enforcement of order relies on documentation and can be bypassed, leading to inconsistent state and test flakiness.

## Solution

Model the phases as distinct types and use `PhantomData` to encode the current phase. Methods consume `self` and return the next state, making it impossible to call methods out of order. Chicago TDD Tools uses this to enforce AAA semantics internally.

## Forces

- Safety vs. ergonomic: type transitions should read naturally without verbose syntax
- Flexibility vs. constraints: provide escape hatches only when absolutely necessary
- Zero-cost vs. clarity: type state should erase at compile time

## Examples

```rust
pub struct TestState<Phase> {
    context: Context,
    _phase: PhantomData<Phase>,
}

impl TestState<Arrange> {
    pub fn act(self) -> TestState<Act> { /* ... */ }
}
```

## Related Patterns

- Pattern 1: AAA Pattern
- Pattern 11: Zero-Cost Abstractions
- Pattern 14: Compile-Time Validation
