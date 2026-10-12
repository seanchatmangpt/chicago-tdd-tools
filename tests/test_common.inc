// Common test utilities for Chicago TDD Tools tests
//
// Provides shared test helpers used across multiple test files.
// Consolidates duplicate code to reduce maintenance burden.
//
// # Docker Availability Helper
//
// 🚨 CRITICAL - Returns `false` if the Docker daemon is not running.
//
// This helper verifies the Docker daemon is actually running and responding,
// not just that the `docker` command executed successfully.
//
// **Kaizen improvement**: Added timeout to prevent hanging when Docker is stopped.
// Pattern: All external commands should have timeouts to fail fast.
// Benefits: Prevents tests from hanging indefinitely and provides fast feedback.
//
// # Returns
//
// `true` if the Docker daemon is running and responding, `false` otherwise.
//
// This module provides shared utilities for integration tests, including
// Docker availability checks and other common test infrastructure.

/// Check if Docker is available and running
///
/// Returns `true` if Docker daemon is running and responding, `false` otherwise.
#[allow(missing_docs)]
pub fn docker_available() -> bool {
    use std::process::Command;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    // Kaizen improvement: Add timeout to prevent hanging (fail fast)
    //
    // Pattern: All external commands should have timeouts to fail fast
    // Implementation: Spawn command in thread, use mpsc channel with recv_timeout
    // Timeout duration: 5000ms (5 seconds) - increased to handle Docker Desktop startup delays
    // and parallel test execution. Fast enough to fail within test timeout, enough time for
    // docker info when Docker is running under load.
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
    const DOCKER_CHECK_TIMEOUT_MILLIS: u64 = 5000;
    const MAX_RETRIES: u32 = 2;

    // Retry logic for parallel test execution - Docker may be slow to respond under load
    for attempt in 0..=MAX_RETRIES {
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
                    if attempt < MAX_RETRIES {
                        thread::sleep(Duration::from_millis(100 * u64::from(attempt + 1)));
                        continue;
                    }
                    eprintln!("🚨 Docker command not found");
                    return false;
                }
            },
            Err(_) => {
                // 🚨 Timeout - Docker command hung (likely Docker daemon not running or under heavy load)
                if attempt < MAX_RETRIES {
                    // Retry on timeout - Docker might be slow under parallel test load
                    thread::sleep(Duration::from_millis(100 * u64::from(attempt + 1)));
                    continue;
                }
                eprintln!(
                    "🚨 Docker check timed out after {}ms after {} attempts (Docker daemon likely not running or under heavy load)",
                    DOCKER_CHECK_TIMEOUT_MILLIS,
                    attempt + 1
                );
                // Don't wait for thread - let it be killed when test ends
                return false;
            }
        };

        // Verify command succeeded and daemon is responding
        if !output.status.success() {
            // 🚨 Docker daemon not running
            if attempt < MAX_RETRIES {
                thread::sleep(Duration::from_millis(100 * u64::from(attempt + 1)));
                continue;
            }
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
            return true;
        } else {
            // 🚨 Docker daemon not responding correctly
            if attempt < MAX_RETRIES {
                thread::sleep(Duration::from_millis(100 * u64::from(attempt + 1)));
                continue;
            }
            eprintln!("🚨 Docker daemon is not responding correctly");
            return false;
        }
    }

    // Should never reach here, but provide fallback
    false
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
#[allow(missing_docs)]
pub fn require_docker() {
    assert!(
        docker_available(),
        "🚨 Docker is required for this test but Docker daemon is not running.\n\
         ⚠️  STOP: Cannot proceed with test\n\
         💡 FIX: Start Docker Desktop or Docker daemon\n\
         📋 macOS: Open Docker Desktop\n\
         📋 Linux: sudo systemctl start docker\n\
         📋 Windows: Start Docker Desktop"
    );
    // ✅ Docker is available, test can proceed
}

/// Skip-or-panic guard for Docker-dependent tests (A121 fix)
///
/// Returns `true` when the test should be skipped: Docker daemon is not
/// reachable AND `WEAVER_ALLOW_SKIP=1` is set (same env convention as the
/// weaver integration tests). Returns `false` when Docker is available and
/// the test should proceed. Panics with the original hard-stop message when
/// Docker is unavailable and `WEAVER_ALLOW_SKIP` is not set (contract
/// unchanged).
///
/// Usage in a test body (callers in `tests/testcontainers/*.rs`):
///
/// ```rust,ignore
/// if common::skip_if_no_docker() {
///     return;
/// }
/// ```
#[allow(missing_docs)]
pub fn skip_if_no_docker() -> bool {
    if docker_available() {
        return false;
    }

    let allow_skip = matches!(
        std::env::var("WEAVER_ALLOW_SKIP"),
        Ok(ref value) if matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES")
    );

    if allow_skip {
        eprintln!(
            "⏭️  SKIPPED (typed): Docker-dependent test bypassed — \
             Docker daemon not available (reason: daemon unreachable) \
             [escape hatch: WEAVER_ALLOW_SKIP=1]"
        );
        true
    } else {
        panic!(
            "🚨 Docker is required for this test but Docker daemon is not running.\n\
             ⚠️  STOP: Cannot proceed with test\n\
             💡 FIX: Start Docker Desktop or Docker daemon\n\
             📋 macOS: Open Docker Desktop\n\
             📋 Linux: sudo systemctl start docker\n\
             📋 Windows: Start Docker Desktop\n\
             💡 ALT: Set WEAVER_ALLOW_SKIP=1 to bypass intentionally"
        );
    }
}
