// Shared test utilities for Chicago TDD Tools tests
//
// Provides common test helpers used across multiple test files.
// Consolidates duplicate code to reduce maintenance burden.

/// Check if Docker is available for testcontainers tests
///
/// This helper is used across multiple testcontainers test files to check
/// if Docker is running before attempting container operations.
///
/// # Returns
///
/// `true` if Docker is available (docker ps succeeds), `false` otherwise.
pub fn docker_available() -> bool {
    std::process::Command::new("docker").arg("ps").output().is_ok()
}

/// Require Docker to be available, panic if not
///
/// Integration tests that require Docker should use this function.
/// If Docker is not available, the test will fail with a clear error message.
///
/// # Panics
///
/// Panics if Docker is not available, with a message indicating Docker is required.
///
/// # Example
///
/// ```rust,no_run
/// #[test]
/// fn test_container_operation() {
///     require_docker();
///     // Test code here...
/// }
/// ```
pub fn require_docker() {
    if !docker_available() {
        panic!("Docker is required for this test but is not available. Please ensure Docker is running.");
    }
}

/// Skip test if Docker is not available
///
/// Helper function to skip a test if Docker is not available.
/// Prints a message and returns `true` if Docker is unavailable.
///
/// **Note**: Use `require_docker()` for integration tests that require Docker.
/// This function is only for tests where Docker is optional.
///
/// # Returns
///
/// `true` if Docker is not available (test should be skipped), `false` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// #[test]
/// fn test_optional_docker_operation() {
///     if skip_if_docker_unavailable() {
///         return;
///     }
///     // Test code here...
/// }
/// ```
pub fn skip_if_docker_unavailable() -> bool {
    if !docker_available() {
        eprintln!("Skipping test: Docker not available");
        return true;
    }
    false
}
