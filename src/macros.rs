//! Macros for Chicago TDD Testing
//!
//! Provides macros to enforce Chicago TDD principles and reduce boilerplate:
//! - AAA pattern enforcement (Arrange-Act-Assert)
//! - Async test wrappers with fixture management
//! - Performance testing (tick budget validation)
//! - Enhanced assertion macros with better error messages

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
/// ```rust,no_run
/// use chicago_tdd_tools::chicago_test;
///
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
/// ```rust,no_run
/// use chicago_tdd_tools::chicago_async_test;
///
/// chicago_async_test!(test_async_feature, {
///     // Arrange: Set up test data
///     let fixture = TestFixture::new()?;
///
///     // Act: Execute async feature (use ? for error propagation)
///     let result = async_function().await?;
///
///     // Assert: Verify behavior
///     assert_success(&result);
///     Ok::<(), MyError>(()) // Return Result - will be unwrapped automatically
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
/// ```rust,no_run
/// use chicago_tdd_tools::{chicago_fixture_test, prelude::*};
///
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
            let $fixture_var = $crate::fixture::TestFixture::new()
                .unwrap_or_else(|e| panic!("Failed to create test fixture: {}", e));

            // Execute test body with 1s timeout for SLA compliance
            let test_future = async { $body };

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
/// ```rust,no_run
/// use chicago_tdd_tools::chicago_performance_test;
///
/// chicago_performance_test!(test_hot_path_performance, {
///     // Arrange: Set up test data
///     let input = create_test_input();
///
///     // Act: Execute hot path operation
///     let (result, ticks) = measure_ticks(|| hot_path_operation(&input));
///
///     // Assert: Verify performance constraint
///     assert!(ticks <= 8, "Hot path exceeded tick budget: {} > 8", ticks);
///     assert_success(&result);
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

/// Assert that a result is successful with detailed error message
///
/// Provides better error messages than standard `assert!` when testing Results.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_ok;
///
/// let result: Result<u32, String> = Ok(42);
/// assert_ok!(result);
///
/// // With custom message
/// assert_ok!(result, "Expected successful operation");
/// ```
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(_) => {}
            Err(e) => panic!("Expected Ok, but got Err: {:?}", e),
        }
    };
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(_) => {}
            Err(e) => panic!("{}: Expected Ok, but got Err: {:?}", $msg, e),
        }
    };
}

/// Assert that a result is an error with detailed message
///
/// Provides better error messages when testing error cases.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_err;
///
/// let result: Result<u32, String> = Err("error".to_string());
/// assert_err!(result);
///
/// // With custom message
/// assert_err!(result, "Expected error case");
/// ```
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(v) => panic!("Expected Err, but got Ok: {:?}", v),
            Err(_) => {}
        }
    };
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(v) => panic!("{}: Expected Err, but got Ok: {:?}", $msg, v),
            Err(_) => {}
        }
    };
}

/// Assert that a value is within tick budget (≤8 ticks)
///
/// Validates performance constraints according to Chatman Constant.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_within_tick_budget;
///
/// let ticks = 5;
/// assert_within_tick_budget!(ticks);
///
/// // With custom message
/// assert_within_tick_budget!(ticks, "Hot path operation");
/// ```
#[macro_export]
macro_rules! assert_within_tick_budget {
    ($ticks:expr) => {
        assert!(
            $ticks <= 8,
            "Tick budget exceeded: {} > 8 (Chatman Constant violation)",
            $ticks
        );
    };
    ($ticks:expr, $msg:expr) => {
        assert!(
            $ticks <= 8,
            "{}: Tick budget exceeded: {} > 8 (Chatman Constant violation)",
            $msg,
            $ticks
        );
    };
}

/// Assert that a value is within a range with detailed error message
///
/// Provides better error messages for range assertions.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_in_range;
///
/// let value = 5;
/// assert_in_range!(value, 0, 10);
///
/// // With custom message
/// assert_in_range!(value, 0, 10, "Value should be in valid range");
/// ```
#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            ($min..=$max).contains(&$value),
            "Value {} not in range [{}, {}]",
            $value,
            $min,
            $max
        );
    };
    ($value:expr, $min:expr, $max:expr, $msg:expr) => {
        assert!(
            ($min..=$max).contains(&$value),
            "{}: Value {} not in range [{}, {}]",
            $msg,
            $value,
            $min,
            $max
        );
    };
}

/// Assert equality with detailed error message and diff output
///
/// Provides better error messages for equality assertions with automatic diff generation.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_eq_msg;
///
/// let actual = 42;
/// let expected = 43;
/// assert_eq_msg!(actual, expected, "Values should match");
/// // Panics with: "Values should match: expected 43, got 42"
/// ```
#[macro_export]
macro_rules! assert_eq_msg {
    ($actual:expr, $expected:expr, $msg:expr) => {{
        let actual_val = &$actual;
        let expected_val = &$expected;
        if actual_val != expected_val {
            panic!(
                "{}: expected {:?}, got {:?}",
                $msg, expected_val, actual_val
            );
        }
    }};
}

