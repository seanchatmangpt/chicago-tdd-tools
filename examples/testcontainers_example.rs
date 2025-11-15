//! # Testcontainers Example - Comprehensive Guide
//!
//! Demonstrates minimal 80/20 testcontainers usage for Chicago TDD integration testing.
//! This example shows how to use Docker containers in tests with automatic lifecycle management.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through container testing patterns:
//!
//! 1. **Create Container Client**: Initialize the container client to manage Docker containers
//! 2. **Start Basic Container**: Create a simple container (exits immediately, for basic use)
//! 3. **Expose Ports**: Map container ports to host ports for service testing
//! 4. **Set Environment Variables**: Configure container environment
//! 5. **Execute Commands**: Run commands in running containers
//! 6. **Handle Entrypoints**: Override container entrypoints when needed (e.g., otel/weaver)
//! 7. **Wait Conditions**: Ensure containers are ready before use
//!
//! ## Explanation: Concepts
//!
//! **Container Lifecycle**: Containers are automatically created, started, and cleaned up.
//! The `GenericContainer` type implements `Drop` to ensure cleanup even on panic.
//!
//! **Container Types**:
//! - **Basic containers** (`GenericContainer::new()`): Start and exit immediately
//! - **Service containers**: Stay running (postgres, redis, nginx)
//! - **Command containers** (`GenericContainer::with_command()`): Run custom commands to keep alive
//!
//! **Entrypoint Override**: Some images (e.g., `otel/weaver`) have entrypoints that interfere
//! with custom commands. Use `with_command()` with `entrypoint` parameter to override.
//!
//! **Port Mapping**: Container ports are mapped to random host ports. Use `get_host_port()`
//! to get the host port for a container port.
//!
//! **Unified API**: `with_command()` is the unified method for all containers:
//! - Regular containers: `entrypoint = None` (uses testcontainers API)
//! - Containers needing override: `entrypoint = Some(&["/bin/sh"])` (uses Docker CLI workaround)
//!
//! ## How-to: Common Tasks
//!
//! - Create a basic container: See `example_basic_container()`
//! - Expose container ports: See `example_container_with_ports()`
//! - Set environment variables: See `example_container_with_env()`
//! - Execute commands: See `example_command_execution()`
//! - Override entrypoint: See `example_entrypoint_override()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `ContainerClient`: Manages Docker container lifecycle
//! - `GenericContainer`: Represents a Docker container
//! - `ExecResult`: Result of command execution
//!
//! **Key Functions**:
//! - `ContainerClient::new() -> ContainerClient`
//! - `GenericContainer::new(client, image, tag) -> Result<GenericContainer, TestcontainersError>`
//! - `GenericContainer::with_command(client, image, tag, command, args, entrypoint) -> Result<GenericContainer, TestcontainersError>`
//! - `GenericContainer::with_ports(client, image, tag, ports) -> Result<GenericContainer, TestcontainersError>`
//! - `GenericContainer::with_env(client, image, tag, env_vars) -> Result<GenericContainer, TestcontainersError>`
//! - `GenericContainer::get_host_port(container_port) -> Result<u16, TestcontainersError>`
//! - `GenericContainer::exec(command, args) -> Result<ExecResult, TestcontainersError>`
//!
//! **Key Constants**:
//! - `DEFAULT_HTTP_PORT`: Default HTTP port (80)
//! - `SUCCESS_EXIT_CODE`: Successful command exit code (0)

use chicago_tdd_tools::testcontainers::exec::SUCCESS_EXIT_CODE;
use chicago_tdd_tools::testcontainers::*;

