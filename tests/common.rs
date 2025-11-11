//! Common test utilities for Chicago TDD Tools tests
//!
//! Provides shared test helpers used across multiple test files.
//! Consolidates duplicate code to reduce maintenance burden.
//!
//! # Docker Availability Helper
//!
//! 🚨 CRITICAL - Returns `false` if the Docker daemon is not running.
//!
//! This helper verifies the Docker daemon is actually running and responding,
//! not just that the `docker` command executed successfully.
//!
//! **Kaizen improvement**: Added timeout to prevent hanging when Docker is stopped.
//! Pattern: All external commands should have timeouts to fail fast.
//! Benefits: Prevents tests from hanging indefinitely and provides fast feedback.
//!
//! # Returns
//!
//! `true` if the Docker daemon is running and responding, `false` otherwise.
//!
//! This module provides shared utilities for integration tests, including
//! Docker availability checks and other common test infrastructure.

/// Check if Docker is available and running
///
/// Returns `true` if Docker daemon is running and responding, `false` otherwise.
pub fn docker_available() -> bool {
    use std::process::Command;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    // Kaizen improvement: Add timeout to prevent hanging (fail fast)
    //
    // Pattern: All external commands should have timeouts to fail fast
    // Implementation: Spawn command in thread, use mpsc channel with recv_timeout
    // Timeout duration: 500ms (fast enough to fail within 1s test timeout, enough time for docker info when Docker is running)
    //
    // When to apply:
    // - External command calls (docker, git, etc.)
    // - Network operations
    // - Any operation that could hang indefinitely
    //
    // Benefits:
    // - Prevents tests from hanging indefinitely
    // - Provides fast feedback when dependencies unavailable
    // - Aligns with codebase timeout standards (see docs/TIMEOUT_ENFORCEMENT.md)
    const DOCKER_CHECK_TIMEOUT_MILLIS: u64 = 500;

    // Use docker info to verify daemon is running
    // Spawn command in thread to enable timeout
    let (tx, rx) = mpsc::channel();
    let _handle = thread::spawn(move || {
        let output = Command::new("docker").args(["info"]).output();
        tx.send(output).ok();
    });

    // Wait for result with timeout
    let output = match rx.recv_timeout(Duration::from_millis(DOCKER_CHECK_TIMEOUT_MILLIS)) {
        Ok(result) => match result {
            Ok(output) => output,
            Err(_) => {
                // 🚨 Docker command not found
                eprintln!("🚨 Docker command not found");
                return false;
            }
        },
        Err(_) => {
            // 🚨 Timeout - Docker command hung (likely Docker daemon not running)
            eprintln!(
                "🚨 Docker check timed out after {}ms (Docker daemon likely not running)",
                DOCKER_CHECK_TIMEOUT_MILLIS
            );
            // Don't wait for thread - let it be killed when test ends
            return false;
        }
    };

    // Verify command succeeded and daemon is responding
    if !output.status.success() {
        // 🚨 Docker daemon not running
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("🚨 Docker daemon is not running");
        eprintln!("   Error: {}", stderr);
        return false;
    }

    // Verify Docker daemon is actually responding
    let stdout = String::from_utf8_lossy(&output.stdout);
    let is_available = stdout.contains("Server Version") || stdout.contains("Docker Root Dir");

    if is_available {
        // ✅ Docker daemon is running and responding
    } else {
        // 🚨 Docker daemon not responding correctly
        eprintln!("🚨 Docker daemon is not responding correctly");
    }

    is_available
}

/// Require Docker to be available, panic if not
///
/// 🚨 CRITICAL - Stops test immediately if Docker is unavailable.
///
/// Integration tests that require Docker should use this function.
/// If Docker is not available, the test will fail with a clear error message.
///
/// # Panics
///
/// Panics if Docker is not available, with a message indicating Docker is required
/// and how to start Docker.
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
        panic!(
            "🚨 Docker is required for this test but Docker daemon is not running.\n\
             ⚠️  STOP: Cannot proceed with test\n\
             💡 FIX: Start Docker Desktop or Docker daemon\n\
             📋 macOS: Open Docker Desktop\n\
             📋 Linux: sudo systemctl start docker\n\
             📋 Windows: Start Docker Desktop"
        );
    }
    // ✅ Docker is available, test can proceed
}
