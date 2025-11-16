# Troubleshooting Guide: Common Issues

> 🔧 **HOW-TO** | Fix common problems when using Chicago TDD patterns

Quick answers to the 10 most common issues users hit.

---

## Issue 1: "Test Passes Locally but Fails in CI"

### Error:
```
test test_database_query ... ok (local)
test test_database_query ... FAILED (CI)
```

### Root Cause:
Usually **timing** or **environment** differences:
- Local: Single-threaded tests, warm disk cache
- CI: Parallel tests, Docker container with cold cache

### Solution:

**Step 1: Enable single-threaded mode locally**

```bash
cargo test -- --test-threads=1
```

If it fails, problem is **race condition** or **test isolation**.

**Step 2: Check test isolation (Pattern 4)**

```rust
// ❌ WRONG: Tests share state
static mut DB: Option<Database> = None;

#[test]
fn test_a() {
    unsafe { DB = Some(Database::new()); }
    // Test runs
}

#[test]
fn test_b() {
    unsafe { /* DB from test_a still exists! */ }
}
```

**✅ RIGHT: Each test has its own fixture**

```rust
#[test]
fn test_a() {
    let db = Database::temporary();  // Unique DB per test
    // Test runs
    // DB cleaned up automatically (Drop)
}

#[test]
fn test_b() {
    let db = Database::temporary();  // Different DB
    // Test runs
}
```

**Step 3: Use cargo-nextest for parallel safety**

```bash
cargo install cargo-nextest
cargo nextest run
```

Nextest isolates tests better than `cargo test`.

---

## Issue 2: "Docker Container Not Cleaning Up"

### Error:
```
thread 'test_with_container' panicked at 'Docker containers accumulating: 15 orphaned containers'
```

### Root Cause:
Pattern 5 (Real Collaborators) requires cleanup. Not using Drop properly.

### Solution:

