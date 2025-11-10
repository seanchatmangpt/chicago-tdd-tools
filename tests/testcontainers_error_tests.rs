//! Error path and boundary condition tests for testcontainers module
//!
//! These tests focus on critical paths (80% of bugs):
//! - Error path testing (all error variants)
//! - Boundary condition testing
//! - Error recovery testing
//!
//! Note: These tests require Docker to be running and the testcontainers feature enabled.

#[cfg(all(feature = "testcontainers", test))]
mod error_tests {
    use chicago_tdd_tools::assert_err;
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::testcontainers::*;

    // Helper to check if Docker is available
    fn docker_available() -> bool {
        std::process::Command::new("docker")
            .arg("ps")
            .output()
            .is_ok()
    }

    // 1. ERROR PATH TESTING - Test all error variants
    #[test]
    fn test_exec_error_paths() {
        // Skip if Docker not available
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let client = ContainerClient::new();

        // Test: Exec on non-existent container (should fail)
        // Note: We can't easily test this without creating a container first,
        // but we can test exec with invalid commands

        // Test: Exec with invalid command (command not found)
        let container = GenericContainer::new(client.client(), "alpine", "latest")
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Try to exec a non-existent command
        let result = container.exec("nonexistent_command_xyz", &[]);

        // Should fail with CommandExecutionFailed
        assert_err!(&result, "Executing non-existent command should fail");
        match result {
            Err(TestcontainersError::CommandExecutionFailed(_)) => {
                // Expected error variant
            }
            Err(e) => panic!("Expected CommandExecutionFailed, got: {:?}", e),
            Ok(_) => panic!("Expected error, got success"),
        }
    }

    // 2. BOUNDARY CONDITION TESTING
    #[test]
    fn test_exec_boundary_conditions() {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "alpine", "latest")
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Test: Empty command args
        let result = container.exec("echo", &[]);
        assert_ok!(&result, "Exec with empty args should work");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result.exit_code, 0, "Empty echo should succeed");

        // Test: Single arg
        let result = container.exec("echo", &["hello"]);
        assert_ok!(&result, "Exec with single arg should work");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result.stdout.trim(), "hello");

        // Test: Multiple args
        let result = container.exec("echo", &["hello", "world", "test"]);
        assert_ok!(&result, "Exec with multiple args should work");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert!(exec_result.stdout.contains("hello"));
        assert!(exec_result.stdout.contains("world"));

        // Test: Command that produces stderr (non-zero exit)
        let result = container.exec("sh", &["-c", "echo error >&2; exit 1"]);
        assert_ok!(&result, "Exec should succeed even if command fails");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result.exit_code, 1, "Command should exit with code 1");
        assert!(
            exec_result.stderr.contains("error"),
            "Should capture stderr"
        );
    }

    // 5. ERROR RECOVERY TESTING
    #[test]
    fn test_exec_error_recovery() {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "alpine", "latest")
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // First exec fails (invalid command)
        let result1 = container.exec("nonexistent_command", &[]);
        assert_err!(&result1, "First exec should fail");

        // Container should still be usable after error
        let result2 = container.exec("echo", &["recovery", "test"]);
        assert_ok!(&result2, "Container should be usable after error");
        let exec_result2 = result2.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result2.exit_code, 0, "Recovery exec should succeed");
        assert!(
            exec_result2.stdout.contains("recovery"),
            "Should capture recovery output"
        );
    }
}
