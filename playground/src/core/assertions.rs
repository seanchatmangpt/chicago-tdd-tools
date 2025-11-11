//! Assertions Examples
//!
//! Demonstrates comprehensive assertion utilities, including advanced assertion types.

use chicago_tdd_tools::core::assertions::*;
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::{AssertionBuilder, ValidatedAssertion};

/// Example: Result assertions
pub fn example_result_assertions() {
    // Arrange: Create results
    let ok_result: Result<u32, String> = Ok(42);
    let err_result: Result<u32, String> = Err("error".to_string());

    // Act-Assert: Verify Result assertions
    assert_success(&ok_result);
    assert_error(&err_result);
}

/// Example: Predicate assertions
pub fn example_predicate_assertions() {
    // Arrange: Create value
    let value = 42;

    // Act-Assert: Verify predicate assertions
    assert_that(&value, |v| *v > 0);
    assert_that_with_msg(&value, |v| *v < 100, "Value should be less than 100");
}

/// Example: Range assertions
pub fn example_range_assertions() {
    // Arrange: Create value
    let value = 42;

    // Act-Assert: Verify range assertions
    assert_in_range(&value, &0, &100, "Value should be in range");
    assert_eq_with_msg(&value, &42, "Value should equal 42");
}

/// Example: Assertion macros (macros are available via prelude)
pub fn example_assertion_macros() {
    // Arrange: Create results
    let ok_result: Result<u32, String> = Ok(42);
    let err_result: Result<u32, String> = Err("error".to_string());

    // Act-Assert: Verify assertion macros
    assert_ok!(&ok_result);
    assert_err!(&err_result);
    assert_in_range!(42, 0, 100, "Value in range");
    assert_eq_msg!(42, 42, "Values equal");
}

/// Example: AssertionBuilder for fluent assertion API
pub fn example_assertion_builder() {
    // Arrange: Create value
    let value = 42;

    // Act-Assert: Use fluent assertion builder
    let assertion = AssertionBuilder::new(value)
        .assert_that(|v| *v > 0)
        .assert_that(|v| *v < 100);

    // Assert: Verify assertion built
    assert_eq!(assertion.into_value(), 42);
}

#[cfg(feature = "otel")]
/// Example: ValidatedAssertion with type-level validation
pub fn example_validated_assertion() {
    // Arrange: Create value
    let value = 42;

    // Act-Assert: Use validated assertion with OTEL
    let assertion = ValidatedAssertion::new(value, "test.assertion")
        .assert_that(|v| *v > 0)
        .assert_that(|v| *v < 100);

    // Assert: Verify validated assertion works
    assert_eq!(assertion.into_value(), 42);
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_result_assertions, {
        // Arrange-Act-Assert: Run example
        example_result_assertions();
    });

    test!(test_predicate_assertions, {
        // Arrange-Act-Assert: Run example
        example_predicate_assertions();
    });

    test!(test_range_assertions, {
        // Arrange-Act-Assert: Run example
        example_range_assertions();
    });

    test!(test_assertion_macros, {
        // Arrange-Act-Assert: Run example
        example_assertion_macros();
    });

    test!(test_assertion_builder, {
        // Arrange-Act-Assert: Run example
        example_assertion_builder();
    });

    #[cfg(feature = "otel")]
    test!(test_validated_assertion, {
        // Arrange-Act-Assert: Run example
        example_validated_assertion();
    });
}

