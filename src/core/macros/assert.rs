//! Assertion Macros for Chicago TDD Testing
//!
//! Provides enhanced assertion macros with better error messages:
//! - Result assertions (`assert_ok`, `assert_err`, `assert_fail`)
//! - Performance assertions (`assert_within_tick_budget`)
//! - Range assertions (`assert_in_range`)
//! - Equality assertions (`assert_eq_msg`, `assert_eq_enhanced`)
//! - Guard constraint assertions (`assert_guard_constraint`)
//! - Collection assertions (`assert_contains`, `assert_not_contains`, `assert_subset`, `assert_superset`) - v1.3.0
//! - JSON assertions (`assert_json_eq`) - v1.3.0
//! - Approximate equality (`assert_approx_eq`) - v1.3.0

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

// ============================================================================
// v1.3.0 NEW ASSERTIONS
// ============================================================================

/// Assert that a collection contains an item
///
/// **New in v1.3.0**: Simplifies common collection testing scenarios.
///
/// Provides better error messages than manual iteration checks.
/// Works with any type that implements `IntoIterator` where items implement `PartialEq + Debug`.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_contains;
///
/// let numbers = vec![1, 2, 3, 4, 5];
/// assert_contains!(numbers, 3);
///
/// // With custom message
/// let users = vec!["alice", "bob", "charlie"];
/// assert_contains!(users, "bob", "User should exist");
/// ```
#[macro_export]
macro_rules! assert_contains {
    ($collection:expr, $item:expr) => {{
        let collection_ref = &$collection;
        let item_ref = &$item;
        let found = collection_ref.into_iter().any(|x| x == item_ref);
        if !found {
            panic!(
                "Collection does not contain item.\n  collection: {:?}\n  missing item: {:?}",
                collection_ref, item_ref
            );
        }
    }};
    ($collection:expr, $item:expr, $msg:expr) => {{
        let collection_ref = &$collection;
        let item_ref = &$item;
        let found = collection_ref.into_iter().any(|x| x == item_ref);
        if !found {
            panic!(
                "{}: Collection does not contain item.\n  collection: {:?}\n  missing item: {:?}",
                $msg, collection_ref, item_ref
            );
        }
    }};
}

/// Assert that a collection does not contain an item
///
/// **New in v1.3.0**: Inverse of `assert_contains!`.
///
/// Provides better error messages for negative collection checks.
/// Works with any type that implements `IntoIterator` where items implement `PartialEq + Debug`.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_not_contains;
///
/// let numbers = vec![1, 2, 3, 4, 5];
/// assert_not_contains!(numbers, 6);
///
/// // With custom message
/// let banned_users = vec!["alice", "bob"];
/// assert_not_contains!(banned_users, "charlie", "User should not be banned");
/// ```
#[macro_export]
macro_rules! assert_not_contains {
    ($collection:expr, $item:expr) => {{
        let collection_ref = &$collection;
        let item_ref = &$item;
        let found = collection_ref.into_iter().any(|x| x == item_ref);
        if found {
            panic!(
                "Collection contains item that should not be present.\n  collection: {:?}\n  unexpected item: {:?}",
                collection_ref, item_ref
            );
        }
    }};
    ($collection:expr, $item:expr, $msg:expr) => {{
        let collection_ref = &$collection;
        let item_ref = &$item;
        let found = collection_ref.into_iter().any(|x| x == item_ref);
        if found {
            panic!(
                "{}: Collection contains item that should not be present.\n  collection: {:?}\n  unexpected item: {:?}",
                $msg, collection_ref, item_ref
            );
        }
    }};
}

