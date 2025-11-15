//! Testcontainers Example
//!
//! Demonstrates minimal 80/20 testcontainers usage for Chicago TDD integration testing.
//!
//! This example shows how to:
//! - Create a container client
//! - Start a generic container
//! - Get host ports for container ports
//! - Use environment variables
//! - Execute commands in containers
//! - Use wait conditions
//! - Automatic cleanup on drop

#[cfg(feature = "testcontainers")]
use chicago_tdd_tools::testcontainers::exec::SUCCESS_EXIT_CODE;
#[cfg(feature = "testcontainers")]
use chicago_tdd_tools::testcontainers::*;

#[cfg(feature = "testcontainers")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testcontainers Example - Minimal 80/20 Implementation");

    // Arrange: Create container client
    let client = ContainerClient::new();

    // Example 1: Basic container
    println!("\n1. Creating basic Alpine container...");
    // Note: GenericContainer::new() creates container but it exits immediately.
    // This container cannot be used for exec() operations.
    // For exec operations, use with_command() (see Example 4).
    let _container = GenericContainer::new(client.client(), "alpine", "latest")?;
    println!("   ✓ Container created successfully");
    println!("   Note: This container exits immediately - cannot use exec()");
    // Container automatically cleaned up on drop

    // Example 2: Container with exposed ports
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

    // Example 3: Container with environment variables
    println!("\n3. Creating container with environment variables...");
    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
    env_vars.insert("ANOTHER_VAR".to_string(), "another_value".to_string());
    let _container_with_env =
        GenericContainer::with_env(client.client(), "alpine", "latest", env_vars)?;
    println!("   ✓ Container created with environment variables");

    // Example 4: Command execution
    // Note: For exec to work, container must be running.
    // **Root Cause Prevention**: For images that exit immediately (e.g., alpine, otel/weaver),
    // use with_command() to keep container running. Service containers (postgres, redis, nginx)
    // typically stay running by default.
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

    // Example 4b: Container with entrypoint override (e.g., weaver)
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

    // Example 5: Wait conditions
    println!("\n5. Using wait conditions...");
    println!("   Note: Wait conditions ensure containers are ready before use");
    println!("   Example: WaitFor::http(\"/\", {}) for HTTP health checks", DEFAULT_HTTP_PORT);
    println!("   Example: WaitFor::message(\"ready\") for log message waiting");
    println!("   ✓ Wait conditions available via GenericContainer::with_wait_for()");

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
