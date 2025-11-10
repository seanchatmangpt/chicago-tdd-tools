//! Integration and resource cleanup tests for testcontainers module
//!
//! These tests focus on reliability:
//! - Integration testing with real containers
//! - Resource cleanup testing (all paths)
//!
//! Note: These tests require Docker to be running and the testcontainers feature enabled.

#[cfg(all(feature = "testcontainers", test))]
mod integration_tests {
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

    // 3. RESOURCE CLEANUP TESTING
    #[test]
    fn test_container_cleanup_all_paths() {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        // Test 1: Normal cleanup (container dropped at end of scope)
        {
            let client = ContainerClient::new();
            let _container = GenericContainer::new(client.client(), "alpine", "latest")
                .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
            // Container should be dropped here
        }

        // Test 2: Cleanup in error path
        let result: Result<(), TestcontainersError> = (|| {
            let client = ContainerClient::new();
            let _container = GenericContainer::new(client.client(), "alpine", "latest")
                .map_err(|e| TestcontainersError::CreationFailed(e.to_string()))?;
            Err(TestcontainersError::OperationFailed(
                "test error".to_string(),
            ))
            // Container should still drop even though we return error
        })();
        assert_err!(&result, "Should return error");
        // Container should have been cleaned up via Drop

        // Test 3: Multiple containers cleanup
        {
            let client = ContainerClient::new();
            let _container1 = GenericContainer::new(client.client(), "alpine", "latest")
                .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
            let _container2 = GenericContainer::new(client.client(), "alpine", "latest")
                .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
            // Both containers should be cleaned up
        }
    }

    // 4. INTEGRATION TESTING WITH REAL CONTAINERS
    #[test]
    fn test_integration_real_container_exec() {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        // Arrange: Create real container
        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "alpine", "latest")
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act: Execute real commands
        let result1 = container.exec("echo", &["integration", "test"]);
        assert_ok!(&result1, "Should execute echo command");
        let exec_result1 = result1.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result1.exit_code, 0, "Echo should succeed");
        assert!(
            exec_result1.stdout.contains("integration"),
            "Should capture stdout"
        );

        // Execute another command (verify container is still usable)
        let result2 = container.exec("sh", &["-c", "echo 'second command'"]);
        assert_ok!(&result2, "Should execute second command");
        let exec_result2 = result2.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result2.exit_code, 0, "Second command should succeed");

        // Assert: Verify state changes persist
        // (In this case, each exec is independent, but container remains usable)
        assert!(
            exec_result1.stdout != exec_result2.stdout,
            "Commands should produce different output"
        );
    }
}
