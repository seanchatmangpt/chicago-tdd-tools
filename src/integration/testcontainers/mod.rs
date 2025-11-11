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

#[cfg(feature = "testcontainers")]
use std::collections::HashMap;

use thiserror::Error;

/// Default HTTP port for examples and tests
///
/// **Kaizen improvement**: Extracted magic number `80` to named constant.
/// Pattern: Use named constants for commonly used port numbers.
pub const DEFAULT_HTTP_PORT: u16 = 80;

/// Testcontainers error type
#[derive(Error, Debug)]
pub enum TestcontainersError {
    /// Docker daemon is not running or unavailable
    #[error("🚨 Docker daemon is not running or unavailable: {0}\n   ⚠️  STOP: Cannot proceed with container operations\n   💡 FIX: Start Docker Desktop or Docker daemon\n   📋 macOS: Open Docker Desktop\n   📋 Linux: sudo systemctl start docker\n   📋 Windows: Start Docker Desktop")]
    DockerUnavailable(String),
    /// Failed to create container
    #[error("🚨 Failed to create container: {0}\n   ⚠️  STOP: Container creation failed\n   💡 FIX: Check Docker image exists and Docker daemon is running")]
    CreationFailed(String),
    /// Container operation failed
    #[error("⚠️  Container operation failed: {0}\n   ⚠️  WARNING: Container operation did not complete successfully")]
    OperationFailed(String),
    /// Invalid configuration
    #[error("🚨 Invalid configuration: {0}\n   ⚠️  STOP: Configuration is invalid\n   💡 FIX: Check configuration parameters")]
    InvalidConfig(String),
    /// Command execution failed
    #[error("⚠️  Command execution failed: {0}\n   ⚠️  WARNING: Command did not execute successfully\n   💡 FIX: Check command syntax and container state")]
    CommandExecutionFailed(String),
    /// Failed to read stdout
    #[error("⚠️  Failed to read stdout: {0}\n   ⚠️  WARNING: Could not read command output\n   💡 FIX: Check container is running and command completed")]
    StdoutReadFailed(String),
    /// Failed to read stderr
    #[error("⚠️  Failed to read stderr: {0}\n   ⚠️  WARNING: Could not read command error output\n   💡 FIX: Check container is running and command completed")]
    StderrReadFailed(String),
    /// Failed to get exit code
    #[error("⚠️  Failed to get exit code: {0}\n   ⚠️  WARNING: Could not determine command exit status\n   💡 FIX: Check container is running and command completed")]
    ExitCodeFailed(String),
}

/// Result type for testcontainers operations
pub type TestcontainersResult<T> = Result<T, TestcontainersError>;

// Re-export exec and wait functionality
pub mod exec;
pub mod wait;
pub use exec::ExecResult;

#[cfg(feature = "testcontainers")]
/// Implementation module for testcontainers functionality
///
/// Contains the actual implementation of `ContainerClient` and `GenericContainer`.
/// These types are feature-gated and only available when the `testcontainers` feature is enabled.
pub mod implementation {
    use super::{HashMap, TestcontainersError, TestcontainersResult};
    use testcontainers::core::ContainerPort;
    use testcontainers::runners::SyncRunner;
    use testcontainers::Container;
    use testcontainers::GenericImage;
    use testcontainers::ImageExt;

