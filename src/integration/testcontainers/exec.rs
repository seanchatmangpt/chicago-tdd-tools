//! Command Execution for Testcontainers
//!
//! Provides command execution functionality for containers.

use super::{TestcontainersError, TestcontainersResult};

/// Exit code indicating successful command execution
///
/// **Kaizen improvement**: Extracted magic number `0` to named constant.
/// Pattern: Use named constants instead of magic numbers for semantic values.
pub const SUCCESS_EXIT_CODE: i32 = 0;

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
    use crate::integration::testcontainers::implementation::GenericContainer;
    use std::io::Read;
    use testcontainers::core::ExecCommand;

    impl GenericContainer {
        /// Execute a command in the container
        ///
        /// **FMEA Fix (RPN 210)**: Improved error messages to help diagnose container lifecycle issues.
        /// Added guidance for common failure modes (container not running, command not found, etc.).
        ///
        /// # Arguments
        ///
        /// * `command` - The command to execute (e.g., "echo", "sh")
        /// * `args` - Command arguments
        ///
        /// # Errors
        ///
        /// Returns error if command execution fails (command not found, container not running, etc.)
        ///
        /// # Returns
        ///
        /// Returns `ExecResult` with stdout, stderr, and exit code
        ///
        /// # Note
        ///
        /// The container must be running for exec to work. This works best with service containers
        /// (postgres, redis, nginx, etc.) that stay running.
        ///
        /// **Container Lifecycle**: If exec fails with "container is not running", ensure:
        /// 1. Container was created with a command that keeps it running (e.g., `sleep infinity`)
        /// 2. Container has had time to start (add small delay if needed)
        /// 3. Container hasn't exited unexpectedly (check container logs)
        pub fn exec(&self, command: &str, args: &[&str]) -> TestcontainersResult<ExecResult> {
            // Build command + args into Vec<String> for ExecCommand::new
            // ExecCommand requires owned strings, so convert &str to String
            let mut cmd_args = vec![command.to_string()];
            cmd_args.extend(args.iter().map(|s| s.to_string()));

            let mut exec_result =
                self.container().exec(ExecCommand::new(cmd_args)).map_err(|e| {
                    let error_msg = format!("{e}");
                    // **FMEA Fix**: Provide more helpful error messages based on failure mode
                    if error_msg.contains("not running") || error_msg.contains("stopped") {
                        TestcontainersError::CommandExecutionFailed(format!(
                            "⚠️  Container is not running - cannot execute command '{command}'\n\
                             ⚠️  WARNING: Container must be running to execute commands\n\
                             💡 FIX: Ensure container was created with a command that keeps it running\n\
                             💡 FIX: Use GenericContainer::with_command() with 'sleep infinity' or similar\n\
                             💡 FIX: Add a small delay after container creation to ensure it's ready\n\
                             💡 FIX: Check container logs if container exited unexpectedly\n\
                             \n\
                             Error: {e}"
                        ))
                    } else {
                        TestcontainersError::CommandExecutionFailed(format!(
                            "⚠️  Failed to execute command '{command}': {e}\n\
                             ⚠️  WARNING: Command did not execute successfully\n\
                             💡 FIX: Check command syntax and container state\n\
                             💡 FIX: Verify container is running and command exists in container\n\
                             \n\
                             Error: {e}"
                        ))
                    }
                })?;

            let mut stdout = String::new();
            exec_result.stdout().read_to_string(&mut stdout).map_err(|e| {
                TestcontainersError::StdoutReadFailed(format!("⚠️  Failed to read stdout: {e}\n   ⚠️  WARNING: Could not read command output\n   💡 FIX: Check container is running and command completed"))
            })?;

            let mut stderr = String::new();
            exec_result.stderr().read_to_string(&mut stderr).map_err(|e| {
                TestcontainersError::StderrReadFailed(format!("⚠️  Failed to read stderr: {e}\n   ⚠️  WARNING: Could not read command error output\n   💡 FIX: Check container is running and command completed"))
            })?;

            // exit_code() returns Result<Option<i64>, ...>, convert to i32
            // testcontainers uses i64 for exit codes, but we use i32 for compatibility
            let exit_code_i64 = exec_result
                .exit_code()
                .map_err(|e| {
                    TestcontainersError::ExitCodeFailed(format!("⚠️  Failed to get exit code: {e}\n   ⚠️  WARNING: Could not determine command exit status\n   💡 FIX: Check container is running and command completed"))
                })?
                .ok_or_else(|| {
                    TestcontainersError::ExitCodeFailed("⚠️  Exit code not available\n   ⚠️  WARNING: Could not determine command exit status\n   💡 FIX: Check container is running and command completed".to_string())
                })?;

            // Convert i64 to i32 (standard Unix exit codes fit in i32 range: -128 to 127)
            let exit_code: i32 = exit_code_i64.try_into().map_err(|_| {
                TestcontainersError::ExitCodeFailed("⚠️  Exit code out of i32 range\n   ⚠️  WARNING: Exit code conversion failed\n   💡 FIX: Check command exit code is within valid range".to_string())
            })?;

            Ok(ExecResult { stdout, stderr, exit_code })
        }
    }
}

