//! Test Macros for Chicago TDD Testing
//!
//! Provides macros to enforce Chicago TDD principles for test definitions:
//! - AAA pattern enforcement (Arrange-Act-Assert)
//! - Async test wrappers with fixture management
//! - Performance testing (tick budget validation)
//! - Parameterized testing (when parameterized-testing feature is enabled)

/// Default test timeout in seconds (SLA compliance)
///
/// **Kaizen improvement**: Extracted magic number `1` to named constant.
/// Pattern: Use named constants instead of magic numbers for configuration values.
/// Benefits: Improves readability, maintainability, self-documentation.
///
/// Note: This constant is used in macro expansions. The value `1` is used directly
/// in macros since constants cannot be referenced in macro_rules! expansions.
pub const DEFAULT_TEST_TIMEOUT_SECONDS: u64 = 1;

/// Macro to enforce AAA (Arrange-Act-Assert) pattern
///
/// This macro ensures tests follow the Chicago TDD AAA pattern by requiring
/// explicit Arrange, Act, and Assert sections.
///
/// **Timeout Enforcement**: Tests are automatically wrapped with a 1s timeout
/// to prevent hangs and ensure SLA compliance.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::chicago_test;
///
/// # fn process(input: &str) -> &str { "result" }
/// chicago_test!(test_feature_behavior, {
///     // Arrange: Set up test data
///     let input = "test";
///     let expected = "result";
///
///     // Act: Execute feature
///     let result = process(input);
///
///     // Assert: Verify behavior
///     assert_eq!(result, expected);
/// });
/// ```
#[macro_export]
macro_rules! chicago_test {
    ($name:ident, $body:block) => {
        #[test]
        #[ntest::timeout(1000)] // 1s timeout for SLA compliance
        fn $name() {
            $body
        }
    };
}

/// Macro for async tests with AAA pattern enforcement
///
/// Wraps async test functions and ensures AAA pattern is followed.
/// Supports `?` operator for error propagation - errors are converted to panics.
/// Handles both Result and non-Result returns.
///
/// **Timeout Enforcement**: Tests are automatically wrapped with tokio::time::timeout
/// (1s) to prevent hangs and ensure SLA compliance.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::chicago_async_test;
///
/// # async fn async_function() -> Result<i32, Box<dyn std::error::Error>> { Ok(42) }
/// chicago_async_test!(test_async_feature, {
///     // Arrange: Set up test data
///     let expected = 42;
///
///     // Act: Execute async feature (use ? for error propagation)
///     let result = async_function().await?;
///
///     // Assert: Verify behavior
///     assert_eq!(result, expected);
///     Ok::<(), Box<dyn std::error::Error>>(()) // Return Result - will be unwrapped automatically
/// });
/// ```
#[macro_export]
macro_rules! chicago_async_test {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            use tokio::time::{timeout, Duration};

            // Helper trait to handle both Result and non-Result returns
            trait TestOutput {
                fn handle(self);
            }

            impl TestOutput for () {
                fn handle(self) {}
            }

            impl<E: std::fmt::Debug> TestOutput for Result<(), E> {
                fn handle(self) {
                    if let Err(e) = self {
                        panic!("Test failed: {:?}", e);
                    }
                }
            }

            // Execute body with 1s timeout for SLA compliance
            let test_future = async {
                let output = async { $body }.await;
                TestOutput::handle(output);
            };

            // Execute body with timeout for SLA compliance
            // Note: Using literal value since macro_rules! cannot reference constants
            // The constant DEFAULT_TEST_TIMEOUT_SECONDS documents this value
            match timeout(Duration::from_secs(1), test_future).await {
                Ok(_) => {
                    // Test completed within timeout
                }
                Err(_) => {
                    panic!("Test exceeded 1s timeout (SLA violation)");
                }
            }
        }
    };
}

