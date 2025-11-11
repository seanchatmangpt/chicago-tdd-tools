# Pattern 13: Sealed Traits for API Safety

## Context

A trait defines extension points (fixtures, validators) but external implementations could break invariants.

## Problem

If downstream crates implement the trait arbitrarily, the framework cannot guarantee lifecycle management or error semantics. Breaking changes become impossible.

## Solution

Use the sealed trait pattern: define a `private` module with a `Sealed` trait implemented only within the crate, and require `Sealed` as a supertrait. Consumers can use the trait, but only framework-defined implementations exist.

## Forces

- Safety vs. openness: sealing protects invariants while letting users compose functionality
- Flexibility vs. versioning: internal changes become possible without breaking downstream code
- Testability vs. encapsulation: tests can still construct fixtures via provided builders

## Examples

```rust
mod private {
    pub trait Sealed {}
}

pub trait AsyncFixtureProvider: private::Sealed {
    // ...
}
```

## Related Patterns

- Pattern 12: Type Safety with GATs
- Pattern 17: Builder-Driven Test Data
- Pattern 20: Macro Pattern Enforcement
