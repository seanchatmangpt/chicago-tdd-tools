//! Testcontainers Support
//!
//! Provides minimal generic container support for integration testing with Docker.
//! Follows Chicago TDD principles by using real containers instead of mocks.
//!
//! ## Features (80/20 Minimal)
//!
//! - **Generic Containers**: Support for any Docker image
//! - **Port Mapping**: Get host ports for container ports
//! - **Environment Variables**: Basic environment variable support
//! - **Command Execution**: Execute commands inside containers and get stdout/stderr/exit code
//! - **Wait Conditions**: Wait for containers to be ready (e.g., HTTP health checks)
//! - **Automatic Cleanup**: Containers cleaned up automatically on Drop
//!
//! ## Chicago TDD Alignment
//!
//! - **Real Collaborators**: Actual Docker containers, not mocks
//! - **State Verification**: Verify container state and responses
//! - **Automatic Cleanup**: Containers cleaned up via Drop trait
//! - **AAA Pattern**: Arrange (start container), Act (test), Assert (verify state)
//!
//! ## Usage
//!
//! ```rust
//! # #[cfg(feature = "testcontainers")]
//! use chicago_tdd_tools::testcontainers::{*, exec::SUCCESS_EXIT_CODE};
//!
//! # #[cfg(feature = "testcontainers")]
//! # #[tokio::test]
//! # async fn test_with_container() -> Result<(), TestcontainersError> {
//!     // Arrange: Create client and container
//! # #[cfg(feature = "testcontainers")]
//!     let client = ContainerClient::new();
//! # #[cfg(feature = "testcontainers")]
//!     let container = GenericContainer::new(
//!         client.client(),
//!         "alpine",
//!         "latest"
//!     )?;
//!
//!     // Act: Use container (e.g., get port or execute command)
//!     let host_port = container.get_host_port(DEFAULT_HTTP_PORT)?;
//!
//!     // Assert: Verify port is valid
//!     assert!(host_port > 0);
//!
//!     // Execute a command in the container
//!     let result = container.exec("echo", &["hello"])?;
//!     assert_eq!(result.stdout.trim(), "hello");
//!     assert_eq!(result.exit_code, SUCCESS_EXIT_CODE);
//!
//!     // Container automatically cleaned up on drop
//!     Ok(())
//! }
//! ```
//!
//! ## Container Lifecycle Notes
//!
//! **Command Execution**: The `exec()` method requires the container to be running.
//! This works best with:
//! - Service containers (postgres, redis, nginx, etc.) that stay running
//! - Containers with long-running default commands
//!
//! For containers that exit immediately, consider using service images or
//! accessing the underlying container via `container()` for advanced configuration.

use thiserror::Error;

#[cfg(feature = "testcontainers")]
use std::collections::HashMap;

/// Default HTTP port for examples and tests
///
/// **Kaizen improvement**: Extracted magic number `80` to named constant.
/// Pattern: Use named constants for commonly used port numbers.
pub const DEFAULT_HTTP_PORT: u16 = 80;

/// Testcontainers error type
#[derive(Error, Debug)]
pub enum TestcontainersError {
    /// Failed to create container
    #[error("Failed to create container: {0}")]
    CreationFailed(String),
    /// Container operation failed
    #[error("Container operation failed: {0}")]
    OperationFailed(String),
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    /// Command execution failed
    #[error("Command execution failed: {0}")]
    CommandExecutionFailed(String),
    /// Failed to read stdout
    #[error("Failed to read stdout: {0}")]
    StdoutReadFailed(String),
    /// Failed to read stderr
    #[error("Failed to read stderr: {0}")]
    StderrReadFailed(String),
    /// Failed to get exit code
    #[error("Failed to get exit code: {0}")]
    ExitCodeFailed(String),
}

/// Result type for testcontainers operations
pub type TestcontainersResult<T> = Result<T, TestcontainersError>;

// Re-export exec and wait functionality
pub mod exec;
pub mod wait;
pub use exec::ExecResult;

#[cfg(feature = "testcontainers")]
pub mod implementation {
    use super::*;
    use futures::executor::block_on;
    use futures_util::TryFutureExt;
    use testcontainers::core::ContainerPort;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::Container;
    use testcontainers::GenericImage;
    use testcontainers::ImageExt;

