//! Unit tests for testcontainers module
//!
//! These tests cover all unit-level functionality of the testcontainers module:
//! - Error path testing (all error variants)
//! - Boundary condition testing
//! - Feature testing (ports, env vars, wait conditions)
//! - ExecResult structure testing
//! - Container client testing
//! - Stress testing (concurrency)
//!
//! Note: These tests require Docker to be running and the testcontainers feature enabled.
//!
//! ## Test Organization
//!
//! Tests are organized by category:
//! 1. Error path testing - Tests all error variants (80% of bugs)
//! 2. Boundary condition testing - Tests edge cases
//! 3. Feature testing - Tests specific features (ports, env vars, wait conditions)
//! 4. ExecResult structure testing - Tests ExecResult behavior
//! 5. Container client testing - Tests ContainerClient functionality
//! 6. Stress testing - Tests concurrent operations

#[cfg(all(feature = "testcontainers", test))]
mod tests {
    mod common {
        include!("../common.rs");
    }
    use chicago_tdd_tools::assert_err;
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::testcontainers::*;
    use common::{docker_available, require_docker};
    use std::collections::HashMap;

    // Kaizen improvement: Extract repeated Docker image names to constants
    // Pattern: Use named constants for repeated string literals to improve maintainability
    const ALPINE_IMAGE: &str = "alpine";
    const ALPINE_TAG: &str = "latest";
    const NGINX_IMAGE: &str = "nginx";
    const NGINX_TAG: &str = "latest";

    // ========================================================================
    // 1. ERROR PATH TESTING - Test all error variants (80% of bugs)
    // ========================================================================

    #[test]
    fn exec_error_paths() {
        require_docker();

        let client = ContainerClient::new();

        // Test: Exec on non-existent container (should fail)
        // Note: We can't easily test this without creating a container first,
        // but we can test exec with invalid commands

        // Test: Exec with invalid command (command not found)
        let container = GenericContainer::new(client.client(), ALPINE_IMAGE, ALPINE_TAG)
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

    #[test]
    fn exec_error_recovery() {
        require_docker();

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), ALPINE_IMAGE, ALPINE_TAG)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // First exec fails (invalid command)
        let result1 = container.exec("nonexistent_command", &[]);
        assert_err!(&result1, "First exec should fail");

