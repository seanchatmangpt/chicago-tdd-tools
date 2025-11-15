# Running Core Examples Tutorial

**Estimated time**: 15-20 minutes

Learn about core testing features by running interactive examples. This tutorial focuses on the fundamental building blocks: fixtures, builders, assertions, and macros.

## What Are Core Features?

Core features are always available and form the foundation of Chicago TDD Tools:

- **Fixtures** - Automatic test setup and teardown
- **Builders** - Fluent API for test data construction
- **Assertions** - Helper macros for verifying behavior
- **Macros** - Test and assertion macros enforcing AAA pattern
- **State** - Type-level state machines for correctness
- **Const Assert** - Compile-time assertions

## Prerequisites

- Complete [Getting Started Tutorial](getting-started.md)
- You're in the `playground` directory
- All core features are available by default

## Example 1: Running the Fixtures Example

Fixtures handle test setup and teardown automatically.

### Run it

```bash
cargo run -- core exec --names "fixtures"
```

### What you'll see

```json
{
  "example": "fixtures",
  "status": "success",
  "message": "Fixtures provide automatic setup and cleanup",
  "details": {
    "fixture_count": 5,
    "setup_ms": 12,
    "cleanup_ms": 8,
    "assertions": 20
  }
}
```

### What it demonstrates

```rust
fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture created automatically
    let counter = fixture.test_counter();

    // Act: Use the fixture
    let incremented = counter + 1;

    // Assert: Verify behavior
    assert_eq!(incremented, counter + 1);
    // Cleanup: Automatic on scope exit
});
```

**Key learnings:**
- Fixtures are created fresh for each test
- Setup happens automatically before test
- Cleanup happens automatically after test
- No manual teardown code needed

## Example 2: Running the Builders Example

Builders provide a fluent API for creating test data.

### Run it

```bash
cargo run -- core exec --names "builders"
```

### What you'll see

```json
{
  "example": "builders",
  "status": "success",
  "message": "Builders enable fluent test data construction",
  "details": {
    "builders_created": 8,
    "objects_built": 32,
    "assertions": 24
  }
}
```

### What it demonstrates

```rust
test!(test_with_builder, {
    // Arrange: Use fluent builder
    let user = UserBuilder::new()
        .with_name("Alice")
        .with_email("alice@example.com")
        .with_active(true)
        .build();

    // Act: Verify builder results
    let data = user.serialize();

    // Assert
    assert!(data.contains("Alice"));
});
```

**Key learnings:**
- Builders provide a readable way to construct test objects
- Chain methods with `.with_*()` for fluent API
- `.build()` finalizes the object
- Makes test intent clear

## Example 3: Running the Assertions Example

Specialized assertions make test failures clear and readable.

### Run it

```bash
cargo run -- core exec --names "assertions"
```

### What you'll see

```json
{
  "example": "assertions",
  "status": "success",
  "message": "Assertion helpers verify behavior clearly",
  "details": {
    "assertions": 42,
    "failures_caught": 8,
    "clarity_improved": true
  }
}
```

### What it demonstrates

```rust
test!(test_assertions, {
    let result: Result<i32, String> = Ok(42);

    // Arrange-Act (combined for clarity)

    // Assert with specialized helpers
    assert_ok!(result);                           // Verify success
    assert_eq_msg!(result.ok(), Some(42), "Expected 42");  // Custom message
    assert_in_range!(result.ok().unwrap(), 40, 50); // Range check
});
```

**Key learnings:**
- `assert_ok!()` verifies `Result::Ok`
- `assert_err!()` verifies `Result::Err`
- `assert_eq_msg!()` adds custom error messages
- `assert_in_range!()` validates numeric bounds

## Example 4: Running the Macros Example

Test macros enforce the AAA pattern at compile time.

### Run it

```bash
cargo run -- core exec --names "macros"
```

### What you'll see

