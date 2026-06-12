# Pattern 16: Fixture Lifecycle Management

> 🔧 How-to

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Manual lifecycle logic is error-prone; forgotten teardown cascades to other tests |
| **Solution** | Wrap lifecycle in TestFixture or AsyncFixtureManager; Let Drop/teardown guarantee cleanup |
| **When to Use** | Database connections, containers, async resource setup, temporary directories |
| **When NOT to Use** | Stateless data (no cleanup needed), shared state between tests (use isolation) |
| **Trade-offs** | Adds slight structure overhead, but ensures cleanup even on test failures or panics |
| **Complexity** | Medium |
| **Real-World Example** | [src/core/async_fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/async_fixture.rs) |

## The Problem

Complex tests require deterministic setup and teardown. Manual lifecycle logic is scattered across tests and error-prone. Forgetting teardown causes cascading failures in subsequent tests. Async setup adds complexity with explicit boundaries.

## The Solution

Wrap all setup/teardown in `TestFixture` or `AsyncFixtureManager`. Use `AsyncFixtureProvider` trait to define resource creation. The manager handles both setup and guaranteed teardown, even on test failure.

## Essential Code Example

```rust
use chicago_tdd_tools::core::async_fixture::*;

struct DatabaseProvider;

impl AsyncFixtureProvider for DatabaseProvider {
    type Fixture<'a> = DatabaseHandle;
    type Error = DbError;

    async fn create_fixture<'a>(&'a self) -> Result<Self::Fixture<'a>, Self::Error> {
        DatabaseHandle::connect().await
    }
}

async_test!(test_query, fixture, {
    let db = fixture.get_database().await?;
    let result = db.query("SELECT 1").await?;
    assert_eq!(result, 1);
    Ok(())
});
```

## Implementation Checklist

- [ ] All resources are wrapped in fixtures
- [ ] Setup happens in create/setup methods
- [ ] Teardown is automatic (Drop or explicit teardown())
- [ ] Fresh fixtures per test (no shared state)
- [ ] Async resources have explicit lifecycle boundaries
- [ ] Error messages explain cleanup failures

## The Gotcha (Most Common Mistake)

Manual cleanup after assertions, which doesn't run if assertion fails:

```rust
// ❌ WRONG: Cleanup doesn't run if assertion fails
async_test!(test_bad, {
    let db = Database::connect().await?;
    let result = db.query().await?;
    assert_eq!(result, 42);  // If this fails...
    db.close().await;  // ...this never runs!
});

// ✅ RIGHT: Fixture guarantees cleanup
async_test!(test_good, fixture, {
    let db = fixture.get_database().await?;
    let result = db.query().await?;
    assert_eq!(result, 42);  // Cleanup happens regardless
    Ok(())
});
```

**Why**: Explicit cleanup is bypassed by early returns and panics. Fixtures guarantee cleanup via Drop.

## Real-World Example

- **Code location**: [src/core/async_fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/async_fixture.rs)
- **Explanation**: The `AsyncFixtureManager` orchestrates the setup and teardown lifecycle of asynchronous resources automatically.

## Related Patterns

- **Before this**: [Pattern 4: Resource Cleanup](../testing-patterns/resource-cleanup.md) (basic cleanup)
- **Use with**: [Pattern 12: Type Safety](type-safety-patterns.md) (GATs for lifetimes)
- **Next**: [Pattern 18: Timeout Defense](timeout-defense.md) (timeout long-running fixtures)

---

**Why It Works**: Fixtures hold resources and guarantee cleanup via Drop, even on early return or panic.

**Production Checklist**:
- [ ] Every resource-allocating test uses a fixture
- [ ] No manual cleanup code
- [ ] Async resources have explicit setup/teardown
- [ ] Tests run in any order without pollution
- [ ] Cleanup failures are reported clearly