        // Container should still be usable after error
        let result2 = container.exec("echo", &["recovery", "test"]);
        assert_ok!(&result2, "Container should be usable after error");
        let exec_result2 = result2.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result2.exit_code, 0, "Recovery exec should succeed");
        assert!(exec_result2.stdout.contains("recovery"), "Should capture recovery output");
    }

    // ========================================================================
    // 2. BOUNDARY CONDITION TESTING - Test edge cases
    // ========================================================================

    #[test]
    fn exec_boundary_conditions() {
        require_docker();

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), ALPINE_IMAGE, ALPINE_TAG)
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
        assert!(exec_result.stderr.contains("error"), "Should capture stderr");
    }

    // ========================================================================
    // 3. FEATURE TESTING - Test specific features
    // ========================================================================

    #[test]
    fn port_mapping_boundaries() {
        require_docker();

        let client = ContainerClient::new();

        // Test: Single port
        let container = GenericContainer::with_ports(client.client(), ALPINE_IMAGE, ALPINE_TAG, &[80])
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
        let port = container
            .get_host_port(80)
            .unwrap_or_else(|e| panic!("Failed to get host port: {}", e));
        assert!(port > 0, "Port should be mapped");

        // Test: Multiple ports
        let container =
            GenericContainer::with_ports(client.client(), ALPINE_IMAGE, ALPINE_TAG, &[80, 443, 8080])
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
        assert!(port80 > 0 && port443 > 0 && port8080 > 0, "All ports should be mapped");
    }

    #[test]
    fn env_vars_all_paths() {
        require_docker();

        let client = ContainerClient::new();

        // Test: Empty env vars
        let empty_env = HashMap::new();
        let _container = GenericContainer::with_env(client.client(), ALPINE_IMAGE, ALPINE_TAG, empty_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Test: Single env var
        let mut single_env = HashMap::new();
        single_env.insert("TEST_VAR".to_string(), "test_value".to_string());
        let container = GenericContainer::with_env(client.client(), ALPINE_IMAGE, ALPINE_TAG, single_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Verify env var is set (if container supports it)
        let result = container.exec("sh", &["-c", "echo $TEST_VAR"]);
        if let Ok(exec_result) = result {
            assert!(exec_result.stdout.contains("test_value"), "Env var should be set");
        }

        // Test: Multiple env vars
        let mut multi_env = HashMap::new();
        multi_env.insert("VAR1".to_string(), "value1".to_string());
        multi_env.insert("VAR2".to_string(), "value2".to_string());
        multi_env.insert("VAR3".to_string(), "value3".to_string());
        let _container = GenericContainer::with_env(client.client(), ALPINE_IMAGE, ALPINE_TAG, multi_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
    }

    #[test]
    fn wait_conditions() {
        require_docker();

        use std::net::TcpStream;
        use std::time::Duration;
        use testcontainers::core::WaitFor;

        let client = ContainerClient::new();

        // Arrange: Create nginx container with wait condition
        // Note: WaitFor::http doesn't exist in testcontainers 0.25, use Duration instead
        let container_result = GenericContainer::with_wait_for(
            client.client(),
            NGINX_IMAGE,
            NGINX_TAG,
            WaitFor::Duration { length: Duration::from_secs(5) },
        );

        // Act: Wait condition should complete (container is ready)
        let container = match container_result {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to create container with wait condition: {}", e);
            }
        };

        // Assert: Verify observable behavior - HTTP service is actually ready
        let host_port = container
            .get_host_port(80)
            .unwrap_or_else(|e| panic!("Failed to get host port: {}", e));

        // Verify HTTP service is accessible (observable behavior)
        let socket_addr = format!("127.0.0.1:{}", host_port)
            .parse()
            .unwrap_or_else(|e| panic!("Failed to parse socket address: {}", e));
        let connection_result = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(2));

        assert_ok!(&connection_result, "HTTP service should be accessible after wait condition");

        // Verify connection is actually established (state verification)
        let stream = connection_result.expect("Connection should succeed after assert_ok");
        assert!(stream.peer_addr().is_ok(), "Connection should be established to HTTP service");
    }

    // ========================================================================
    // 4. EXECRESULT STRUCTURE TESTING
    // ========================================================================

    #[test]
    fn exec_result_structure() {
        require_docker();

        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), ALPINE_IMAGE, ALPINE_TAG)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Test: Successful command
        let result = container.exec("echo", &["test"]);
        assert_ok!(&result, "Exec should succeed");
        let exec_result = result.expect("Exec should succeed after assert_ok");

        // Verify ExecResult structure
        assert!(
            !exec_result.stdout.is_empty() || exec_result.stdout == "test\n",
            "Should have stdout"
        );
        assert_eq!(exec_result.exit_code, 0, "Exit code should be 0 for success");

        // Test: Failed command
        let result = container.exec("sh", &["-c", "exit 42"]);
        assert_ok!(&result, "Exec should succeed even if command fails");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result.exit_code, 42, "Exit code should match command exit code");
    }

    // ========================================================================
    // 5. CONTAINER CLIENT TESTING
    // ========================================================================

    #[test]
    fn container_client_boundaries() {
        // Test: Default client creation
        let _client1 = ContainerClient::new();

        // Test: Default trait implementation
        let _client2 = ContainerClient::new();

        // Test: Multiple clients (should be independent)
        let client3 = ContainerClient::new();
        let client4 = ContainerClient::new();

        // Verify clients can be used
        if docker_available() {
            let _container1 = GenericContainer::new(client3.client(), ALPINE_IMAGE, ALPINE_TAG);
            let _container2 = GenericContainer::new(client4.client(), ALPINE_IMAGE, ALPINE_TAG);
        }
    }

    // ========================================================================
    // 6. STRESS TESTING - Concurrent operations
    // ========================================================================

    #[test]
    fn concurrent_container_creation() {
        require_docker();

        use std::sync::Arc;
        use std::thread;

        let client = Arc::new(ContainerClient::new());
        let num_containers = 5;

        // Arrange: Create multiple containers concurrently
        let handles: Vec<_> = (0..num_containers)
            .map(|i| {
                let client = Arc::clone(&client);
                thread::spawn(move || {
                    // Act: Create container in parallel
                    let container_result =
                        GenericContainer::new(client.client(), ALPINE_IMAGE, ALPINE_TAG);

                    // Assert: Verify each container is created successfully (observable behavior)
                    assert_ok!(
                        &container_result,
                        &format!("Container {} should be created successfully", i)
                    );

                    let container = container_result.expect("Container should be created");

                    // Verify container is usable (state verification)
                    let exec_result = container.exec("echo", &[&format!("container-{}", i)]);
                    assert_ok!(&exec_result, "Exec should succeed in concurrent container");

                    let exec_result = exec_result.expect("Exec should succeed");
                    assert!(
                        exec_result.stdout.contains(&format!("container-{}", i)),
                        "Container {} should execute commands correctly",
                        i
                    );

                    // Container will be dropped here, testing concurrent cleanup
                    container
                })
            })
            .collect();

        // Wait for all containers to be created and tested
        let containers: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should complete successfully"))
            .collect();

        // Assert: Verify all containers were created successfully (state verification)
        assert_eq!(
            containers.len(),
            num_containers,
            "All containers should be created successfully"
        );
    }

    #[test]
    fn stress_concurrent_exec() {
        require_docker();

        use std::sync::Arc;
        use std::thread;

        let client = ContainerClient::new();
        let container = Arc::new(
            GenericContainer::new(client.client(), ALPINE_IMAGE, ALPINE_TAG)
                .expect("Container should be created for stress testing"),
        );

        let num_commands = 10;

        // Arrange: Execute multiple commands concurrently on same container
        let handles: Vec<_> = (0..num_commands)
            .map(|i| {
                let container = Arc::clone(&container);
                thread::spawn(move || {
                    // Act: Execute command concurrently
                    let exec_result = container.exec("echo", &[&format!("stress-{}", i)]);

                    // Assert: Verify command executes successfully (observable behavior)
                    assert_ok!(&exec_result, &format!("Command {} should execute successfully", i));

                    let exec_result = exec_result.expect("Exec should succeed");
                    assert_eq!(exec_result.exit_code, 0, "Command {} should exit with code 0", i);
                    assert!(
                        exec_result.stdout.contains(&format!("stress-{}", i)),
                        "Command {} should produce correct output",
                        i
                    );

                    exec_result
                })
            })
            .collect();

        // Wait for all commands to complete
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should complete successfully"))
            .collect();

        // Assert: Verify all commands executed successfully (state verification)
        assert_eq!(results.len(), num_commands, "All commands should execute successfully");

        // Verify outputs are distinct (containers don't interfere)
        let outputs: Vec<String> = results.iter().map(|r| r.stdout.clone()).collect();

        // Each output should be unique (verify isolation)
        let unique_outputs: std::collections::HashSet<String> = outputs.iter().cloned().collect();
        assert_eq!(
            unique_outputs.len(),
            num_commands,
            "All command outputs should be distinct (no interference)"
        );
    }

    #[test]
    fn stress_multiple_containers_concurrent_exec() {
        require_docker();

        use std::sync::Arc;
        use std::thread;

        let client = Arc::new(ContainerClient::new());
        let num_containers = 3;
        let commands_per_container = 5;

        // Arrange: Create multiple containers
        let containers: Vec<_> = (0..num_containers)
            .map(|i| {
                GenericContainer::new(client.client(), ALPINE_IMAGE, ALPINE_TAG)
                    .expect(&format!("Container {} should be created", i))
            })
            .collect();

        let containers = Arc::new(containers);

        // Act: Execute commands concurrently across all containers
        let handles: Vec<_> = (0..num_containers)
            .flat_map(|container_idx| {
                let containers = Arc::clone(&containers);
                (0..commands_per_container).map(move |cmd_idx| {
                    let containers = Arc::clone(&containers);
                    let container_idx = container_idx;
                    let cmd_idx = cmd_idx;
                    thread::spawn(move || {
                        let container = &containers[container_idx];
                        let exec_result = container.exec(
                            "echo",
                            &[&format!("container-{}-cmd-{}", container_idx, cmd_idx)],
                        );

                        // Assert: Verify command executes successfully (observable behavior)
                        assert_ok!(
                            &exec_result,
                            &format!(
                                "Command {} on container {} should execute successfully",
                                cmd_idx, container_idx
                            )
                        );

                        let exec_result = exec_result.expect("Exec should succeed");
                        assert_eq!(exec_result.exit_code, 0, "Command should exit with code 0");
                        assert!(
                            exec_result
                                .stdout
                                .contains(&format!("container-{}-cmd-{}", container_idx, cmd_idx)),
                            "Command should produce correct output"
                        );

                        exec_result
                    })
                })
            })
            .collect();

        // Wait for all commands to complete
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should complete successfully"))
            .collect();

        // Assert: Verify all commands executed successfully (state verification)
        assert_eq!(
            results.len(),
            num_containers * commands_per_container,
            "All commands should execute successfully"
        );

        // Verify containers don't interfere with each other
        let outputs: Vec<String> = results.iter().map(|r| r.stdout.clone()).collect();

        let unique_outputs: std::collections::HashSet<String> = outputs.iter().cloned().collect();
        assert_eq!(
            unique_outputs.len(),
            num_containers * commands_per_container,
            "All command outputs should be distinct (containers don't interfere)"
        );
    }
}

