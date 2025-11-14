//! Assertion Macros for Chicago TDD Testing
//!
//! Provides enhanced assertion macros with better error messages:
//! - Result assertions (`assert_ok`, `assert_err`, `assert_fail`)
//! - Performance assertions (`assert_within_tick_budget`)
//! - Range assertions (`assert_in_range`)
//! - Equality assertions (`assert_eq_msg`, `assert_eq_enhanced`)
//! - Guard constraint assertions (`assert_guard_constraint`)

/// Assert that a result is successful with detailed error message
///
/// Provides better error messages than standard `assert!` when testing Results.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_ok;
///
/// let result: Result<u32, String> = Ok(42);
/// assert_ok!(result);
///
/// // With custom message
/// let result2: Result<u32, String> = Ok(42);
/// assert_ok!(result2, "Expected successful operation");
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
/// ```rust
/// use chicago_tdd_tools::assert_err;
///
/// let result: Result<u32, String> = Err("error".to_string());
/// assert_err!(result);
///
/// // With custom message
/// let result2: Result<u32, String> = Err("error".to_string());
/// assert_err!(result2, "Expected error case");
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

/// Assert that a function call fails, returning the error value
///
/// Convenience macro for testing error paths. Calls the function and asserts it returns `Err`,
/// then returns the error value for further assertions.
///
/// **Ergonomics**: With `test!` macro's new `Result` return type support, this provides
/// a concise way to test error cases without intermediate variables.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::{assert_fail, test};
///
/// # fn fallible_function() -> Result<u32, String> { Err("error".to_string()) }
/// test!(test_should_fail, {
///     // Arrange: Function that should fail
///
///     // Act & Assert: Verify function fails and extract error
///     let error = assert_fail!(fallible_function());
///     assert_eq!(error, "error");
/// });
/// ```
///
/// # Example with custom message
///
/// ```rust
/// use chicago_tdd_tools::{assert_fail, test};
///
/// # fn fallible_function() -> Result<u32, String> { Err("error".to_string()) }
/// test!(test_should_fail_with_msg, {
///     // Act & Assert: Verify function fails with custom message
///     let error = assert_fail!(fallible_function(), "Operation should fail");
///     assert_eq!(error, "error");
/// });
/// ```
#[macro_export]
macro_rules! assert_fail {
    ($call:expr) => {
        match $call {
            Ok(v) => panic!("Expected function to fail, but got Ok: {:?}", v),
            Err(e) => e,
        }
    };
    ($call:expr, $msg:expr) => {
        match $call {
            Ok(v) => panic!("{}: Expected function to fail, but got Ok: {:?}", $msg, v),
            Err(e) => e,
        }
    };
}

/// Assert that a value is within tick budget (≤8 ticks)
///
/// Validates performance constraints according to Chatman Constant.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_within_tick_budget;
///
/// let ticks = 5;
/// assert_within_tick_budget!(ticks);
///
/// // With custom message
/// let ticks2 = 5;
/// assert_within_tick_budget!(ticks2, "Hot path operation");
/// ```
#[macro_export]
macro_rules! assert_within_tick_budget {
    ($ticks:expr) => {
        assert!($ticks <= 8, "Tick budget exceeded: {} > 8 (Chatman Constant violation)", $ticks);
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
/// ```rust
/// use chicago_tdd_tools::assert_in_range;
///
/// let value = 5;
/// assert_in_range!(value, 0, 10);
///
/// // With custom message
/// let value2 = 5;
/// assert_in_range!(value2, 0, 10, "Value should be in valid range");
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
/// ```rust,should_panic
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
            panic!("{}: expected {:?}, got {:?}", $msg, expected_val, actual_val);
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
/// Validates guard constraints like `max_run_len` ≤ 8.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_guard_constraint;
///
/// let max_run_len = 5;
/// assert_guard_constraint!(max_run_len <= 8, "max_run_len");
/// ```
#[macro_export]
macro_rules! assert_guard_constraint {
    ($condition:expr, $constraint_name:expr) => {
        assert!($condition, "Guard constraint violation: {}", $constraint_name);
    };
}

#[cfg(test)]
#[allow(unnameable_test_items)] // Macro-generated tests trigger this warning
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use crate::test;

    test!(test_assert_ok_macro, {
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

    test!(test_assert_err_macro, {
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

    test!(test_assert_fail_macro, {
        // Arrange: Function that returns error
        fn fallible_function() -> Result<u32, String> {
            Err("error".to_string())
        }

        // Act & Assert: Verify assert_fail! macro works and returns error
        let error = assert_fail!(fallible_function());
        assert_eq!(error, "error");

        // With custom message
        let error2 = assert_fail!(fallible_function(), "Operation should fail");
        assert_eq!(error2, "error");
    });

    #[test]
    #[should_panic(expected = "Expected function to fail")]
    fn test_assert_fail_macro_fails() {
        // Arrange: Function that succeeds
        fn successful_function() -> Result<u32, String> {
            Ok(42)
        }

        // Act & Assert: Should panic
        let _ = assert_fail!(successful_function());
    }

    test!(test_assert_within_tick_budget_macro, {
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

    test!(test_assert_in_range_macro, {
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

    test!(test_assert_eq_msg_macro, {
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

    test!(test_assert_guard_constraint_macro, {
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
