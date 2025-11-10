//! Expert-level tests for testcontainers module
//!
//! These tests cover expert-level scenarios that aren't covered in unit or integration tests:
//! - Warmup testing (pre-pull commonly used images)
//!
//! Note: Most expert-level tests have been consolidated into:
//! - `tests.rs` - Unit tests (error paths, boundaries, features, stress tests)
//! - `integration.rs` - Integration tests (cleanup, real containers)
//!
//! This file focuses on unique expert scenarios that don't fit the standard categories.
//!
//! Note: These tests require Docker to be running and the testcontainers feature enabled.

#[cfg(all(feature = "testcontainers", test))]
mod expert_tests {
    mod common {
        include!("../common.rs");
    }
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::testcontainers::*;
    use common::require_docker;

    // Kaizen improvement: Extract repeated Docker image names to constants
    // Pattern: Use named constants for repeated string literals to improve maintainability
    const ALPINE_IMAGE: &str = "alpine";
    const ALPINE_TAG: &str = "latest";
    const NGINX_IMAGE: &str = "nginx";
    const NGINX_TAG: &str = "latest";

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
    fn warmup_image_pull() {
        require_docker();

        let client = ContainerClient::new();

        // Arrange: Pre-pull commonly used images to speed up subsequent tests
        // Chicago TDD: Use real Docker images (real collaborators)
        let images = vec![(ALPINE_IMAGE, ALPINE_TAG), (NGINX_IMAGE, NGINX_TAG)];

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

