# Learning Testing Patterns: 90-Minute Mastery

> 🎓 **TUTORIAL** | Master the 5 fundamental testing patterns

This tutorial guides you through the 5 testing patterns in a natural progression. Each builds on the previous one.

**Time**: ~90 minutes | **Difficulty**: Beginner | **Prerequisites**: Basic Rust knowledge

---

## Module Overview

| Pattern | Time | Focus |
|---------|------|-------|
| Pattern 1: AAA | 15 min | Test structure and readability |
| Pattern 2: Error Paths | 20 min | Testing failures and error cases |
| Pattern 3: Boundaries | 15 min | Edge cases and limits |
| Pattern 4: Resource Cleanup | 15 min | Automatic cleanup and fixtures |
| Pattern 5: Real Collaborators | 15 min | Testing with real dependencies |
| **Practice Exercises** | **15 min** | **Apply what you've learned** |

---

## Part 1: AAA Pattern (15 minutes)

**Goal**: Understand how to structure tests for readability

### The Three Phases

Every test has three parts:

1. **Arrange**: Set up the test data and environment
2. **Act**: Execute the behavior you're testing
3. **Assert**: Verify the result is correct

### Example Structure

```rust
test!(test_name, {
    // Arrange: Set up
    let input = 42;

    // Act: Execute
    let result = process(input);

    // Assert: Verify
    assert_eq!(result, 84);
});
```

### Key Points

✅ **Do**: Label each phase with comments
✅ **Do**: Test one behavior per test
✅ **Do**: Make test names describe what they test

❌ **Don't**: Mix phases together
❌ **Don't**: Put logic in assertions
❌ **Don't**: Test multiple behaviors in one test

### Checkpoint Question

How would you structure a test that:
1. Creates a user
2. Verifies the user was created correctly

**Answer**: Create → Verify (Pattern 1: AAA structure)

---

## Part 2: Error Path Testing (20 minutes)

**Goal**: Test both success AND failure cases

### Why Error Testing Matters

Tests that only verify success hide bugs. Real systems fail. You must test how code behaves when things go wrong.

### Success Path vs. Error Path

```rust
// Success path: Everything works
test!(test_add_user_success, {
    // Arrange
    let mut db = Database::new();

    // Act
    let result = db.add_user("alice@example.com");

    // Assert
    assert_ok!(&result);  // Should succeed
});

// Error path: Something goes wrong
test!(test_add_duplicate_user_fails, {
    // Arrange
    let mut db = Database::new();
    db.add_user("alice@example.com").ok();  // Add once

    // Act
    let result = db.add_user("alice@example.com");  // Try to add again

    // Assert
    assert_err!(&result);  // Should fail
});
```

### What to Test

For each behavior, test:

1. **Normal case** - Everything works perfectly
2. **Invalid input** - Bad data
3. **Boundary case** - Limits (empty, max size, etc.)
4. **Error case** - Something fails
5. **Concurrent case** (if applicable) - Multiple threads

### Checkpoint Question

You have a function `divide(a, b)` that returns `Result<i32, Error>`.

**What test cases should you write?**

Answers:
1. ✅ Normal: `divide(10, 2)` → `Ok(5)`
2. ✅ Error: `divide(10, 0)` → `Err(Division by zero)`
3. ✅ Boundary: Negative numbers?  Large numbers?

---

## Part 3: Boundary Conditions (15 minutes)

**Goal**: Systematically test edge cases

### What Are Boundaries?

Boundaries are the limits where bugs often hide:

- Empty collections (size 0)
- Full collections (size = capacity)
- Negative numbers
- Maximum values
- Null/None cases
- First vs. Last elements

### Boundary Testing Pattern

```rust
test!(test_boundaries, {
    // Arrange
    let mut list = List::new();

    // Test: Empty list (boundary)
    assert_eq!(list.len(), 0);

    // Test: Add one (crossing boundary)
    list.add(42);
    assert_eq!(list.len(), 1);

    // Test: Add many (stress)
    for i in 0..1000 {
        list.add(i);
    }
    assert_eq!(list.len(), 1001);
});
```

### Common Boundaries to Test

| Boundary | Test Values | Example |
|----------|-------------|---------|
| Empty | 0 items | `vec![]` |
| Single | 1 item | `vec![1]` |
| Multiple | 2-many items | `vec![1, 2, 3...]` |
| Negative | Negative numbers | `-1, -100` |
| Maximum | Max value | `u32::MAX` |
| Minimum | Min value | `u32::MIN` |

### Checkpoint Question

You're testing a `split_string(s, limit)` function.

**What boundary cases should you test?**

Answers:
1. ✅ Empty string
2. ✅ Limit = 0
3. ✅ Limit = 1
4. ✅ Limit > length
5. ✅ Very long string

---

## Part 4: Resource Cleanup (15 minutes)

**Goal**: Ensure tests clean up automatically

### Why Cleanup Matters

Tests often create resources:
- Files
- Database connections
- Network sockets
- Memory allocations

If not cleaned up, tests **leak resources** and fail.

### The Fixture Pattern