    /// Container client for managing Docker containers
    ///
    /// Minimal 80/20 implementation - provides basic container management.
    /// For advanced features (pools, determinism, policies), see clnrm.
    pub struct ContainerClient;

    impl ContainerClient {
        /// Create a new container client
        pub fn new() -> Self {
            Self
        }

        /// Get a reference for compatibility (no-op in minimal implementation)
        pub fn client(&self) -> &Self {
            self
        }
    }

    impl Default for ContainerClient {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Generic container wrapper for any Docker image
    ///
    /// Minimal 80/20 implementation - supports basic container operations:
    /// - Start any Docker image
    /// - Map container ports to host ports
    /// - Set environment variables
    /// - Execute commands
    /// - Automatic cleanup on Drop
    ///
    /// For advanced features (volume mounts, resource limits, determinism),
    /// see clnrm's TestcontainerBackend.
    pub struct GenericContainer {
        container: Container<GenericImage>,
    }

    impl GenericContainer {
        /// Create a new generic container from any Docker image
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (unused in minimal implementation)
        /// * `image` - Docker image name (e.g., "alpine", "postgres")
        /// * `tag` - Docker image tag (e.g., "latest", "14")
        ///
        /// # Errors
        ///
        /// Returns error if container creation fails (Docker not running, image not found, etc.)
        pub fn new(
            _client: &ContainerClient,
            image: &str,
            tag: &str,
        ) -> TestcontainersResult<Self> {
            let image = GenericImage::new(image, tag);
            let container = block_on(image.start()).map_err(|e| {
                TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
            })?;

            Ok(Self { container })
        }

        /// Create a new generic container with environment variables
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (unused in minimal implementation)
        /// * `image` - Docker image name
        /// * `tag` - Docker image tag
        /// * `env_vars` - Environment variables to set in the container
        ///
        /// # Errors
        ///
        /// Returns error if container creation fails
        pub fn with_env(
            _client: &ContainerClient,
            image: &str,
            tag: &str,
            env_vars: HashMap<String, String>,
        ) -> TestcontainersResult<Self> {
            let image = GenericImage::new(image, tag);
            // Build container request with all env vars
            // Move env_vars into the request (no need to clone since we consume the HashMap)
            let mut request: testcontainers::core::ContainerRequest<GenericImage> = image.into();
            for (key, value) in env_vars {
                request = request.with_env_var(key, value);
            }
            let container = block_on(request.start()).map_err(|e| {
                TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
            })?;

            Ok(Self { container })
        }

        /// Create a new generic container with port mappings
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (unused in minimal implementation)
        /// * `image` - Docker image name
        /// * `tag` - Docker image tag
        /// * `ports` - Container ports to map to host ports
        ///
        /// # Errors
        ///
        /// Returns error if container creation fails
        pub fn with_ports(
            _client: &ContainerClient,
            image: &str,
            tag: &str,
            ports: &[u16],
        ) -> TestcontainersResult<Self> {
            let mut image = GenericImage::new(image, tag);
            for port in ports {
                image = image.with_exposed_port(ContainerPort::Tcp(*port));
            }
            let container = image.start().map_err(|e| {
                TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
            })?;

            Ok(Self { container })
        }

        /// Get the host port for a container port
        ///
        /// # Arguments
        ///
        /// * `container_port` - The container port to get the host port for
        ///
        /// # Errors
        ///
        /// Returns error if port mapping fails or port is not mapped
        pub fn get_host_port(&self, container_port: u16) -> TestcontainersResult<u16> {
            let port = self.container.get_host_port_ipv4(container_port).map_err(|e| {
                TestcontainersError::OperationFailed(format!(
                    "Failed to get host port for container port {}: {}",
                    container_port, e
                ))
            })?;
            Ok(port)
        }

        /// Get the underlying testcontainers Container
        ///
        /// Allows access to advanced testcontainers features if needed.
        pub fn container(&self) -> &Container<GenericImage> {
            &self.container
        }
    }
}

#[cfg(feature = "testcontainers")]
pub use implementation::*;

#[cfg(not(feature = "testcontainers"))]
mod stubs {
    use super::*;

    /// Stub for ContainerClient when testcontainers feature is disabled
    pub struct ContainerClient;

    impl ContainerClient {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for ContainerClient {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Stub for GenericContainer when testcontainers feature is disabled
    pub struct GenericContainer;