/// Example: Creating a basic container
///
/// ## How-to: Create a Basic Container
///
/// Use `GenericContainer::new()` to create a simple container. This container
/// starts and exits immediately, suitable for basic container creation testing.
///
/// **Note**: This container cannot be used for `exec()` operations since it exits immediately.
/// For exec operations, use `with_command()` (see `example_command_execution()`).
///
/// ## Reference
///
/// - **Function**: `GenericContainer::new(client, image, tag) -> Result<GenericContainer, TestcontainersError>`
/// - **Parameters**:
///   - `client`: Container client from `ContainerClient::new()`
///   - `image`: Docker image name (e.g., "alpine")
///   - `tag`: Image tag (e.g., "latest")
/// - **Returns**: `Ok(GenericContainer)` on success
/// - **Errors**: Returns error if container creation fails
/// - **Cleanup**: Container automatically cleaned up on drop
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::testcontainers::*;
///
/// let client = ContainerClient::new();
/// let container = GenericContainer::new(client.client(), "alpine", "latest")?;
/// // Container automatically cleaned up on drop
/// ```
fn example_basic_container(client: &ContainerClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n1. Creating basic Alpine container...");
    // Note: GenericContainer::new() creates container but it exits immediately.
    // This container cannot be used for exec() operations.
    // For exec operations, use with_command() (see Example 4).
    let _container = GenericContainer::new(client.client(), "alpine", "latest")?;
    println!("   ✓ Container created successfully");
    println!("   Note: This container exits immediately - cannot use exec()");
    // Container automatically cleaned up on drop
    Ok(())
}

/// Example: Creating a container with exposed ports
///
/// ## How-to: Expose Container Ports
///
/// Use `GenericContainer::with_ports()` to create a container with exposed ports.
/// Container ports are mapped to random host ports. Use `get_host_port()` to
/// get the host port for a container port.
///
/// ## Reference
///
/// - **Function**: `GenericContainer::with_ports(client, image, tag, ports) -> Result<GenericContainer, TestcontainersError>`
/// - **Parameters**:
///   - `ports`: Array of container ports to expose (e.g., `&[80, 443]`)
/// - **Returns**: `Ok(GenericContainer)` with port mappings
/// - **Method**: `get_host_port(container_port) -> Result<u16, TestcontainersError>`
///   - Returns the host port mapped to the container port
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::testcontainers::*;
///
/// let client = ContainerClient::new();
/// let container = GenericContainer::with_ports(
///     client.client(),
///     "alpine",
///     "latest",
///     &[DEFAULT_HTTP_PORT, 443],
/// )?;
/// let host_port = container.get_host_port(DEFAULT_HTTP_PORT)?;
/// ```
fn example_container_with_ports(
    client: &ContainerClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Creating container with exposed ports...");
    let container_with_ports = GenericContainer::with_ports(
        client.client(),
        "alpine",
        "latest",
        &[DEFAULT_HTTP_PORT, 443],
    )?;
    let host_port_80 = container_with_ports.get_host_port(DEFAULT_HTTP_PORT)?;
    let host_port_443 = container_with_ports.get_host_port(443)?;
    println!("   ✓ Container port {} -> host port {}", DEFAULT_HTTP_PORT, host_port_80);
    println!("   ✓ Container port 443 -> host port {}", host_port_443);
    Ok(())
}

/// Example: Creating a container with environment variables
///
/// ## How-to: Set Environment Variables
///
/// Use `GenericContainer::with_env()` to create a container with environment variables.
/// Pass a `HashMap<String, String>` of environment variable names and values.
///
/// ## Reference
///
/// - **Function**: `GenericContainer::with_env(client, image, tag, env_vars) -> Result<GenericContainer, TestcontainersError>`
/// - **Parameters**:
///   - `env_vars`: `HashMap<String, String>` of environment variables
/// - **Returns**: `Ok(GenericContainer)` with environment variables set
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::testcontainers::*;
/// use std::collections::HashMap;
///
/// let mut env_vars = HashMap::new();
/// env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
/// let container = GenericContainer::with_env(
///     client.client(),
///     "alpine",
///     "latest",
///     env_vars,
/// )?;
/// ```
fn example_container_with_env(client: &ContainerClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Creating container with environment variables...");
    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
    env_vars.insert("ANOTHER_VAR".to_string(), "another_value".to_string());
    let _container_with_env =
        GenericContainer::with_env(client.client(), "alpine", "latest", env_vars)?;
    println!("   ✓ Container created with environment variables");
    Ok(())
}

