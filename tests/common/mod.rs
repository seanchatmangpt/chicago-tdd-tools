//! Shared test utilities for Chicago TDD Tools tests
//!
//! Provides common test helpers used across multiple test files.
//! Consolidates duplicate code to reduce maintenance burden.

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

/// Skip test if Docker is not available
///
/// Helper function to skip a test if Docker is not available.
/// Prints a message and returns `true` if Docker is unavailable.
///
/// # Returns
///
/// `true` if Docker is not available (test should be skipped), `false` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// #[test]
/// fn test_container_operation() {
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

