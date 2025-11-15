//! Macro Examples for Chicago TDD Tools
//!
//! Demonstrates usage of macros provided by chicago-tdd-tools.
//!
//! Note: These macros expand to test functions, so they're typically used
//! in test files rather than examples. This file shows the macro syntax
//! and usage patterns.

// Macros are automatically exported from the crate root
// They can be used with: use chicago_tdd_tools::*;

// Example macro usage patterns (these would be in test files):

#[cfg(test)]
mod macro_examples {
    use chicago_tdd_tools::test;
    use chicago_tdd_tools::{assert_err, assert_ok};

    // Example 1: Basic synchronous test with AAA pattern
    test!(test_basic_aaa_pattern, {
        // Arrange: Set up test data
        let input = 5;
        let expected = 10;

        // Act: Execute feature under test
        let result = input * 2;

        // Assert: Verify behavior
        assert_eq!(result, expected);
    });

    // Example 2: Test with Result handling using assert_ok!
    test!(test_result_handling, {
        // Arrange: Create a Result
        let result: Result<u32, String> = Ok(42);

        // Act & Assert: Verify Result is Ok and check value
        assert_ok!(&result, "Result should be Ok");
        if let Ok(value) = result {
            assert_eq!(value, 42, "Value should be 42");
        }
    });

    // Example 3: Test with error Result using assert_err!
    test!(test_error_handling, {
        // Arrange: Create an error Result
        let result: Result<u32, String> = Err("test error".to_string());

        // Act & Assert: Verify Result is Err
        assert_err!(&result, "Result should be Err");
    });

    // Example 4: Test with assertions using custom message
    // Note: For fixture_test! macro usage, see fixture_test! documentation
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
