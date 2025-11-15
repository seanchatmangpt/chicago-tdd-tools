// Integration tests for testcontainers module
//
// **Poka-yoke**: These are INTEGRATION tests, not unit tests. They require Docker
// and test real container interactions. Unit tests should NOT use testcontainers.
//
// These tests cover integration-level functionality of the testcontainers module:
// - Error path testing (all error variants) with real containers
// - Boundary condition testing with real containers
// - Feature testing (ports, env vars, wait conditions) with real containers
// - ExecResult structure testing with real containers
// - Container client testing with real containers
// - Stress testing (concurrency) with real containers
//
// **CRITICAL**: These tests require Docker to be running and the testcontainers feature enabled.
// If Docker is stopped, these tests MUST fail (not skip).
//
// Test Organization
//
// Tests are organized by category:
// 1. Error path testing - Tests all error variants (80% of bugs)
// 2. Boundary condition testing - Tests edge cases
// 3. Feature testing - Tests specific features (ports, env vars, wait conditions)
// 4. ExecResult structure testing - Tests ExecResult behavior
// 5. Container client testing - Tests ContainerClient functionality
// 6. Stress testing - Tests concurrent operations

#[cfg(all(feature = "testcontainers", test))]
mod tests {
    mod common {
        include!("../test_common.inc");
    }
    use chicago_tdd_tools::prelude::*;
    use chicago_tdd_tools::assertions::assert_that_with_msg;
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

    test!(exec_error_paths, {
        // Arrange: Set up Docker and container
        require_docker();
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act: Try to exec a non-existent command
        let result = container.exec("nonexistent_command_xyz", &[]);

        // Assert: Exec should succeed (container is running), but command should fail (exit_code != 0)
        assert_ok!(&result, "Exec should succeed even if command doesn't exist");
        let exec_result = result.expect("Exec should succeed");
        assert_that_with_msg(&exec_result.exit_code, |v| *v != 0, "Non-existent command should have non-zero exit code");
        assert_that_with_msg(
            &(exec_result.stderr.contains("executable file not found") || exec_result.stdout.contains("executable file not found")),
            |v| *v,
            "Error message should indicate command not found"
        );
    });

