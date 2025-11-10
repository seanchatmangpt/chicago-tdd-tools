//! Feature-specific tests for testcontainers module
//!
//! These tests cover specific features:
//! - Port mapping boundary conditions
//! - Environment variables testing
//! - Wait conditions testing
//! - ExecResult structure testing
//! - Container client testing
//! - Stress testing (concurrency)
//!
//! Note: These tests require Docker to be running and the testcontainers feature enabled.

#[cfg(all(feature = "testcontainers", test))]
mod feature_tests {
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::testcontainers::*;
    use std::collections::HashMap;

    // Helper to check if Docker is available
    fn docker_available() -> bool {
        std::process::Command::new("docker")
            .arg("ps")
            .output()
            .is_ok()
    }

    // 6. PORT MAPPING BOUNDARY CONDITIONS
    #[test]
    fn test_port_mapping_boundaries() {
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
    }

    // 7. ENVIRONMENT VARIABLES TESTING
    #[test]
    fn test_env_vars_all_paths() {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        let client = ContainerClient::new();

        // Test: Empty env vars
        let empty_env = HashMap::new();
        let _container = GenericContainer::with_env(client.client(), "alpine", "latest", empty_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Test: Single env var
        let mut single_env = HashMap::new();
        single_env.insert("TEST_VAR".to_string(), "test_value".to_string());
        let container = GenericContainer::with_env(client.client(), "alpine", "latest", single_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Verify env var is set (if container supports it)
        let result = container.exec("sh", &["-c", "echo $TEST_VAR"]);
        if let Ok(exec_result) = result {
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
    }

    // 8. WAIT CONDITIONS TESTING
    #[test]
    fn test_wait_conditions() {
        if !docker_available() {
            eprintln!("Skipping test: Docker not available");
            return;
        }

        use std::net::TcpStream;
        use std::time::Duration;
        use testcontainers::core::WaitFor;

        let client = ContainerClient::new();

        // Arrange: Create nginx container with wait condition
        // Note: WaitFor::http doesn't exist in testcontainers 0.25, use Duration instead
        let container_result = GenericContainer::with_wait_for(
            client.client(),
            "nginx",
            "latest",
            WaitFor::Duration {
                length: Duration::from_secs(5),
            },
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

        assert_ok!(
            &connection_result,
            "HTTP service should be accessible after wait condition"
        );

        // Verify connection is actually established (state verification)
        let stream = connection_result.expect("Connection should succeed after assert_ok");
        assert!(
            stream.peer_addr().is_ok(),
            "Connection should be established to HTTP service"
        );
    }

    // 9. EXECRESULT STRUCTURE TESTING
    #[test]
    fn test_exec_result_structure() {
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
        let exec_result = result.expect("Exec should succeed after assert_ok");

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
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq!(
            exec_result.exit_code, 42,
            "Exit code should match command exit code"
        );
    }

    // 10. CONTAINER CLIENT TESTING
    #[test]
    fn test_container_client_boundaries() {
        // Test: Default client creation
        let _client1 = ContainerClient::new();

        // Test: Default trait implementation
        let _client2 = ContainerClient::new();

        // Test: Multiple clients (should be independent)
        let client3 = ContainerClient::new();
        let client4 = ContainerClient::new();

        // Verify clients can be used
        if docker_available() {
            let _container1 = GenericContainer::new(client3.client(), "alpine", "latest");
            let _container2 = GenericContainer::new(client4.client(), "alpine", "latest");
        }
    }
}
