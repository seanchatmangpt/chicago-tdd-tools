# Pattern 12: Type Safety with GATs

> 🔧 **HOW-TO** | Use Generic Associated Types to bind lifetimes to fixtures

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | References escape fixture scope; dangling pointers and logic bugs |
| **Core Solution** | Generic Associated Types (GATs) bind returned data to fixture lifetime |
| **When to Use** | ✅ Fixtures with references, ✅ Async providers, ✅ Lifetime-bound APIs |
| **When NOT to Use** | ❌ Owned values only (no lifetime constraints needed), ❌ Static data (use associated consts) |
| **Difficulty** | Hard - Requires understanding GATs and lifetimes |

## The Problem

Fixtures allocate resources and return references. Without explicit lifetime constraints, consumers can hold references after cleanup, causing dangling pointers or use-after-free bugs.

## The Solution

Use Generic Associated Types (GATs) to encode that returned fixtures are bound to the provider's lifetime. The type system then prevents dangling references. Pair with sealed traits to prevent unsafe downstream implementations.

## Essential Code Example

```rust
pub trait AsyncFixtureProvider: private::Sealed {
    // Fixture's lifetime is tied to self's lifetime
    type Fixture<'a>: Send where Self: 'a;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn create_fixture<'a>(
        &'a self
    ) -> Result<Self::Fixture<'a>, Self::Error>;
}

// Usage: fixture cannot outlive the provider
async fn test_with_fixture(provider: &MyFixture) {
    let fixture = provider.create_fixture().await.unwrap();
    // fixture is bound to provider's lifetime
    // compile error if you try to return fixture from function
}
```

## Implementation Checklist

- [ ] GATs tie returned lifetime to provider lifetime
- [ ] Compile error if reference outlives provider
- [ ] Sealed trait prevents unsafe downstream implementations
- [ ] Lifetimes are documented
- [ ] Tests verify lifetime constraints work
- [ ] Error types are static (independent of lifetime)

## The Gotcha (Most Common Mistake)

Forgetting to tie the lifetime to `Self`:

```rust
// ❌ WRONG: Lifetime not tied to self
type Fixture<'a>;  // 'a is free; can outlive provider

// ✅ RIGHT: Lifetime bound to self
type Fixture<'a> where Self: 'a;  // 'a cannot exceed self's lifetime
```

**Why**: Without the `where Self: 'a` bound, the lifetime parameter is unconstrained. References can escape.

## Codebase Example

File: `src/core/fixture.rs`
Purpose: Defines AsyncFixtureProvider with GATs binding lifetimes

## Related Patterns

- **Before this**: [Pattern 13: Sealed Traits](sealed-traits.md) (prevent unsafe implementations)
- **Use with**: [Pattern 11: Zero-Cost](zero-cost-abstractions.md) (GATs compile away)
- **Next**: [Pattern 16: Fixture Lifecycle](fixture-lifecycle.md) (manage complex fixtures)

---

**Why It Works**: The Rust compiler enforces lifetime constraints. If a reference is bound to a fixture, you cannot use it after the fixture drops.

**Production Checklist**:
- [ ] All fixture references are lifetime-bound
- [ ] Compiler prevents dangling references
- [ ] Error types don't leak lifetimes
- [ ] Documentation explains lifetime constraints
- [ ] Tests verify lifetime safety
