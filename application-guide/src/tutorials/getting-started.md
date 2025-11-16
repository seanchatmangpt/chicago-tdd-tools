# Getting Started: 25-Minute Tutorial

> 🎓 **TUTORIAL** | Learn the basics of Chicago TDD in 25 minutes

This tutorial teaches you everything you need to write your first test with Chicago TDD Tools. No prior knowledge required.

**Time**: ~25 minutes
**Level**: Beginner
**What you'll learn**: Write a complete test from start to finish

---

## Part 1: Your First Test (5 minutes)

### Step 1: Create a test function

Chicago TDD uses the `test!` macro for declaring tests:

```rust
use chicago_tdd_tools::prelude::*;

test!(my_first_test, {
    // Test code goes here
});
```

### Step 2: Add test logic (Arrange-Act-Assert)

Every test follows three steps:

```rust
test!(test_number_parsing, {
    // Arrange: Set up test data
    let input = "42";

    // Act: Execute the code being tested
    let result = input.parse::<u32>();

    // Assert: Verify the result
    assert_ok!(&result);
    assert_eq!(result.unwrap(), 42);
});
```

**That's it!** You've written your first test. Let's break it down:

- **Arrange** (`let input = "42"`) - Prepare test data
- **Act** (`let result = input.parse()`) - Call the code you're testing
- **Assert** (`assert_ok!` and `assert_eq!`) - Verify it worked

---

## Part 2: Testing Success and Failure (7 minutes)

### Test the Happy Path

When code works:

```rust
test!(test_valid_number, {
    let result = "100".parse::<u32>();

    // Verify it's Ok
    assert_ok!(&result);

    // Verify the value is correct
    assert_eq!(result.unwrap(), 100);
});
```

### Test the Error Path

When code fails (most bugs hide here!):

```rust
test!(test_invalid_number, {
    let result = "not_a_number".parse::<u32>();

    // Verify it's an error
    assert_err!(&result);
});
```

### Test Boundary Conditions

Edge cases are important:

```rust
test!(test_boundaries, {
    // Zero (special case)
    assert_ok!(&"0".parse::<u32>());

    // Maximum value
    assert_ok!(&u32::MAX.to_string().parse::<u32>());

    // Negative (not valid for u32)
    assert_err!(&"-1".parse::<u32>());
});
```

### Key Assertion Helpers

Chicago TDD provides clear assertion helpers:

| Helper | What it checks | Example |
|--------|-----------------|---------|
| `assert_ok!(&result)` | Is this an `Ok`? | `assert_ok!(&my_result)` |
| `assert_err!(&result)` | Is this an `Err`? | `assert_err!(&my_result)` |
| `assert_eq!(a, b)` | Are these equal? | `assert_eq!(value, 42)` |
| `assert!(condition)` | Is this true? | `assert!(value > 0)` |

---

## Part 3: Using Fixtures for Test Setup (6 minutes)

For complex tests, use `TestFixture` to set up test data:

### Creating a Fixture

```rust
use chicago_tdd_tools::fixture::*;

test!(test_with_fixture, {
    // Create a fixture (isolated test environment)
    let fixture = TestFixture::new()?;

    // Store test data
    fixture.set_metadata("user_id", "123");
    fixture.set_metadata("username", "alice");

    // Retrieve test data
    let user_id = fixture.get_metadata("user_id");
    assert_eq!(user_id, Some("123"));
});
```

### Capturing Test State

You can save snapshots of your test data:

```rust
test!(test_with_snapshots, {
    let fixture = TestFixture::new()?;

    // ... do some work ...

    // Capture the current state
    let state = HashMap::from([
        ("step".to_string(), "completed".to_string()),
        ("items_processed".to_string(), "5".to_string()),
    ]);
    fixture.capture_snapshot(state);

    // Access all snapshots
    let snapshots = fixture.snapshots();
    assert_eq!(snapshots.len(), 1);
});
```

### Benefits of Fixtures

✅ Each test gets its own isolated environment
✅ Automatic cleanup when test ends
✅ State tracking with metadata and snapshots
✅ Great for complex multi-step tests

---

## Part 4: Building Test Data (5 minutes)

For complex test data, use `TestDataBuilder`:

### Basic Data Building

```rust
use chicago_tdd_tools::builders::*;

test!(test_with_builder, {
    // Create a builder
    let builder = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("email", "alice@example.com");

    // Build as JSON
    let json = builder.build_json()?;
    assert!(json["name"] == "Alice");
});
```

### Building Maps

```rust
test!(test_building_map, {
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_var("key2", "value2")
        .build();

    // Returns HashMap<String, String>
    assert_eq!(data.get("key1"), Some(&"value1".to_string()));
});
```

### Fluent API Style

Builders use a fluent API - chain method calls:

```rust
test!(test_fluent_style, {
    let data = TestDataBuilder::new()
        .with_var("first", "hello")
        .with_var("second", "world")
        .with_var("third", "!")
        .build_json()?;

    // Clean, readable syntax
});
```

---

## Part 5: Complete Example (2 minutes)

Putting it all together:

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::builders::*;
use chicago_tdd_tools::fixture::*;

test!(complete_example, {
    // Arrange: Create fixture and test data
    let fixture = TestFixture::new()?;

    let user_data = TestDataBuilder::new()
        .with_var("username", "alice")
        .with_var("email", "alice@example.com")
        .build_json()?;

    // Act: Parse the email
    let email = user_data["email"].as_str().unwrap();
    let valid = email.contains("@");

    // Assert: Verify the result
    assert!(valid, "Email should contain @");

    // Store result in fixture
    fixture.set_metadata("test_passed", "true");
});
```

---

## Summary: What You've Learned

✅ Basic test structure with `test!` macro
✅ Arrange-Act-Assert pattern
✅ Assertion helpers (`assert_ok!`, `assert_err!`, etc.)
✅ Testing success, error, and boundary cases
✅ Using fixtures for test isolation
✅ Building complex test data

## Next Steps

**Ready to dive deeper?**

1. **[Fixtures Deep Dive](fixtures-tutorial.md)** (10 minutes) - Master test isolation
2. **[Error Path Testing](../core/error-paths.md)** - Learn where bugs hide
3. **[Real-World Examples](../guides/real-world.md)** - See complete projects

**Or choose your path:**

- **I want to learn more patterns** → [Core Testing Patterns](../core/README.md)
- **I want advanced techniques** → [Advanced Testing](../advanced/README.md)
- **I want to test a web service** → [Web Service Tutorial](../guides/web-service.md)
- **I want to test a CLI tool** → [CLI Application Tutorial](../guides/cli-application.md)

---

## Quick Reference Card

```rust
// Basic test
test!(test_name, { /* code */ });

// Fixtures
let fixture = TestFixture::new()?;
fixture.set_metadata("key", "value");

// Data builders
TestDataBuilder::new()
    .with_var("key", "value")
    .build_json()?

// Assertions
assert_ok!(&result);          // Result is Ok
assert_err!(&result);         // Result is Err
assert_eq!(actual, expected); // Values match
assert!(condition);           // Condition is true
```

---

**Congratulations!** You can now write tests with Chicago TDD Tools. The rest is practice and learning specific patterns for your use case.
