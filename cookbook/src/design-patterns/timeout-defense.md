# Pattern 18: Timeout Defense in Depth

> 🔧 How-to

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Hung tests freeze the suite; single-layer timeouts fail silently; unclear which test stalled |
| **Solution** | Layer timeouts at test level (macro), runner level (nextest), and process level (Makefile) |
| **When to Use** | Async tests, integration tests, container interactions, network calls |
| **When NOT to Use** | Inherently fast operations (overhead not worth it), intentionally long operations |
| **Trade-offs** | Requires configuration across multiple layers, but prevents test suites from hanging indefinitely |
| **Complexity** | Low |
| **Real-World Example** | [nextest.toml](file:///Users/sac/chicago-tdd-tools/nextest.toml) and [src/core/macros/test.rs](file:///Users/sac/chicago-tdd-tools/src/core/macros/test.rs) |

## The Problem

Async tests interact with containers, networks, or external services. A hung future could freeze the entire suite. Single-layer timeouts (like process-level) kill the run without explaining which test stalled. No diagnostic information available.

## The Solution

Layer timeouts at three levels, each catching failures the lower level misses:
1. **Test-level** - tokio::time::timeout inside macros; clear error message on failure
2. **Runner-level** - cargo-nextest profiles with SLA timeouts per profile
3. **Process-level** - timeout wrapper in Makefile; emergency stop if lower levels fail

## Essential Code Example

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::core::macros::DEFAULT_INTEGRATION_TEST_TIMEOUT_SECONDS;

// Test-level timeout (clearest diagnostics)
fixture_test_with_timeout!(test_container_query, fixture, 30, {
    let container = fixture.postgres_container().await?;
    let result = container.query("SELECT 1").await?;
    assert_eq!(result, 1);
    Ok(())
});

// Runner-level (from nextest.toml)
// [profile.integration]
// slow-timeout = { period = "30s", terminate-after = 1 }
```

## Implementation Checklist

- [ ] Test-level timeouts with *_with_timeout! macros
- [ ] Timeout constants defined in one place (Pattern 9)
- [ ] Runner profiles configured in nextest.toml
- [ ] Process-level timeout in Makefile.toml as safety net
- [ ] Timeout constants match runner configuration
- [ ] Error messages identify which test timed out

## The Gotcha (Most Common Mistake)

Using only one layer of timeout or using loose timeouts everywhere:

```rust
// ❌ WRONG: No test-level timeout, only process-level (unclear which test fails)
async fn test_slow_op() { /* ... */ }

// ❌ WRONG: Loose timeouts hide real problems
fixture_test_with_timeout!(test, fixture, 300, { /* should be 30 */ })

// ✅ RIGHT: Layered timeouts, each appropriately tight
fixture_test_with_timeout!(test, fixture, 30, {  // Test-level: clear error
    let result = container.query().await?;
    Ok(())
});
// Plus runner-level: 30s per test, 300s total timeout
// Plus process-level: 600s emergency stop
```

**Why**: Layered timeouts provide defense in depth. If one layer fails, another catches it.

## Real-World Example

- **Code location**: [nextest.toml](file:///Users/sac/chicago-tdd-tools/nextest.toml) and [src/core/macros/test.rs](file:///Users/sac/chicago-tdd-tools/src/core/macros/test.rs)
- **Explanation**: Combines Tokios time limit in macros with Nextest profiles to enforce layered timeouts.

## Related Patterns

- **Before this**: [Pattern 9: Single Source](../architecture-patterns/single-source-of-truth.md) (timeout constants)
- **Use with**: [Pattern 4: Resource Cleanup](../testing-patterns/resource-cleanup.md) (cleanup after timeout)
- **Next**: [Pattern 20: Macro Enforcement](macro-enforcement.md) (enforces via macros)

---

**Why It Works**: Multiple timeout layers catch hangs at different points. Each layer provides better diagnostics than the one below.

**Production Checklist**:
- [ ] Test-level timeouts use *_with_timeout! macros
- [ ] Runner-level timeouts match test-level expectations
- [ ] Process-level timeout is generous (2x runner timeout)
- [ ] Timeout error messages identify the test
- [ ] Timeout constants are centralized (no duplication)
