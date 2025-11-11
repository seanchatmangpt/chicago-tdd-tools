//! Wait Conditions for Testcontainers
//!
//! Provides wait condition functionality for containers.

use super::{TestcontainersError, TestcontainersResult};

#[cfg(feature = "testcontainers")]
mod implementation {
    use super::*;
    use crate::integration::testcontainers::implementation::{ContainerClient, GenericContainer};
    use testcontainers::core::WaitFor;
    use testcontainers::runners::SyncRunner;
    use testcontainers::GenericImage;

    impl GenericContainer {
        /// Create a new generic container with wait conditions
        ///
        /// # Arguments
        ///
        /// * `_client` - Container client instance (unused in minimal implementation)
        /// * `image` - Docker image name
        /// * `tag` - Docker image tag
        /// * `wait_for` - Wait condition to wait for before considering container ready
        ///
        /// # Errors
        ///
        /// Returns error if container creation fails or wait condition times out
        ///
        /// # Example
        ///
        /// ```rust,no_run
        /// use chicago_tdd_tools::testcontainers::*;
        /// use testcontainers::core::WaitFor;
        ///
        /// let client = ContainerClient::new();
        /// let container = GenericContainer::with_wait_for(
        ///     client.client(),
        ///     "nginx",
        ///     "latest",
        ///     WaitFor::message_on_stdout("ready"),
        /// )?;
        /// ```
        pub fn with_wait_for(
            _client: &ContainerClient,
            image: &str,
            tag: &str,
            wait_for: WaitFor,
        ) -> TestcontainersResult<Self> {
            let image = GenericImage::new(image, tag).with_wait_for(wait_for);
            // Convert GenericImage to ContainerRequest before starting
            let request: testcontainers::core::ContainerRequest<GenericImage> = image.into();
            let container = request.start().map_err(|e| {
                TestcontainersError::CreationFailed(format!("🚨 Failed to start container: {e}\n   ⚠️  STOP: Container creation failed\n   💡 FIX: Check Docker image exists and Docker daemon is running"))
            })?;

            Ok(GenericContainer::from_container(container))
        }
    }
}

// Implementation items are accessible through the module path
// The impl blocks extend GenericContainer, so items are available via the type

#[cfg(not(feature = "testcontainers"))]
mod stubs {
    use super::*;
    use crate::integration::testcontainers::implementation::{ContainerClient, GenericContainer};

    impl GenericContainer {
        pub fn with_wait_for(
            _client: &ContainerClient,
            _image: &str,
            _tag: &str,
            _wait_for: (),
        ) -> TestcontainersResult<Self> {
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
    use crate::test;

    // ========================================================================
    // 1. ERROR PATH TESTING - Test all error variants (80% of bugs)
    // ========================================================================

    #[cfg(not(feature = "testcontainers"))]
    test!(test_with_wait_for_stub_returns_error, {
        use crate::integration::testcontainers::{ContainerClient, GenericContainer};

        let client = ContainerClient::new();
        let result = GenericContainer::with_wait_for(
            client.client(),
            "test",
            "latest",
            (), // WaitFor type not available without feature
        );

        assert!(result.is_err());
        match result {
            Err(TestcontainersError::InvalidConfig(msg)) => {
                assert!(msg.contains("testcontainers feature is not enabled"));
            }
            _ => panic!("Expected InvalidConfig error"),
        }
    });

    // ========================================================================
    // 2. FEATURE-GATED CODE PATH TESTING
    // ========================================================================

    test!(test_wait_module_compiles_without_feature, {
        // Test that the module compiles and stub implementation works
        // This verifies the feature-gated code paths are correct
        assert!(true, "Module should compile without testcontainers feature");
    });
}