    impl GenericContainer {
        pub fn new(
            _client: &ContainerClient,
            _image: &str,
            _tag: &str,
        ) -> TestcontainersResult<Self> {
            Err(TestcontainersError::InvalidConfig(
                "testcontainers feature is not enabled".to_string(),
            ))
        }

        pub fn with_env(
            _client: &ContainerClient,
            _image: &str,
            _tag: &str,
            _env_vars: HashMap<String, String>,
        ) -> TestcontainersResult<Self> {
            Err(TestcontainersError::InvalidConfig(
                "testcontainers feature is not enabled".to_string(),
            ))
        }

        pub fn with_ports(
            _client: &ContainerClient,
            _image: &str,
            _tag: &str,
            _ports: &[u16],
        ) -> TestcontainersResult<Self> {
            Err(TestcontainersError::InvalidConfig(
                "testcontainers feature is not enabled".to_string(),
            ))
        }

        pub fn get_host_port(&self, _container_port: u16) -> TestcontainersResult<u16> {
            Err(TestcontainersError::InvalidConfig(
                "testcontainers feature is not enabled".to_string(),
            ))
        }

        pub fn container(&self) -> &Self {
            self
        }
    }
}

#[cfg(not(feature = "testcontainers"))]
pub use stubs::*;

#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::*;

    // Test error types (critical - 80% of bugs)
    #[test]
    fn test_testcontainers_error_display() {
        // Test all error variants display correctly
        let errors = vec![
            TestcontainersError::CreationFailed("test".to_string()),
            TestcontainersError::OperationFailed("test".to_string()),
            TestcontainersError::InvalidConfig("test".to_string()),
            TestcontainersError::CommandExecutionFailed("test".to_string()),
            TestcontainersError::StdoutReadFailed("test".to_string()),
            TestcontainersError::StderrReadFailed("test".to_string()),
            TestcontainersError::ExitCodeFailed("test".to_string()),
        ];

        for error in errors {
            let display = format!("{error}");
            assert!(!display.is_empty(), "Error should have display message");
            assert!(display.contains("test"), "Error should contain message");
        }
    }

    #[test]
    fn test_exec_result_structure() {
        // Test ExecResult creation and access
        let result = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: exec::SUCCESS_EXIT_CODE,
        };

        assert_eq!(result.stdout, "output");
        assert_eq!(result.stderr, "error");
        assert_eq!(result.exit_code, exec::SUCCESS_EXIT_CODE);
    }

    #[test]
    fn test_exec_result_clone() {
        // Test ExecResult is cloneable
        let result1 = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: exec::SUCCESS_EXIT_CODE,
        };

        let result2 = result1.clone();
        assert_eq!(result1.stdout, result2.stdout);
        assert_eq!(result1.stderr, result2.stderr);
        assert_eq!(result1.exit_code, result2.exit_code);
    }

    #[test]
    fn test_exec_result_debug() {
        // Test ExecResult is debuggable
        let result = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: exec::SUCCESS_EXIT_CODE,
        };

        let debug = format!("{result:?}");
        assert!(debug.contains("output"));
        assert!(debug.contains("error"));
        assert!(debug.contains("0"));
    }

    // Test stubs when feature is disabled (important for mocking)
    #[cfg(not(feature = "testcontainers"))]
    #[test]
    fn test_stubs_return_errors() {
        let client = ContainerClient::new();

        // Test: All stub methods return InvalidConfig error
        let result = GenericContainer::new(&client, "alpine", "latest");
        assert!(result.is_err());
        match result {
            Err(TestcontainersError::InvalidConfig(msg)) => {
                assert!(msg.contains("testcontainers feature is not enabled"));
            }
            _ => panic!("Expected InvalidConfig error"),
        }

        let container = GenericContainer;
        let port_result = container.get_host_port(DEFAULT_HTTP_PORT);
        assert!(port_result.is_err());

        let exec_result = container.exec("echo", &["test"]);
        assert!(exec_result.is_err());
    }

    #[cfg(not(feature = "testcontainers"))]
    #[test]
    fn test_stub_container_client() {
        // Test: Stub client can be created
        let client1 = ContainerClient::new();
        let client2 = ContainerClient::default();

        // Both should work (no panic)
        let _ref1 = client1.client();
        let _ref2 = client2.client();
    }
}