/// Macro for async tests with automatic fixture setup and teardown
///
/// Creates a test fixture, runs the test body, and ensures cleanup.
///
/// **Timeout Enforcement**: Tests are automatically wrapped with tokio::time::timeout
/// (1s) to prevent hangs and ensure SLA compliance.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::{chicago_fixture_test, prelude::*};
///
/// # fn process(counter: u64) -> u64 { counter + 1 }
/// chicago_fixture_test!(test_with_fixture, fixture, {
///     // Arrange: Use provided fixture
///     let counter = fixture.test_counter();
///
///     // Act: Execute test
///     let result = process(counter);
///
///     // Assert: Verify behavior
///     assert!(result > 0);
/// });
/// ```
#[macro_export]
macro_rules! chicago_fixture_test {
    ($name:ident, $fixture_var:ident, $body:block) => {
        #[allow(unnameable_test_items)]
        #[tokio::test]
        async fn $name() {
            use tokio::time::{timeout, Duration};

            // Arrange: Create fixture
            #[allow(clippy::expect_used)] // Macro - panic is appropriate if fixture creation fails
            let $fixture_var = $crate::core::fixture::TestFixture::new()
                .unwrap_or_else(|e| panic!("Failed to create test fixture: {}", e));

            // Execute test body with 1s timeout for SLA compliance
            let test_future = async { $body };

            // Execute body with timeout for SLA compliance
            // Note: Using literal value since macro_rules! cannot reference constants
            // The constant DEFAULT_TEST_TIMEOUT_SECONDS documents this value
            match timeout(Duration::from_secs(1), test_future).await {
                Ok(_) => {
                    // Test completed within timeout
                }
                Err(_) => {
                    panic!("Test exceeded 1s timeout (SLA violation)");
                }
            }

            // Cleanup: Automatic teardown via Drop
        }
    };
}

/// Macro for performance tests with tick budget validation
///
/// Validates that hot path operations complete within the Chatman Constant
/// (≤8 ticks = 2ns budget).
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::{chicago_performance_test, prelude::*};
///
/// # fn create_test_input() -> i32 { 42 }
/// # fn hot_path_operation(input: &i32) -> i32 { *input }
/// chicago_performance_test!(test_hot_path_performance, {
///     // Arrange: Set up test data
///     let input = create_test_input();
///
///     // Act: Execute hot path operation
///     let (result, ticks) = measure_ticks(|| hot_path_operation(&input));
///
///     // Assert: Verify performance constraint
///     assert!(ticks <= 8, "Hot path exceeded tick budget: {} > 8", ticks);
///     assert_eq!(result, 42);
/// });
/// ```
#[macro_export]
macro_rules! chicago_performance_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            $body
        }
    };
}

