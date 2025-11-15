# AAA Pattern in Tests

**Understanding** the Arrange-Act-Assert pattern and how it's enforced in the playground.

## What Is AAA?

AAA stands for **Arrange-Act-Assert**, a simple structure for writing clear tests:

```rust
test!(test_addition, {
    // Arrange: Set up test data
    let x = 5;
    let y = 3;

    // Act: Execute code under test
    let result = x + y;

    // Assert: Verify behavior
    assert_eq!(result, 8);
});
```

Three distinct phases, each with a clear purpose.

## Phase 1: Arrange

**Set up everything needed for the test.**

### Purpose

Prepare test data, fixtures, mocks, and context so the code under test can run.

### Examples

```rust
// Arrange: Create test data
let user = User {
    name: "Alice",
    email: "alice@example.com",
};

let config = ConfigBuilder::new()
    .with_timeout(5000)
    .build();

let fixture = test_fixture();
```

### Key Principle

Arrange should be **boring and obvious**. Readers should instantly understand the setup.

❌ Clever setup:
```rust
let user = create_user_from_json(r#"{"name":"Alice",...}"#);
```

✅ Clear setup:
```rust
let user = User {
    name: "Alice",
    email: "alice@example.com",
};
```

## Phase 2: Act

**Execute the code under test.**

### Purpose

Call the function/method you're testing with the arranged data.

### Examples

```rust
// Act: Execute function
let result = add(5, 3);

// Act: Call method
let count = cache.insert(user);

// Act: Async operation
let response = api.fetch_user(id).await;
```

### Key Principle

Act should be **minimal and focused**. Usually one or two lines.

❌ Too much in act:
```rust
// Act
let cache = create_cache();
let user = User::new();
let result = cache.insert(user);
let fetched = cache.get(user.id);
assert_eq!(fetched, Some(user));
```

✅ Clean act:
```rust
// Arrange
let cache = create_cache();
let user = User::new();
cache.insert(user.clone());

// Act
let result = cache.get(user.id);

// Assert
assert_eq!(result, Some(user));
```

## Phase 3: Assert

**Verify the behavior.**

### Purpose

Check that the code under test produced the expected result.

### Examples

```rust
// Assert: Direct comparison
assert_eq!(result, 8);

// Assert: Multiple conditions
assert!(result > 0);
assert!(result < 100);

// Assert: Result type
assert_ok!(result);
assert_err!(result);
```

### Key Principle

Assertions should be **clear and focused**. Test one behavior per test.

❌ Multiple behaviors:
```rust
#[test]
fn test_user_everything() {
    let user = create_user();
    assert_eq!(user.name, "Alice");      // Testing name
    assert_eq!(user.email, "a@ex.com");  // Testing email
    assert!(user.is_active);              // Testing status
    // Plus 10 more assertions...
}
```

✅ One behavior per test:
```rust
#[test]
fn test_user_has_correct_name() {
    let user = create_user();
    assert_eq!(user.name, "Alice");
}

#[test]
fn test_user_has_correct_email() {
    let user = create_user();
    assert_eq!(user.email, "alice@ex.com");
}

#[test]
fn test_user_is_active() {
    let user = create_user();
    assert!(user.is_active);
}
```

## Real-World Example

### Without AAA (Messy)

```rust
#[test]
fn test_cache() {
    let cache = create_cache();
    let user = User::new("Alice");
    cache.insert(user.clone());
    let found = cache.get(user.id);
    assert_eq!(found, Some(user));
    cache.clear();
    assert_eq!(cache.size(), 0);
}
```

- Hard to understand what's being tested
- Multiple behaviors mixed together
- Hard to write clearly

### With AAA (Clear)

```rust
#[test]
fn test_cache_stores_user() {
    // Arrange
    let cache = create_cache();
    let user = User::new("Alice");

    // Act
    cache.insert(user.clone());

    // Assert
    assert_eq!(cache.get(user.id), Some(user));
}

#[test]
fn test_cache_clear_removes_all() {
    // Arrange
    let cache = create_cache();
    cache.insert(User::new("Alice"));

    // Act
    cache.clear();

    // Assert
    assert_eq!(cache.size(), 0);
}
```

- Clear, readable structure
- One behavior per test
- Easy to understand

## AAA Pattern With Fixtures

