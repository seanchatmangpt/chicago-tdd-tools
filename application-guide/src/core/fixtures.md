# Getting Started with Fixtures

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

    // Use the fixture in your test
    let counter = fixture.test_counter();
    assert!(counter >= 0);
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
            let _ = fixture.test_counter();
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
    let _ = fixture.test_counter();
});
```

## Fixture Properties

The `TestFixture` provides utilities for tests:

```rust
test!(test_fixture_properties, {
    let fixture = TestFixture::new()?;

    // Get test counter (how many tests have run)
    let counter = fixture.test_counter();
    assert!(counter >= 0);

    // Additional fixture features are available
    // depending on enabled features (docker, etc.)
});
```

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
    let counter1 = fixture1.test_counter();
    // Uses fixture1
});

test!(test_isolation_2, {
    let fixture2 = TestFixture::new()?;
    let counter2 = fixture2.test_counter();
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
    let counter1 = fixture1.test_counter();
    let counter2 = fixture2.test_counter();

    // Both are cleaned up when the test ends
});
```

## Real-World Example

### Scenario: Testing a User Service

```rust
test!(test_user_service, {
    // Arrange: Set up fixture with data
    let fixture = TestFixture::new()?;

    // Act: Use the fixture
    let user_id = fixture.test_counter();

    // Assert: Verify
    assert!(user_id >= 0);

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

