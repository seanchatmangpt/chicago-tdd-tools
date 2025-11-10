//! Testcontainers verification tests for Weaver integration
//!
//! These tests verify that Weaver integration works correctly using testcontainers.
//! This demonstrates Chicago TDD principles:
//! - Real Collaborators: Actual Weaver container, not mocks
//! - State Verification: Verify Weaver binary works in container
//! - Behavior Verification: Verify Weaver can execute commands
//!
//! Note: These tests require Docker to be running and both testcontainers and weaver features enabled.

#[cfg(all(feature = "testcontainers", feature = "weaver", test))]
mod weaver_tests {
    mod common {
        include!("common.rs");
    }
    use common::skip_if_docker_unavailable;

    /// Test that Weaver Docker image is available and can execute commands
    ///
    /// This test verifies:
    /// 1. Weaver Docker image can be pulled and run
    /// 2. Weaver binary exists in the container
    /// 3. Weaver can execute basic commands (--version)
    ///
    /// This is Chicago TDD: Real Collaborators (actual Docker container),
    /// State Verification (weaver binary exists), Behavior Verification (weaver works)
    #[test]
    fn test_weaver_container_available() {
        if skip_if_docker_unavailable() {
            return;
        }

        // Arrange: Create Weaver container
        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "otel/weaver", "latest")
            .unwrap_or_else(|e| panic!("Failed to create Weaver container: {}", e));

        // Act: Execute weaver --version in container
        let result = container.exec("weaver", &["--version"]);

        // Assert: Verify Weaver works (state verification)
        assert_ok!(&result, "Weaver should execute --version successfully");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result.exit_code, 0, "Weaver --version should succeed");
        assert!(
            exec_result.stdout.contains("weaver") || exec_result.stdout.contains("Weaver"),
            "Weaver version output should contain 'weaver' or 'Weaver'"
        );
    }

    /// Test that Weaver can execute registry check command in container
    ///
    /// This test verifies:
    /// 1. Weaver can execute registry commands
    /// 2. Weaver provides helpful error messages when registry not found
    ///
    /// This is Chicago TDD: Behavior Verification (weaver command behavior),
    /// Error Path Testing (80% of bugs)
    #[test]
    fn test_weaver_container_registry_check() {
        if skip_if_docker_unavailable() {
            return;
        }

        // Arrange: Create Weaver container
        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "otel/weaver", "latest")
            .unwrap_or_else(|e| panic!("Failed to create Weaver container: {}", e));

        // Act: Execute weaver registry check with non-existent registry
        let result =
            container.exec("weaver", &["registry", "check", "-r", "/nonexistent/registry"]);

        // Assert: Verify Weaver provides helpful error (behavior verification)
        assert_ok!(&result, "Weaver should execute command (even if it fails)");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        // Weaver should return non-zero exit code for invalid registry
        assert_ne!(exec_result.exit_code, 0, "Weaver should fail with invalid registry");
        // Verify error message is helpful (behavior verification)
        assert!(
            exec_result.stderr.contains("registry")
                || exec_result.stderr.contains("not found")
                || exec_result.stderr.contains("error"),
            "Weaver should provide helpful error message"
        );
    }

    /// Test that Weaver container can be used for live-check verification
    ///
    /// This test verifies:
    /// 1. Weaver container has weaver binary available
    /// 2. Weaver can be used for integration testing
    ///
    /// This is Chicago TDD: Real Collaborators (actual Weaver container),
    /// Working Capability Testing (verify Weaver works in containerized environment)
    #[test]
    fn test_weaver_container_live_check_capability() {
        if skip_if_docker_unavailable() {
            return;
        }

        // Arrange: Create Weaver container
        let client = ContainerClient::new();
        let container = GenericContainer::new(client.client(), "otel/weaver", "latest")
            .unwrap_or_else(|e| panic!("Failed to create Weaver container: {}", e));

        // Act: Check if weaver binary exists and can show help
        let result = container.exec("weaver", &["--help"]);

        // Assert: Verify Weaver is functional (working capability)
        assert_ok!(&result, "Weaver should execute --help successfully");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq!(exec_result.exit_code, 0, "Weaver --help should succeed");
        // Verify help output contains expected commands (behavior verification)
        assert!(
            exec_result.stdout.contains("registry") || exec_result.stdout.contains("live-check"),
            "Weaver help should mention registry or live-check commands"
        );
    }
}
