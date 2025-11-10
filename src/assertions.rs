//! Assertion Helpers
//!
//! Provides assertion utilities following Chicago TDD principles.
//! Uses Higher-Ranked Trait Bounds (HRTB) for flexible predicate functions.

/// Assert that a result is successful
pub fn assert_success<T, E: std::fmt::Debug>(result: &Result<T, E>) {
    assert!(
        result.is_ok(),
        "Expected success, but got error: {:?}",
        result.as_ref().err()
    );
}

/// Assert that a result is an error
pub fn assert_error<T: std::fmt::Debug, E>(result: &Result<T, E>) {
    assert!(
        result.is_err(),
        "Expected error, but got success: {:?}",
        result.as_ref().ok()
    );
}

/// Assert that two values are equal with a custom message
pub fn assert_eq_with_msg<T: std::fmt::Debug + PartialEq>(actual: &T, expected: &T, msg: &str) {
    assert_eq!(
        actual, expected,
        "{}: expected {:?}, got {:?}",
        msg, expected, actual
    );
}

/// Assert that a value is within a range
pub fn assert_in_range<T: PartialOrd + std::fmt::Debug>(value: &T, min: &T, max: &T, msg: &str) {
    assert!(
        value >= min && value <= max,
        "{}: value {:?} not in range [{:?}, {:?}]",
        msg,
        value,
        min,
        max
    );
}

/// Assert that a value satisfies a predicate using Higher-Ranked Trait Bounds (HRTB)
///
/// HRTB allows the predicate to work with any lifetime, making it more flexible
/// than a regular `Fn(&T) -> bool` bound.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::assertions::assert_that;
///
/// let value = 42;
/// assert_that(&value, |v| *v > 0);
///
/// // Works with references of any lifetime
/// let vec = vec![1, 2, 3];
/// assert_that(&vec, |v| v.len() == 3);
/// ```
pub fn assert_that<T, F>(value: &T, predicate: F)
where
    T: std::fmt::Debug,
    F: for<'a> Fn(&'a T) -> bool,
{
    assert!(predicate(value), "Assertion failed for value: {:?}", value);
}

/// Assert that a value satisfies a predicate with a custom message
pub fn assert_that_with_msg<T, F>(value: &T, predicate: F, msg: &str)
where
    T: std::fmt::Debug,
    F: for<'a> Fn(&'a T) -> bool,
{
    assert!(
        predicate(value),
        "{}: Assertion failed for value: {:?}",
        msg,
        value
    );
}
