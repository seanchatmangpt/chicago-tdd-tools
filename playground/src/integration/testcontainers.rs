//! Testcontainers Examples
//!
//! Demonstrates Docker container support for integration testing.
//!
//! **Note**: Requires Docker to be running. Tests will be skipped if Docker is unavailable.

#[cfg(feature = "testcontainers")]
use chicago_tdd_tools::integration::testcontainers::*;
#[cfg(feature = "testcontainers")]
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "testcontainers")]
/// Example: Basic container creation
pub fn example_container_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create container client
    let client = ContainerClient::new();

    // Act: Create container
    let _container = GenericContainer::new(client.client(), "alpine", "latest")?;

    // Assert: Container created successfully
    // Container automatically cleaned up on drop
    Ok(())
}

#[cfg(feature = "testcontainers")]
/// Example: Container with ports
pub fn example_container_ports() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create container client
    let client = ContainerClient::new();

    // Act: Create container with exposed ports
    let container = GenericContainer::with_ports(client.client(), "alpine", "latest", &[80, 443])?;
    let host_port_80 = container.get_host_port(80)?;
    let host_port_443 = container.get_host_port(443)?;

    // Assert: Verify port mapping
    assert!(host_port_80 > 0);
    assert!(host_port_443 > 0);
    Ok(())
}

#[cfg(feature = "testcontainers")]
/// Example: Container with environment variables
pub fn example_container_env() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create container client and env vars
    let client = ContainerClient::new();
    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    // Act: Create container with environment variables
    let _container = GenericContainer::with_env(client.client(), "alpine", "latest", env_vars)?;

    // Assert: Container created successfully
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "testcontainers")]
    use super::*;
    #[cfg(feature = "testcontainers")]
    use chicago_tdd_tools::prelude::*;

    #[cfg(feature = "testcontainers")]
    async_test_with_timeout!(test_container_basic, 30, {
        // Arrange-Act-Assert: Run example
        // Note: 30s timeout for Docker operations
        assert_ok!(example_container_basic());
    });

    #[cfg(feature = "testcontainers")]
    async_test_with_timeout!(test_container_ports, 30, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_container_ports());
    });

    #[cfg(feature = "testcontainers")]
    async_test_with_timeout!(test_container_env, 30, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_container_env());
    });
}

