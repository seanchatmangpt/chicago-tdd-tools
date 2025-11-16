# Integration Testing with Docker

> 🔧 **HOW-TO** | 📚 **REFERENCE** | Test with real services using Docker

Test with real services using Docker containers.

## Why Docker for Testing?

Docker provides:
- Real service instances (not mocks)
- Isolated test environment
- Reproducible results
- Easy cleanup

## Prerequisites

```bash
# Ensure Docker is running
docker --version

# Enable testcontainers feature
[dev-dependencies]
chicago-tdd-tools = { version = "1.3", features = ["testcontainers"] }
```

## Docker Compose for Tests

Create `docker-compose.test.yml`:

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: test_user
      POSTGRES_PASSWORD: test_password
      POSTGRES_DB: test_db
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
```

Run tests:

```bash
# Start services
docker-compose -f docker-compose.test.yml up -d

# Run tests
cargo make test-integration

# Stop services
docker-compose -f docker-compose.test.yml down
```

## Testing with Database

```rust
test!(test_with_postgres, {
    // Fixture provides database connection
    let fixture = TestFixture::new()?;

    // Create test user
    fixture.db().create_user("alice", "alice@example.com")?;

    // Query database
    let user = fixture.db().get_user_by_email("alice@example.com")?;
    assert_eq!(user.name, "alice");
});
```

## Testing with Redis

```rust
test!(test_with_redis, {
    let fixture = TestFixture::new()?;

    // Use Redis from fixture
    let cache = fixture.redis();

    // Set value
    cache.set("key", "value")?;

    // Get value
    let value = cache.get("key")?;
    assert_eq!(value, "value");
});
```

## Complete Integration Test

```rust
test!(complete_integration_test, {
    let fixture = TestFixture::new()?;
    let db = fixture.db();
    let cache = fixture.redis();

    // 1. Create user in database
    let user = db.create_user("alice", "alice@example.com")?;

    // 2. Cache user
    cache.set(&format!("user:{}", user.id), &user.to_json())?;

    // 3. Verify database
    let retrieved = db.get_user(user.id)?;
    assert_eq!(retrieved.email, "alice@example.com");

    // 4. Verify cache
    let cached = cache.get(&format!("user:{}", user.id))?;
    assert!(!cached.is_empty());
});
```

## Handling Docker Failures

If Docker is unavailable:

```bash
# Skip integration tests
WEAVER_ALLOW_SKIP=1 cargo make test-unit

# Or just run unit tests
cargo test --lib
```

## Performance Optimization

Docker containers have overhead:

- Slow: 30-60 seconds per test
- Solution: Batch related tests

```rust
test!(test_db_operations_batch, {
    let fixture = TestFixture::new()?;

    // Test 1: Create
    let user = fixture.db().create_user("alice", "alice@example.com")?;
    assert_ok!(&user);

    // Test 2: Read
    let retrieved = fixture.db().get_user(user.id)?;
    assert_ok!(&retrieved);

    // Test 3: Update
    fixture.db().update_user(user.id, "new_email@example.com")?;
    let updated = fixture.db().get_user(user.id)?;
    assert_eq!(updated.email, "new_email@example.com");

    // Test 4: Delete
    fixture.db().delete_user(user.id)?;
    let result = fixture.db().get_user(user.id);
    assert_err!(&result);

    // One test, multiple operations, one fixture overhead
});
```

## CI/CD Pipeline

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15-alpine
        env:
          POSTGRES_PASSWORD: password
        options: --health-cmd pg_isready

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo make test-integration
```

## Best Practices

✅ **Do:**
- Use Docker for real services
- Batch related tests
- Use fixtures for isolation
- Clean up after tests

❌ **Don't:**
- Mock Docker services
- Share containers between tests
- Run Docker tests in CI for every commit
- Forget about cleanup

## Troubleshooting

### "Docker daemon not running"

Start Docker:

```bash
# macOS
open /Applications/Docker.app

# Linux
sudo systemctl start docker

# Windows
Start Docker Desktop
```

### "Port already in use"

Check ports:

```bash
docker ps  # See running containers
docker stop <container>
```

### Tests Timeout

Increase timeout:

```bash
cargo test --lib -- --test-threads=1  # Sequential
```

## Next Steps

See: [Best Practices](best-practices.md)

