# Pattern 4: Resource Cleanup

> 🔧 **HOW-TO** | Use fixtures to ensure resources are released, even when tests fail

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Leaked containers, orphaned connections, state pollution between tests |
| **Core Solution** | Use `fixture_test!`/`fixture_test_with_timeout!` with RAII to cleanup automatically |
| **When to Use** | ✅ Database connections, ✅ Docker containers, ✅ File handles, ✅ Network ports |
| **When NOT to Use** | ❌ Simple data (no cleanup needed), ❌ Stateless services (no state to clean) |
| **Difficulty** | Low - Framework handles most cleanup automatically |

## The Problem

Manual cleanup logic scattered across tests is easy to skip. If an assertion fails, cleanup never runs, leaking containers or connections. Tests fail intermittently due to port conflicts or database locks from previous test runs.

## The Solution

Use the fixture's RAII guarantees. Wrap resource allocation inside `fixture_test!` or `fixture_test_with_timeout!`. The framework automatically calls Drop on resources when the test ends, even on assertion failure.

## Essential Code Example

```rust
use chicago_tdd_tools::prelude::*;

fixture_test!(test_database_roundtrip, fixture, {
    // Arrange: Fixture allocates database connection
    let db = fixture.get_database()?;

    // Act: Execute query
    let result = db.query("SELECT COUNT(*) FROM users")?;

    // Assert: Verify result
    assert_eq!(result, 0);

    // Cleanup: Automatic when fixture drops (even if assertion fails)
    Ok(())
});
```

## Implementation Checklist

- [ ] Use `fixture_test!` or `fixture_test_with_timeout!` for resource allocation
- [ ] Store resource handles in the fixture, not local variables
- [ ] Resource Drop implementations are called when test ends
- [ ] Resources are released even if assertions fail
- [ ] Tests run in isolation (no state leakage between runs)
- [ ] Cleanup behavior is tested (run tests in sequence to detect leaks)

## The Gotcha (Most Common Mistake)

Allocating resources outside the fixture scope, so they don't get cleaned up:

```rust
// ❌ WRONG: Cleanup doesn't run if assertion fails
test!(test_bad_cleanup, {
    let container = docker.run("postgres:16");  // Allocated outside fixture
    let result = container.query("SELECT 1");
    assert_ok!(&result);
    drop(container);  // May not run if assertion above fails
});

// ✅ RIGHT: Fixture guarantees cleanup via Drop
fixture_test!(test_good_cleanup, fixture, {
    let container = fixture.postgres_container()?;  // In fixture scope
    let result = container.query("SELECT 1");
    assert_ok!(&result);  // If this fails, drop still runs
    Ok(())
});
```

**Why**: If the assertion fails before cleanup, resources leak. RAII guarantees cleanup happens via Drop, not explicit code.

## Codebase Example

File: `src/core/fixture.rs`
Purpose: Defines fixture Drop implementation that cleans up all allocated resources

## Related Patterns

- **Before this**: [Pattern 5: Real Collaborators](real-collaborators.md) (uses fixtures for resources)
- **Next**: [Pattern 16: Fixture Lifecycle](../design-patterns/fixture-lifecycle.md) (advanced fixture management)
- **Combine**: [Pattern 18: Timeout Defense](../design-patterns/timeout-defense.md) (prevent hanging cleanups)

---

**Why It Works**: Drop is guaranteed to run when the value goes out of scope, even on panic/early return. This provides automatic, reliable cleanup.

**Production Checklist**:
- [ ] All resource-allocating tests use fixtures
- [ ] No manual cleanup code (let Drop handle it)
- [ ] Tests run in any order without state leakage
- [ ] Docker containers exit successfully after test
- [ ] Database connections are returned to pools
- [ ] File handles are closed