**❌ WRONG: Manual cleanup (doesn't work on panic)**

```rust
#[test]
fn test_with_container() {
    let container = GenericContainer::new(client, "postgres", "latest")?;

    // Test runs...

    container.stop()?;  // Only runs if test passes!
    // If test panics before here, container stays running
}
```

**✅ RIGHT: Drop trait (works always)**

```rust
#[test]
fn test_with_container() {
    let container = GenericContainer::new(client, "postgres", "latest")?;

    // Test runs...

    // Drop called automatically (even on panic)
    drop(container);
}
```

**Step 1: Verify you're using Drop**

Check if your container type implements Drop:

```rust
// src/integration/testcontainers/mod.rs
impl Drop for GenericContainer {
    fn drop(&mut self) {
        // Automatically stop container
    }
}
```

**Step 2: Don't call stop() manually**

Let Drop handle cleanup. Only call stop if you need early cleanup:

```rust
#[test]
fn test_with_early_cleanup() {
    let container = GenericContainer::new(client, "postgres", "latest")?;
    container.execute("SELECT 1")?;

    // Optional: Stop early if you need resources
    container.stop()?;

    // At end of test, Drop called again (safe, idempotent)
}
```

---

## Issue 3: "Timeout Still Firing After Adding Pattern 18"

### Error:
```
thread 'test_slow_operation' panicked at 'test timed out after 1s'
```

### Root Cause:
Pattern 18 (Timeout Defense) has **multiple layers**. If inner timeout fails, outer timeout fires.

### Solution:

**Understand the layers (top priority first):**

1. **Test-level timeout** (outermost, last resort)
2. **Operation timeout** (what you added)
3. **Resource timeout** (cleanup timeout)

**✅ Correct order:**

```rust
#[test]
fn test_with_timeout_defense() {
    // Arrange: Operation with internal timeout
    let operation = SlowOperation::new()
        .with_timeout(Duration::from_millis(100));  // 1st: Operation timeout

    // Act: Execute (should timeout at 100ms)
    let result = operation.execute();

    // Assert
    assert_err!(result);  // Should error, not panic

    // Test timeout: 1s (last resort, won't fire if operation handled timeout)
}
```

**If test still times out at 1s:**
- Operation didn't respect its internal timeout
- Operation is actually hanging (network wait, deadlock)

**Debug: Add logging**

```rust
#[test]
fn test_with_timeout_defense_debug() {
    alert_info!("Starting operation");
    let operation = SlowOperation::new()
        .with_timeout(Duration::from_millis(100));

    alert_info!("About to execute");
    let result = operation.execute();

    alert_info!("Operation completed: {:?}", result);
    assert_err!(result);
}
```

Run with:
```bash
RUST_LOG=debug cargo test test_with_timeout_defense_debug -- --nocapture
```

---

## Issue 4: "Fixture Metadata Not Tracking"

### Error:
```rust
let metadata = fixture.metadata();
assert!(metadata.has_event("user.created"));  // Fails: no events tracked
```

### Root Cause:
Metadata tracking requires you to **emit events**. Fixture tracks what you tell it to.

### Solution:

**❌ WRONG: Expecting automatic tracking**

```rust
#[test]
fn test_tracking() {
    let fixture = TestFixture::new()?;

    // Manually create user (fixture doesn't know)
    create_user("Alice");

    // This fails: No events were captured
    assert!(fixture.metadata().has_event("user.created"));
}
```

**✅ RIGHT: Emit events explicitly**

```rust
#[test]
fn test_tracking() {
    let mut fixture = TestFixture::new()?;

    // Emit event
    fixture.emit_event("user.created", "Alice");

    // Now it's tracked
    assert!(fixture.metadata().has_event("user.created"));
}
```

**If you want automatic tracking:**

Use Pattern 5 (Real Collaborators) + observability:

```rust
#[test]
fn test_with_automatic_tracking() {
    let weaver = WeaverTestFixture::new()?;

    // Code that emits spans automatically
    create_user_with_instrumentation("Alice");

    // Weaver captured the spans
    let spans = weaver.collect_spans();
    assert!(spans.iter().any(|s| s.name == "user.created"));
}
```

---

## Issue 5: "Weaver Connection Refused"

### Error:
```
thread 'test_with_weaver' panicked at 'Failed to connect to Weaver: connection refused on localhost:4317'
```

### Root Cause:
Weaver binary not running or Docker daemon not available.

### Solution:

**Step 1: Check if Weaver is enabled**

Cargo.toml must have `weaver` feature:

```toml
[dev-dependencies]
chicago-tdd-tools = { features = ["observability-full"] }  # Includes weaver
```

**Step 2: Download Weaver**

```bash
cargo make weaver-bootstrap
```

This downloads the Weaver binary and registry.

**Step 3: Verify Docker running**

```bash
docker ps
```

If error: Start Docker Desktop / daemon

**Step 4: Run Weaver smoke test**

```bash
cargo make weaver-smoke
```

If this passes, Weaver is ready.

**Step 5: If tests still fail, allow skip**

```bash
export WEAVER_ALLOW_SKIP=1
cargo test
```

This skips Weaver validation if not available. Good for CI with restricted Docker.

---

## Issue 6: "Unwrap in Error Handling After Pattern 2"

### Error:
```
error: use of `unwrap_used` is not allowed
    |
18 |     error_handler.get().unwrap()  // WRONG!
    |                       ^^^^^^
```

### Root Cause:
Pattern 2 (Error Paths) teaches `?` operator instead of `.unwrap()`. Clippy denies unwrap.

### Solution:

**❌ WRONG: Using unwrap**

```rust
let value = result.unwrap();  // Panics on error
```

**✅ RIGHT: Option 1 - Propagate with ?**

```rust
let value = result?;  // Propagates error
```

**✅ RIGHT: Option 2 - Handle explicitly**

```rust
let value = match result {
    Ok(v) => v,
    Err(e) => {
        alert_warning!("Operation failed: {}", e);
        DEFAULT_VALUE
    }
};
```

**✅ RIGHT: Option 3 - Use unwrap_or**

```rust
let value = result.unwrap_or(DEFAULT_VALUE);
```

**✅ RIGHT: Option 4 - Use ? (test code)**

In tests, you can use `?`:

```rust
#[test]
fn test_operation() -> Result<(), Box<dyn std::error::Error>> {
    let value = result?;  // Propagates error
    assert_eq!(value, expected);
    Ok(())
}
```

---

## Issue 7: "Pattern X Doesn't Match My Codebase"

### Problem:
"We use mocks everywhere. Pattern 5 (Real Collaborators) doesn't fit."

### Solution:

**Understand pattern dependencies:**

Not all patterns fit all codebases at the same time.

Check [Pattern Dependencies](pattern-dependencies.md):
- Pattern 5 requires **ability to use real services**
- If you can't use Docker, use Pattern 5 just for local tests
- Pattern 2 (Error Paths) works everywhere (no dependencies)

**Start with patterns that fit:**

1. Pattern 1 (AAA) - Works everywhere ✅
2. Pattern 2 (Error Paths) - Works everywhere ✅
3. Pattern 10 (Capability Groups) - Works everywhere ✅

Then add patterns that require your infrastructure:
4. Pattern 5 (Real Collaborators) - Requires Docker capability
5. Pattern 4 (Resource Cleanup) - Requires Drop support

---

## Issue 8: "Tests Pass with Feature A, Fail without Feature B"

### Error:
```
cargo test                      # PASSES
cargo test --no-default-features # FAILS
```

### Root Cause:
Using features without enabling them in Cargo.toml.

### Solution:

**Check your dev-dependencies:**

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.3.0", features = ["testing-extras"] }
```

**If test uses Pattern 5 (Real Collaborators):**

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.3.0", features = ["testing-full"] }
```

**If test uses observability:**

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.3.0", features = ["observability-full"] }
```

**Always test both ways:**

```bash
cargo test                           # With default features
cargo test --no-default-features    # Without features
cargo test --all-features           # With all features
```

---

## Issue 9: "How do I Know If Pattern is Working?"

### Problem:
"We added Pattern 2 (Error Paths). How do we measure improvement?"

### Solution:

**Track these metrics:**

| Metric | How to Measure | Expected Change |
|--------|--|--|
| **Tests that check errors** | `grep "assert_err\|Err" tests/ \| wc -l` | Should increase |
| **Time to write tests** | Measure in PR reviews | Should decrease (Pattern helps) |
| **Bugs found before shipping** | Count bugs in QA | Should increase |
| **Test readability** | Ask team (1-5 scale) | Should increase (after Pattern 1) |
| **Time to debug failures** | Measure test debugging time | Should decrease (Pattern 1) |

**Simple before/after:**

Before Pattern 2:
```
Happy path tests: 20
Error path tests: 2
```

After Pattern 2:
```
Happy path tests: 20
Error path tests: 12 ← 6x improvement!
```

---

## Issue 10: "Team Resistance to Patterns"

### Problem:
"Developers think patterns are 'too much overhead'."

### Solution:

**Start with Pattern 1 only:**

Spend 1 week. Ask team:
- Is code harder to read? (Usually: no)
- Is it easier to debug? (Usually: yes)
- Does it take longer? (Usually: no)

**Show metrics:**
- Time to write test: 3min → 2.5min (faster due to structure)
- Time to debug failure: 10min → 3min (clearer structure)

**Add Pattern 2 after 2 weeks:**

Team sees error cases missing. Pattern 2 feels natural.

**Key insight:** Patterns should **reduce** work, not add it.

If pattern makes work harder → use it differently (or skip it).

---

## Still Stuck?

**Check these in order:**

1. Reread the pattern docs (easy stuff missed)
2. Check [Pattern Dependencies](pattern-dependencies.md) (dependency issue)
3. Check [Common Mistakes](common-mistakes.md) (typical gotcha)
4. Check [Real-World Scenarios](real-world-scenarios.md) (similar example)
5. Run with `RUST_LOG=debug` (see what's happening)
6. Ask in [GitHub Discussions](https://github.com/seanchatmangpt/chicago-tdd-tools/discussions)

---

## Common Success Pattern

Most users report similar progression:

**Week 1:** "Pattern 1 is just structure, why bother?" → Adoption
**Week 2:** "Pattern 2 finds bugs we missed" → Enthusiasm
**Week 3:** "Patterns make testing faster" → Full adoption
**Month 2:** "We can't imagine testing without patterns" → Spreading to team

---

## Quick Reference: Error Messages

| Error | Likely Cause | See Issue |
|-------|---|---|
| "test timed out" | Timeout layer failure | Issue 3 |
| "Docker containers accumulating" | No cleanup | Issue 2 |
| "unwrap_used is not allowed" | Pattern 2 not applied | Issue 6 |
| "feature X not enabled" | Missing Cargo.toml | Issue 8 |
| "test passes local, fails CI" | Race condition | Issue 1 |
| "Weaver connection refused" | Docker not running | Issue 5 |
| "metadata.has_event fails" | Events not emitted | Issue 4 |
| "Pattern doesn't fit" | Not all patterns for all codebases | Issue 7 |
| "How to measure improvement" | Track metrics | Issue 9 |
| "Team pushes back" | Start with Pattern 1 only | Issue 10 |

