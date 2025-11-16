# TestFixture API Reference

> 📚 **REFERENCE** | Complete API documentation for test fixtures

## Overview

`TestFixture` provides isolated test environments with metadata storage and state snapshots.

**Module**: `chicago_tdd_tools::fixture`
**Stability**: Stable
**Feature flag**: Core (always available)

---

## Constructor

### `TestFixture::new() -> Result<TestFixture, FixtureError>`

Creates a new isolated test fixture.

**Returns:**
- `Ok(TestFixture)` - New fixture instance
- `Err(FixtureError)` - Fixture creation failed

**Example:**
```rust
let fixture = TestFixture::new()?;
```

**Error cases:**
- IO errors during fixture initialization
- Permission errors
- System resource unavailability

---

## Metadata Methods

### `set_metadata(key: impl AsRef<str>, value: impl AsRef<str>) -> ()`

Stores a key-value pair in fixture metadata.

**Parameters:**
- `key: impl AsRef<str>` - Metadata key (e.g., "user_id")
- `value: impl AsRef<str>` - Metadata value (e.g., "123")

**Returns:** Unit (always succeeds)

**Example:**
```rust
fixture.set_metadata("user_id", "123");
fixture.set_metadata("status", "active");
```

**Notes:**
- Overwrites existing key if present
- Keys and values are strings
- No size limits on metadata

### `get_metadata(key: &str) -> Option<&String>`

Retrieves a value from fixture metadata.

**Parameters:**
- `key: &str` - Metadata key to look up

**Returns:**
- `Some(&String)` - Value if key exists
- `None` - Key not found

**Example:**
```rust
let user_id = fixture.get_metadata("user_id");
match user_id {
    Some(id) => println!("User: {}", id),
    None => println!("User ID not set"),
}
```

**Notes:**
- Returns reference, not owned value
- Reference is valid only while fixture exists
- Returns None for non-existent keys

---

## Snapshot Methods

### `capture_snapshot(state: HashMap<String, String>) -> ()`

Captures the current test state as a snapshot.

**Parameters:**
- `state: HashMap<String, String>` - State to capture

**Returns:** Unit (always succeeds)

**Example:**
```rust
use std::collections::HashMap;

let state = HashMap::from([
    ("step".to_string(), "processing".to_string()),
    ("count".to_string(), "5".to_string()),
]);
fixture.capture_snapshot(state);
```

**Notes:**
- Creates immutable snapshot of provided state
- State is copied, original HashMap can be modified
- Snapshots are stored in order

### `snapshots() -> &[HashMap<String, String>]`

Returns all captured snapshots.

**Returns:**
- `&[HashMap<String, String>]` - Slice of all snapshots

**Example:**
```rust
let all = fixture.snapshots();
println!("Total snapshots: {}", all.len());

for (i, snapshot) in all.iter().enumerate() {
    println!("Snapshot {}: {:?}", i, snapshot);
}
```

**Notes:**
- Returns empty slice if no snapshots captured
- Snapshots are in capture order (chronological)
- Reference is valid only while fixture exists

### `latest_snapshot() -> Option<&HashMap<String, String>>`

Returns the most recently captured snapshot.

**Returns:**
- `Some(&HashMap)` - Last captured snapshot
- `None` - No snapshots captured yet

**Example:**
```rust
if let Some(latest) = fixture.latest_snapshot() {
    if let Some(step) = latest.get("step") {
        println!("Current step: {}", step);
    }
}
```

**Notes:**
- Convenience method for accessing most recent state
- Returns None if `snapshots().is_empty()`
- Equivalent to `snapshots().last()`

---

## Lifecycle

### Creation and Cleanup

```rust
test!(fixture_lifecycle, {
    // Creation: new() initializes resources
    let fixture = TestFixture::new()?;

    // Usage: store state and snapshots
    fixture.set_metadata("key", "value");

    // Cleanup: automatic when fixture is dropped
    // (at end of test scope)
}); // Fixture dropped here - cleanup happens
```

### Automatic Cleanup

When `TestFixture` is dropped:
1. All metadata is cleared
2. All snapshots are cleared
3. File handles/resources are released
4. Directory/files created by fixture are removed

**Note:** You don't need to manually call cleanup. It happens automatically.

---

## Common Patterns

### Pattern: Setup and Verify State

```rust
test!(verify_state_pattern, {
    let fixture = TestFixture::new()?;

    // Setup phase
    fixture.set_metadata("initialized", "false");
    assert_eq!(fixture.get_metadata("initialized"), Some("false"));

    // Initialization happens
    // ...

    // Verify state changed
    fixture.set_metadata("initialized", "true");
    assert_eq!(fixture.get_metadata("initialized"), Some("true"));
});
```

