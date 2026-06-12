# Fixtures Deep Dive: 15-Minute Tutorial

> 🎓 Tutorial | Master test isolation with fixtures

Fixtures are the foundation of isolated testing. This tutorial builds on the basics and shows you how to use fixtures for real-world scenarios.

**Prerequisites**: [Getting Started](getting-started.md)
**Time**: ~15 minutes
**What you'll learn**: Advanced fixture patterns and isolation techniques

---

## What Are Fixtures?

A **fixture** is a isolated test environment. Each test gets its own copy:

```rust
test!(test_isolation_example, {
    // Each test gets a completely separate fixture
    let fixture = TestFixture::new()?;

    // Changes here don't affect other tests
    fixture.set_metadata("value", "100");
    assert_eq!(fixture.get_metadata("value"), Some("100"));
});
```

### Why Fixtures Matter

✅ **Isolation**: Tests don't interfere with each other
✅ **Cleanup**: Resources are automatically cleaned up
✅ **Repeatability**: Same test always behaves the same way
✅ **Parallel execution**: Tests can run safely in parallel

---

## Storing Test State

### Simple Key-Value Storage

Store and retrieve test data:

```rust
test!(test_storing_state, {
    let fixture = TestFixture::new()?;

    // Store data
    fixture.set_metadata("user_id", "123");
    fixture.set_metadata("status", "active");

    // Retrieve data
    let user_id = fixture.get_metadata("user_id");
    assert_eq!(user_id, Some("123"));

    // Non-existent key returns None
    let missing = fixture.get_metadata("missing");
    assert_eq!(missing, None);
});
```

### Practical Example: User Setup

```rust
test!(test_user_creation, {
    let fixture = TestFixture::new()?;

    // Define helper structs and functions locally
    struct User { id: u32, name: String }
    let create_user = |name: &str, _email: &str| -> Result<User, &'static str> {
        Ok(User { id: 42, name: name.to_string() })
    };
    let get_user = |_id: u32| -> Result<User, &'static str> {
        Ok(User { id: 42, name: "alice".to_string() })
    };

    // Setup: Create a user
    let user = create_user("alice", "alice@example.com")?;
    fixture.set_metadata("user_id", &user.id.to_string());
    fixture.set_metadata("username", &user.name);

    // Test: Can we retrieve the user?
    let stored_id = fixture.get_metadata("user_id").unwrap();
    let retrieved = get_user(stored_id.parse().map_err(|_| "parse error")?)?;
    assert_eq!(retrieved.name, "alice");
});
```

---

## Snapshots: Capturing State Over Time

Snapshots record your test's state at different points:

### Taking a Snapshot

```rust
use std::collections::HashMap;

test!(test_with_snapshots, {
    let fixture = TestFixture::new()?;

    // Perform some operations
    let data = vec![1, 2, 3];

    // Capture state as a snapshot
    let state = HashMap::from([
        ("step".to_string(), "initial".to_string()),
        ("count".to_string(), data.len().to_string()),
    ]);
    fixture.capture_snapshot(state);

    // Perform additional operations to process elements
    let mut data = data;
    data.push(4);
    data.push(5);

    let state2 = HashMap::from([
        ("step".to_string(), "processed".to_string()),
        ("count".to_string(), data.len().to_string()),
    ]);
    fixture.capture_snapshot(state2);

    // Access all snapshots
    let snapshots = fixture.snapshots();
    assert_eq!(snapshots.len(), 2);

    // Access latest snapshot
    let latest = fixture.latest_snapshot();
    assert_eq!(latest.unwrap().get("step"), Some(&"processed".to_string()));
});
```

### Real-World Example: Multi-Step Workflow

```rust
test!(test_order_workflow, {
    let fixture = TestFixture::new()?;

    struct Order { id: u32 }
    let create_order = |_user: &str, _amount: f64| -> Result<Order, &'static str> {
        Ok(Order { id: 987 })
    };
    let process_payment = |_order: &Order| -> Result<(), &'static str> { Ok(()) };
    let ship_order = |_order: &Order| -> Result<(), &'static str> { Ok(()) };

    // Step 1: Create order
    let order = create_order("alice", 100.0)?;
    fixture.capture_snapshot(HashMap::from([
        ("stage".to_string(), "created".to_string()),
        ("order_id".to_string(), order.id.to_string()),
        ("amount".to_string(), "100.0".to_string()),
    ]));

    // Step 2: Process payment
    process_payment(&order)?;
    fixture.capture_snapshot(HashMap::from([
        ("stage".to_string(), "paid".to_string()),
        ("payment_status".to_string(), "completed".to_string()),
    ]));

    // Step 3: Ship order
    ship_order(&order)?;
    fixture.capture_snapshot(HashMap::from([
        ("stage".to_string(), "shipped".to_string()),
        ("tracking".to_string(), "12345".to_string()),
    ]));

    // Verify all stages completed
    let snapshots = fixture.snapshots();
    assert_eq!(snapshots.len(), 3);
});
```

---

## Multiple Fixtures in One Test

You can use multiple fixtures for complex scenarios:

```rust
test!(test_multiple_fixtures, {
    // Fixture 1: First test environment
    let fixture1 = TestFixture::new()?;
    fixture1.set_metadata("context", "database");

    // Fixture 2: Separate test environment
    let fixture2 = TestFixture::new()?;
    fixture2.set_metadata("context", "cache");

    // They don't interfere
    let ctx1 = fixture1.get_metadata("context");
    let ctx2 = fixture2.get_metadata("context");

    assert_eq!(ctx1, Some("database"));
    assert_eq!(ctx2, Some("cache"));
    assert_ne!(ctx1, ctx2);  // Different values
});
```

---

## Fixtures with Error Handling

