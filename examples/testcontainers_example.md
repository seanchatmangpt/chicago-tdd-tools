# Testcontainers Example

**Category:** How-To Guide
**Level:** Intermediate
**Prerequisites:** Docker installed and running
**Features Required:** `testcontainers`

---

## Overview

This example demonstrates minimal 80/20 testcontainers usage for Chicago TDD integration testing. Shows how to use Docker containers in tests with automatic lifecycle management.

**What you'll learn:**
- Creating and managing Docker containers
- Exposing ports and mapping to host
- Setting environment variables in containers
- Executing commands in running containers
- Automatic container cleanup

---

## Quick Start

```bash
# Ensure Docker is running
docker ps

# Run example
cargo run --example testcontainers_example --features testcontainers
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Docker Desktop installed and running
- Chicago TDD Tools with `testcontainers` feature

**Install Docker:**
- [Docker Desktop](https://www.docker.com/products/docker-desktop)

**Add to Cargo.toml:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["testcontainers"] }
```

**Verify Docker:**
```bash
docker ps  # Should show running containers or empty list (not error)
docker info  # Should show Docker info
```

---

## Key Concepts

### Container Lifecycle

Containers are automatically created, started, and cleaned up:

1. **Create:** `GenericContainer::new()`
2. **Start:** Automatic on creation
3. **Use:** Execute commands, access ports
4. **Cleanup:** Automatic on drop

### Container Types

- **Basic containers:** Start and exit immediately
- **Service containers:** Stay running (postgres, redis, nginx)
- **Command containers:** Run custom commands to keep alive

---

## Code Examples

### Example 1: Basic Container

```rust
use chicago_tdd_tools::testcontainers::*;

let client = ContainerClient::new();
let container = GenericContainer::new(client.client(), "alpine", "latest")?;
println!("✓ Container created successfully");
// Container automatically cleaned up on drop
```

**Note:** Basic containers exit immediately - cannot use `exec()`.

### Example 2: Container with Exposed Ports

```rust
use chicago_tdd_tools::testcontainers::*;

let client = ContainerClient::new();
let container = GenericContainer::with_ports(
    client.client(),
    "alpine",
    "latest",
    &[DEFAULT_HTTP_PORT, 443],
)?;

let host_port_80 = container.get_host_port(DEFAULT_HTTP_PORT)?;
let host_port_443 = container.get_host_port(443)?;

println!("Container port {} -> host port {}", DEFAULT_HTTP_PORT, host_port_80);
println!("Container port 443 -> host port {}", host_port_443);
```

**Key Points:**
- Container ports mapped to random host ports
- Use `get_host_port()` to get host port

### Example 3: Container with Environment Variables

```rust
use chicago_tdd_tools::testcontainers::*;
use std::collections::HashMap;

let mut env_vars = HashMap::new();
env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
env_vars.insert("ANOTHER_VAR".to_string(), "another_value".to_string());

let container = GenericContainer::with_env(
    client.client(),
    "alpine",
    "latest",
    env_vars,
)?;
```

### Example 4: Executing Commands

```rust
use chicago_tdd_tools::testcontainers::*;
use chicago_tdd_tools::testcontainers::exec::SUCCESS_EXIT_CODE;

// Create container that stays running
let container = GenericContainer::with_command(
    client.client(),
    "alpine",
    "latest",
    "sleep",
    &["infinity"],
    None,  // No entrypoint override
)?;

// Execute command
let result = container.exec("echo", &["hello", "from", "container"])?;
assert_eq!(result.exit_code, SUCCESS_EXIT_CODE);
println!("Output: {}", result.stdout.trim());
```

**Key Points:**
- Use `with_command()` to keep container running
- `exec()` executes commands in running container
- Check `exit_code` with `SUCCESS_EXIT_CODE` constant

### Example 5: Entrypoint Override

```rust
// For images like otel/weaver that need entrypoint override:
let container = GenericContainer::with_command(
    client.client(),
    "otel/weaver",
    "latest",
    "sleep",
    &["infinity"],
    Some(&["/bin/sh"]),  // Override entrypoint
)?;

let result = container.exec("weaver", &["--version"])?;
```

