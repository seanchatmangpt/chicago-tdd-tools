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
//! ```rust,no_run
//! use chicago_tdd_tools::testcontainers::*;
//!
//! #[test]
//! fn test_with_container() -> Result<(), TestcontainersError> {
//!     // Arrange: Create client and container
//!     let client = ContainerClient::new();
//!     let container = GenericContainer::new(
//!         client.client(),
//!         "alpine",
//!         "latest"
//!     )?;
//!
//!     // Act: Use container (e.g., get port or execute command)
//!     let host_port = container.get_host_port(80)?;
//!
//!     // Assert: Verify port is valid
//!     assert!(host_port > 0);
//!
//!     // Execute a command in the container
//!     let result = container.exec("echo", &["hello"])?;
//!     assert_eq!(result.stdout.trim(), "hello");
//!     assert_eq!(result.exit_code, 0);
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

use std::collections::HashMap;
use thiserror::Error;

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

/// Result of executing a command in a container
#[derive(Debug, Clone)]
pub struct ExecResult {
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code of the command (0 for success, non-zero for failure)
    pub exit_code: i32,
}

#[cfg(feature = "testcontainers")]
mod implementation {
    use super::*;
    use std::io::Read;
    use testcontainers::core::ContainerPort;
    use testcontainers::core::ContainerRequest;
    use testcontainers::core::ExecCommand;
    use testcontainers::core::WaitFor;
    use testcontainers::runners::SyncRunner;
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
            let container_request: ContainerRequest<GenericImage> = image.into();
            let container = container_request.start().map_err(|e| {
                TestcontainersError::CreationFailed(format!("Failed to start container: {}", e))
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
        /// * `env_vars` - Environment variables to set in container
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
            let mut container_request: ContainerRequest<GenericImage> = image.into();

            // Add environment variables
            for (key, value) in env_vars {
                container_request = container_request.with_env_var(&key, &value);
            }

            let container = container_request.start().map_err(|e| {
                TestcontainersError::CreationFailed(format!("Failed to start container: {}", e))
            })?;
            Ok(Self { container })
        }

        /// Create a new generic container with exposed ports
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (unused in minimal implementation)
        /// * `image` - Docker image name
        /// * `tag` - Docker image tag
        /// * `ports` - Container ports to expose
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

            // Expose ports on the image before converting to ContainerRequest
            for &port in ports {
                image = image.with_exposed_port(ContainerPort::Tcp(port));
            }

            let container_request: ContainerRequest<GenericImage> = image.into();
            let container = container_request.start().map_err(|e| {
                TestcontainersError::CreationFailed(format!("Failed to start container: {}", e))
            })?;
            Ok(Self { container })
        }

        /// Get host port for a container port
        ///
        /// # Arguments
        ///
        /// * `container_port` - Port number inside the container
        ///
        /// # Returns
        ///
        /// Host port number that maps to the container port
        ///
        /// # Errors
        ///
        /// Returns error if the port was not exposed when creating the container
        pub fn get_host_port(&self, container_port: u16) -> TestcontainersResult<u16> {
            self.container
                .get_host_port_ipv4(ContainerPort::Tcp(container_port))
                .map_err(|e| {
                    TestcontainersError::OperationFailed(format!(
                        "Failed to get host port for container port {}: {}",
                        container_port, e
                    ))
                })
        }

        /// Create a new generic container with wait condition
        ///
        /// Wait conditions allow waiting for containers to be ready before use.
        /// Common use cases include waiting for HTTP endpoints or log messages.
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (unused in minimal implementation)
        /// * `image` - Docker image name
        /// * `tag` - Docker image tag
        /// * `wait_for` - Wait condition (e.g., `WaitFor::http` for HTTP health checks)
        ///
        /// # Errors
        ///
        /// Returns error if container creation fails
        ///
        /// # Example
        ///
        /// ```rust,no_run
        /// use chicago_tdd_tools::testcontainers::*;
        /// use testcontainers::core::WaitFor;
        ///
        /// let client = ContainerClient::new();
        /// // Wait for HTTP endpoint to be ready
        /// let container = GenericContainer::with_wait_for(
        ///     client.client(),
        ///     "nginx",
        ///     "latest",
        ///     WaitFor::http("/", 80),
        /// )?;
        /// ```
        pub fn with_wait_for(
            _client: &ContainerClient,
            image: &str,
            tag: &str,
            wait_for: WaitFor,
        ) -> TestcontainersResult<Self> {
            let image = GenericImage::new(image, tag).with_wait_for(wait_for);
            let container_request: ContainerRequest<GenericImage> = image.into();
            let container = container_request.start().map_err(|e| {
                TestcontainersError::CreationFailed(format!("Failed to start container: {}", e))
            })?;

            Ok(Self { container })
        }