    /// Check if Docker daemon is actually running and responding
    ///
    /// This function verifies Docker daemon is running by checking:
    /// 1. Docker command exists
    /// 2. Docker daemon is responding (not just command execution)
    /// 3. Docker daemon is accessible
    ///
    /// Returns 🚨 CRITICAL signal if Docker is unavailable.
    /// This is a fail-fast check - operations should stop immediately.
    ///
    /// # Returns
    ///
    /// `Ok(())` if Docker daemon is running and responding
    /// `Err(TestcontainersError::DockerUnavailable)` if Docker is stopped or unavailable
    pub fn check_docker_available() -> TestcontainersResult<()> {
        use std::process::Command;

        // Check Docker command exists
        let docker_check = Command::new("docker").args(["info"]).output();

        match docker_check {
            Ok(output) => {
                if output.status.success() {
                    // Verify Docker daemon is responding by checking output
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("Server Version") || stdout.contains("Docker Root Dir") {
                        // ✅ Docker daemon is running and responding
                        Ok(())
                    } else {
                        Err(TestcontainersError::DockerUnavailable(
                            "Docker daemon is not responding correctly. Output does not contain expected Docker info.".to_string()
                        ))
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(TestcontainersError::DockerUnavailable(format!(
                        "Docker daemon is not running. Error: {stderr}"
                    )))
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(TestcontainersError::DockerUnavailable(
                        "Docker command not found. Please install Docker.".to_string(),
                    ))
                } else {
                    Err(TestcontainersError::DockerUnavailable(format!(
                        "Failed to check Docker availability: {e}"
                    )))
                }
            }
        }
    }

    /// Docker error message patterns that indicate Docker daemon is unavailable
    ///
    /// **Kaizen improvement**: Extracted duplicated error detection strings to named constants.
    /// Pattern: Use constants for repeated string patterns to reduce duplication and improve maintainability.
    /// Benefits: Single source of truth, easier to maintain, consistent error detection.
    const DOCKER_CONNECTION_ERROR_PATTERNS: &[&str] =
        &["Cannot connect to the Docker daemon", "docker daemon", "connection refused"];

    /// Check if an error message indicates Docker daemon is unavailable
    ///
    /// **Kaizen improvement**: Extracted duplicated error detection logic to helper function.
    /// Pattern: Extract repeated logic to function for DRY (Don't Repeat Yourself) principle.
    /// Benefits: Single implementation, easier to maintain, consistent behavior.
    fn is_docker_unavailable_error(error_msg: &str) -> bool {
        DOCKER_CONNECTION_ERROR_PATTERNS
            .iter()
            .any(|pattern| error_msg.contains(pattern))
    }

    /// Container client for managing Docker containers
    ///
    /// Minimal 80/20 implementation - provides basic container management.
    /// For advanced features (pools, determinism, policies), see clnrm.
    pub struct ContainerClient;

    impl ContainerClient {
        /// Create a new container client
        ///
        /// **FMEA Fix (RPN 216)**: Check Docker availability at client creation to fail-fast.
        /// Previously, Docker was only checked when containers were created, allowing false positives
        /// (tests pass when Docker unavailable). Now checks Docker immediately to prevent false positives.
        ///
        /// # Panics
        ///
        /// Panics if Docker is unavailable, with a clear error message.
        #[must_use]
        pub fn new() -> Self {
            // **FMEA Fix**: Verify Docker is available at client creation (fail-fast)
            // This prevents false positives where tests pass when Docker is unavailable
            check_docker_available().unwrap_or_else(|e| {
                panic!(
                    "🚨 Docker is required for testcontainers but Docker daemon is not available.\n\
                     ⚠️  STOP: Cannot create container client\n\
                     💡 FIX: Start Docker Desktop or Docker daemon\n\
                     📋 macOS: Open Docker Desktop\n\
                     📋 Linux: sudo systemctl start docker\n\
                     📋 Windows: Start Docker Desktop\n\
                     \n\
                     Error: {e}"
                )
            });
            Self
        }

        /// Get a reference for compatibility (no-op in minimal implementation)
        #[must_use]
        pub const fn client(&self) -> &Self {
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
    /// see clnrm's `TestcontainerBackend`.
    pub struct GenericContainer {
        container: Container<GenericImage>,
    }

    impl GenericContainer {
        /// Create a new generic container from any Docker image
        ///
        /// 🚨 CRITICAL - Stops immediately if Docker is unavailable.
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (should have been validated via `ContainerClient::new()`)
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
            // 🚨 Verify Docker is still available before container operations
            check_docker_available()?;

            let image = GenericImage::new(image, tag);
            let container = image.start().map_err(|e| {
                // Check if error indicates Docker is unavailable
                let error_msg = format!("{e}");
                if is_docker_unavailable_error(&error_msg) {
                    TestcontainersError::DockerUnavailable(format!(
                        "Docker daemon connection failed during container start: {e}\n   ⚠️  STOP: Cannot connect to Docker daemon\n   💡 FIX: Start Docker Desktop or Docker daemon"
                    ))
                } else {
                    TestcontainersError::CreationFailed(format!("Failed to start container: {e}\n   ⚠️  STOP: Container creation failed\n   💡 FIX: Check Docker image exists and Docker daemon is running"))
                }
            })?;

            // ✅ Container created successfully
            Ok(Self { container })
        }

        /// Create a `GenericContainer` from an existing Container
        ///
        /// This is used internally by other methods (e.g., `with_wait_for`) to construct
        /// a `GenericContainer` from a Container that was created with additional configuration.
        pub(crate) const fn from_container(container: Container<GenericImage>) -> Self {
            Self { container }
        }

        /// Create a new generic container with environment variables and optional command
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (unused in minimal implementation)
        /// * `image` - Docker image name
        /// * `tag` - Docker image tag
        /// * `env_vars` - Environment variables to set in the container
        /// * `command` - Optional command to run (e.g., Some(("sleep", &["infinity"])) to keep container running)
        ///
        /// # Errors
        ///
        /// Returns error if container creation fails
        pub fn with_env_and_command(
            _client: &ContainerClient,
            image: &str,
            tag: &str,
            env_vars: HashMap<String, String>,
            command: Option<(&str, &[&str])>,
        ) -> TestcontainersResult<Self> {
            // 🚨 Verify Docker is still available
            check_docker_available()?;

            let image = GenericImage::new(image, tag);
            // Build container request with all env vars
            let mut request: testcontainers::core::ContainerRequest<GenericImage> = image.into();
            for (key, value) in env_vars {
                request = request.with_env_var(key, value);
            }
            // Add command if provided
            if let Some((cmd, args)) = command {
                let mut cmd_vec = vec![cmd.to_string()];
                cmd_vec.extend(args.iter().map(|s| (*s).to_string()));
                request = request.with_cmd(cmd_vec);
            }
            let container = request.start().map_err(|e| {
                let error_msg = format!("{e}");
                if is_docker_unavailable_error(&error_msg) {
                    TestcontainersError::DockerUnavailable(format!(
                        "Docker daemon connection failed during container start: {e}\n   ⚠️  STOP: Cannot connect to Docker daemon\n   💡 FIX: Start Docker Desktop or Docker daemon"
                    ))
                } else {
                    TestcontainersError::CreationFailed(format!("Failed to start container: {e}\n   ⚠️  STOP: Container creation failed\n   💡 FIX: Check Docker image exists and Docker daemon is running"))
                }
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
            // 🚨 Verify Docker is still available
            check_docker_available()?;

            let image = GenericImage::new(image, tag);
            // Build container request with all env vars
            // Move env_vars into the request (no need to clone since we consume the HashMap)
            let mut request: testcontainers::core::ContainerRequest<GenericImage> = image.into();
            for (key, value) in env_vars {
                request = request.with_env_var(key, value);
            }
            let container = request.start().map_err(|e| {
                let error_msg = format!("{e}");
                if is_docker_unavailable_error(&error_msg) {
                    TestcontainersError::DockerUnavailable(format!(
                        "Docker daemon connection failed during container start: {e}\n   ⚠️  STOP: Cannot connect to Docker daemon\n   💡 FIX: Start Docker Desktop or Docker daemon"
                    ))
                } else {
                    TestcontainersError::CreationFailed(format!("Failed to start container: {e}\n   ⚠️  STOP: Container creation failed\n   💡 FIX: Check Docker image exists and Docker daemon is running"))
                }
            })?;

            Ok(Self { container })
        }

        /// Create a new generic container with a command that keeps it running
        ///
        /// This is useful for containers like Alpine that exit immediately without a command.
        /// The command will be executed when the container starts and will keep it running.
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance
        /// * `image` - Docker image name
        /// * `tag` - Docker image tag
        /// * `command` - Command to run (e.g., "sleep", "sh")
        /// * `args` - Command arguments (e.g., &["infinity"] for sleep)
        ///
        /// # Errors
        ///
        /// Returns error if container creation fails
        ///
        /// # Example
        ///
        /// ```rust
        /// // Create Alpine container with sleep to keep it running
        /// let container = GenericContainer::with_command(
        ///     client.client(),
        ///     "alpine",
        ///     "latest",
        ///     "sleep",
        ///     &["infinity"]
        /// )?;
        /// ```
        pub fn with_command(
            _client: &ContainerClient,
            image: &str,
            tag: &str,
            command: &str,
            args: &[&str],
        ) -> TestcontainersResult<Self> {
            // 🚨 Verify Docker is still available
            check_docker_available()?;

            let image = GenericImage::new(image, tag);
            // Build container request with command
            let mut request: testcontainers::core::ContainerRequest<GenericImage> = image.into();
            // Set command and args to keep container running
            let mut cmd_vec = vec![command.to_string()];
            cmd_vec.extend(args.iter().map(|s| (*s).to_string()));
            request = request.with_cmd(cmd_vec);

            let container = request.start().map_err(|e| {
                let error_msg = format!("{e}");
                if is_docker_unavailable_error(&error_msg) {
                    TestcontainersError::DockerUnavailable(format!(
                        "Docker daemon connection failed during container start: {e}\n   ⚠️  STOP: Cannot connect to Docker daemon\n   💡 FIX: Start Docker Desktop or Docker daemon"
                    ))
                } else {
                    TestcontainersError::CreationFailed(format!("Failed to start container: {e}\n   ⚠️  STOP: Container creation failed\n   💡 FIX: Check Docker image exists and Docker daemon is running"))
                }
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
            // 🚨 Verify Docker is still available
            check_docker_available()?;

            let mut image = GenericImage::new(image, tag);
            for port in ports {
                image = image.with_exposed_port(ContainerPort::Tcp(*port));
            }
            let container = image.start().map_err(|e| {
                let error_msg = format!("{e}");
                if is_docker_unavailable_error(&error_msg) {
                    TestcontainersError::DockerUnavailable(format!(
                        "Docker daemon connection failed during container start: {e}\n   ⚠️  STOP: Cannot connect to Docker daemon\n   💡 FIX: Start Docker Desktop or Docker daemon"
                    ))
                } else {
                    TestcontainersError::CreationFailed(format!("Failed to start container: {e}"))
                }
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
                    "Failed to get host port for container port {container_port}: {e}"
                ))
            })?;
            Ok(port)
        }

        /// Get the underlying testcontainers Container
        ///
        /// Allows access to advanced testcontainers features if needed.
        #[must_use]
        pub const fn container(&self) -> &Container<GenericImage> {
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
    use crate::assert_eq_msg;
    use crate::assert_err;
    use crate::assertions::assert_that_with_msg;
    use crate::test;

    // ========================================================================
    // 1. ERROR PATH TESTING - Test all error variants (80% of bugs)
    // ========================================================================

    test!(test_testcontainers_error_display, {
        // Arrange: Create all error variants
        let errors = vec![
            TestcontainersError::CreationFailed("test".to_string()),
            TestcontainersError::OperationFailed("test".to_string()),
            TestcontainersError::InvalidConfig("test".to_string()),
            TestcontainersError::CommandExecutionFailed("test".to_string()),
            TestcontainersError::StdoutReadFailed("test".to_string()),
            TestcontainersError::StderrReadFailed("test".to_string()),
            TestcontainersError::ExitCodeFailed("test".to_string()),
        ];

        // Act & Assert: Verify all error variants display correctly
        for error in errors {
            let display = format!("{error}");
            assert_that_with_msg(&!display.is_empty(), |v| *v, "Error should have display message");
            assert_that_with_msg(&display.contains("test"), |v| *v, "Error should contain message");
        }
    });

    test!(test_exec_result_structure, {
        // Arrange: Create ExecResult
        let result = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: exec::SUCCESS_EXIT_CODE,
        };

        // Act & Assert: Verify ExecResult structure
        assert_eq_msg!(&result.stdout, &"output".to_string(), "Stdout should match");
        assert_eq_msg!(&result.stderr, &"error".to_string(), "Stderr should match");
        assert_eq_msg!(&result.exit_code, &exec::SUCCESS_EXIT_CODE, "Exit code should match");
    });

    test!(test_exec_result_clone, {
        // Arrange: Create ExecResult
        let result1 = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: exec::SUCCESS_EXIT_CODE,
        };

        // Act: Clone the result
        let result2 = result1.clone();

        // Assert: Verify cloned fields match original
        assert_eq_msg!(&result1.stdout, &result2.stdout, "Cloned stdout should match");
        assert_eq_msg!(&result1.stderr, &result2.stderr, "Cloned stderr should match");
        assert_eq_msg!(&result1.exit_code, &result2.exit_code, "Cloned exit code should match");
    });

    test!(test_exec_result_debug, {
        // Arrange: Create ExecResult
        let result = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: exec::SUCCESS_EXIT_CODE,
        };

        // Act: Format as debug
        let debug = format!("{result:?}");

        // Assert: Verify debug output contains expected fields
        assert_that_with_msg(&debug.contains("output"), |v| *v, "Debug should contain stdout");
        assert_that_with_msg(&debug.contains("error"), |v| *v, "Debug should contain stderr");
        assert_that_with_msg(&debug.contains("0"), |v| *v, "Debug should contain exit code");
    });

    // ========================================================================
    // 2. STUB BEHAVIOR TESTING - Test feature-gated code paths
    // ========================================================================

    #[cfg(not(feature = "testcontainers"))]
    test!(test_stubs_return_errors, {
        // Arrange: Create container client
        let client = ContainerClient::new();

        // Act: Attempt to create container (should fail in stub mode)
        let result = GenericContainer::new(&client, "alpine", "latest");

        // Assert: Verify stub returns InvalidConfig error
        assert_err!(&result, "Stub should return error");
        match result {
            Err(TestcontainersError::InvalidConfig(msg)) => {
                assert_that_with_msg(
                    &msg.contains("testcontainers feature is not enabled"),
                    |v| *v,
                    "Error message should indicate feature not enabled",
                );
            }
            _ => panic!("Expected InvalidConfig error"),
        }

        // Act: Attempt to use stub container methods
        let container = GenericContainer;
        let port_result = container.get_host_port(DEFAULT_HTTP_PORT);
        let exec_result = container.exec("echo", &["test"]);

        // Assert: Verify all stub methods return errors
        assert_err!(&port_result, "Port result should be error");
        assert_err!(&exec_result, "Exec result should be error");
    });

    #[cfg(not(feature = "testcontainers"))]
    test!(test_stub_container_client, {
        // Arrange: Create container clients
        let client1 = ContainerClient::new();
        let client2 = ContainerClient::default();

        // Act: Access client references
        let _ref1 = client1.client();
        let _ref2 = client2.client();

        // Assert: Both should work (no panic) - stub clients are valid
    });
}
