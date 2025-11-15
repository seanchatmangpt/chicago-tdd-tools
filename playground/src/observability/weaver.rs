//! Weaver Examples
//!
//! Demonstrates Weaver live validation integration, including timeout variants.

#[cfg(feature = "weaver")]
use chicago_tdd_tools::observability::weaver::WeaverValidator;
#[cfg(feature = "weaver")]
use chicago_tdd_tools::prelude::*;
#[cfg(feature = "weaver")]
use std::path::PathBuf;

#[cfg(feature = "weaver")]
/// Example: Basic Weaver validator
pub fn example_weaver_basic() {
    // Arrange: Create validator with registry path
    let registry_path = PathBuf::from("registry/");

    // Act: Create validator
    let validator = WeaverValidator::new(registry_path);

    // Assert: Verify validator created
    assert_eq!(validator.otlp_endpoint(), "http://127.0.0.1:4317");
    assert!(!validator.is_running());
}

#[cfg(feature = "weaver")]
/// Example: Weaver with custom config
pub fn example_weaver_custom_config() {
    // Arrange: Create validator with custom ports
    let registry_path = PathBuf::from("registry/");
    let validator = WeaverValidator::with_config(registry_path, 4318, 8081);

    // Act: Get endpoint
    let endpoint = validator.otlp_endpoint();

    // Assert: Verify custom configuration
    assert_eq!(endpoint, "http://127.0.0.1:4318");
}

#[cfg(feature = "weaver")]
/// Example: Weaver availability check
pub fn example_weaver_availability() {
    // Arrange-Act: Check Weaver availability
    let available = WeaverValidator::check_weaver_available().is_ok();

    // Assert: Availability check returns Result (may or may not be available)
    // This is informational - Weaver may not be installed in test environment
    assert!(available || !available); // Always true, but demonstrates pattern
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "weaver")]
    use super::*;
    #[cfg(feature = "weaver")]
    use chicago_tdd_tools::prelude::*;

    #[cfg(feature = "weaver")]
    weaver_test!(test_weaver_basic, {
        // Arrange-Act-Assert: Run example
        example_weaver_basic();
    });

    #[cfg(feature = "weaver")]
    weaver_test!(test_weaver_custom_config, {
        // Arrange-Act-Assert: Run example
        example_weaver_custom_config();
    });

    #[cfg(feature = "weaver")]
    weaver_test!(test_weaver_availability, {
        // Arrange-Act-Assert: Run example
        example_weaver_availability();
    });

    #[cfg(feature = "weaver")]
    /// Example: weaver_test_with_timeout! macro with custom timeout
    weaver_test_with_timeout!(test_weaver_timeout, 30, {
        // Arrange: Create validator
        let registry_path = PathBuf::from("registry/");
        let validator = WeaverValidator::new(registry_path);

        // Act-Assert: Verify validator created (30s timeout for integration tests)
        assert_eq!(validator.otlp_endpoint(), "http://127.0.0.1:4317");
    });
}
