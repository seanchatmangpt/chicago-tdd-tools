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
        pub fn exec(&self, command: &str, args: &[&str]) -> TestcontainersResult<ExecResult> {
            // Build command + args into Vec<String> for ExecCommand::new
            // ExecCommand requires owned strings, so convert &str to String
            let mut cmd_args = vec![command.to_string()];
            cmd_args.extend(args.iter().map(|s| s.to_string()));

            let mut exec_result =
                self.container().exec(ExecCommand::new(cmd_args)).map_err(|e| {
                    TestcontainersError::CommandExecutionFailed(format!(
                        "Failed to execute command '{command}': {e}"
                    ))
                })?;

            let mut stdout = String::new();
            exec_result.stdout().read_to_string(&mut stdout).map_err(|e| {
                TestcontainersError::StdoutReadFailed(format!("Failed to read stdout: {e}"))
            })?;

            let mut stderr = String::new();
            exec_result.stderr().read_to_string(&mut stderr).map_err(|e| {
                TestcontainersError::StderrReadFailed(format!("Failed to read stderr: {e}"))
            })?;

            // exit_code() returns Result<Option<i64>, ...>, convert to i32
            // testcontainers uses i64 for exit codes, but we use i32 for compatibility
            let exit_code_i64 = exec_result
                .exit_code()
                .map_err(|e| {
                    TestcontainersError::ExitCodeFailed(format!("Failed to get exit code: {e}"))
                })?
                .ok_or_else(|| {
                    TestcontainersError::ExitCodeFailed("Exit code not available".to_string())
                })?;

            // Convert i64 to i32 (standard Unix exit codes fit in i32 range: -128 to 127)
            let exit_code: i32 = exit_code_i64.try_into().map_err(|_| {
                TestcontainersError::ExitCodeFailed("Exit code out of i32 range".to_string())
            })?;

            Ok(ExecResult { stdout, stderr, exit_code })
        }
    }
}

// Re-export implementation
#[cfg(feature = "testcontainers")]
pub use implementation::*;

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

    // ========================================================================
    // 1. ERROR PATH TESTING - Test all error variants (80% of bugs)
    // ========================================================================

    #[test]
    fn test_exec_result_debug() {
        let result = ExecResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: SUCCESS_EXIT_CODE,
        };
        let debug = format!("{result:?}");
        assert!(debug.contains("ExecResult"));
        assert!(debug.contains("output"));
        assert!(debug.contains("error"));
    }

    #[test]
    fn test_exec_result_clone() {
        let result =
            ExecResult { stdout: "test".to_string(), stderr: "".to_string(), exit_code: 42 };
        let cloned = result.clone();
        assert_eq!(result.stdout, cloned.stdout);
        assert_eq!(result.stderr, cloned.stderr);
        assert_eq!(result.exit_code, cloned.exit_code);
    }

    #[test]
    fn test_success_exit_code_constant() {
        assert_eq!(SUCCESS_EXIT_CODE, 0);
    }

    #[test]
    fn test_exec_result_success() {
        let result = ExecResult {
            stdout: "success".to_string(),
            stderr: "".to_string(),
            exit_code: SUCCESS_EXIT_CODE,
        };
        assert_eq!(result.exit_code, SUCCESS_EXIT_CODE);
        assert_eq!(result.stdout, "success");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_exec_result_failure() {
        let result = ExecResult {
            stdout: "".to_string(),
            stderr: "command not found".to_string(),
            exit_code: 127,
        };
        assert_ne!(result.exit_code, SUCCESS_EXIT_CODE);
        assert_eq!(result.exit_code, 127);
        assert!(result.stderr.contains("not found"));
    }

    // ========================================================================
    // 2. STUB BEHAVIOR TESTING - Test feature-gated code paths
    // ========================================================================

    #[cfg(not(feature = "testcontainers"))]
    #[test]
    fn test_exec_stub_returns_error() {
        use crate::integration::testcontainers::{ContainerClient, GenericContainer};

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "test", "latest").unwrap();
        let result = container.exec("echo", &["hello"]);

        assert!(result.is_err());
        match result {
            Err(TestcontainersError::InvalidConfig(msg)) => {
                assert!(msg.contains("testcontainers feature is not enabled"));
            }
            _ => panic!("Expected InvalidConfig error"),
        }
    }
}