        /// Execute a command in the container
        ///
        /// Executes a command inside the running container and returns stdout, stderr, and exit code.
        ///
        /// # Important
        ///
        /// The container must be running for exec to work. This works best with:
        /// - Service containers (postgres, redis, nginx, etc.) that stay running
        /// - Containers with long-running default commands
        ///
        /// For containers that exit immediately, consider using service images or
        /// accessing the underlying container via `container()` for advanced configuration.
        ///
        /// # Arguments
        ///
        /// * `command` - Command to execute (e.g., "sh", "echo", "ls")
        /// * `args` - Command arguments (e.g., ["-c", "echo hello"])
        ///
        /// # Errors
        ///
        /// Returns error if:
        /// - Command execution fails
        /// - Reading stdout/stderr fails
        /// - Getting exit code fails
        ///
        /// # Example
        ///
        /// ```rust,no_run
        /// use chicago_tdd_tools::testcontainers::*;
        ///
        /// let client = ContainerClient::new();
        /// let container = GenericContainer::new(client.client(), "alpine", "latest")?;
        ///
        /// // Execute a command
        /// let result = container.exec("echo", &["hello", "world"])?;
        /// assert_eq!(result.stdout.trim(), "hello world");
        /// assert_eq!(result.exit_code, 0);
        /// ```
        pub fn exec(&self, command: &str, args: &[&str]) -> TestcontainersResult<ExecResult> {
            // Build command arguments as Vec<&str> (like clnrm lines 512-514)
            let cmd_args: Vec<&str> = std::iter::once(command)
                .chain(args.iter().copied())
                .collect();

            // Create ExecCommand (like clnrm line 519)
            let exec_cmd = ExecCommand::new(cmd_args);

            // Execute command (like clnrm lines 520-522)
            let mut exec_result = self.container.exec(exec_cmd).map_err(|e| {
                TestcontainersError::CommandExecutionFailed(format!(
                    "Failed to execute command '{}': {}",
                    command, e
                ))
            })?;

            // Read stdout (like clnrm lines 533-536)
            let mut stdout = String::new();
            exec_result
                .stdout()
                .read_to_string(&mut stdout)
                .map_err(|e| {
                    TestcontainersError::StdoutReadFailed(format!("Failed to read stdout: {}", e))
                })?;

            // Read stderr (like clnrm lines 537-540)
            let mut stderr = String::new();
            exec_result
                .stderr()
                .read_to_string(&mut stderr)
                .map_err(|e| {
                    TestcontainersError::StderrReadFailed(format!("Failed to read stderr: {}", e))
                })?;

            // Get exit code (like clnrm lines 545-553)
            // exit_code() returns Result<Option<i32>, Error> - may return None
            let exit_code = exec_result.exit_code().map_err(|e| {
                TestcontainersError::ExitCodeFailed(format!("Failed to get exit code: {}", e))
            })?;

            // Handle None exit code by defaulting to -1 (like clnrm lines 548-553)
            let exit_code = exit_code.unwrap_or(-1) as i32;

            Ok(ExecResult {
                stdout,
                stderr,
                exit_code,
            })
        }

        /// Get the underlying testcontainers Container
        ///
        /// Allows access to advanced testcontainers features if needed.
        /// For most use cases, use the methods on GenericContainer instead.
        pub fn container(&self) -> &Container<GenericImage> {
            &self.container
        }
    }

    // Automatic cleanup via Drop - containers are cleaned up when GenericContainer is dropped
    // This follows RAII principles and ensures no resource leaks
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

        pub fn with_wait_for(
            _client: &ContainerClient,
            _image: &str,
            _tag: &str,
            _wait_for: (),
        ) -> TestcontainersResult<Self> {
            Err(TestcontainersError::InvalidConfig(
                "testcontainers feature is not enabled".to_string(),
            ))
        }

        pub fn container(&self) -> &Self {
            self
        }

        pub fn exec(&self, _command: &str, _args: &[&str]) -> TestcontainersResult<ExecResult> {
            Err(TestcontainersError::InvalidConfig(
                "testcontainers feature is not enabled".to_string(),
            ))
        }
    }
}

#[cfg(not(feature = "testcontainers"))]
pub use stubs::*;
