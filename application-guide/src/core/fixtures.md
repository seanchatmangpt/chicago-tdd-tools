# Getting Started with Fixtures

> 🔧 **HOW-TO** | 📚 **REFERENCE** | Learn to use fixtures for test isolation

Fixtures are isolated test environments that provide controlled setup and automatic cleanup.

## What is a Fixture?

A fixture is a test object that:
- Provides fresh, isolated state for each test
- Handles setup automatically
- Cleans up resources when the test ends
- Prevents state leakage between tests

## Creating a Fixture

### Basic Fixture Creation

```rust
use chicago_tdd_tools::prelude::*;

test!(test_with_fixture, {
    // Create a fresh fixture
    let fixture = TestFixture::new()?;

    // Fixtures provide isolation and utilities
    // Store metadata for the test
    fixture.set_metadata("test_id".to_string(), "123".to_string());

    // Retrieve metadata
    let test_id = fixture.get_metadata("test_id");
    assert_eq!(test_id, Some(&"123".to_string()));
});
```

### Error Handling

Fixtures return `Result` - always handle the error:

```rust
test!(test_fixture_error_handling, {
    // ✅ Handle the Result properly
    match TestFixture::new() {
        Ok(fixture) => {
            // Use fixture
            fixture.set_metadata("key".to_string(), "value".to_string());
        }
        Err(e) => {
            alert_critical!("Fixture creation failed: {}", e);
            return Err(e.into());
        }
    }
});
```

Or use the `?` operator:

```rust
test!(test_fixture_with_question_mark, {
    let fixture = TestFixture::new()?;  // Propagates error
    fixture.set_metadata("test_data".to_string(), "setup_complete".to_string());
});
```

## Fixture Features

The `TestFixture` provides utilities for tests:

```rust
test!(test_fixture_features, {
    let mut fixture = TestFixture::new()?;

    // Store and retrieve metadata
    fixture.set_metadata("user_id".to_string(), "42".to_string());
    assert_eq!(
        fixture.get_metadata("user_id"),
        Some(&"42".to_string())
    );

    // Capture snapshots of test state
    let mut state = HashMap::new();
    state.insert("status".to_string(), "initialized".to_string());
    fixture.capture_snapshot(state);

    // Retrieve snapshots
    let snapshots = fixture.snapshots();
    assert!(!snapshots.is_empty());
});
```

## Quick Reference: TestFixture API

| Method | Parameters | Returns | Purpose |
|--------|-----------|---------|---------|
| `new()` | none | `Result<TestFixture, FixtureError>` | Create new isolated fixture |
| `set_metadata()` | `key: String`, `value: String` | `()` | Store test state |
| `get_metadata()` | `key: &str` | `Option<&String>` | Retrieve stored state |
| `capture_snapshot()` | `state: HashMap<String, String>` | `()` | Save test state snapshot |
| `snapshots()` | none | `&[HashMap<String, String>]` | Get all snapshots |
| `latest_snapshot()` | none | `Option<&HashMap<...>>` | Get most recent snapshot |

## Fixture Lifecycle

### Setup Phase

Setup happens when `TestFixture::new()` is called:

```rust
test!(test_setup, {
    // This is the setup phase
    let fixture = TestFixture::new()?;
    // Fixture is fully initialized here
});
```

### Cleanup Phase

Cleanup happens automatically when the fixture is dropped (at the end of the test):

```rust
test!(test_cleanup, {
    let fixture = TestFixture::new()?;
    // Use fixture

    // When this scope ends, fixture is dropped and cleaned up
    // No explicit cleanup needed!
});
```

## Test Isolation

Each test gets a fresh fixture:

```rust
test!(test_isolation_1, {
    let fixture1 = TestFixture::new()?;
    fixture1.set_metadata("test".to_string(), "isolation_1".to_string());
    // Uses fixture1
});

test!(test_isolation_2, {
    let fixture2 = TestFixture::new()?;
    fixture2.set_metadata("test".to_string(), "isolation_2".to_string());
    // Uses fixture2
    // fixture1 and fixture2 are completely independent
});
```

Both tests can run in parallel with no interference.

## Advanced: Multiple Fixtures

Create multiple fixtures in one test:

```rust
test!(test_with_multiple_fixtures, {
    let fixture1 = TestFixture::new()?;
    let fixture2 = TestFixture::new()?;

    // Both fixtures exist independently
    fixture1.set_metadata("fixture".to_string(), "first".to_string());
    fixture2.set_metadata("fixture".to_string(), "second".to_string());

    assert_eq!(fixture1.get_metadata("fixture"), Some(&"first".to_string()));
    assert_eq!(fixture2.get_metadata("fixture"), Some(&"second".to_string()));

    // Both are cleaned up when the test ends
});
```

## Real-World Example

### Scenario: Testing a User Service

```rust
test!(test_user_service, {
    // Arrange: Set up fixture with data
    let mut fixture = TestFixture::new()?;
    fixture.set_metadata("user_id".to_string(), "123".to_string());

    // Act: Perform test operations
    // Use fixture metadata for test coordination
    let user_id = fixture.get_metadata("user_id");

    // Assert: Verify
    assert_eq!(user_id, Some(&"123".to_string()));

    // Cleanup: Automatic! No explicit cleanup needed.
});
```

## Common Patterns

### Pattern: Reusable Fixture Setup

Create a helper function:

```rust
fn setup_user_fixture() -> Result<TestFixture, Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    // Additional setup here if needed
    Ok(fixture)
}

test!(test_with_helper, {
    let fixture = setup_user_fixture()?;
    // Use configured fixture
});
```

### Pattern: Nested Fixtures

Fixtures can use other fixtures:

```rust
test!(test_nested, {
    let outer = TestFixture::new()?;
    let inner = TestFixture::new()?;
    // Both are available
    // Inner is cleaned up first (LIFO order)
});
```

## Troubleshooting

### "Failed to create fixture"

This usually means an environment issue:

```rust
test!(test_fixture_error, {
    match TestFixture::new() {
        Ok(fixture) => {
            // Successfully created
        }
        Err(e) => {
            // Check your environment configuration
            alert_critical!("Environment issue: {}", e);
            return Err(e.into());
        }
    }
});
```

### Tests Running Sequentially

Chicago TDD Tools tests run in parallel by default. If you see sequential execution:

1. Check for shared state (file I/O, network)
2. Ensure each test has its own fixture
3. Use `cargo test -- --test-threads=1` to force sequential (for debugging)

## Best Practices

✅ **Do:**
- Create a fresh fixture in each test
- Handle the `Result` with `?` or `match`
- Let fixtures clean up automatically
- Use multiple fixtures if needed

❌ **Don't:**
- Share fixtures between tests
- Manually clean up (let the fixture drop)
- Rely on global state
- Use `unwrap()` on fixture creation

## Next Steps

Learn how to use fixtures with data builders: [Building Test Data](data-builders.md)

---

## Summary

| Concept | Purpose |
|---------|---------|
| **Fixture** | Isolated test state |
| **Setup** | Happens in `TestFixture::new()` |
| **Cleanup** | Automatic on drop |
| **Isolation** | Each test gets fresh fixture |
| **Error Handling** | Use `?` operator or `match` |