Chicago TDD provides `TestFixture` for automatic cleanup:

```rust
test!(test_with_cleanup, {
    // Create fixture - sets up
    let fixture = TestFixture::new()?;

    // Use the fixture
    fixture.set_metadata("key", "value");

    // No need to cleanup - happens automatically
    // when fixture is dropped!
});  // Cleanup happens here automatically
```

### What Gets Cleaned Up

- File handles closed
- Connections closed
- Memory freed
- Temporary directories removed

All automatic! ✅

### Checkpoint Question

You're writing tests that use a database. What should you do?

Answer: Use a `TestFixture` that:
- Connects to test database on creation
- Runs tests
- Closes connection automatically on drop

---

## Part 5: Real Collaborators (15 minutes)

**Goal**: Test with real implementations, not mocks

### Mocks vs. Real Implementations

```rust
// ❌ Using a mock (hides integration bugs)
test!(test_with_mock, {
    let mock_client = MockApiClient::new();  // Fake
    mock_client.set_response(Ok(User { ... }));
    let result = my_service.fetch_user(&mock_client);
    assert_ok!(&result);
    // But does real client work? Who knows!
});

// ✅ Using real implementation (catches real bugs)
test!(test_with_real_client, {
    let real_client = RealApiClient::new();  // Actually calls API
    let result = my_service.fetch_user(&real_client);
    assert_ok!(&result);
    // Proves it works with the real API
});
```

### When to Use Real Collaborators

✅ **Always preferred**:
- Database queries
- File I/O
- Web services
- Network calls

✅ **When it's fast** (< 100ms):
- Real implementations

❌ **Only when necessary**:
- Slow external services (use test doubles)
- Non-deterministic behavior
- Expensive operations

### The Philosophy

**Chicago TDD principle**: Test behavior with real dependencies. This proves code actually works, not just in the test's imagination.

### Checkpoint Question

You're testing a payment service. What should you test against?

Answer: A real (test) payment processor, not a mock. Why? Because the real one is what you'll use in production.

---

## Putting It Together: Practice Exercise (15 minutes)

### Exercise: User Registration Service

Write tests for this service:

```rust
pub struct UserService {
    db: UserDatabase,
}

impl UserService {
    pub fn register(&mut self, email: &str, password: &str) -> Result<User, Error> {
        // Validate input
        if email.is_empty() || password.len() < 8 {
            return Err(Error::InvalidInput);
        }

        // Check if user exists
        if self.db.user_exists(email) {
            return Err(Error::UserExists);
        }

        // Create user
        let user = User::new(email, password);
        self.db.save(&user)?;
        Ok(user)
    }
}
```

### What Tests Should You Write?

Using all 5 patterns, write tests for:

1. **Pattern 1 (AAA)**: Structure tests clearly
2. **Pattern 2 (Error Paths)**:
   - Empty email ❌
   - Short password ❌
   - User already exists ❌
   - Valid registration ✅
3. **Pattern 3 (Boundaries)**:
   - Minimum password length (7, 8, 9 chars)
   - Very long email
4. **Pattern 4 (Resource Cleanup)**:
   - Use fixture for database cleanup
5. **Pattern 5 (Real Collaborators)**:
   - Use real UserDatabase, not mock

### Example Solution Framework

```rust
test!(test_register_success, {
    // Arrange
    let fixture = TestFixture::new()?;
    let db = fixture.test_database();  // Real database
    let mut service = UserService::new(db);

    // Act
    let result = service.register("alice@example.com", "password123");

    // Assert
    assert_ok!(&result);
    let user = result.unwrap();
    assert_eq!(user.email, "alice@example.com");
});
```

---

## Summary: The 5 Testing Patterns

| Pattern | Goal | Use When |
|---------|------|----------|
| **Pattern 1: AAA** | Read­able tests | Every test |
| **Pattern 2: Error Paths** | Test failures | Every function |
| **Pattern 3: Boundaries** | Test limits | Every input |
| **Pattern 4: Resource Cleanup** | Auto cleanup | Complex tests |
| **Pattern 5: Real Collaborators** | Real deps | Integration tests |

---

## Next Steps

### Immediate (Today)

Write 5-10 tests using all patterns for something in your codebase.

### Short-term (This Week)

Review existing tests. Rewrite any that don't follow these patterns.

### Long-term (This Month)

Learn [Architecture Patterns](learning-architecture-patterns.md) to organize your code structure.

---

## Checkpoint: Do You Know...?

- [ ] How to structure a test with AAA?
- [ ] How to test both success and error cases?
- [ ] What boundary conditions to test?
- [ ] How fixtures provide automatic cleanup?
- [ ] Why real collaborators are better than mocks?

If you answered yes to all, you've mastered Testing Patterns! 🎉

---

## Resources

- **Full Pattern Details**: [Testing Patterns](../testing-patterns/)
- **Decision Guide**: [Choosing Your Pattern](../choosing-your-pattern.md)
- **All Patterns**: [Quick Reference](../all-patterns-reference.md)

---

**Congratulations!** You now understand the 5 fundamental testing patterns. Next, learn how to organize your code with Architecture Patterns.