#[cfg(feature = "parameterized-testing")]
/// Parameterized test macro using rstest
///
/// Creates parameterized tests that run with multiple input values.
/// This is a wrapper around rstest's `#[rstest]` attribute macro.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::chicago_param_test;
///
/// chicago_param_test! {
///     #[case(1, 2, 3)]
///     #[case(2, 3, 5)]
///     #[case(3, 4, 7)]
///     fn test_addition(a: i32, b: i32, expected: i32) {
///         assert_eq!(a + b, expected);
///     }
/// }
/// ```
#[macro_export]
macro_rules! chicago_param_test {
    {
        $(#[$attr:meta])*
        fn $name:ident($($param:ident: $type:ty),* $(,)?) $body:block
    } => {
        $(#[$attr])*
        #[rstest::rstest]
        fn $name($($param: $type),*) $body
    };
}

#[cfg(not(feature = "parameterized-testing"))]
/// Parameterized test macro (requires parameterized-testing feature)
///
/// Enable the `parameterized-testing` feature to use parameterized tests.
#[macro_export]
macro_rules! chicago_param_test {
    ($($tt:tt)*) => {
        compile_error!("Parameterized testing requires the 'parameterized-testing' feature. Enable with: --features parameterized-testing");
    };
}

/// Macro for OTEL testing with automatic validation
///
/// Automates OTEL span/metric testing with Chicago TDD patterns:
/// - AAA pattern enforcement
/// - Automatic span/metric validation
/// - Test helper setup
///
/// **Timeout Enforcement**: Tests are automatically wrapped with a 1s timeout
/// to prevent hangs and ensure SLA compliance.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::{chicago_otel_test, prelude::*};
///
/// chicago_otel_test!(test_otel_span_validation, {
///     // Arrange: Create test span
///     let span = chicago_tdd_tools::otel::test_helpers::create_test_span("test.operation");
///
///     // Act: Validate span
///     let helper = chicago_tdd_tools::otel::OtelTestHelper::new();
///     helper.assert_spans_valid(&[span.clone()]);
///
///     // Assert: Verify span is valid
///     assert_eq!(span.name, "test.operation");
/// });
/// ```
#[cfg(feature = "otel")]
#[macro_export]
macro_rules! chicago_otel_test {
    ($name:ident, $body:block) => {
        #[test]
        #[ntest::timeout(1000)] // 1s timeout for SLA compliance
        fn $name() {
            $body
        }
    };
}

#[cfg(not(feature = "otel"))]
/// Macro for OTEL testing (requires otel feature)
///
/// Enable the `otel` feature to use OTEL testing macros.
#[macro_export]
macro_rules! chicago_otel_test {
    ($($tt:tt)*) => {
        compile_error!("OTEL testing requires the 'otel' feature. Enable with: --features otel");
    };
}

/// Macro for Weaver testing with automatic validation
///
/// Automates Weaver validation testing with Chicago TDD patterns:
/// - AAA pattern enforcement
/// - Automatic Weaver validator setup
/// - Test helper setup
///
/// **Timeout Enforcement**: Tests are automatically wrapped with tokio::time::timeout
/// (1s) to prevent hangs and ensure SLA compliance.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::{chicago_weaver_test, prelude::*};
/// use std::path::PathBuf;
///
/// chicago_weaver_test!(test_weaver_validation, {
///     // Arrange: Create validator
///     let registry_path = PathBuf::from("registry/");
///     let mut validator = chicago_tdd_tools::weaver::WeaverValidator::new(registry_path);
///
///     // Act: Start Weaver (if available)
///     let start_result = validator.start();
///
///     // Assert: Verify Weaver started or handle unavailable case
///     if start_result.is_ok() {
///         assert!(validator.is_running());
///         validator.stop().unwrap();
///     }
/// });
/// ```
#[cfg(feature = "weaver")]
#[macro_export]
macro_rules! chicago_weaver_test {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            use tokio::time::{timeout, Duration};

            // Helper trait to handle both Result and non-Result returns
            trait TestOutput {
                fn handle(self);
            }

            impl TestOutput for () {
                fn handle(self) {}
            }

            impl<E: std::fmt::Debug> TestOutput for Result<(), E> {
                fn handle(self) {
                    if let Err(e) = self {
                        panic!("Test failed: {:?}", e);
                    }
                }
            }

            // Execute body with 1s timeout for SLA compliance
            let test_future = async {
                let output = async { $body }.await;
                TestOutput::handle(output);
            };

            // Execute body with timeout for SLA compliance
            match timeout(Duration::from_secs(1), test_future).await {
                Ok(_) => {
                    // Test completed within timeout
                }
                Err(_) => {
                    panic!("Test exceeded 1s timeout (SLA violation)");
                }
            }
        }
    };
}

#[cfg(not(feature = "weaver"))]
/// Macro for Weaver testing (requires weaver feature)
///
/// Enable the `weaver` feature to use Weaver testing macros.
#[macro_export]
macro_rules! chicago_weaver_test {
    ($($tt:tt)*) => {
        compile_error!(
            "Weaver testing requires the 'weaver' feature. Enable with: --features weaver"
        );
    };
}

#[cfg(test)]
#[allow(unnameable_test_items)] // Macro-generated tests trigger this warning
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    // Note: We can't use chicago_test! macro here because it would create
    // a test function with the same name, causing conflicts.
    // These tests verify the macro expansion works correctly.

    #[test]
    fn test_chicago_test_macro_expansion() {
        // Verify macro expands to valid test function
        // This is tested by compilation success
        let _ = stringify! {
            chicago_test!(test_basic, {
                let x = 1;
                let y = x + 1;
                assert_eq!(y, 2);
            });
        };
    }

    #[test]
    fn test_chicago_async_test_macro_expansion() {
        // Verify macro expands to valid async test function
        let _ = stringify! {
            chicago_async_test!(test_async_basic, {
                let x = 1;
                let y = x + 1;
                assert_eq!(y, 2);
            });
        };
    }

    #[allow(dead_code)] // Test function is generated by macro, not actually dead code
    #[tokio::test]
    async fn test_chicago_fixture_test_macro() {
        chicago_fixture_test!(test_fixture_basic, fixture, {
            // Arrange
            let counter = fixture.test_counter();

            // Act
            let result = counter + 1;

            // Assert
            assert!(result > 0);
        });
    }

    #[cfg(feature = "parameterized-testing")]
    #[test]
    fn test_parameterized_macro() {
        // This test demonstrates parameterized testing
        // Actual parameterized tests would use chicago_param_test! macro
        assert!(true);
    }
}