Fixtures handle errors gracefully:

```rust
test!(test_fixture_error_handling, {
    // Create fixture might fail
    let fixture = TestFixture::new()?;

    let risky_operation = || -> Result<(), &'static str> {
        Err("operation failed")
    };

    // Operations might fail
    if let Err(e) = risky_operation() {
        // Record the error
        fixture.set_metadata("error", &e.to_string());

        // Assert error was expected
        let recorded = fixture.get_metadata("error");
        assert!(recorded.is_some());
    }

    // Fixture cleanup still happens automatically
});
```

---

## Fixture Initialization Pattern

For repeated setup, create a helper function:

```rust
struct User { id: u32, name: String, email: String }

fn setup_user_fixture() -> Result<(TestFixture, User), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let user = User { id: 101, name: "test_user".to_string(), email: "test@example.com".to_string() };
    fixture.set_metadata("user_id", &user.id.to_string());
    fixture.set_metadata("username", &user.name);

    Ok((fixture, user))
}

test!(test_using_setup, {
    let (fixture, user) = setup_user_fixture()?;

    // Test can now use pre-initialized fixture
    assert_eq!(user.name, "test_user");
    assert_eq!(user.email, "test@example.com");
});
```

---

## Best Practices for Fixtures

### ✅ Do:

1. **Create one fixture per test**
   ```rust
   test!(test1, {
       let f = TestFixture::new()?;
       assert!(f.get_metadata("id").is_none());
   });
   test!(test2, {
       let f = TestFixture::new()?;
       assert!(f.get_metadata("id").is_none());
   }); // Separate!
   ```

2. **Use metadata for state tracking**
   ```rust
   fixture.set_metadata("step", "processing");
   ```

3. **Capture snapshots at key points**
   ```rust
   fixture.capture_snapshot(state);
   ```

4. **Use descriptive metadata keys**
   ```rust
   fixture.set_metadata("user_id", "123");  // Clear
   // Not: fixture.set_metadata("id", "123");  // Vague
   ```

### ❌ Don't:

1. **Share fixtures between tests**
   ```rust
   // WRONG - fixture is shared!
   // let shared_fixture = TestFixture::new();
   // test!(test1, { /* use shared_fixture */ });
   ```

2. **Use global state**
   ```rust
   // WRONG - global state affects test isolation
   // static FIXTURE: Lazy<TestFixture> = Lazy::new(|| TestFixture::new().unwrap());
   ```

3. **Forget to handle errors**
   ```rust
   // WRONG - TestFixture::new() can fail without handling
   // let fixture = TestFixture::new().unwrap();
   ```

---

## Common Fixture Patterns

### Pattern 1: Setup-Teardown

```rust
test!(test_setup_teardown, {
    let fixture = TestFixture::new()?;

    let setup_database = |_f: &TestFixture| -> Result<(), &'static str> {
        Ok(())
    };

    // Setup
    setup_database(&fixture)?;
    fixture.set_metadata("db_ready", "true");

    // Test logic
    assert_eq!(fixture.get_metadata("db_ready"), Some("true"));

    // Teardown happens automatically when fixture is dropped
});
```

### Pattern 2: State Validation

```rust
test!(test_state_validation, {
    let fixture = TestFixture::new()?;

    let do_work = || -> Result<(), &'static str> {
        Ok(())
    };

    // Perform operations
    let _result = do_work()?;

    // Capture and validate state
    let state = HashMap::from([
        ("success".to_string(), "true".to_string()),
        ("items".to_string(), "5".to_string()),
    ]);
    fixture.capture_snapshot(state);

    // Verify final state
    let latest = fixture.latest_snapshot().unwrap();
    assert_eq!(latest.get("success"), Some(&"true".to_string()));
});
```

### Pattern 3: Progressive Testing

```rust
test!(test_progressive, {
    let fixture = TestFixture::new()?;

    let operation1 = || -> Result<(), &'static str> { Ok(()) };
    let operation2 = || -> Result<(), &'static str> { Ok(()) };
    let operation3 = || -> Result<(), &'static str> { Ok(()) };

    // Phase 1
    let result1 = operation1()?;
    fixture.set_metadata("phase", "1");
    assert_ok!(&result1);

    // Phase 2
    let result2 = operation2()?;
    fixture.set_metadata("phase", "2");
    assert_ok!(&result2);

    // Phase 3
    let result3 = operation3()?;
    fixture.set_metadata("phase", "3");
    assert_ok!(&result3);

    // Verify we completed all phases
    assert_eq!(fixture.get_metadata("phase"), Some("3"));
});
```

---

## Summary

Fixtures provide:

✅ **Isolation** - Each test has its own environment
✅ **State tracking** - Store and retrieve metadata
✅ **Snapshots** - Capture state at different points
✅ **Cleanup** - Automatic resource cleanup
✅ **Parallel safety** - Tests can run in parallel

## Next Steps

**Ready for more?**

1. **[Error Path Testing](../core/error-paths.md)** - Test error cases thoroughly
2. **[Advanced Fixtures](../core/fixtures.md)** - API reference and examples
3. **[Real-World Applications](../guides/real-world.md)** - See fixtures in action

---

## Quick Reference

```rust
// Create fixture
let fixture = TestFixture::new()?;

// Store metadata
fixture.set_metadata("key", "value");

// Retrieve metadata
let value = fixture.get_metadata("key");  // Returns Option<&str>

// Capture snapshot
let state = HashMap::from([("key".to_string(), "value".to_string())]);
fixture.capture_snapshot(state);

// Access snapshots
let all_snapshots = fixture.snapshots();
let latest = fixture.latest_snapshot();
```

**Congratulations!** You've mastered fixtures. You can now write tests with proper isolation and state tracking.
