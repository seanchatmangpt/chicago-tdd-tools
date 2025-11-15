# Testing Philosophy

**Understanding** the philosophy behind Chicago-style TDD and how it shapes the playground.

## Chicago-Style TDD (Classicist)

The framework and playground are built on **Chicago-style TDD**, also called the **Classicist** approach.

### Core Principle

**Test behavior, not implementation. Use real dependencies.**

## What Makes Chicago-Style Different?

### Chicago-Style (Classicist)

- ✅ Test **behavior** - What does the code do?
- ✅ Use **real collaborators** - Database, cache, services
- ✅ **Minimize mocking** - Only mock external systems
- ✅ **Integration focus** - How do components work together?

```rust
#[test]
fn test_cache_stores_user() {
    // Use real cache (not mocked)
    let cache = Cache::new();
    let user = User::new("Alice");

    cache.insert(user.clone());

    // Verify actual behavior
    assert_eq!(cache.get(user.id), Some(user));
}
```

### London-Style (Mockist) - By Contrast

- ❌ Test **implementation details** - How does it work?
- ❌ Use **mocks everywhere** - Mock all dependencies
- ❌ **Maximize mocking** - Every dependency is mocked
- ❌ **Specification focus** - Does it match spec?

```rust
#[test]
fn test_service_calls_repo() {
    // Mock the repository
    let repo = MockRepository::new();
    repo.expect_get().returning(|| Ok(user));

    let service = UserService::new(Box::new(repo));

    service.fetch_user(1)?;

    repo.assert_called_once();  // Tests implementation!
}
```

## Why Chicago-Style?

### Reason 1: Closer to Reality

Tests using real dependencies better match what happens in production.

```rust
// Chicago: Tests real cache behavior
let cache = Cache::new();
cache.insert(user);
assert_eq!(cache.get(id), Some(user));

// London: Tests that service calls cache
repo.expect_get().returning(...);
assert!(repo.was_called());
```

### Reason 2: More Maintainable

Mocking couples tests to implementation. Changes to implementation break tests even if behavior is correct.

```rust
// Chicago: If cache implementation changes,
// test still verifies it works
cache.insert(user);
assert_eq!(cache.get(id), Some(user));

// London: If we use different cache, mock breaks
// (even though behavior is identical)
```

### Reason 3: Clearer Intent

Real dependencies show **what you're actually testing**.

```rust
// Chicago: Clear - we're testing cache behavior
let cache = Cache::new();
let result = cache.insert(user);
assert!(result.is_ok());

// London: Unclear - we're testing that code called a mock
repo.expect_insert().returning(Ok(()));
service.process(user);
assert!(repo.was_called());
```

### Reason 4: Better Refactoring

You can refactor without changing tests (if behavior is same).

```rust
// Original implementation
fn add(a: i32, b: i32) -> i32 { a + b }

// Refactored implementation
fn add(a: i32, b: i32) -> i32 {
    [a, b].iter().sum()
}

// Chicago test still passes - behavior unchanged
assert_eq!(add(5, 3), 8);

// London test fails - implementation details changed
```

## Core Testing Principles

### Principle 1: Test Behavior

Test **what** the code does, not **how** it does it.

```rust
// ✅ Tests behavior
fn test_user_can_login() {
    let user = create_user();
    let result = user.verify_password(correct_password);
    assert!(result);
}

// ❌ Tests implementation
fn test_user_stores_password_hash() {
    let user = create_user();
    assert!(user.password_hash.contains('$'));  // Implementation detail!
}
```

### Principle 2: Use Real Objects

Use real objects unless you're testing external systems.

```rust
// ✅ Real cache
let cache = Cache::new();
cache.insert(user);
assert_eq!(cache.get(id), Some(user));

// ❌ Mocked cache (unless testing code that calls cache)
let cache = MockCache::new();
cache.expect_insert().returning(Ok(()));
```

### Principle 3: Test One Behavior Per Test

Each test verifies one specific behavior.

```rust
// ✅ One behavior
#[test]
fn test_user_validates_email() {
    assert!(validate_email("alice@example.com"));
}

// ❌ Multiple behaviors
#[test]
fn test_user_validation() {
    assert!(validate_email("alice@example.com"));
    assert!(validate_password("secureP@ss123"));
    assert!(validate_phone("+1234567890"));
}
```

### Principle 4: Test Business Rules

Focus on **business value**, not implementation details.

