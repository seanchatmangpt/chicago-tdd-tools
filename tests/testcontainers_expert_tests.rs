//! Expert-level tests for testcontainers module
//!
//! These tests cover expert-level scenarios that aren't covered in unit or integration tests:
//! - Warmup testing (pre-pull commonly used images)
//!
//! Note: Most expert-level tests have been consolidated into:
//! - `testcontainers_tests.rs` - Unit tests (error paths, boundaries, features, stress tests)
//! - `testcontainers_integration_tests.rs` - Integration tests (cleanup, real containers)
//!
//! This file focuses on unique expert scenarios that don't fit the standard categories.
//!
//! Note: These tests require Docker to be running and the testcontainers feature enabled.

#[cfg(all(feature = "testcontainers", test))]
mod expert_tests {
    mod common {
        include!("common.rs");
    }
    use common::skip_if_docker_unavailable;
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::testcontainers::*;

    // ========================================================================
    // WARMUP TESTING - Pre-pull commonly used images
    // ========================================================================

    /// Pre-pull commonly used images to speed up subsequent tests
    ///
    /// This test verifies:
    /// 1. Commonly used Docker images can be pulled successfully
    /// 2. Containers can be created from pre-pulled images
    ///
    /// This is Chicago TDD: Real Collaborators (actual Docker images),
    /// Behavior Verification (images can be pulled and used)
    #[test]
    fn test_warmup_image_pull() {
        if skip_if_docker_unavailable() {
            return;
        }

        let client = ContainerClient::new();

        // Arrange: Pre-pull commonly used images to speed up subsequent tests
        // Chicago TDD: Use real Docker images (real collaborators)
        let images = vec![("alpine", "latest"), ("nginx", "latest")];

        // Act: Create containers to trigger image pulls
        for (image, tag) in images {
            let container_result = GenericContainer::new(client.client(), image, tag);

            // Assert: Verify images are pulled successfully (observable behavior)
            assert_ok!(
                &container_result,
                &format!("Image {}:{} should be pulled successfully", image, tag)
            );

            // Verify container is usable (state verification)
            let _container = container_result.expect("Container should be created");
            // Container will be dropped here, testing cleanup
        }
    }
}