### Pattern: Multi-Phase Test

```rust
test!(multi_phase_pattern, {
    let fixture = TestFixture::new()?;

    // Phase 1
    fixture.set_metadata("phase", "1");
    let result1 = do_phase_1()?;
    fixture.capture_snapshot(state_at_phase1());
    assert_ok!(&result1);

    // Phase 2
    fixture.set_metadata("phase", "2");
    let result2 = do_phase_2()?;
    fixture.capture_snapshot(state_at_phase2());
    assert_ok!(&result2);

    // Verify all phases completed
    assert_eq!(fixture.get_metadata("phase"), Some("2"));
    assert_eq!(fixture.snapshots().len(), 2);
});
```

### Pattern: Context Association

```rust
test!(context_pattern, {
    let fixture = TestFixture::new()?;

    // Associate test context
    fixture.set_metadata("test_name", "my_test");
    fixture.set_metadata("test_version", "1.0");
    fixture.set_metadata("test_category", "integration");

    // ... test code ...
});
```

---

## Error Handling

### Handling Fixture Creation Errors

```rust
test!(error_handling, {
    match TestFixture::new() {
        Ok(fixture) => {
            // Use fixture
            fixture.set_metadata("key", "value");
        }
        Err(e) => {
            // Log error or skip test
            eprintln!("Failed to create fixture: {}", e);
            return;  // Skip this test
        }
    }
});
```

### Propagating Errors with `?`

```rust
test!(error_propagation, {
    let fixture = TestFixture::new()?;
    // If creation fails, entire test returns error
});
```

---

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| `new()` | ~1ms | I/O heavy, system dependent |
| `set_metadata()` | <1μs | In-memory |
| `get_metadata()` | <1μs | In-memory lookup |
| `capture_snapshot()` | <1μs | Copy HashMap to internal storage |
| `snapshots()` | <1μs | Returns slice reference |
| `latest_snapshot()` | <1μs | Slice operation |

---

## Memory Usage

- **Metadata**: ~50 bytes per key-value pair (plus string data)
- **Snapshots**: ~100 bytes per snapshot (plus HashMap overhead)
- **Total per test**: Usually <1MB unless storing large amounts of state

---

## Limitations and Constraints

| Constraint | Limit | Workaround |
|-----------|-------|-----------|
| Metadata key length | Unlimited | Use short, memorable keys |
| Metadata value length | Unlimited | Avoid storing binary data |
| Number of snapshots | Unlimited | Limit snapshots to key points |
| Parallel tests | Fully safe | Tests run with separate fixtures |

---

## Integration with Other Tools

### With TestDataBuilder

```rust
test!(integration_builder, {
    let fixture = TestFixture::new()?;

    let data = TestDataBuilder::new()
        .with_var("key", "value")
        .build_json()?;

    fixture.set_metadata("data", &data.to_string());
});
```

### With Async Tests

```rust
#[tokio::test]
async fn async_with_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    fixture.set_metadata("async", "true");

    // async operations

    Ok(())
}
```

---

## Examples

### Example 1: Simple Setup

```rust
test!(simple_setup, {
    let fixture = TestFixture::new()?;
    fixture.set_metadata("user", "alice");
    assert_eq!(fixture.get_metadata("user"), Some("alice"));
});
```

### Example 2: Complex State Tracking

```rust
test!(complex_tracking, {
    let fixture = TestFixture::new()?;

    for i in 0..3 {
        fixture.set_metadata("iteration", &i.to_string());

        let state = HashMap::from([
            ("step".to_string(), format!("step_{}", i)),
            ("count".to_string(), i.to_string()),
        ]);
        fixture.capture_snapshot(state);
    }

    assert_eq!(fixture.snapshots().len(), 3);
});
```

### Example 3: Error Recovery

```rust
test!(error_recovery, {
    let fixture = TestFixture::new()?;

    fixture.set_metadata("status", "starting");

    if let Err(e) = risky_operation() {
        fixture.set_metadata("status", "failed");
        fixture.set_metadata("error", &e.to_string());

        // Can still verify error was recorded
        assert!(fixture.get_metadata("error").is_some());
    }
});
```

---

## Related Types

- `FixtureError` - Error type for fixture operations
- `HashMap<String, String>` - Type for snapshot state

---

## See Also

- [Getting Started](../tutorials/getting-started.md) - Tutorial on basic fixtures
- [Fixtures Deep Dive](../tutorials/fixtures-tutorial.md) - Advanced patterns
- [Fixtures How-to](../core/fixtures.md) - Practical guide
