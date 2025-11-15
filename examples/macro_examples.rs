//! # Macro Examples for Chicago TDD Tools - Comprehensive Guide
//!
//! Demonstrates usage of macros provided by chicago-tdd-tools for test writing.
//! Macros expand to test functions and enforce AAA (Arrange-Act-Assert) pattern.
//!
//! ## Tutorial: Getting Started
//!
//! This example demonstrates macro usage patterns:
//!
//! 1. **Basic Test**: Use `test!` macro for synchronous tests with AAA pattern
//! 2. **Result Handling**: Use `assert_ok!` and `assert_err!` for Result types
//! 3. **Custom Messages**: Add custom messages to assertions
//! 4. **Async Tests**: Use `async_test!` for async tests (see async examples)
//!
//! **Note**: These macros expand to test functions, so they're typically used
//! in test files rather than examples. This file shows the macro syntax and usage patterns.
//!
//! ## Explanation: Concepts
//!
//! **Macro Expansion**: Macros expand at compile time to standard Rust test functions.
//! They provide syntactic sugar for common testing patterns while maintaining zero-cost abstractions.
//!
//! **AAA Pattern**: All macros enforce Arrange-Act-Assert structure:
//! - **Arrange**: Set up test data and fixtures
//! - **Act**: Execute the code under test
//! - **Assert**: Verify expected behavior
//!
//! **Assertion Macros**: Specialized macros for common assertion patterns:
//! - `assert_ok!`: Assert `Result` is `Ok`
//! - `assert_err!`: Assert `Result` is `Err`
//! - `assert_within_tick_budget!`: Validate tick budget (≤8 ticks)
//! - `assert_in_range!`: Assert value is in range
//!
//! **Test Macros**: Macros for different test types:
//! - `test!`: Synchronous test
//! - `async_test!`: Async test
//! - `fixture_test!`: Test with automatic fixture setup
//! - `performance_test!`: Performance test with tick validation
//!
//! ## How-to: Common Tasks
//!
//! - Write a basic test: See `test_basic_aaa_pattern`
//! - Handle Result types: See `test_result_handling` and `test_error_handling`
//! - Add custom messages: See `test_with_custom_message`
//!
//! ## Reference: Quick Lookup
//!
//! **Test Macros**:
//! - `test!(name, { body })` - Synchronous test with AAA pattern
//! - `async_test!(name, { body })` - Async test with AAA pattern
//! - `fixture_test!(name, { body })` - Test with automatic fixture setup
//! - `performance_test!(name, { body })` - Performance test with tick validation
//!
//! **Assertion Macros**:
//! - `assert_ok!(result, message?)` - Assert Result is Ok
//! - `assert_err!(result, message?)` - Assert Result is Err
//! - `assert_within_tick_budget!(ticks, max)` - Validate tick budget
//! - `assert_in_range!(value, min, max)` - Assert value is in range
//! - `assert_eq_msg!(left, right, message)` - Assert equality with message
//! - `assert_guard_constraint!(guard, constraint)` - Validate guard constraints
//!
//! **Usage**: Macros are automatically exported from crate root:
//! ```rust
//! use chicago_tdd_tools::*;
//! ```

// Example macro usage patterns (these would be in test files):

#[cfg(test)]
mod macro_examples {
    use chicago_tdd_tools::test;
    use chicago_tdd_tools::{assert_err, assert_ok};

    /// Example: Basic synchronous test with AAA pattern
    ///
    /// ## How-to: Write a Basic Test
    ///
    /// Use `test!` macro to create a synchronous test with AAA pattern.
    /// The macro expands to a standard `#[test]` function with AAA structure.
    ///
    /// ## Reference
    ///
    /// - **Macro**: `test!(name, { body })`
    /// - **Pattern**: AAA (Arrange-Act-Assert)
    /// - **Expansion**: Expands to `#[test] fn name() { body }`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chicago_tdd_tools::test;
    ///
    /// test!(my_test, {
    ///     // Arrange
    ///     let input = 5;
    ///     // Act
    ///     let result = input * 2;
    ///     // Assert
    ///     assert_eq!(result, 10);
    /// });
    /// ```
    test!(test_basic_aaa_pattern, {
        // Arrange: Set up test data
        let input = 5;
        let expected = 10;

        // Act: Execute feature under test
        let result = input * 2;

        // Assert: Verify behavior
        assert_eq!(result, expected);
    });

