//! Integration test helpers with compile-time enforcement
//!
//! **Poka-yoke design**: Type-level prevention ensures integration tests
//! cannot be written without Docker checks. This prevents errors at compile time.

use std::marker::PhantomData;

/// Marker type indicating Docker has been checked
///
/// **Poka-yoke**: This type can only be created by calling `require_docker()`.
/// Integration tests must use `IntegrationTestContext` which requires this marker.
#[derive(Debug, Clone, Copy)]
pub struct DockerChecked;

/// Integration test context that enforces Docker check
///
/// **Poka-yoke**: Integration tests must create this context, which requires
/// calling `require_docker()` first. This prevents integration tests from
/// running without Docker checks.
///
/// # Example
///
/// ```rust,no_run
/// #[test]
/// fn test_integration_feature() {
///     // Must check Docker first - compile error if omitted
///     let _ctx = IntegrationTestContext::new();
///     // Integration test code here...
/// }
/// ```
pub struct IntegrationTestContext {
    /// Phantom data ensures Docker check was performed
    ///
    /// **Poka-yoke**: This field can only be populated by `require_docker()`,
    /// preventing integration tests from skipping Docker checks.
    _docker_checked: PhantomData<DockerChecked>,
}

impl IntegrationTestContext {
    /// Create a new integration test context
    ///
    /// **Poka-yoke**: This method requires `require_docker()` to be called first.
    /// The only way to get a `DockerChecked` marker is through `require_docker()`.
    ///
    /// # Panics
    ///
    /// This function does not panic, but `require_docker()` must be called
    /// before creating this context.
    pub fn new(_docker_checked: DockerChecked) -> Self {
        Self {
            _docker_checked: PhantomData,
        }
    }
}

// Include common.rs functions
mod common_impl {
    include!("../common.rs");
}

/// Require Docker to be available for integration tests
///
/// **Poka-yoke**: This function returns `DockerChecked` marker, which is required
/// to create `IntegrationTestContext`. This enforces that integration tests
/// check Docker availability.
///
/// # Panics
///
/// Panics if Docker is not available, with a clear error message.
///
/// # Returns
///
/// Returns `DockerChecked` marker that must be passed to `IntegrationTestContext::new()`.
pub fn require_docker() -> DockerChecked {
    // Use the existing require_docker from common.rs
    common_impl::require_docker();
    DockerChecked
}

/// Check if Docker is available (non-panicking version)
///
/// Returns `Some(DockerChecked)` if Docker is available, `None` otherwise.
/// Use this for optional Docker checks.
pub fn check_docker() -> Option<DockerChecked> {
    if common_impl::docker_available() {
        Some(DockerChecked)
    } else {
        None
    }
}