// Implementation items are accessible through the module path
// The impl blocks extend GenericContainer, so items are available via the type

#[cfg(not(feature = "testcontainers"))]
mod stubs {
    use super::*;
    use crate::integration::testcontainers::implementation::GenericContainer;

    impl GenericContainer {
        pub fn exec(&self, _command: &str, _args: &[&str]) -> TestcontainersResult<ExecResult> {
            Err(TestcontainersError::InvalidConfig(
                "testcontainers feature is not enabled".to_string(),
            ))
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::*;
    use crate::test;

    // ========================================================================
    // 1. ERROR PATH TESTING - Test all error variants (80% of bugs)
    // ========================================================================

    test!(test_exec_result_debug, {
        // Arrange: Create ExecResult
        let result = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: SUCCESS_EXIT_CODE,
        };

        // Act: Format as debug
        let debug = format!("{result:?}");

        // Assert: Verify debug output contains expected fields
        assert!(debug.contains("ExecResult"));
        assert!(debug.contains("output"));
        assert!(debug.contains("error"));
    });

    // Kaizen improvement: Extract magic number to named constant for clarity
    const TEST_EXIT_CODE: i32 = 42;

    test!(test_exec_result_clone, {
        // Arrange: Create ExecResult
        let result = ExecResult {
            stdout: "test".to_string(),
            stderr: "".to_string(),
            exit_code: TEST_EXIT_CODE,
        };

        // Act: Clone the result
        let cloned = result.clone();

        // Assert: Verify cloned fields match original
        assert_eq!(result.stdout, cloned.stdout);
        assert_eq!(result.stderr, cloned.stderr);
        assert_eq!(result.exit_code, cloned.exit_code);
    });

    test!(test_success_exit_code_constant, {
        // Arrange: SUCCESS_EXIT_CODE constant

        // Act & Assert: Verify constant value
        assert_eq!(SUCCESS_EXIT_CODE, 0);
    });

    test!(test_exec_result_success, {
        // Arrange: Create successful ExecResult
        let result = ExecResult {
            stdout: "success".to_string(),
            stderr: "".to_string(),
            exit_code: SUCCESS_EXIT_CODE,
        };

        // Act & Assert: Verify success indicators
        assert_eq!(result.exit_code, SUCCESS_EXIT_CODE);
        assert_eq!(result.stdout, "success");
        assert!(result.stderr.is_empty());
    });

    // Kaizen improvement: Extract magic number to named constant for clarity
    // Standard Unix exit code for "command not found"
    const COMMAND_NOT_FOUND_EXIT_CODE: i32 = 127;

    test!(test_exec_result_failure, {
        // Arrange: Create failed ExecResult
        let result = ExecResult {
            stdout: "".to_string(),
            stderr: "command not found".to_string(),
            exit_code: COMMAND_NOT_FOUND_EXIT_CODE,
        };

        // Act & Assert: Verify failure indicators
        assert_ne!(result.exit_code, SUCCESS_EXIT_CODE);
        assert_eq!(result.exit_code, COMMAND_NOT_FOUND_EXIT_CODE);
        assert!(result.stderr.contains("not found"));
    });

    // ========================================================================
    // 2. STUB BEHAVIOR TESTING - Test feature-gated code paths
    // ========================================================================

    #[cfg(not(feature = "testcontainers"))]
    test!(test_exec_stub_returns_error, {
        // Arrange: Create container client and container (stub mode)
        use crate::integration::testcontainers::{ContainerClient, GenericContainer};

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "test", "latest").unwrap();

        // Act: Attempt to exec command
        let result = container.exec("echo", &["hello"]);

        // Assert: Verify stub returns error
        assert!(result.is_err());
        match result {
            Err(TestcontainersError::InvalidConfig(msg)) => {
                assert!(msg.contains("testcontainers feature is not enabled"));
            }
            _ => panic!("Expected InvalidConfig error"),
        }
    });
}