/// Assert that one collection is a subset of another
///
/// **New in v1.3.0**: Validates subset relationships between collections.
///
/// Checks that all items in `subset` are present in `superset`.
/// Works with any type that implements `IntoIterator` where items implement `PartialEq + Debug`.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_subset;
///
/// let all_features = vec!["feature_a", "feature_b", "feature_c"];
/// let enabled_features = vec!["feature_a", "feature_c"];
/// assert_subset!(enabled_features, all_features);
///
/// // With custom message
/// let allowed_roles = vec!["admin", "user", "guest"];
/// let user_roles = vec!["user"];
/// assert_subset!(user_roles, allowed_roles, "User roles must be allowed");
/// ```
#[macro_export]
macro_rules! assert_subset {
    ($subset:expr, $superset:expr) => {{
        let subset_ref = &$subset;
        let superset_ref = &$superset;
        let subset_vec: Vec<_> = subset_ref.into_iter().collect();
        let superset_vec: Vec<_> = superset_ref.into_iter().collect();

        let missing: Vec<_> = subset_vec
            .iter()
            .filter(|item| !superset_vec.contains(item))
            .collect();

        if !missing.is_empty() {
            panic!(
                "Subset contains items not in superset.\n  subset: {:?}\n  superset: {:?}\n  missing from superset: {:?}",
                subset_vec, superset_vec, missing
            );
        }
    }};
    ($subset:expr, $superset:expr, $msg:expr) => {{
        let subset_ref = &$subset;
        let superset_ref = &$superset;
        let subset_vec: Vec<_> = subset_ref.into_iter().collect();
        let superset_vec: Vec<_> = superset_ref.into_iter().collect();

        let missing: Vec<_> = subset_vec
            .iter()
            .filter(|item| !superset_vec.contains(item))
            .collect();

        if !missing.is_empty() {
            panic!(
                "{}: Subset contains items not in superset.\n  subset: {:?}\n  superset: {:?}\n  missing from superset: {:?}",
                $msg, subset_vec, superset_vec, missing
            );
        }
    }};
}

/// Assert that one collection is a superset of another
///
/// **New in v1.3.0**: Validates superset relationships between collections.
///
/// Checks that all items in `subset` are present in `superset`.
/// This is the inverse of `assert_subset!` for better readability in some contexts.
/// Works with any type that implements `IntoIterator` where items implement `PartialEq + Debug`.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_superset;
///
/// let all_features = vec!["feature_a", "feature_b", "feature_c"];
/// let enabled_features = vec!["feature_a", "feature_c"];
/// assert_superset!(all_features, enabled_features);
///
/// // With custom message
/// let all_permissions = vec!["read", "write", "execute"];
/// let user_permissions = vec!["read"];
/// assert_superset!(all_permissions, user_permissions, "All permissions must include user permissions");
/// ```
#[macro_export]
macro_rules! assert_superset {
    ($superset:expr, $subset:expr) => {{
        $crate::assert_subset!($subset, $superset);
    }};
    ($superset:expr, $subset:expr, $msg:expr) => {{
        $crate::assert_subset!($subset, $superset, $msg);
    }};
}

/// Assert that two JSON values are semantically equal
///
/// **New in v1.3.0**: Semantic JSON comparison with pretty-printed diffs.
///
/// Compares JSON values semantically:
/// - Ignores key order in objects
/// - Ignores whitespace differences
/// - Provides pretty-printed diff on failure
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_json_eq;
/// use serde_json::json;
///
/// let actual = json!({
///     "name": "Alice",
///     "age": 30
/// });
/// let expected = json!({
///     "age": 30,
///     "name": "Alice"
/// });
/// assert_json_eq!(actual, expected); // Passes despite different key order
///
/// // With custom message
/// let response = json!({"status": "ok"});
/// let expected_response = json!({"status": "ok"});
/// assert_json_eq!(response, expected_response, "API response should match");
/// ```
#[macro_export]
macro_rules! assert_json_eq {
    ($actual:expr, $expected:expr) => {{
        let actual_ref = &$actual;
        let expected_ref = &$expected;
        if actual_ref != expected_ref {
            let actual_pretty = serde_json::to_string_pretty(actual_ref)
                .unwrap_or_else(|_| format!("{:?}", actual_ref));
            let expected_pretty = serde_json::to_string_pretty(expected_ref)
                .unwrap_or_else(|_| format!("{:?}", expected_ref));
            panic!(
                "JSON values are not equal.\n  actual:\n{}\n  expected:\n{}",
                actual_pretty, expected_pretty
            );
        }
    }};
    ($actual:expr, $expected:expr, $msg:expr) => {{
        let actual_ref = &$actual;
        let expected_ref = &$expected;
        if actual_ref != expected_ref {
            let actual_pretty = serde_json::to_string_pretty(actual_ref)
                .unwrap_or_else(|_| format!("{:?}", actual_ref));
            let expected_pretty = serde_json::to_string_pretty(expected_ref)
                .unwrap_or_else(|_| format!("{:?}", expected_ref));
            panic!(
                "{}: JSON values are not equal.\n  actual:\n{}\n  expected:\n{}",
                $msg, actual_pretty, expected_pretty
            );
        }
    }};
}