    /// Example: Test with Result handling
    ///
    /// ## How-to: Handle Result Types in Tests
    ///
    /// Use `assert_ok!` macro to assert that a `Result` is `Ok`. This provides
    /// better error messages than manual `match` or `unwrap()`.
    ///
    /// ## Reference
    ///
    /// - **Macro**: `assert_ok!(result, message?)`
    /// - **Parameters**:
    ///   - `result`: `Result<T, E>` to check
    ///   - `message`: Optional custom error message
    /// - **Behavior**: Panics if result is `Err`, otherwise continues
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chicago_tdd_tools::{test, assert_ok};
    ///
    /// test!(my_test, {
    ///     let result: Result<u32, String> = Ok(42);
    ///     assert_ok!(&result, "Result should be Ok");
    ///     if let Ok(value) = result {
    ///         assert_eq!(value, 42);
    ///     }
    /// });
    /// ```
    test!(test_result_handling, {
        // Arrange: Create a Result
        let result: Result<u32, String> = Ok(42);

        // Act & Assert: Verify Result is Ok and check value
        assert_ok!(&result, "Result should be Ok");
        if let Ok(value) = result {
            assert_eq!(value, 42, "Value should be 42");
        }
    });

    /// Example: Test with error Result
    ///
    /// ## How-to: Test Error Cases
    ///
    /// Use `assert_err!` macro to assert that a `Result` is `Err`. This is useful
    /// for testing error paths and error handling.
    ///
    /// ## Reference
    ///
    /// - **Macro**: `assert_err!(result, message?)`
    /// - **Parameters**:
    ///   - `result`: `Result<T, E>` to check
    ///   - `message`: Optional custom error message
    /// - **Behavior**: Panics if result is `Ok`, otherwise continues
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chicago_tdd_tools::{test, assert_err};
    ///
    /// test!(my_test, {
    ///     let result: Result<u32, String> = Err("error".to_string());
    ///     assert_err!(&result, "Result should be Err");
    /// });
    /// ```
    test!(test_error_handling, {
        // Arrange: Create an error Result
        let result: Result<u32, String> = Err("test error".to_string());

        // Act & Assert: Verify Result is Err
        assert_err!(&result, "Result should be Err");
    });

    /// Example: Test with custom assertion messages
    ///
    /// ## How-to: Add Custom Messages to Assertions
    ///
    /// Use custom messages in assertions to provide context when tests fail.
    /// Standard Rust assertions support custom messages as the last parameter.
    ///
    /// ## Reference
    ///
    /// - **Pattern**: `assert_eq!(left, right, "message")`
    /// - **Macro**: `assert_eq_msg!(left, right, message)` - Alternative with explicit message
    /// - **Usage**: Custom messages help debug test failures
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chicago_tdd_tools::test;
    ///
    /// test!(my_test, {
    ///     let value = 42;
    ///     let expected = 42;
    ///     assert_eq!(value, expected, "Value should equal expected");
    /// });
    /// ```
    test!(test_with_custom_message, {
        // Arrange: Set up test data
        let value = 42;
        let expected = 42;

        // Act & Assert: Verify with custom message
        assert_eq!(value, expected, "Value should equal expected");
    });
}

fn main() {
    println!("Chicago TDD Tools - Macro Examples");
    println!("===================================");
    println!();
    println!("This file demonstrates macro usage patterns.");
    println!("Macros expand to test functions, so they're typically used in test files.");
    println!();
    println!("Available macros:");
    println!("  - test!: Synchronous test with AAA pattern");
    println!("  - async_test!: Async test with AAA pattern");
    println!("  - fixture_test!: Test with automatic fixture setup");
    println!("  - performance_test!: Performance test with tick validation");
    println!("  - assert_ok!: Assert Result is Ok");
    println!("  - assert_err!: Assert Result is Err");
    println!("  - assert_within_tick_budget!: Validate tick budget (≤8 ticks)");
    println!("  - assert_in_range!: Assert value is in range");
    println!("  - assert_eq_msg!: Assert equality with custom message");
    println!("  - assert_guard_constraint!: Validate guard constraints");
    println!();
    println!("See README.md for complete usage examples.");
}
