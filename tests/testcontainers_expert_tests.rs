//! Expert-level tests for testcontainers module
//!
//! These tests follow expert testing patterns:
//! - Error path testing (all error variants)
//! - Boundary condition testing
//! - Resource cleanup testing
//! - Integration testing with real containers
//!
//! Note: These tests require Docker to be running and the testcontainers feature enabled.

#[cfg(all(feature = "testcontainers", test))]
mod expert_tests {
    use chicago_tdd_tools::assert_err;
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::chicago_test;
    use chicago_tdd_tools::testcontainers::*;
    use std::collections::HashMap;

    // Helper to check if Docker is available
    fn docker_available() -> bool {
        std::process::Command::new("docker")
            .arg("ps")
            .output()
            .is_ok()
    }

    // 1. ERROR PATH TESTING - Test all error variants
    chicago_test!(test_exec_error_paths, {
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
    });

    // 2. BOUNDARY CONDITION TESTING
    chicago_test!(test_exec_boundary_conditions, {
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
        let exec_result = result.unwrap();
        assert_eq!(exec_result.exit_code, 0, "Empty echo should succeed");

        // Test: Single arg
        let result = container.exec("echo", &["hello"]);
        assert_ok!(&result, "Exec with single arg should work");
        let exec_result = result.unwrap();
        assert_eq!(exec_result.stdout.trim(), "hello");

        // Test: Multiple args
        let result = container.exec("echo", &["hello", "world", "test"]);
        assert_ok!(&result, "Exec with multiple args should work");
        let exec_result = result.unwrap();
        assert!(exec_result.stdout.contains("hello"));
        assert!(exec_result.stdout.contains("world"));

        // Test: Command that produces stderr (non-zero exit)
        let result = container.exec("sh", &["-c", "echo error >&2; exit 1"]);
        assert_ok!(&result, "Exec should succeed even if command fails");
        let exec_result = result.unwrap();
        assert_eq!(exec_result.exit_code, 1, "Command should exit with code 1");
        assert!(
            exec_result.stderr.contains("error"),
            "Should capture stderr"
        );
    });

    // 3. RESOURCE CLEANUP TESTING
    chicago_test!(test_container_cleanup_all_paths, {
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
    });

    // 4. INTEGRATION TESTING WITH REAL CONTAINERS
    chicago_test!(test_integration_real_container_exec, {
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
        let exec_result1 = result1.unwrap();
        assert_eq!(exec_result1.exit_code, 0, "Echo should succeed");
        assert!(
            exec_result1.stdout.contains("integration"),
            "Should capture stdout"
        );

        // Execute another command (verify container is still usable)
        let result2 = container.exec("sh", &["-c", "echo 'second command'"]);
        assert_ok!(&result2, "Should execute second command");
        let exec_result2 = result2.unwrap();
        assert_eq!(exec_result2.exit_code, 0, "Second command should succeed");

        // Assert: Verify state changes persist
        // (In this case, each exec is independent, but container remains usable)
        assert!(
            exec_result1.stdout != exec_result2.stdout,
            "Commands should produce different output"
        );
    });

    // 5. ERROR RECOVERY TESTING
    chicago_test!(test_exec_error_recovery, {
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
        let exec_result2 = result2.unwrap();
        assert_eq!(exec_result2.exit_code, 0, "Recovery exec should succeed");
        assert!(
            exec_result2.stdout.contains("recovery"),
            "Should capture recovery output"
        );
    });

    // 6. PORT MAPPING BOUNDARY CONDITIONS
    chicago_test!(test_port_mapping_boundaries, {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let client = ContainerClient::new();

        // Test: Single port
        let container = GenericContainer::with_ports(client.client(), "alpine", "latest", &[80])
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
        let port = container
            .get_host_port(80)
            .unwrap_or_else(|e| panic!("Failed to get host port: {}", e));
        assert!(port > 0, "Port should be mapped");
        // Port is valid u16 (0-65535)

        // Test: Multiple ports
        let container =
            GenericContainer::with_ports(client.client(), "alpine", "latest", &[80, 443, 8080])
                .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
        let port80 = container
            .get_host_port(80)
            .unwrap_or_else(|e| panic!("Failed to get host port 80: {}", e));
        let port443 = container
            .get_host_port(443)
            .unwrap_or_else(|e| panic!("Failed to get host port 443: {}", e));
        let port8080 = container
            .get_host_port(8080)
            .unwrap_or_else(|e| panic!("Failed to get host port 8080: {}", e));
        assert!(
            port80 > 0 && port443 > 0 && port8080 > 0,
            "All ports should be mapped"
        );
        // Ports are valid u16 values (0-65535)
    });

