# Best Practices & Migration

> 🔧 **HOW-TO** | 📚 **REFERENCE** | Proven patterns and migration strategies

Proven patterns and migration strategies for Chicago TDD.

## Testing Best Practices

### 1. Write Tests First (TDD)

```rust
// 1. Write failing test
test!(test_parse_positive_number, {
    let result = parse_number("42");
    assert_ok!(&result);
    assert_eq!(result.unwrap(), 42);
});

// 2. Implement minimal code
pub fn parse_number(input: &str) -> Result<u32, String> {
    input.parse().map_err(|e| format!("Parse failed: {e}"))
}

// 3. Refactor (improve design, remove duplication)
// 4. Test passes ✓
```

### 2. Test Both Paths

```rust
test!(test_complete_behavior, {
    // Success path
    assert_ok!(&parse_number("42"));

    // Error path
    assert_err!(&parse_number("invalid"));
});
```

### 3. Focus on Error Cases

80% of bugs hide in error paths:

```rust
test!(test_error_cases, {
    // Test invalid input
    assert_err!(&parse_number(""));
    assert_err!(&parse_number("not_a_number"));
    assert_err!(&parse_number("-1"));  // If negative not allowed

    // Test boundaries
    assert_ok!(&parse_number("0"));
    assert_ok!(&parse_number("4294967295"));  // u32::MAX
});
```

### 4. Keep Tests Focused

One test per behavior:

```rust
// ✅ Focused
test!(test_parse_valid_number, {
    let result = parse_number("42");
    assert_ok!(&result);
});

test!(test_parse_invalid_number, {
    let result = parse_number("invalid");
    assert_err!(&result);
});

// ❌ Too many behaviors
test!(test_parsing, {
    // Tests both valid and invalid
    // Hard to know what failed
});
```

### 5. Use Descriptive Names

```rust
// ✅ Clear intent
test!(test_parse_handles_negative_numbers_gracefully, { /* */ });

// ❌ Vague
test!(test_parse, { /* */ });
```

## Organization Best Practices

### 1. Mirror Source Structure

```
src/
├── users/
│   └── service.rs
└── orders/
    └── service.rs

tests/
├── users/
│   └── service_tests.rs
└── orders/
    └── service_tests.rs
```

### 2. Shared Utilities

```
tests/
├── common.rs           # Shared utilities
├── users_tests.rs
└── orders_tests.rs
```

In `common.rs`:

```rust
pub fn create_test_user() -> Result<User, String> {
    TestDataBuilder::new()
        .with_var("name", "Test User")
        .build_json()?
}
```

### 3. Fixture Factory Pattern

```rust
fn setup_database_fixture() -> Result<TestFixture, String> {
    let fixture = TestFixture::new()?;
    // Additional setup
    Ok(fixture)
}
```

## Performance Best Practices

### 1. Isolate Slow Tests

Mark slow tests:

```rust
#[ignore]  // Run with --ignored flag
test!(slow_integration_test, {
    // Takes 10 seconds
});
```

Run separately:

```bash
cargo test --ignored  # Only slow tests
```

### 2. Parallel Execution

Tests run in parallel by default:

```bash
cargo test -- --test-threads=4  # 4 threads (default: CPU count)
cargo test -- --test-threads=1  # Sequential (slow, for debugging)
```

### 3. Cache Expensive Operations

```rust
test!(test_expensive_setup, {
    // Reuse expensive setup
    lazy_static::lazy_static! {
        static ref EXPENSIVE_DATA: Data = { /* expensive */ };
    }

    // Use cached data
    assert_ok!(&process(&EXPENSIVE_DATA));
});
```

## Migration from Traditional Testing

### From: No Tests → To: Core Tests

1. Start with core patterns (fixtures, builders, assertions)
2. Test public APIs
3. Focus on error cases
4. Gradually increase coverage

### From: Mocks → To: Real Dependencies

1. Replace mocks with real implementations
2. Use fixtures for isolation
3. Only mock external services (APIs, DBs)

```rust
// Before: Mock-heavy
let mock_db = MockDatabase::new();
let result = process(&mock_db);

// After: Real implementations
let fixture = TestFixture::new()?;
let result = process(&fixture)?;
```

### From: Global State → To: Fixtures

1. Remove global state
2. Create fixtures for test isolation
3. Pass fixtures as parameters

```rust
// Before: Global
static mut TEST_DATA: Option<Data> = None;

// After: Fixture-based
test!(test_with_data, {
    let fixture = TestFixture::new()?;
    // Use fixture
});
```

### From: 100% Coverage → To: 80% + Error Paths

1. Stop obsessing over coverage
2. Focus on critical paths
3. Test error cases thoroughly

## Common Pitfalls & Solutions

### Pitfall 1: Tests Coupled to Implementation

```rust
// ❌ Brittle - depends on internal structure
test!(test_struct_format, {
    let user = create_user();
    assert_eq!(format!("{:?}", user), "User { id: 123, ... }");
});

// ✅ Robust - tests behavior
test!(test_user_creation, {
    let user = create_user();
    assert_eq!(user.id, 123);
    assert_eq!(user.name, "Alice");
});
```

### Pitfall 2: Flaky Tests

```rust
// ❌ Flaky - depends on time
test!(test_timing_dependent, {
    let start = Instant::now();
    operation();
    assert!(start.elapsed() < Duration::from_secs(1));  // Unreliable
});

// ✅ Reliable - deterministic
test!(test_result_correct, {
    let result = operation();
    assert_eq!(result, expected);  // Same result every time
});
```

### Pitfall 3: Test Interdependencies

```rust
// ❌ Tests depend on order
test!(test_1_setup, { /* setup */ });
test!(test_2_use_setup_from_1, { /* depends on test_1 */ });

// ✅ Each test is independent
test!(test_setup, {
    let fixture = TestFixture::new()?;
    // setup complete
});

test!(test_use, {
    let fixture = TestFixture::new()?;
    // independent
});
```

## Quality Checklist

For each test, verify:

- [ ] **AAA Pattern**: Arrange, Act, Assert clearly separated
- [ ] **Isolation**: No dependencies on other tests
- [ ] **Error Paths**: Tests both success and failure
- [ ] **Clear Name**: Describes what's being tested
- [ ] **One Assertion**: Focused on one behavior
- [ ] **Deterministic**: Same result every run
- [ ] **Fast**: <100ms per test (unless integration)

## Continuous Integration

### Pre-Commit

```bash
cargo make pre-commit  # Format + lint + unit tests
```

### Before Push

```bash
cargo make ci-local    # Simulate full CI pipeline
```

### In CI

```bash
cargo make test-all    # All tests including integration
```

## Graduation Path

```
Learning
  ↓
Core Patterns (fixtures, builders, assertions)
  ↓
Error Path Testing
  ↓
Advanced Techniques (properties, mutations, snapshots)
  ↓
Observability (OTEL, Weaver)
  ↓
Expert
```

## Resources

- [Core Patterns](../core/README.md)
- [Advanced Techniques](../advanced/README.md)
- [Real-World Examples](real-world.md)
- [Pattern Cookbook](../../cookbook/README.md)
- [API Reference](https://docs.rs/chicago-tdd-tools/)

## Next Steps

1. Pick a project to refactor
2. Start with core patterns
3. Add tests incrementally
4. Build confidence with error paths
5. Add advanced techniques where beneficial

---

**Chicago TDD Tools**: Testing with confidence, errors prevented at compile time.