```rust
fixture_test!(test_database_insert, fixture, {
    // Arrange: Fixture provides database
    let db = fixture.database();
    let user = User::new("Alice");

    // Act: Execute operation
    let id = db.insert(user.clone())?;

    // Assert: Verify behavior
    let found = db.get(id)?;
    assert_eq!(found, user);
});
```

Fixtures handle setup/teardown automatically.

## AAA Pattern With Builders

```rust
test!(test_config_builder, {
    // Arrange: Use builder
    let config = ConfigBuilder::new()
        .with_timeout(5000)
        .with_retries(3)
        .build();

    // Act: Use config
    let duration = config.timeout_duration();

    // Assert: Verify
    assert_eq!(duration.as_millis(), 5000);
});
```

Builders make complex test data readable.

## AAA Pattern With Property Testing

```rust
test!(test_addition_commutative, {
    for _ in 0..100 {
        // Arrange: Generate random data
        let a = rand::random::<u32>();
        let b = rand::random::<u32>();

        // Act: Perform operation
        let result1 = a + b;
        let result2 = b + a;

        // Assert: Verify property holds
        assert_eq!(result1, result2);
    }
});
```

AAA applies to property tests too!

## Benefits of AAA

### Benefit 1: Clarity

Readers instantly understand what's being tested.

### Benefit 2: Maintainability

Changes are easy to make. Want to change setup? Modify Arrange. Bug in test? Check Assert.

### Benefit 3: Reusability

Test structure is consistent. Easy to create new tests using the same pattern.

### Benefit 4: Debugging

When a test fails, you know exactly where the problem is:
- Arrange phase: Bad setup
- Act phase: Exception during execution
- Assert phase: Behavior didn't match expectation

### Benefit 5: Test-Driven Development

AAA structure encourages writing tests first:
1. What data do you need? (Arrange)
2. What operation? (Act)
3. What should happen? (Assert)

## Common Mistakes

### Mistake 1: No Clear Separation

```
❌ No sections or comments
let x = 5;
let y = 3;
let result = x + y;
assert_eq!(result, 8);

✅ Clear sections
// Arrange
let x = 5;
let y = 3;

// Act
let result = x + y;

// Assert
assert_eq!(result, 8);
```

### Mistake 2: Arrange Has Too Much

```
❌ Too much setup
// Arrange
let cache = create_complex_cache();
let user = build_user_with_dependencies();
let config = load_config_from_file();

// Act
let result = cache.get(user.id);

// Better: Focus only on what's needed
```

### Mistake 3: Act Has Multiple Operations

```
❌ Multiple acts
// Act
cache.insert(user);
let result = cache.get(user.id);
cache.clear();

✅ Single operation
// Act
let result = cache.get(user.id);
```

### Mistake 4: Assert Doesn't Match Act

```
❌ Inconsistent
// Act
let count = get_count();

// Assert
assert!(count > 0);  // Different than what we acted on

✅ Consistent
// Act
let count = get_count();

// Assert
assert_eq!(count, expected_count);
```

## How chicago-tdd-tools Enforces AAA

The framework enforces AAA at **compile time** using type-level state machines:

```rust
test!(test_example, {
    // ✅ Compiler knows we're in Arrange
    let data = setup();

    // ✅ Act transitions us to Acting phase
    let result = operation();

    // ✅ Assert is the final phase
    assert_eq!(result, expected);

    // ❌ This would be a compile error:
    // setup();  // Can't Arrange after Act!
});
```

Type system prevents violating AAA structure.

## AAA at Different Scales

### Unit Test (AAA applies)

```rust
#[test]
fn test_validation() {
    let input = "test@example.com";
    let is_valid = validate_email(input);
    assert!(is_valid);
}
```

### Integration Test (AAA applies)

```rust
fixture_test!(test_database, fixture, {
    let db = fixture.database();
    let user = User::new("Alice");

    db.insert(user.clone())?;

    assert!(db.exists(user.id)?);
});
```

### Property Test (AAA applies)

```rust
test!(test_property, {
    for _ in 0..100 {
        let input = generate_random();
        let result = algorithm(input);
        assert!(property_holds(result));
    }
});
```

## Next Steps

- **Learn testing philosophy** → [Testing Philosophy](testing-philosophy.md)
- **See examples using AAA** → [Example Inventory](../reference/example-inventory.md)
- **Run examples** → [Getting Started](../tutorials/getting-started.md)

---

AAA makes tests clear, maintainable, and correct.