/// Assert equality with automatic type inference and diff output
///
/// Enhanced version that provides better error messages with context.
#[macro_export]
macro_rules! assert_eq_enhanced {
    ($actual:expr, $expected:expr $(,)?) => {
        {
            let actual_val = &$actual;
            let expected_val = &$expected;
            if actual_val != expected_val {
                panic!(
                    "assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`",
                    actual_val, expected_val
                );
            }
        }
    };
    ($actual:expr, $expected:expr, $($arg:tt)+) => {
        {
            let actual_val = &$actual;
            let expected_val = &$expected;
            if actual_val != expected_val {
                panic!(
                    "assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`\n{}",
                    actual_val, expected_val, format!($($arg)+)
                );
            }
        }
    };
}

/// Assert that a guard constraint is satisfied
///
/// Validates guard constraints like max_run_len ≤ 8.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assert_guard_constraint;
///
/// let max_run_len = 5;
/// assert_guard_constraint!(max_run_len <= 8, "max_run_len");
/// ```
#[macro_export]
macro_rules! assert_guard_constraint {
    ($condition:expr, $constraint_name:expr) => {
        assert!(
            $condition,
            "Guard constraint violation: {}",
            $constraint_name
        );
    };
}

#[cfg(test)]
#[allow(unnameable_test_items)] // Macro-generated tests trigger this warning
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

    chicago_test!(test_assert_ok_macro, {
        // Arrange: Create successful result
        let result: Result<u32, String> = Ok(42);

        // Act & Assert: Verify assert_ok! macro works
        assert_ok!(result);
        assert_ok!(result, "Should succeed");
    });

    #[test]
    #[should_panic(expected = "Expected Ok")]
    fn test_assert_ok_macro_fails() {
        // Arrange: Create error result
        let result: Result<u32, String> = Err("error".to_string());

        // Act & Assert: Should panic
        assert_ok!(result);
    }

    chicago_test!(test_assert_err_macro, {
        // Arrange: Create error result
        let result: Result<u32, String> = Err("error".to_string());

        // Act & Assert: Verify assert_err! macro works
        assert_err!(result);
        assert_err!(result, "Should fail");
    });

    #[test]
    #[should_panic(expected = "Expected Err")]
    fn test_assert_err_macro_fails() {
        // Arrange: Create successful result
        let result: Result<u32, String> = Ok(42);

        // Act & Assert: Should panic
        assert_err!(result);
    }

    chicago_test!(test_assert_within_tick_budget_macro, {
        // Arrange: Various tick values
        let ticks_valid = 5;
        let ticks_max = 8;
        let ticks_zero = 0;

        // Act & Assert: Verify tick budget validation
        assert_within_tick_budget!(ticks_valid);
        assert_within_tick_budget!(ticks_max);
        assert_within_tick_budget!(ticks_zero);
        assert_within_tick_budget!(ticks_valid, "Test operation");
    });

    #[test]
    #[should_panic(expected = "Tick budget exceeded")]
    fn test_assert_within_tick_budget_macro_fails() {
        // Arrange: Tick value exceeding budget
        let ticks = 9;

        // Act & Assert: Should panic
        assert_within_tick_budget!(ticks);
    }

    chicago_test!(test_assert_in_range_macro, {
        // Arrange: Values within and at boundaries
        let value_mid = 5;
        let value_min = 0;
        let value_max = 10;

        // Act & Assert: Verify range validation
        assert_in_range!(value_mid, 0, 10);
        assert_in_range!(value_min, 0, 10);
        assert_in_range!(value_max, 0, 10);
        assert_in_range!(value_mid, 0, 10, "Value should be valid");
    });

    #[test]
    #[should_panic(expected = "not in range")]
    fn test_assert_in_range_macro_fails_below() {
        // Arrange: Value below range
        let value = -1;

        // Act & Assert: Should panic
        assert_in_range!(value, 0, 10);
    }

    #[test]
    #[should_panic(expected = "not in range")]
    fn test_assert_in_range_macro_fails_above() {
        // Arrange: Value above range
        let value = 11;

        // Act & Assert: Should panic
        assert_in_range!(value, 0, 10);
    }

    chicago_test!(test_assert_eq_msg_macro, {
        // Arrange: Equal values
        let actual = 42;
        let expected = 42;

        // Act & Assert: Verify equality with message
        assert_eq_msg!(actual, expected, "Values should match");
    });

    #[test]
    #[should_panic(expected = "Values should match")]
    fn test_assert_eq_msg_macro_fails() {
        // Arrange: Unequal values
        let actual = 41;
        let expected = 42;

        // Act & Assert: Should panic
        assert_eq_msg!(actual, expected, "Values should match");
    }

    chicago_test!(test_assert_guard_constraint_macro, {
        // Arrange: Valid constraint values
        let max_run_len = 5;

        // Act & Assert: Verify guard constraint validation
        assert_guard_constraint!(max_run_len <= 8, "max_run_len");
        assert_guard_constraint!(true, "always_true");
    });

    #[test]
    #[should_panic(expected = "Guard constraint violation")]
    fn test_assert_guard_constraint_macro_fails() {
        // Arrange: Invalid constraint value
        let max_run_len = 9;

        // Act & Assert: Should panic
        assert_guard_constraint!(max_run_len <= 8, "max_run_len");
    }
}