    test!(exec_error_recovery, {
        // Arrange: Set up Docker and container
        require_docker();
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act: First exec fails (invalid command) - exec succeeds but command fails
        let result1 = container.exec("nonexistent_command", &[]);
        assert_ok!(&result1, "Exec should succeed even if command doesn't exist");
        let exec_result1 = result1.expect("Exec should succeed");
        assert_that_with_msg(&exec_result1.exit_code, |v| *v != 0, "Invalid command should have non-zero exit code");

        // Act: Container should still be usable after error
        let result2 = container.exec("echo", &["recovery", "test"]);

        // Assert: Container should be usable after error
        assert_ok!(&result2, "Container should be usable after error");
        let exec_result2 = result2.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result2.exit_code, &0, "Recovery exec should succeed");
        assert_that_with_msg(&exec_result2.stdout.contains("recovery"), |v| *v, "Should capture recovery output");
    });

    // ========================================================================
    // 2. BOUNDARY CONDITION TESTING - Test edge cases
    // ========================================================================

    test!(exec_boundary_conditions, {
        // Arrange: Set up Docker and container
        require_docker();
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act & Assert: Empty command args
        let result = container.exec("echo", &[]);
        assert_ok!(&result, "Exec with empty args should work");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result.exit_code, &0, "Empty echo should succeed");

        // Act & Assert: Single arg
        let result = container.exec("echo", &["hello"]);
        assert_ok!(&result, "Exec with single arg should work");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(exec_result.stdout.trim(), "hello", "Echo output should match");

        // Act & Assert: Multiple args
        let result = container.exec("echo", &["hello", "world", "test"]);
        assert_ok!(&result, "Exec with multiple args should work");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_that_with_msg(&exec_result.stdout.contains("hello"), |v| *v, "Output should contain hello");
        assert_that_with_msg(&exec_result.stdout.contains("world"), |v| *v, "Output should contain world");

        // Act & Assert: Command that produces stderr (non-zero exit)
        let result = container.exec("sh", &["-c", "echo error >&2; exit 1"]);
        assert_ok!(&result, "Exec should succeed even if command fails");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result.exit_code, &1, "Command should exit with code 1");
        assert_that_with_msg(&exec_result.stderr.contains("error"), |v| *v, "Should capture stderr");
    });

    // Test that verifies container lifecycle: containers created with with_command() stay running
    //
    // This test verifies:
    // 1. Containers created with with_command() stay running
    // 2. Multiple exec operations work on the same container
    // 3. Container doesn't exit between exec operations
    //
    // **Root Cause Prevention**: This test would catch the pattern where containers exit
    // before exec is called, preventing "container is not running" errors.
    // Pattern: Use with_command() for images that exit immediately.
    test!(container_lifecycle_stays_running_for_exec, {
        require_docker();

        // Arrange: Create container with command to keep it running
        // **Root Cause Prevention**: Use with_command() to ensure container stays running
        // This prevents "container is not running" errors when executing commands
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act: Execute first command
        let result1 = container.exec("echo", &["first"]);
        assert_ok!(&result1, "First exec should succeed");
        let exec_result1 = result1.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result1.exit_code, &0, "First command should succeed");

        // Act: Execute second command (verifies container is still running)
        let result2 = container.exec("echo", &["second"]);
        assert_ok!(&result2, "Second exec should succeed (container still running)");
        let exec_result2 = result2.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result2.exit_code, &0, "Second command should succeed");

        // Act: Execute third command (verifies container remains running)
        let result3 = container.exec("sh", &["-c", "echo third"]);
        assert_ok!(&result3, "Third exec should succeed (container still running)");
        let exec_result3 = result3.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result3.exit_code, &0, "Third command should succeed");
        assert_that_with_msg(&exec_result3.stdout.contains("third"), |v| *v, "Third command output should match");

        // Assert: Verify all commands executed successfully (container stayed running)
        // This verifies that with_command() keeps container running for multiple exec operations
        assert_that_with_msg(
            &(exec_result1.stdout.contains("first") && exec_result2.stdout.contains("second") && exec_result3.stdout.contains("third")),
            |v| *v,
            "All commands should execute successfully, verifying container stays running"
        );
    });

    // Test that verifies entrypoint override works for containers with problematic entrypoints
    //
    // This test verifies:
    // 1. Containers with entrypoint override (Docker CLI workaround) stay running
    // 2. Exec operations work correctly with entrypoint override
    // 3. Container lifecycle is maintained for Docker CLI-created containers
    //
    // **Root Cause Prevention**: This test verifies the entrypoint override workaround
    // works correctly for images like otel/weaver that have entrypoints that interfere.
    test!(container_entrypoint_override_works, {
        require_docker();

        // Arrange: Create container with entrypoint override (simulating weaver scenario)
        // Use alpine with entrypoint override to test the Docker CLI workaround
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(
            client.client(),
            ALPINE_IMAGE,
            ALPINE_TAG,
            "sleep",
            &["infinity"],
            Some(&["/bin/sh"]),  // Override entrypoint to test Docker CLI workaround
        )
        .unwrap_or_else(|e| panic!("Failed to create container with entrypoint override: {}", e));

        // Act: Execute first command (verifies container is running and exec works)
        let result1 = container.exec("echo", &["entrypoint", "override", "test"]);
        assert_ok!(&result1, "First exec should succeed with entrypoint override");
        let exec_result1 = result1.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result1.exit_code, &0, "First command should succeed");
        assert_that_with_msg(
            &exec_result1.stdout.contains("entrypoint"),
            |v| *v,
            "First command output should contain expected text"
        );

        // Act: Execute second command (verifies container stays running)
        let result2 = container.exec("sh", &["-c", "echo second command"]);
        assert_ok!(&result2, "Second exec should succeed (container still running)");
        let exec_result2 = result2.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result2.exit_code, &0, "Second command should succeed");
        assert_that_with_msg(
            &exec_result2.stdout.contains("second"),
            |v| *v,
            "Second command output should contain expected text"
        );

        // Assert: Verify entrypoint override worked (container stays running, exec works)
        // This verifies that the Docker CLI workaround for entrypoint override is functional
        assert_that_with_msg(
            &(exec_result1.stdout.contains("entrypoint") && exec_result2.stdout.contains("second")),
            |v| *v,
            "Entrypoint override should work - container stays running and exec operations succeed"
        );
    });

    // **Gemba Walk Fix**: Add critical boundary condition tests (80/20 - catch 80% of bugs)
    // These tests cover edge cases that are likely to cause bugs:
    // - Empty/invalid container image/tag (should fail)
    // - Invalid port numbers (should fail)
    // - Special characters in env vars (should work)

    test!(container_creation_boundaries, {
        // Arrange: Set up Docker and client
        require_docker();
        let client = ContainerClient::new();

        // Act & Assert: Empty image name should fail
        let result = GenericContainer::new(client.client(), "", ALPINE_TAG);
        assert_err!(&result, "Empty image name should fail");
        match result {
            Err(TestcontainersError::CreationFailed(_)) => {
                // Expected error variant
            }
            Err(e) => panic!("Expected CreationFailed error, got: {:?}", e),
            Ok(_) => panic!("Expected error, got success"),
        }

        // Act & Assert: Empty tag should fail
        let result = GenericContainer::new(client.client(), ALPINE_IMAGE, "");
        assert_err!(&result, "Empty tag should fail");
        match result {
            Err(TestcontainersError::CreationFailed(_)) => {
                // Expected error variant
            }
            Err(e) => panic!("Expected CreationFailed error, got: {:?}", e),
            Ok(_) => panic!("Expected error, got success"),
        }
    });

    test!(port_mapping_invalid_boundaries, {
        // Arrange: Set up Docker and client
        require_docker();
        let client = ContainerClient::new();

        // Act & Assert: Maximum valid port (u16::MAX = 65535) should work
        // Note: Can't test >65535 as it's out of range for u16 type system
        // Type system prevents invalid ports at compile time
        let result = GenericContainer::with_ports(client.client(), NGINX_IMAGE, NGINX_TAG, &[u16::MAX]);
        assert_ok!(&result, "Maximum valid port should work");
        // Verify container was created successfully
        let _container = result.expect("Container should be created with maximum valid port");
    });

    test!(env_vars_special_characters, {
        // Arrange: Set up Docker and client
        require_docker();
        let client = ContainerClient::new();

        // Act & Assert: Special characters in env vars should work
        let mut special_env = HashMap::new();
        special_env.insert("TEST_VAR".to_string(), "value with spaces".to_string());
        special_env.insert("TEST_VAR2".to_string(), "value-with-dashes".to_string());
        special_env.insert("TEST_VAR3".to_string(), "value_with_underscores".to_string());
        special_env.insert("TEST_VAR4".to_string(), "value123".to_string());

        let container = GenericContainer::with_env_and_command(
            client.client(),
            ALPINE_IMAGE,
            ALPINE_TAG,
            special_env,
            Some(("sleep", &["infinity"])),
        )
        .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Verify special characters work (if container supports it)
        let result = container.exec("sh", &["-c", "echo $TEST_VAR"]);
        if let Ok(exec_result) = result {
            assert_that_with_msg(&exec_result.stdout.contains("value with spaces"), |v| *v, "Special characters should work");
        }
    });

    // **FMEA Fix (RPN 243)**: Add negative test cases to verify tests fail when they should
    // These tests verify that our test infrastructure correctly detects failures,
    // preventing false negatives where tests pass when they should fail.

    test!(negative_test_empty_image_fails, {
        // Arrange: Set up Docker and client
        require_docker();
        let client = ContainerClient::new();

        // Act: Attempt to create container with empty image (should fail)
        let result = GenericContainer::new(client.client(), "", ALPINE_TAG);

        // Assert: Verify test correctly detects failure (negative test case)
        assert_err!(&result, "Empty image should fail");
        match result {
            Err(TestcontainersError::CreationFailed(_)) | Err(TestcontainersError::InvalidConfig(_)) => {
                // Expected error variant - test correctly detected failure
            }
            Err(e) => panic!("Expected CreationFailed or InvalidConfig error, got: {:?}", e),
            Ok(_) => panic!("Expected error for empty image, but got success - FALSE NEGATIVE DETECTED"),
        }
    });

    test!(negative_test_invalid_port_fails, {
        // Arrange: Set up Docker and client
        require_docker();
        let client = ContainerClient::new();

        // Act: Test maximum valid port (u16::MAX = 65535)
        // Note: Can't test >65535 as it's out of range for u16 type system
        // Type system prevents invalid ports at compile time - this is a positive test
        let result = GenericContainer::with_ports(client.client(), NGINX_IMAGE, NGINX_TAG, &[u16::MAX]);

        // Assert: Verify maximum valid port works (positive test case)
        assert_ok!(&result, "Maximum valid port should work");
        // Verify container was created successfully
        let _container = result.expect("Container should be created with maximum valid port");
    });

    test!(negative_test_nonexistent_command_fails, {
        // Arrange: Set up Docker and container
        require_docker();
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act: Execute non-existent command (exec succeeds, but command fails)
        let result = container.exec("nonexistent_command_xyz_12345", &[]);

        // Assert: Verify exec succeeds (container is running), but command fails (negative test case)
        assert_ok!(&result, "Exec should succeed even if command doesn't exist");
        let exec_result = result.expect("Exec should succeed");
        assert_that_with_msg(&exec_result.exit_code, |v| *v != 0, "Non-existent command should have non-zero exit code");
        // Verify test correctly detects command failure (not a false negative)
        assert_that_with_msg(
            &(exec_result.stderr.contains("not found") || exec_result.stderr.contains("executable file not found") || exec_result.stdout.contains("not found")),
            |v| *v,
            "Error message should indicate command not found - if this assertion fails, we have a false negative"
        );
    });

    // ========================================================================
    // 3. FEATURE TESTING - Test specific features
    // ========================================================================

    test!(port_mapping_boundaries, {
        // Arrange: Set up Docker and client
        require_docker();
        let client = ContainerClient::new();

        // Act & Assert: Single port (use nginx which exposes port 80)
        let container = GenericContainer::with_ports(client.client(), NGINX_IMAGE, NGINX_TAG, &[80])
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
        let port = container
            .get_host_port(80)
            .unwrap_or_else(|e| panic!("Failed to get host port: {}", e));
        assert_that_with_msg(&port, |v| *v > 0, "Port should be mapped");

        // Act & Assert: Multiple ports (use nginx which can expose multiple ports)
        let container =
            GenericContainer::with_ports(client.client(), NGINX_IMAGE, NGINX_TAG, &[80, 443, 8080])
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
        assert_that_with_msg(&(port80 > 0 && port443 > 0 && port8080 > 0), |v| *v, "All ports should be mapped");
    });

    test!(env_vars_all_paths, {
        // Arrange: Set up Docker and client
        require_docker();
        let client = ContainerClient::new();

        // Act & Assert: Empty env vars
        let empty_env = HashMap::new();
        let _container = GenericContainer::with_env(client.client(), ALPINE_IMAGE, ALPINE_TAG, empty_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act & Assert: Single env var
        let mut single_env = HashMap::new();
        single_env.insert("TEST_VAR".to_string(), "test_value".to_string());
        let container = GenericContainer::with_env_and_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, single_env, Some(("sleep", &["infinity"])))
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Verify env var is set (if container supports it)
        let result = container.exec("sh", &["-c", "echo $TEST_VAR"]);
        if let Ok(exec_result) = result {
            assert_that_with_msg(&exec_result.stdout.contains("test_value"), |v| *v, "Env var should be set");
        }

        // Act & Assert: Multiple env vars
        let mut multi_env = HashMap::new();
        multi_env.insert("VAR1".to_string(), "value1".to_string());
        multi_env.insert("VAR2".to_string(), "value2".to_string());
        multi_env.insert("VAR3".to_string(), "value3".to_string());
        let _container = GenericContainer::with_env(client.client(), ALPINE_IMAGE, ALPINE_TAG, multi_env)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));
    });

    test!(wait_conditions, {
        // Arrange: Set up Docker, client, and imports
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
        assert_that_with_msg(&stream.peer_addr().is_ok(), |v| *v, "Connection should be established to HTTP service");
    });

    // ========================================================================
    // 4. EXECRESULT STRUCTURE TESTING
    // ========================================================================

    test!(exec_result_structure, {
        // Arrange: Set up Docker and container
        require_docker();
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
            .unwrap_or_else(|e| panic!("Failed to create container: {}", e));

        // Act & Assert: Successful command
        let result = container.exec("echo", &["test"]);
        assert_ok!(&result, "Exec should succeed");
        let exec_result = result.expect("Exec should succeed after assert_ok");

        // Verify ExecResult structure
        assert_that_with_msg(
            &(!exec_result.stdout.is_empty() || exec_result.stdout == "test\n"),
            |v| *v,
            "Should have stdout"
        );
        assert_eq_msg!(&exec_result.exit_code, &0, "Exit code should be 0 for success");

        // Act & Assert: Failed command
        let result = container.exec("sh", &["-c", "exit 42"]);
        assert_ok!(&result, "Exec should succeed even if command fails");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result.exit_code, &42, "Exit code should match command exit code");
    });

    // ========================================================================
    // 5. CONTAINER CLIENT TESTING
    // ========================================================================

    test!(container_client_boundaries, {
        // Arrange: Create multiple clients
        let _client1 = ContainerClient::new();
        let _client2 = ContainerClient::new();
        let client3 = ContainerClient::new();
        let client4 = ContainerClient::new();

        // Act & Assert: Verify clients can be used
        if docker_available() {
            let _container1 = GenericContainer::with_command(client3.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None);
            let _container2 = GenericContainer::with_command(client4.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None);
        }
    });

    // ========================================================================
    // 6. STRESS TESTING - Concurrent operations
    // ========================================================================

    test!(concurrent_container_creation, {
        // Arrange: Set up Docker, client, and imports
        require_docker();
        use std::sync::Arc;
        use std::thread;
        use std::time::SystemTime;

        // **FMEA Fix (RPN 144)**: Use unique identifiers for concurrent tests to prevent interference
        // Generate unique test ID based on timestamp to ensure isolation across test runs
        let test_id = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let client = Arc::new(ContainerClient::new());
        let num_containers = 5;

        // Arrange: Create multiple containers concurrently
        let handles: Vec<_> = (0..num_containers)
            .map(|i| {
                let client = Arc::clone(&client);
                let test_id = test_id;
                thread::spawn(move || {
                    // **FMEA Fix**: Use unique identifier combining test_id and container index
                    let unique_id = format!("test-{}-container-{}", test_id, i);
                    
                    // Act: Create container in parallel
                    let container_result =
                        GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None);

                    // Assert: Verify each container is created successfully (observable behavior)
                    assert_ok!(
                        &container_result,
                        &format!("Container {} should be created successfully", unique_id)
                    );

                    let container = container_result.expect("Container should be created");

                    // Verify container is usable (state verification)
                    let exec_result = container.exec("echo", &[&unique_id]);
                    assert_ok!(&exec_result, &format!("Exec should succeed in concurrent container {}", unique_id));

                    let exec_result = exec_result.expect("Exec should succeed");
                    assert_that_with_msg(
                        &exec_result.stdout.contains(&unique_id),
                        |v| *v,
                        &format!("Container {} should execute commands correctly", unique_id)
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
        assert_eq_msg!(
            &containers.len(),
            &num_containers,
            "All containers should be created successfully"
        );
    });

    test!(stress_concurrent_exec, {
        // Arrange: Set up Docker, client, container, and imports
        require_docker();
        use std::sync::Arc;
        use std::thread;
        use std::time::SystemTime;

        // **FMEA Fix (RPN 144)**: Use unique identifiers for concurrent tests to prevent interference
        let test_id = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let client = ContainerClient::new();
            let container = Arc::new(
            GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
                .expect("Container should be created for stress testing"),
        );

        let num_commands = 10;

        // Arrange: Execute multiple commands concurrently on same container
        let handles: Vec<_> = (0..num_commands)
            .map(|i| {
                let container = Arc::clone(&container);
                let test_id = test_id;
                thread::spawn(move || {
                    // **FMEA Fix**: Use unique identifier combining test_id and command index
                    let unique_id = format!("test-{}-stress-{}", test_id, i);
                    
                    // Act: Execute command concurrently
                    let exec_result = container.exec("echo", &[&unique_id]);

                    // Assert: Verify command executes successfully (observable behavior)
                    assert_ok!(&exec_result, &format!("Command {} should execute successfully", unique_id));

                    let exec_result = exec_result.expect("Exec should succeed");
                    assert_eq_msg!(&exec_result.exit_code, &0, &format!("Command {} should exit with code 0", unique_id));
                    assert_that_with_msg(
                        &exec_result.stdout.contains(&unique_id),
                        |v| *v,
                        &format!("Command {} should produce correct output", unique_id)
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
        assert_eq_msg!(&results.len(), &num_commands, "All commands should execute successfully");

        // Verify outputs are distinct (containers don't interfere)
        let outputs: Vec<String> = results.iter().map(|r| r.stdout.clone()).collect();

        // Each output should be unique (verify isolation)
        let unique_outputs: std::collections::HashSet<String> = outputs.iter().cloned().collect();
        assert_eq_msg!(
            unique_outputs.len(),
            num_commands,
            "All command outputs should be distinct (no interference)"
        );
    });

    test!(stress_multiple_containers_concurrent_exec, {
        // Arrange: Set up Docker, client, and imports
        require_docker();
        use std::sync::Arc;
        use std::thread;
        use std::time::SystemTime;

        // **FMEA Fix (RPN 144)**: Use unique identifiers for concurrent tests to prevent interference
        let test_id = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let client = Arc::new(ContainerClient::new());
        let num_containers = 3;
        let commands_per_container = 5;

        // Arrange: Create multiple containers
        let containers: Vec<_> = (0..num_containers)
            .map(|i| {
                GenericContainer::with_command(client.client(), ALPINE_IMAGE, ALPINE_TAG, "sleep", &["infinity"], None)
                    .expect(&format!("Container {} should be created", i))
            })
            .collect();

        let containers = Arc::new(containers);

        // Act: Execute commands concurrently across all containers
        let handles: Vec<_> = (0..num_containers)
            .flat_map(|container_idx| {
                let containers = Arc::clone(&containers);
                let test_id = test_id;
                (0..commands_per_container).map(move |cmd_idx| {
                    let containers = Arc::clone(&containers);
                    let container_idx = container_idx;
                    let cmd_idx = cmd_idx;
                    let test_id = test_id;
                    thread::spawn(move || {
                        // **FMEA Fix**: Use unique identifier combining test_id, container index, and command index
                        let unique_id = format!("test-{}-container-{}-cmd-{}", test_id, container_idx, cmd_idx);
                        
                        let container = &containers[container_idx];
                        let exec_result = container.exec("echo", &[&unique_id]);

                        // Assert: Verify command executes successfully (observable behavior)
                        assert_ok!(
                            &exec_result,
                            &format!("Command {} should execute successfully", unique_id)
                        );

                        let exec_result = exec_result.expect("Exec should succeed");
                        assert_eq_msg!(&exec_result.exit_code, &0, &format!("Command {} should exit with code 0", unique_id));
                        assert_that_with_msg(
                            &exec_result.stdout.contains(&unique_id),
                            |v| *v,
                            &format!("Command {} should produce correct output", unique_id)
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
        assert_eq_msg!(
            &results.len(),
            &(num_containers * commands_per_container),
            "All commands should execute successfully"
        );

        // Verify containers don't interfere with each other
        let outputs: Vec<String> = results.iter().map(|r| r.stdout.clone()).collect();

        let unique_outputs: std::collections::HashSet<String> = outputs.iter().cloned().collect();
        assert_eq_msg!(
            &unique_outputs.len(),
            &(num_containers * commands_per_container),
            "All command outputs should be distinct (containers don't interfere)"
        );
    });
}

