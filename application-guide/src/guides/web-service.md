# Testing a Web Service

Complete example of testing a web service with Chicago TDD Tools.

## Project Structure

```
myservice/
├── src/
│   ├── main.rs
│   ├── handlers/
│   │   ├── users.rs
│   │   ├── orders.rs
│   │   └── auth.rs
│   └── models/
└── tests/
    ├── integration_tests.rs
    └── api_tests.rs
```

## Testing Endpoints

### Example: GET /users

```rust
test!(test_get_users, {
    let client = TestClient::new()?;

    // Act
    let response = client.get("/users")?;

    // Assert
    assert_eq!(response.status, 200);
    let users: Vec<User> = response.json()?;
    assert!(!users.is_empty());
});
```

### Example: POST /users

```rust
test!(test_create_user, {
    let client = TestClient::new()?;

    let user_data = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("email", "alice@example.com")
        .build_json()?;

    // Act
    let response = client.post("/users", &user_data)?;

    // Assert
    assert_eq!(response.status, 201);  // Created
    assert_ok!(&response.json::<User>());
});
```

### Example: Error Cases

```rust
test!(test_create_user_validation_error, {
    let client = TestClient::new()?;

    let invalid_data = TestDataBuilder::new()
        .with_var("email", "not_an_email")
        .build_json()?;

    // Act
    let response = client.post("/users", &invalid_data)?;

    // Assert
    assert_eq!(response.status, 400);  // Bad request
});
```

## Testing with Real Database

Use integration tests with fixtures:

```rust
test!(test_with_database, {
    let fixture = TestFixture::new()?;

    // Fixture provides database connection
    let db = fixture.db_connection();

    // Create user in database
    let user = db.create_user("Alice", "alice@example.com")?;

    // Test retrieval
    let retrieved = db.get_user(user.id)?;
    assert_eq!(retrieved.name, "Alice");
});
```

## Testing Authentication

```rust
test!(test_auth_required, {
    let client = TestClient::new()?;

    // No authentication
    let response = client.get("/protected")?;
    assert_eq!(response.status, 401);  // Unauthorized

    // With authentication
    let token = client.login("alice", "password")?;
    let response = client.get_with_auth("/protected", &token)?;
    assert_eq!(response.status, 200);
});
```

## Best Practices

✅ **Do:**
- Test with real database (in tests)
- Test all HTTP methods (GET, POST, PUT, DELETE)
- Test error cases (400, 401, 404, 500)
- Test response structure
- Use fixtures for isolation

❌ **Don't:**
- Mock the entire HTTP layer
- Test framework code
- Hard-code full responses

## Testing Workflow

```bash
# 1. Unit tests (fast)
cargo make test-unit

# 2. Integration tests (requires database)
cargo make test-integration

# 3. Full CI simulation
cargo make ci-local
```

See: [Advanced Techniques](../advanced/README.md)