/// Example: Executing commands in a container
///
/// ## How-to: Execute Commands in Containers
///
/// Use `GenericContainer::with_command()` to create a container that stays running,
/// then use `exec()` to execute commands. For images that exit immediately (e.g., alpine),
/// use `with_command()` with a keep-alive command like `sleep infinity`.
///
/// **Best Practice**: Use `SUCCESS_EXIT_CODE` constant instead of magic number 0.
///
/// ## Reference
///
/// - **Function**: `GenericContainer::with_command(client, image, tag, command, args, entrypoint) -> Result<GenericContainer, TestcontainersError>`
/// - **Parameters**:
///   - `command`: Command to run (e.g., "sleep")
///   - `args`: Command arguments (e.g., `&["infinity"]`)
///   - `entrypoint`: Optional entrypoint override (e.g., `Some(&["/bin/sh"])` or `None`)
/// - **Method**: `exec(command, args) -> Result<ExecResult, TestcontainersError>`
///   - Returns `ExecResult` with `stdout`, `stderr`, and `exit_code`
/// - **Constants**: `SUCCESS_EXIT_CODE` (0) for successful command execution
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::testcontainers::*;
/// use chicago_tdd_tools::testcontainers::exec::SUCCESS_EXIT_CODE;
///
/// let container = GenericContainer::with_command(
///     client.client(),
///     "alpine",
///     "latest",
///     "sleep",
///     &["infinity"],
///     None,
/// )?;
/// let result = container.exec("echo", &["hello"])?;
/// assert_eq!(result.exit_code, SUCCESS_EXIT_CODE);
/// ```
fn example_command_execution(client: &ContainerClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Executing commands in container...");
    println!("   Note: Using with_command() - the unified method for all containers");

    // Use Alpine with sleep to keep it running (correct pattern for images that exit)
    // **Unified API**: with_command() works for all containers:
    // - Regular containers: entrypoint = None (uses testcontainers API)
    // - Containers needing entrypoint override: entrypoint = Some(&["/bin/sh"]) (uses Docker CLI workaround)
    let alpine_container = GenericContainer::with_command(
        client.client(),
        "alpine",
        "latest",
        "sleep",
        &["infinity"],
        None, // No entrypoint override needed for alpine (uses testcontainers API)
    )?;

    // Execute a simple command
    let exec_result = alpine_container.exec("echo", &["hello", "from", "container"])?;
    println!("   ✓ Command executed: {}", exec_result.stdout.trim());
    // **Best Practice**: Use SUCCESS_EXIT_CODE constant instead of magic number 0
    assert_eq!(exec_result.exit_code, SUCCESS_EXIT_CODE, "Command should succeed");
    println!("   ✓ Exit code: {} (SUCCESS)", exec_result.exit_code);

    // **Best Practice**: Demonstrate error path handling - show how to check for failures
    let error_result = alpine_container.exec("nonexistent_command", &[]);
    match error_result {
        Ok(result) => {
            // Command succeeded but might have non-zero exit code
            if result.exit_code != SUCCESS_EXIT_CODE {
                println!(
                    "   ✓ Error handling demonstrated - command failed with exit code: {}",
                    result.exit_code
                );
            }
        }
        Err(e) => {
            // Command execution failed (e.g., command not found)
            println!("   ✓ Error handling demonstrated - exec failed: {e}");
            // **Best Practice**: In production code, handle errors appropriately
        }
    }

    println!("   (Pattern: Use with_command() for images that exit immediately)");
    Ok(())
}

