# How to Run Integration Feature Examples

**Quick reference** for integration testing: Docker containers, testcontainers support.

## Quick Commands

```bash
# Show all integration features
cargo run -- integ stat

# List available integration examples
cargo run -- integ list

# Run Docker container tests
cargo run -- integ contain

# Run all integration examples
cargo run -- integ exec --names "contain"
```

## Integration Features Overview

| Feature | Purpose | Requirements |
|---------|---------|--------------|
| **Testcontainers** | Docker container support | Docker running, feature: `testcontainers` |

## Prerequisites

### Docker Installation

```bash
# Verify Docker is installed
docker ps

# If needed, install Docker
# Visit: https://www.docker.com/products/docker-desktop
```

### Feature Enablement

```bash
cargo run --features testcontainers -- integ stat
```

Or add to `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1", features = ["testcontainers"] }
```

## Testcontainers Integration

```bash
cargo run --features testcontainers -- integ contain
```

**What it does:**
Automatically spins up Docker containers for testing. No manual Docker commands needed.

**Use when:**
- Testing with real databases (PostgreSQL, MySQL)
- Testing with Redis, Elasticsearch, etc.
- Integration testing with services
- End-to-end testing

**Supported containers:**
- PostgreSQL
- MySQL
- Redis
- Elasticsearch
- MongoDB
- And many more

### Example: PostgreSQL Testing

```rust
use chicago_tdd_tools::integration::testcontainers::*;

fixture_test!(test_with_postgres, fixture, {
    // Arrange: Spin up Postgres automatically
    let container = fixture
        .postgres_container()
        .expect("Postgres should start");

    let conn_string = container.connection_string();

    // Act: Execute database operations
    let result = container
        .execute_query("SELECT COUNT(*) FROM information_schema.tables;")
        .await;

    // Assert
    assert_ok!(result);

    // Cleanup: Container stops automatically
});
```

### Example: Redis Testing

```rust
fixture_test!(test_with_redis, fixture, {
    // Arrange
    let redis = fixture
        .redis_container()
        .expect("Redis should start");

    let conn = redis.connection();

    // Act: Use Redis
    conn.set("key", "value")?;
    let value = conn.get("key")?;

    // Assert
    assert_eq!(value, "value");

    // Cleanup: Automatic
});
```

### Example: Multiple Containers

```rust
fixture_test!(test_full_stack, fixture, {
    // Arrange: Start multiple services
    let db = fixture.postgres_container()?;
    let cache = fixture.redis_container()?;

    // Act: Use both services together
    let user_id = db.insert_user("Alice")?;
    cache.set(&format!("user:{}", user_id), "alice_data")?;

    // Assert
    let cached = cache.get(&format!("user:{}", user_id))?;
    assert_eq!(cached, "alice_data");
});
```

## Running with Docker

### Verify Docker is Ready

```bash
docker ps
```

Should show running containers (might be empty).

### Run Container Tests

```bash
cargo run --features testcontainers -- integ contain
```

### Run Full Integration Suite

```bash
cargo run --features testcontainers -- test-all
```

Includes both unit and integration tests.

## Combining Integration with Testing Features

Integration testing + testing features:

```rust
fixture_test!(test_db_mutation_testing, fixture, {
    // Integration: Real database
    let db = fixture.postgres_container()?;

    // Testing: Mutation detection
    let mut data = db.query("SELECT * FROM users")?;

    // Mutation: What if DELETE was used?
    db.insert_record(&record)?;

    // Assert: Must catch the mutation
    assert!(db.record_exists(&record.id)?);
});
```

## Best Practices

1. **Use fixtures** - Automatic container lifecycle management
2. **One container per test** - Fresh isolation
3. **Use connection strings** - Flexible configuration
4. **Cleanup automatic** - Containers stop on fixture drop
5. **Test against real services** - Not mocks

## Performance Considerations

Container startup times:
- PostgreSQL: ~2-3 seconds
- Redis: ~0.5 seconds
- Elasticsearch: ~5-10 seconds

```bash
# Timeout for integration tests
cargo run --features testcontainers -- test-integration
```

Default timeout: 30 seconds per test

## Docker without Testcontainers

If you prefer manual Docker:

```bash
# Start containers manually
docker run -d --name test-db -e POSTGRES_PASSWORD=test postgres:15

# Connect in tests
let conn = connect("postgres://localhost:5432/test")?;

# Cleanup
docker stop test-db && docker rm test-db
```

But testcontainers is much easier!

## Troubleshooting

**Q: "Docker daemon is not running"**
A: Start Docker Desktop or ensure Docker service is running.

**Q: "Feature 'testcontainers' is required"**
A: Enable feature:
```bash
cargo run --features testcontainers -- integ stat
```

**Q: "Container fails to start"**
A: Check Docker logs:
```bash
docker logs test-postgres  # View container logs
docker ps -a              # See all containers including failed
```

**Q: "Tests timeout"**
A: Container startup might be slow. Increase timeout or check Docker resources.

**Q: "Port already in use"**
A: Another container is using the port:
```bash
docker ps  # Find what's running
docker stop <container_id>  # Stop it
```

## Integration Test Workflow

```
1. Write unit tests (core features)
2. Add integration tests (testcontainers)
3. Run with: cargo run --features testcontainers -- integ contain
4. Verify against real services
5. Copy to your project
```

## Copying Integration Tests

Copy testcontainer patterns to your project:

```rust
// Your project's integration test
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::integration::testcontainers::*;

fixture_test!(test_my_db_integration, fixture, {
    // Arrange: Container
    let db = fixture.postgres_container()?;

    // Act: Your code
    let result = your_db_operation(&db)?;

    // Assert
    assert_ok!(result);
});
```

## Combining All Features

Ultimate test suite:

```bash
# Core features
cargo run -- core stat

# Testing features
cargo run --all-features -- test stat

# Validation features
cargo run -- valid stat

# Observability features
cargo run --features otel,weaver -- obs stat

# Integration features
cargo run --features testcontainers -- integ contain
```

## Next Steps

- **Copy to your project** → [Copying Examples](../tutorials/copying-examples.md)
- **Observability features** → [Observability Features](observability-features.md)
- **Testing features** → [Testing Features Guide](testing-features.md)
- **See all examples** → [Example Inventory](../reference/example-inventory.md)

---

Test your application with real services using testcontainers.