```json
{
  "example": "macros",
  "status": "success",
  "message": "Test macros enforce AAA pattern",
  "details": {
    "tests": 6,
    "patterns_enforced": 6,
    "compile_time_checks": true
  }
}
```

### What it demonstrates

```rust
// Synchronous test
test!(test_sync, {
    // Arrange
    let x = 5;

    // Act
    let result = x * 2;

    // Assert
    assert_eq!(result, 10);
});

// Asynchronous test (1s timeout)
async_test!(test_async, {
    // Arrange
    let data = async { vec![1, 2, 3] }.await;

    // Act
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Assert
    assert_eq!(data.len(), 3);
});

// Fixture-based test
fixture_test!(test_fixture, fixture, {
    // Arrange (fixture provided)

    // Act

    // Assert
});
```

**Key learnings:**
- `test!()` for synchronous tests
- `async_test!()` for async code (1s timeout)
- `fixture_test!()` for setup/teardown
- `performance_test!()` for tick budget validation
- Compiler enforces AAA structure

## Example 5: Running All Core Examples Together

You can run multiple examples at once:

```bash
cargo run -- core exec --names "fixtures builders assertions macros state"
```

### What you'll see

```json
[
  { "example": "fixtures", "status": "success", ... },
  { "example": "builders", "status": "success", ... },
  { "example": "assertions", "status": "success", ... },
  { "example": "macros", "status": "success", ... },
  { "example": "state", "status": "success", ... }
]
```

## Building on Core Features

### Combining Fixtures + Builders

```rust
fixture_test!(test_fixture_with_builder, fixture, {
    // Arrange: Use fixture with builder
    let user = fixture.user_builder()
        .with_active(true)
        .build();

    // Act
    let result = user.is_active();

    // Assert
    assert!(result);
});
```

### Using Assertions + Macros

```rust
test!(test_result_handling, {
    // Arrange
    let value: Result<i32, String> = Ok(100);

    // Act - implicitly in assertions

    // Assert with multiple helpers
    assert_ok!(value);
    assert_eq_msg!(value.ok(), Some(100), "Got unexpected value");
    assert_in_range!(value.unwrap(), 90, 110);
});
```

## Copying Core Examples to Your Project

Now that you understand core features, copy them to your project:

```bash
# Copy test files from playground
cp -r playground/src/core/* your-project/tests/

# Add chicago-tdd-tools to your Cargo.toml
cargo add chicago-tdd-tools --dev
```

See [Copying Examples Tutorial](copying-examples.md) for full details.

## Practice Exercises

Try these to deepen understanding:

1. **Create a custom fixture** - Follow the pattern from `fixtures` example
2. **Build a fluent builder** - Model after the `builders` example
3. **Write custom assertions** - Combine helpers from `assertions` example
4. **Mix fixtures + builders** - Combine both approaches

## Next Steps

- **See testing examples** → [Running Feature Examples Tutorial](running-feature-examples.md)
- **Copy to your project** → [Copying Examples Tutorial](copying-examples.md)
- **Understand the philosophy** → [Testing Philosophy](../explanation/testing-philosophy.md)
- **See all examples** → [Example Inventory](../reference/example-inventory.md)

## Troubleshooting

### "Command not found"
Make sure you're in the `playground` directory:
```bash
cd chicago-tdd-tools/playground
```

### "Example returned error"
View detailed error output:
```bash
cargo run --all-features -- core exec --names "fixtures" 2>&1
```

### "Fixture not available"
Ensure you have all features enabled:
```bash
cargo run --all-features -- core stat
```

## Summary

You've learned:
- ✅ How to run core examples
- ✅ How fixtures handle setup/teardown
- ✅ How builders create fluent test data
- ✅ How assertions clarify test intent
- ✅ How macros enforce AAA pattern
- ✅ How to combine core features

---

**Next**: [Running Feature Examples](running-feature-examples.md) or [Copying Examples](copying-examples.md)