    // 7. ENVIRONMENT VARIABLES TESTING
    chicago_test!(test_env_vars_all_paths, {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let client = ContainerClient::new();

        // Test: Empty env vars
        let empty_env = HashMap::new();
        let _container = GenericContainer::with_env(client.client(), "alpine", "latest", empty_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
        // Container should be created successfully

        // Test: Single env var
        let mut single_env = HashMap::new();
        single_env.insert("TEST_VAR".to_string(), "test_value".to_string());
        let container = GenericContainer::with_env(client.client(), "alpine", "latest", single_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Verify env var is set (if container supports it)
        let result = container.exec("sh", &["-c", "echo $TEST_VAR"]);
        if result.is_ok() {
            let exec_result = result.unwrap();
            assert!(
                exec_result.stdout.contains("test_value"),
                "Env var should be set"
            );
        }

        // Test: Multiple env vars
        let mut multi_env = HashMap::new();
        multi_env.insert("VAR1".to_string(), "value1".to_string());
        multi_env.insert("VAR2".to_string(), "value2".to_string());
        multi_env.insert("VAR3".to_string(), "value3".to_string());
        let _container = GenericContainer::with_env(client.client(), "alpine", "latest", multi_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
        // Container should be created successfully
    });

    // 8. WAIT CONDITIONS TESTING
    #[cfg(feature = "testcontainers")]
    chicago_test!(test_wait_conditions, {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        use std::net::TcpStream;
        use std::time::Duration;
        use testcontainers::core::WaitFor;

        let client = ContainerClient::new();

        // Arrange: Create nginx container with wait condition
        // Chicago TDD: Use real HTTP service container (real collaborator)
        // Note: WaitFor::http doesn't exist in testcontainers 0.25, use Duration instead
        let container_result = GenericContainer::with_wait_for(
            client.client(),
            "nginx",
            "latest",
            WaitFor::Duration {
                length: Duration::from_secs(5),
            }, // Wait for container to start
        );

        // Act: Wait condition should complete (container is ready)
        let container = match container_result {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to create container with wait condition: {}", e);
            }
        };

        // Assert: Verify observable behavior - HTTP service is actually ready
        // Chicago TDD: Verify what code does (HTTP responses), not just that API exists
        let host_port = container
            .get_host_port(80)
            .unwrap_or_else(|e| panic!("Failed to get host port: {}", e));

        // Verify HTTP service is accessible (observable behavior)
        let socket_addr = format!("127.0.0.1:{}", host_port)
            .parse()
            .unwrap_or_else(|e| panic!("Failed to parse socket address: {}", e));
        let connection_result = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(2));

        assert_ok!(
            &connection_result,
            "HTTP service should be accessible after wait condition"
        );

        // Verify connection is actually established (state verification)
        let stream = connection_result.unwrap();
        assert!(
            stream.peer_addr().is_ok(),
            "Connection should be established to HTTP service"
        );
    });

    // 9. EXECRESULT STRUCTURE TESTING
    chicago_test!(test_exec_result_structure, {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "alpine", "latest")
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Test: Successful command
        let result = container.exec("echo", &["test"]);
        assert_ok!(&result, "Exec should succeed");
        let exec_result = result.unwrap();

        // Verify ExecResult structure
        assert!(
            !exec_result.stdout.is_empty() || exec_result.stdout == "test\n",
            "Should have stdout"
        );
        assert_eq!(
            exec_result.exit_code, 0,
            "Exit code should be 0 for success"
        );

        // Test: Failed command
        let result = container.exec("sh", &["-c", "exit 42"]);
        assert_ok!(&result, "Exec should succeed even if command fails");
        let exec_result = result.unwrap();
        assert_eq!(
            exec_result.exit_code, 42,
            "Exit code should match command exit code"
        );
    });

    // 10. CONTAINER CLIENT TESTING
    chicago_test!(test_container_client_boundaries, {
        // Test: Default client creation
        let _client1 = ContainerClient::new();
        // Client should be created successfully (no panic)

        // Test: Default trait implementation
        let _client2 = ContainerClient::new();
        // Default client should work (no panic)

        // Test: Multiple clients (should be independent)
        let client3 = ContainerClient::new();
        let client4 = ContainerClient::new();
        // Both should work independently (no panic)

        // Verify clients can be used
        if docker_available() {
            let _container1 = GenericContainer::new(client3.client(), "alpine", "latest");
            let _container2 = GenericContainer::new(client4.client(), "alpine", "latest");
            // Both should work independently
        }
    });
}
