# Real-World Applications

See complete examples of Chicago TDD Tools in action.

## CLI Application Example

The playground includes a complete CLI tool with multiple commands.

### Running the Playground

```bash
# Build the playground
cargo build --release -p playground

# Run CLI help
./target/release/playground help

# Run specific commands
./target/release/playground test --help
./target/release/playground quality --help
./target/release/playground obs --help
```

### Playground Structure

```
playground/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   └── cli/             # Subcommands
│       ├── test.rs      # Testing commands
│       ├── quality.rs   # Quality commands
│       ├── obs.rs       # Observability commands
│       └── ...
└── tests/               # Integration tests
    ├── core_tests.rs
    ├── testing_tests.rs
    └── integration_tests.rs
```

### Example Commands

```bash
# Test fixtures
playground test fixtures

# Run mutation tests
playground quality mutation

# Check OTEL compliance
playground obs validate

# Generate coverage
playground quality coverage
```

## Example-Based Learning

### Example: basic_test.rs

Demonstrates core patterns:
- Fixture creation
- Data builders
- Assertions
- Error handling

```bash
cargo run --example basic_test
```

### Example: property_testing.rs

Property-based testing:
- Random data generation
- Property verification
- Shrinking failed cases

```bash
cargo run --example property_testing --features property-testing
```

### Example: go_extra_mile.rs

Progressive enhancement:
- 1st idea (basic)
- 2nd idea (generic)
- 3rd idea (validated)

```bash
cargo run --example go_extra_mile --features otel,weaver
```

## Integration Testing Patterns

### With Docker Containers

```rust
test!(test_with_docker_db, {
    // Requires Docker to be running
    let fixture = TestFixture::new()?;

    // Fixture provides Docker container support
    // (when testcontainers feature enabled)

    // Run test against real database
    let result = query_database(&fixture)?;
    assert_ok!(&result);
});
```

Run with:

```bash
cargo make test-integration
```

## Testing Workflows

### Quick Feedback Loop (5 seconds)

```bash
# Format + check + unit tests
cargo make pre-commit

# Then fix issues
```

### Comprehensive Testing (1-2 minutes)

```bash
# Format + lint + all tests
cargo make test-all

# Includes integration tests (requires Docker)
```

### Release Validation (5-10 minutes)

```bash
# Full validation before release
cargo make release-validate

# Includes:
# - All tests
# - Coverage
# - Mutation testing
# - Documentation
```

## Architecture Patterns

### Fixture-Based Setup

```rust
test!(test_with_fixture, {
    // Arrange: Create isolated fixture
    let fixture = TestFixture::new()?;

    // Act: Use fixture in test
    let result = process(&fixture)?;

    // Assert: Verify behavior
    assert_ok!(&result);

    // Cleanup: Automatic (fixture dropped)
});
```

### Builder-Driven Test Data

```rust
test!(test_with_builders, {
    let user = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("email", "alice@example.com")
        .build_json()?;

    let result = create_user(&user)?;
    assert_ok!(&result);
});
```

### Property-Based Coverage

```rust
test!(test_property, {
    let strategy = ProptestStrategy::new().with_cases(1000);

    strategy.test(any::<(u32, u32)>(), |(a, b)| {
        a + b == b + a  // Commutativity
    });
});
```

## Complete Example: User Service

```rust
test!(complete_user_service_test, {
    // Setup
    let fixture = TestFixture::new()?;

    // Create user with builder
    let user_data = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("email", "alice@example.com")
        .build_json()?;

    // Act: Create
    let create_result = create_user(&user_data)?;
    assert_ok!(&create_result);
    let user = create_result.unwrap();

    // Act: Read
    let read_result = get_user(user.id)?;
    assert_ok!(&read_result);
    assert_eq!(read_result.unwrap().name, "Alice");

    // Act: Update
    let mut updated = user.clone();
    updated.email = "alice.new@example.com".to_string();
    let update_result = update_user(&updated)?;
    assert_ok!(&update_result);

    // Act: Delete
    let delete_result = delete_user(user.id)?;
    assert_ok!(&delete_result);

    // Verify deleted
    let read_result = get_user(user.id);
    assert_err!(&read_result);
});
```

## Best Practices from Examples

✅ **From examples:**
- Clear Arrange-Act-Assert structure
- Comprehensive error testing
- Progressive complexity
- Reusable patterns

✅ **From playground:**
- Multiple testing techniques
- Integration with Docker
- CLI testing patterns
- Quality metrics

## Next Steps

Apply what you've learned:

1. Start with [Core Patterns](../core/README.md)
2. Add [Advanced Techniques](../advanced/README.md)
3. Implement [Observability](observability.md)
4. Follow [Best Practices](best-practices.md)