/// Example: Container with entrypoint override
///
/// ## How-to: Override Container Entrypoint
///
/// Some Docker images (e.g., `otel/weaver`) have entrypoints that interfere with custom commands.
/// Use `with_command()` with `entrypoint = Some(&["/bin/sh"])` to override the entrypoint.
///
/// **Note**: Docker's `--entrypoint` flag only accepts a single executable path, not multiple arguments.
/// For shell commands with arguments, use the `command` parameter instead.
///
/// ## Reference
///
/// - **Function**: `GenericContainer::with_command(client, image, tag, command, args, entrypoint)`
/// - **Parameters**:
///   - `entrypoint`: `Some(&["/bin/sh"])` to override entrypoint, `None` for default
/// - **Implementation**: Uses Docker CLI workaround when entrypoint override is needed
/// - **Limitation**: Entrypoint must be single executable (Docker CLI limitation)
///
/// # Examples
///
/// ```rust
/// // For images like otel/weaver that need entrypoint override:
/// let weaver_container = GenericContainer::with_command(
///     client.client(),
///     "otel/weaver",
///     "latest",
///     "sleep",
///     &["infinity"],
///     Some(&["/bin/sh"]), // Entrypoint override needed
/// )?;
/// ```
fn example_entrypoint_override() {
    println!("\n4b. Container with entrypoint override (e.g., otel/weaver)...");
    println!("   Note: For images with entrypoints that interfere, use entrypoint parameter");
    println!("   Example pattern (commented out - requires otel/weaver image):");
    println!("   ```rust");
    println!("   let weaver_container = GenericContainer::with_command(");
    println!("       client.client(),");
    println!("       \"otel/weaver\",");
    println!("       \"latest\",");
    println!("       \"sleep\",");
    println!("       &[\"infinity\"],");
    println!(
        "       Some(&[\"/bin/sh\"]), // Entrypoint override needed (uses Docker CLI workaround)"
    );
    println!("   )?;");
    println!("   let weaver_result = weaver_container.exec(\"weaver\", &[\"--version\"])?;");
    println!("   assert_eq!(weaver_result.exit_code, SUCCESS_EXIT_CODE);");
    println!("   ```");
    println!("   (Pattern: Use entrypoint = Some(&[\"/bin/sh\"]) for images like otel/weaver)");
}

/// Example: Using wait conditions
///
/// ## How-to: Wait for Container Readiness
///
/// Use `GenericContainer::with_wait_for()` to ensure containers are ready before use.
/// Wait conditions include HTTP health checks and log message waiting.
///
/// ## Reference
///
/// - **Method**: `GenericContainer::with_wait_for(wait_condition) -> GenericContainer`
/// - **Wait Conditions**:
///   - `WaitFor::http(path, port)`: Wait for HTTP endpoint to respond
///   - `WaitFor::message(text)`: Wait for log message
/// - **Usage**: Chain after container creation, before use
///
/// # Examples
///
/// ```rust
/// let container = GenericContainer::with_ports(client.client(), "nginx", "latest", &[80])?
///     .with_wait_for(WaitFor::http("/", DEFAULT_HTTP_PORT));
/// ```
fn example_wait_conditions() {
    println!("\n5. Using wait conditions...");
    println!("   Note: Wait conditions ensure containers are ready before use");
    println!("   Example: WaitFor::http(\"/\", {}) for HTTP health checks", DEFAULT_HTTP_PORT);
    println!("   Example: WaitFor::message(\"ready\") for log message waiting");
    println!("   ✓ Wait conditions available via GenericContainer::with_wait_for()");
}

#[cfg(feature = "testcontainers")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testcontainers Example - Minimal 80/20 Implementation");

    // Arrange: Create container client
    let client = ContainerClient::new();

    // Run all examples
    example_basic_container(&client)?;
    example_container_with_ports(&client)?;
    example_container_with_env(&client)?;
    example_command_execution(&client)?;
    example_entrypoint_override();
    example_wait_conditions();

    println!("\n✓ All examples completed successfully!");
    println!("  Containers will be automatically cleaned up on drop");

    Ok(())
}

#[cfg(not(feature = "testcontainers"))]
fn main() {
    println!("testcontainers feature is not enabled");
    println!(
        "Enable it with: cargo run --example testcontainers_example --features testcontainers"
    );
}