```rust
// ✅ Business rule
#[test]
fn test_premium_users_get_discount() {
    let user = create_premium_user();
    let discount = calculate_discount(user);
    assert_eq!(discount, 0.20);  // 20% discount
}

// ❌ Implementation detail
#[test]
fn test_discount_multiplied_by_quantity() {
    let discount = calculate_discount(...);
    assert!(discount * quantity == result);
}
```

## How This Shapes the Playground

### The Playground Teaches Chicago-Style

All examples use real objects and real behavior:

```rust
// Fixtures use real setup
fixture_test!(test_example, fixture, {
    // Real database, real cache, real services
    let db = fixture.database();
    let result = db.insert(user);
    assert!(result.is_ok());
});

// Builders create real test data
let user = UserBuilder::new()
    .with_name("Alice")
    .build();

// Assertions verify real behavior
assert_eq!(cache.get(id), Some(expected));
```

### No Mock-Heavy Examples

The playground **doesn't teach mocking** because:

1. Use real objects for unit tests (Chicago)
2. Only mock external systems (APIs, databases you can't run)
3. Mocking is an escape hatch, not primary testing tool

```rust
// ✅ Chicago: Use real objects
let cache = Cache::new();
cache.insert(data);
assert_eq!(cache.get(id), Some(data));

// ❌ London: Mock everything
let cache = MockCache::new();
cache.expect_insert().returning(Ok(()));
// Now testing implementation, not behavior!
```

## When Is Mocking Appropriate?

### Mocking is OK for:

1. **External Services** - APIs, cloud services
   ```rust
   let api = MockExternalApi::new();
   api.expect_fetch().returning(|| Ok(data));
   ```

2. **Expensive Resources** - Databases (for unit tests)
   ```rust
   let db = MockDatabase::new();  // Only if real DB is too slow
   ```

3. **Non-Deterministic Systems** - Random, time-based
   ```rust
   let clock = MockClock::new();
   clock.set_time(specific_time);
   ```

### Mocking is NOT appropriate for:

1. **Collaborating Objects** - Use real objects
   ```rust
   let cache = Cache::new();  // Real, not mocked
   let user = User::new();    // Real, not mocked
   ```

2. **Business Logic** - Should be tested with real objects
   ```rust
   let result = calculate_discount(user);  // Real logic
   ```

3. **Simple Dependencies** - Creating real objects is easier than mocking
   ```rust
   let config = Config::new();  // Easier than MockConfig
   ```

## Testing Pyramid (Chicago-Style)

```
        /\
       /  \  E2E / System
      /    \     (Few tests, slow)
     /______\
    /        \
   /          \  Integration
  /            \ (Some tests, medium speed)
 /              \
/______________\
|                |  Unit
|________________| (Many tests, fast, real objects)
```

- **Unit tests** (bottom) - Fast, many, real objects
- **Integration tests** (middle) - Medium speed, real systems
- **E2E tests** (top) - Slow, few, full system

The playground focuses on **unit + integration tests** with **real objects**.

## Benefits of This Approach

### Benefit 1: Maintainability

Tests don't break when implementation changes (if behavior is same).

### Benefit 2: Confidence

Tests using real objects better simulate production behavior.

### Benefit 3: Simplicity

Fewer mocks = simpler tests = easier to understand.

### Benefit 4: Faster Feedback

Real objects often just work; mocks require setup.

### Benefit 5: Refactoring Safety

You can refactor with confidence; tests ensure behavior didn't change.

## Common Concerns About Chicago-Style

### "What about speed?"

Real objects are usually **fast enough**:
- Database: ~milliseconds
- Cache: ~microseconds
- Services: Network-dependent

Only mock if performance is actually a problem.

### "What about external APIs?"

Mock external services - you have to:
```rust
let api = MockGithubApi::new();
api.expect_get_user().returning(|| Ok(user));
```

But mock the **service**, not internal collaborators.

### "What about isolation?"

Isolation isn't the goal - **correctness** is.

Chicago-style tests are well-isolated by focusing on one behavior per test, not by mocking everything.

## Summary

Chicago-style TDD (used in the playground):

✅ Tests **behavior**, not implementation
✅ Uses **real objects**, minimal mocking
✅ **Integrates** components well
✅ **Maintainable** and refactor-safe
✅ **Closer to production** reality

The playground teaches this approach because it's:
- Simpler to understand
- More maintainable
- Better at catching real bugs
- Less coupled to implementation

## Next Steps

- **See AAA pattern** → [AAA Pattern](aaa-pattern.md)
- **Playground philosophy** → [Playground Philosophy](playground-philosophy.md)
- **Run examples** → [Getting Started](../tutorials/getting-started.md)

---

Chicago-style TDD keeps tests focused on behavior and closer to production reality.
