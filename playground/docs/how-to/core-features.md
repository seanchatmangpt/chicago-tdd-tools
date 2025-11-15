# How to Run Core Feature Examples

**Quick reference** for running and understanding core features: fixtures, builders, assertions, macros, and state machines.

## Quick Commands

```bash
# Show all core features
cargo run -- core stat

# List available core examples
cargo run -- core list

# Run specific example(s)
cargo run -- core exec --names "fixtures"
cargo run -- core exec --names "fixtures builders assertions"

# Run all core examples
cargo run -- core exec --names "fixtures builders assertions macros state const alert"
```

## Core Features Overview

| Feature | Purpose | Example | Learn |
|---------|---------|---------|-------|
| **Fixtures** | Automatic setup/teardown | Test database setup | `core exec --names "fixtures"` |
| **Builders** | Fluent test data | Build complex test objects | `core exec --names "builders"` |
| **Assertions** | Readable assertions | Verify behavior clearly | `core exec --names "assertions"` |
| **Macros** | AAA pattern enforcement | Write tests correctly | `core exec --names "macros"` |
| **State** | Type-level AAA | Compile-time correctness | `core exec --names "state"` |
| **Const Assert** | Compile-time checks | Verify at compile time | `core exec --names "const"` |
| **Alert** | Visual problem indicators | Log with icons | `core exec --names "alert"` |

## Running Individual Features

### Fixtures (Automatic Setup/Teardown)

```bash
cargo run -- core exec --names "fixtures"
```

**Use when:**
- Test needs setup (creating database, connecting to service)
- Test needs cleanup (closing connections, deleting test data)
- Multiple tests share setup logic

**Example:**
```rust
fixture_test!(test_with_fixture, fixture, {
    let resource = fixture.create_resource();
    // Use resource
    // Cleanup automatic
});
```

### Builders (Fluent Test Data)

```bash
cargo run -- core exec --names "builders"
```

**Use when:**
- Building complex test objects
- Many optional fields
- Readable test data construction

**Example:**
```rust
let user = UserBuilder::new()
    .with_name("Alice")
    .with_email("alice@example.com")
    .build();
```

### Assertions (Readable Verifications)

```bash
cargo run -- core exec --names "assertions"
```

**Use when:**
- Checking Results, Options, ranges
- Need custom error messages
- Want clear failure diagnostics

**Available helpers:**
- `assert_ok!(result)` - Verify `Ok` variant
- `assert_err!(result)` - Verify `Err` variant
- `assert_eq_msg!(a, b, msg)` - Equality with custom message
- `assert_in_range!(value, min, max)` - Numeric bounds

### Macros (AAA Pattern Enforcement)

```bash
cargo run -- core exec --names "macros"
```

**Macros available:**
- `test!` - Synchronous test
- `async_test!` - Async test (1s timeout)
- `fixture_test!` - Setup/teardown test
- `performance_test!` - Tick budget validation

**Example:**
```rust
test!(test_sync, {
    // Arrange
    let x = 5;
    // Act
    let result = x * 2;
    // Assert
    assert_eq!(result, 10);
});
```

### State (Type-Level AAA)

```bash
cargo run -- core exec --names "state"
```

**Use when:**
- Need to enforce AAA at compile time
- Want impossible states to be unrepresentable
- Building state machines

**How it works:**
- Type system tracks test phases
- Compiler prevents wrong state transitions
- If it compiles, test structure is correct

### Const Assert (Compile-Time Verification)

```bash
cargo run -- core exec --names "const"
```

**Use when:**
- Verifying invariants at compile time
- Checking size/alignment constraints
- Validating const expressions

## Running Multiple Features Together

### Essential Trio (Most Common)

```bash
cargo run -- core exec --names "fixtures builders assertions"
```

Combines automatic setup, fluent data building, and readable assertions.

### Full Core Suite

```bash
cargo run -- core exec --names "fixtures builders assertions macros state const alert"
```

All core features together.

## Feature Selection Guide

**Starting out?**
```bash
cargo run -- core exec --names "fixtures"
```

**Building test data?**
```bash
cargo run -- core exec --names "builders"
```

**Verifying behavior?**
```bash
cargo run -- core exec --names "assertions"
```

**Writing proper tests?**
```bash
cargo run -- core exec --names "macros"
```

**Complete core experience?**
```bash
cargo run -- core exec --names "fixtures builders assertions macros"
```

## Combining Core Features

Most powerful combination in practice:

```rust
// Fixture provides setup (fixtures)
fixture_test!(test_complete, fixture, {
    // Builder creates test data (builders)
    let user = UserBuilder::new()
        .with_name("Test")
        .build();

    // Macros enforce AAA (macros)
    // Arrange above

    // Act
    let result = user.validate();

    // Assert with helpers (assertions)
    assert_ok!(result);
    assert_eq_msg!(user.name, "Test", "Name mismatch");
});
```

## Output Formats

By default, all commands output JSON:

```json
{
  "example": "fixtures",
  "status": "success",
  "duration_ms": 45,
  "assertions_passed": 15
}
```

## Troubleshooting

**Q: "Example returned error"**
A: Run with verbose output:
```bash
RUST_LOG=debug cargo run -- core exec --names "fixtures"
```

**Q: "Feature not available"**
A: Ensure features are enabled:
```bash
cargo run --all-features -- core exec --names "fixtures"
```

**Q: "Can't find feature"**
A: List available:
```bash
cargo run -- core list
```

## Best Practices

1. **Start with fixtures** - Learn setup/teardown first
2. **Add builders** - Improve test data readability
3. **Use assertions** - Write clearer verifications
4. **Apply macros** - Enforce AAA pattern
5. **Combine all** - Use together for most benefit

## Next Steps

- **Copy to your project** → [Copying Examples Tutorial](../tutorials/copying-examples.md)
- **Explore testing features** → [How to Run Testing Examples](testing-features.md)
- **See all examples** → [Example Inventory](../reference/example-inventory.md)
- **Understand philosophy** → [Testing Philosophy](../explanation/testing-philosophy.md)

---

See [Getting Started](../tutorials/getting-started.md) for installation help.
