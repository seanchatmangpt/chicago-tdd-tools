# Pattern 5: Real Collaborators

> 🔧 **HOW-TO** | Test with real dependencies to catch integration gaps before production

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Mock-heavy tests hide integration gaps; production fails when mocks don't match reality |
| **Core Solution** | Use real collaborators (containers, services) in integration tests; separate from unit tests |
| **When to Use** | ✅ Database interactions, ✅ External APIs, ✅ Message queues, ✅ Telemetry validation |
| **When NOT to Use** | ❌ Unit tests (too slow), ❌ Unreliable external services (use fixtures), ❌ Network unavailable (skip gracefully) |
| **Difficulty** | Medium - Requires Docker; slower tests |

## The Problem

Mocks can hide integration gaps. A function passes all mock-based tests but fails in production because the real service behaves differently. The test suite offers no guidance on what went wrong.

## The Solution

Test critical paths with real collaborators (containers, actual services, real telemetry systems). Run integration tests separately from unit tests so teams can run unit tests quickly and integration tests intentionally.

## Essential Code Example

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::integration::testcontainers::*;

fixture_test_with_timeout!(test_postgres_query, fixture, 30, {
    // Arrange
    let container = fixture.postgres_container().await?;
    let client = container.client();

    // Act
    let rows = client.query("SELECT 1").await?;

    // Assert
    assert_eq!(rows[0].get::<i32, _>(0), 1);
    Ok(())
});
```

## Implementation Checklist

- [ ] Integration tests use real containers/services, not mocks
- [ ] Tests run with `cargo make test-integration` (separate from unit tests)
- [ ] Docker is required; test fails clearly if unavailable
- [ ] Resource cleanup happens automatically (fixture or scope)
- [ ] Timeout is set (30-60 seconds typical)
- [ ] Tests document expected collaborator version/config

## The Gotcha (Most Common Mistake)

Mixing real and mock collaborators in the same test, making it unclear which is being tested:

```rust
// ❌ WRONG: Real database with mocked cache layer
let db = RealDatabase::connect()?;
let cache = MockCache::new();  // Changes behavior unpredictably
let result = query_with_cache(&db, &cache)?;

// ✅ RIGHT: All real, or clearly separated
// Real test: real database + real cache
let db = RealDatabase::connect()?;
let cache = RealCache::connect()?;
let result = query_with_cache(&db, &cache)?;

// Unit test: both mocked for speed
let db = MockDatabase::new();
let cache = MockCache::new();
let result = query_with_cache(&db, &cache)?;
```

**Why**: When tests fail, you need to know if the issue is the code or the test infrastructure. Mixed real/mock makes debugging impossible.

## Codebase Example

File: `tests/go_extra_mile_tests.rs`
Purpose: Integration tests using real containers for database, telemetry, and service validation

## Related Patterns

- **Before this**: [Pattern 1: AAA Pattern](aaa-pattern.md) (foundation), [Pattern 2: Error Paths](error-path-testing.md)
- **Next**: [Pattern 16: Fixture Lifecycle](../design-patterns/fixture-lifecycle.md) (manage complex setups)
- **Combine**: [Pattern 4: Resource Cleanup](resource-cleanup.md) (automatic cleanup with real resources)

---

**Why It Works**: Real collaborators validate actual integration points. When tests pass, you have confidence the code will work in production.

**Production Checklist**:
- [ ] All database interactions tested with real schema
- [ ] API calls tested with real endpoints (or staging)
- [ ] Telemetry validated against semantic conventions
- [ ] Containers have version constraints documented
- [ ] Docker failure message is clear (guides user to fix)
