# Pattern 12: Type Safety with GATs

## Context

You provide fixtures or builders that return references tied to the fixture lifetime.

## Problem

Without explicit lifetimes, consumers can hold onto references after cleanup, causing dangling pointers or logic bugs.

## Solution

Use Generic Associated Types (GATs) to bind returned data to the fixture lifetime. In `AsyncFixtureProvider`, `Fixture<'a>` ensures the borrow cannot outlive the provider. Pair GATs with sealed traits to prevent downstream crates from violating invariants.

## Forces

- Safety vs. ergonomics: GATs constrain lifetimes but keep APIs pleasant
- Extensibility vs. soundness: sealing the trait permits internal evolution while preserving invariants
- Async vs. sync: async fixtures require lifetimes that sync code cannot express without GATs

## Examples

```rust
pub trait AsyncFixtureProvider: private::Sealed {
    type Fixture<'a>: Send where Self: 'a;
    type Error: std::error::Error + Send + Sync + 'static;

    fn create_fixture<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<Self::Fixture<'a>, Self::Error>> + Send + 'a>>;
}
```

## Related Patterns

- Pattern 13: Sealed Traits for API Safety
- Pattern 16: Fixture Lifecycle Management
- Pattern 17: Builder-Driven Test Data
