# Pattern 12: Type Safety with GATs

> 📚 Reference

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | References escape fixture scope; dangling pointers and logic bugs |
| **Solution** | Use Generic Associated Types (GATs) to tie returned data's lifetimes to the fixture provider |
| **When to Use** | Fixtures returning references, async resource providers, lifetime-bound APIs |
| **When NOT to Use** | Owned values only (no lifetime constraints needed), static data (use associated consts) |
| **Trade-offs** | Lifetime constraints make API signatures more complex, but prevent references from escaping scope at compile time |
| **Complexity** | Hard |
| **Real-World Example** | [src/core/fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/fixture.rs) |

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

## Real-World Example

- **Code location**: [src/core/fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/fixture.rs)
- **Explanation**: Tie the lifetimes of temporary directories or mock services to the fixture provider to ensure they cannot outlive the test execution context.

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