---

## Common Patterns

### Pattern 1: Database Testing

```rust
let postgres = GenericContainer::with_ports(
    client.client(),
    "postgres",
    "latest",
    &[5432],
)?;

let port = postgres.get_host_port(5432)?;
let connection_string = format!("postgres://localhost:{port}/test");
// Connect and test
```

### Pattern 2: Redis Testing

```rust
let redis = GenericContainer::with_ports(
    client.client(),
    "redis",
    "latest",
    &[6379],
)?;

let port = redis.get_host_port(6379)?;
// Connect to redis://localhost:{port}
```

### Pattern 3: Custom Service Testing

```rust
let mut env = HashMap::new();
env.insert("SERVICE_PORT".to_string(), "8080".to_string());

let service = GenericContainer::with_env(
    client.client(),
    "my-service",
    "latest",
    env,
)?;
```

---

## Best Practices

### 1. Use Automatic Cleanup

```rust
{
    let container = GenericContainer::new(...)?;
    // Use container
} // Automatic cleanup on drop
```

### 2. Check Command Exit Codes

```rust
let result = container.exec("command", &[])?;
assert_eq!(result.exit_code, SUCCESS_EXIT_CODE, "Command should succeed");
```

### 3. Test Both Success and Error Paths

```rust
// Success path
let result = container.exec("valid_command", &[])?;
assert_eq!(result.exit_code, SUCCESS_EXIT_CODE);

// Error path
let error_result = container.exec("invalid_command", &[]);
assert!(error_result.is_err() || error_result.unwrap().exit_code != SUCCESS_EXIT_CODE);
```

---

## Troubleshooting

### Error: "Cannot connect to Docker daemon"

**Cause:** Docker not running

**Fix:**
1. Start Docker Desktop
2. Verify with `docker ps`
3. Ensure Docker is accessible

### Error: "testcontainers feature required"

**Cause:** Feature not enabled

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["testcontainers"] }
```

### Container Exits Immediately

**Cause:** Image exits after startup

**Fix:** Use `with_command()` to keep container running:
```rust
let container = GenericContainer::with_command(
    client.client(),
    "alpine",
    "latest",
    "sleep",
    &["infinity"],
    None,
)?;
```

### Port Already in Use

**Cause:** Random host port conflicts

**Fix:** Testcontainers uses random ports automatically. If conflict, restart test.

---

## Next Steps

After mastering testcontainers, explore:

1. **[OTEL/Weaver Testing](otel_weaver_testing.md)** - Observability with containers
2. **[CLI Testing](cli_testing.md)** - CLI testing patterns
3. **[Concurrency Testing](concurrency_testing.md)** - Thread safety

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [testcontainers-rs](https://docs.rs/testcontainers/) - Complete guide
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation

---

## Reference

### Key Types

- `ContainerClient` - Manages Docker container lifecycle
- `GenericContainer` - Represents a Docker container
- `ExecResult` - Result of command execution

### Key Functions

- `ContainerClient::new() -> ContainerClient`
- `GenericContainer::new(client, image, tag) -> Result<GenericContainer, Error>`
- `GenericContainer::with_ports(client, image, tag, ports) -> Result<GenericContainer, Error>`
- `GenericContainer::with_env(client, image, tag, env_vars) -> Result<GenericContainer, Error>`
- `GenericContainer::with_command(client, image, tag, command, args, entrypoint) -> Result<GenericContainer, Error>`
- `GenericContainer::get_host_port(container_port) -> Result<u16, Error>`
- `GenericContainer::exec(command, args) -> Result<ExecResult, Error>`

### Constants

- `DEFAULT_HTTP_PORT` - Default HTTP port (80)
- `SUCCESS_EXIT_CODE` - Successful command exit code (0)

---

**Quality is the default. Prevention beats detection.**

*Example: testcontainers_example.rs | Version: 1.2.0 | Updated: 2025-11-15*
