# Pattern 13: Sealed Traits for API Safety

> 📚 Reference

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Downstream crates implement trait arbitrarily, breaking invariants and blocking future updates |
| **Solution** | Require a private `Sealed` supertrait to prevent external trait implementations |
| **When to Use** | Public traits with internal invariants, lifecycle-critical traits, API evolution paths |
| **When NOT to Use** | Designed for extension (use extension traits), plugin systems (need openness) |
| **Trade-offs** | Prevents downstream crates from implementing the trait, but ensures library safety and simplifies updates |
| **Complexity** | Low |
| **Real-World Example** | [src/core/fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/fixture.rs) |

## The Problem

Public traits allow downstream crates to provide their own implementations. If those implementations break invariants (skipping cleanup, violating lifetimes), the framework can't guarantee correctness. Future breaking changes become impossible without hurting users.

## The Solution

Create an internal `Sealed` trait implemented only by the framework. Require it as a supertrait on public traits. Consumers can still use the trait, but only framework-provided implementations exist. This allows safe evolution without breaking users.

## Essential Code Example

```rust
// Internal module - not re-exported
mod private {
    pub trait Sealed {}  // Implementation restricted to crate
}

// Public trait using Sealed supertrait
pub trait AsyncFixtureProvider: private::Sealed {
    type Fixture<'a>: Send where Self: 'a;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn create_fixture<'a>(&'a self) -> Result<Self::Fixture<'a>, Self::Error>;
}

// Only the framework can implement
impl Sealed for DefaultFixture {}
impl AsyncFixtureProvider for DefaultFixture { /* ... */ }
```

## Implementation Checklist

- [ ] Define `private::Sealed` trait (not re-exported)
- [ ] Public trait requires `Sealed` as supertrait
- [ ] Only framework types implement the trait
- [ ] Documentation explains why the trait is sealed
- [ ] Tests use provided builders, not direct trait implementations
- [ ] Consider eventual unsealing if genuinely useful

## The Gotcha (Most Common Mistake)

Forgetting to make Sealed private or accidentally unsealing it:

```rust
// ❌ WRONG: Sealed is public (defeats purpose)
pub mod private { pub trait Sealed {} }  // Wrong!

// ❌ WRONG: Trait doesn't require Sealed
pub trait AsyncFixtureProvider {  // Anyone can implement
    // ...
}

// ✅ RIGHT: Sealed is truly private, trait requires it
mod private { pub trait Sealed {} }
pub trait AsyncFixtureProvider: private::Sealed { /* ... */ }
```

**Why**: Sealed must be private to prevent downstream implementations. Otherwise the seal is useless.

## Real-World Example

- **Code location**: [src/core/fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/fixture.rs)
- **Explanation**: The `AsyncFixtureProvider` and related traits inherit from a private `private::Sealed` trait so they can only be implemented internally.

## Related Patterns

- **Before this**: [Pattern 12: Type Safety](type-safety-patterns.md) (pair with GATs)
- **Use with**: [Pattern 11: Zero-Cost](zero-cost-abstractions.md) (seal generic APIs)
- **Next**: [Pattern 20: Macro Enforcement](macro-enforcement.md) (enforce at compile time)

---

**Why It Works**: Without access to the `Sealed` trait, downstream crates cannot implement the public trait. Only internal code can.

**Production Checklist**:
- [ ] `Sealed` trait is in private module
- [ ] No path to implement trait from outside crate
- [ ] Documentation explains sealing rationale
- [ ] Tests verify only framework types work
- [ ] Consider unsealing if user demand is genuine