/// Assert that two floating-point values are approximately equal
///
/// **New in v1.3.0**: Floating-point comparison with configurable tolerance.
///
/// Compares floating-point values within a specified epsilon (tolerance).
/// Works with `f32` and `f64` types.
/// Provides clear failure messages showing the actual difference.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assert_approx_eq;
///
/// let pi = 3.14159265_f64;
/// let approx_pi = 3.14_f64;
/// assert_approx_eq!(pi, approx_pi, 0.01);
///
/// // With custom message
/// let calculated = 2.0_f64 / 3.0_f64;
/// let expected = 0.6667_f64;
/// assert_approx_eq!(calculated, expected, 0.0001, "Division result should be close");
/// ```
#[macro_export]
macro_rules! assert_approx_eq {
    ($actual:expr, $expected:expr, $epsilon:expr) => {{
        #[allow(clippy::float_cmp)] // Intentional approximate comparison
        {
            let actual_val = $actual as f64;
            let expected_val = $expected as f64;
            let epsilon_val = $epsilon as f64;
            let diff = (actual_val - expected_val).abs();
            if diff > epsilon_val {
                panic!(
                    "Values not approximately equal.\n  actual: {}\n  expected: {}\n  epsilon: {}\n  difference: {}",
                    actual_val, expected_val, epsilon_val, diff
                );
            }
        }
    }};
    ($actual:expr, $expected:expr, $epsilon:expr, $msg:expr) => {{
        #[allow(clippy::float_cmp)] // Intentional approximate comparison
        {
            let actual_val = $actual as f64;
            let expected_val = $expected as f64;
            let epsilon_val = $epsilon as f64;
            let diff = (actual_val - expected_val).abs();
            if diff > epsilon_val {
                panic!(
                    "{}: Values not approximately equal.\n  actual: {}\n  expected: {}\n  epsilon: {}\n  difference: {}",
                    $msg, actual_val, expected_val, epsilon_val, diff
                );
            }
        }
    }};
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

    // ========================================================================
    // v1.3.0 NEW ASSERTION TESTS
    // ========================================================================

    test!(test_assert_contains_macro, {
        // Arrange: Collection with items
        let numbers = vec![1, 2, 3, 4, 5];
        let users = vec!["alice", "bob", "charlie"];

        // Act & Assert: Verify assert_contains! macro works
        assert_contains!(numbers, 3);
        assert_contains!(users, "bob");
        assert_contains!(users, "bob", "User should exist");
    });

    #[test]
    #[should_panic(expected = "Collection does not contain item")]
    fn test_assert_contains_macro_fails() {
        // Arrange: Collection without target item
        let numbers = vec![1, 2, 3];

        // Act & Assert: Should panic
        assert_contains!(numbers, 5);
    }

    test!(test_assert_not_contains_macro, {
        // Arrange: Collection without certain items
        let numbers = vec![1, 2, 3, 4, 5];
        let banned_users = vec!["alice", "bob"];

        // Act & Assert: Verify assert_not_contains! macro works
        assert_not_contains!(numbers, 6);
        assert_not_contains!(banned_users, "charlie");
        assert_not_contains!(banned_users, "charlie", "User should not be banned");
    });

    #[test]
    #[should_panic(expected = "Collection contains item that should not be present")]
    fn test_assert_not_contains_macro_fails() {
        // Arrange: Collection with item
        let numbers = vec![1, 2, 3];

        // Act & Assert: Should panic
        assert_not_contains!(numbers, 2);
    }

    test!(test_assert_subset_macro, {
        // Arrange: Subset and superset collections
        let all_features = vec!["feature_a", "feature_b", "feature_c"];
        let enabled_features = vec!["feature_a", "feature_c"];
        let all_roles = vec!["admin", "user", "guest"];
        let user_roles = vec!["user"];

        // Act & Assert: Verify assert_subset! macro works
        assert_subset!(enabled_features, all_features);
        assert_subset!(user_roles, all_roles);
        assert_subset!(user_roles, all_roles, "User roles must be allowed");
    });

    #[test]
    #[should_panic(expected = "Subset contains items not in superset")]
    fn test_assert_subset_macro_fails() {
        // Arrange: Invalid subset (contains items not in superset)
        let superset = vec![1, 2, 3];
        let subset = vec![2, 3, 4]; // 4 is not in superset

        // Act & Assert: Should panic
        assert_subset!(subset, superset);
    }

    test!(test_assert_superset_macro, {
        // Arrange: Superset and subset collections
        let all_features = vec!["feature_a", "feature_b", "feature_c"];
        let enabled_features = vec!["feature_a", "feature_c"];
        let all_permissions = vec!["read", "write", "execute"];
        let user_permissions = vec!["read"];

        // Act & Assert: Verify assert_superset! macro works
        assert_superset!(all_features, enabled_features);
        assert_superset!(all_permissions, user_permissions);
        assert_superset!(
            all_permissions,
            user_permissions,
            "All permissions must include user permissions"
        );
    });

    #[test]
    #[should_panic(expected = "Subset contains items not in superset")]
    fn test_assert_superset_macro_fails() {
        // Arrange: Invalid relationship (subset has items not in superset)
        let superset = vec![1, 2, 3];
        let subset = vec![2, 3, 4]; // 4 is not in superset

        // Act & Assert: Should panic
        assert_superset!(superset, subset);
    }

    test!(test_assert_json_eq_macro, {
        use serde_json::json;

        // Arrange: JSON values with different key orders
        let actual = json!({
            "name": "Alice",
            "age": 30
        });
        let expected = json!({
            "age": 30,
            "name": "Alice"
        });
        let response = json!({"status": "ok"});
        let expected_response = json!({"status": "ok"});

        // Act & Assert: Verify assert_json_eq! macro works (ignores key order)
        assert_json_eq!(actual, expected);
        assert_json_eq!(response, expected_response);
        assert_json_eq!(response, expected_response, "API response should match");
    });

    #[test]
    #[should_panic(expected = "JSON values are not equal")]
    fn test_assert_json_eq_macro_fails() {
        use serde_json::json;

        // Arrange: Different JSON values
        let actual = json!({"name": "Alice"});
        let expected = json!({"name": "Bob"});

        // Act & Assert: Should panic
        assert_json_eq!(actual, expected);
    }

    test!(test_assert_approx_eq_macro, {
        // Arrange: Floating-point values
        let pi = 3.14159265;
        let approx_pi = 3.14;
        let calculated = 2.0 / 3.0;
        let expected = 0.6667;

        // Act & Assert: Verify assert_approx_eq! macro works
        assert_approx_eq!(pi, approx_pi, 0.01);
        assert_approx_eq!(calculated, expected, 0.0001);
        assert_approx_eq!(calculated, expected, 0.0001, "Division result should be close");
    });

    #[test]
    #[should_panic(expected = "Values not approximately equal")]
    fn test_assert_approx_eq_macro_fails() {
        // Arrange: Values not within epsilon
        let actual = 3.14159;
        let expected = 2.71828;

        // Act & Assert: Should panic
        assert_approx_eq!(actual, expected, 0.01);
    }
}
