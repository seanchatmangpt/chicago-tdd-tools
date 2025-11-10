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

/*
// Example 1: Basic synchronous test with AAA pattern
chicago_test!(test_basic_aaa_pattern, {
    // Arrange: Set up test data
    let input = 5;
    let expected = 10;

    // Act: Execute feature under test
    let result = input * 2;

    // Assert: Verify behavior
    assert_eq!(result, expected);
});

// Example 2: Async test with AAA pattern
chicago_async_test!(test_async_operation, {
    // Arrange: Set up test data
    let input = "test";

    // Act: Execute async operation
    let result = async_operation(input).await;

    // Assert: Verify behavior
    assert_eq!(result, "test_processed");
});

// Example 3: Test with automatic fixture setup/teardown
chicago_fixture_test!(test_with_fixture, fixture, {
    // Arrange: Use provided fixture
    let counter = fixture.test_counter();
    fixture.set_metadata("test_key".to_string(), "test_value".to_string());

    // Act: Execute test operation
    let metadata = fixture.get_metadata("test_key");

    // Assert: Verify behavior
    assert_eq!(metadata, Some(&"test_value".to_string()));
    assert!(counter >= 0);
});

// Example 4: Performance test with tick budget validation
chicago_performance_test!(test_hot_path_performance, {
    // Arrange: Set up test data
    let input = create_test_input();

    // Act: Execute hot path operation and measure ticks
    let (result, ticks) = measure_ticks(|| hot_path_operation(&input));

    // Assert: Verify performance constraint
    assert_within_tick_budget!(ticks, "Hot path operation");
    assert_ok!(result, "Operation should succeed");
});
*/

fn main() {
    println!("Chicago TDD Tools - Macro Examples");
    println!("===================================");
    println!();
    println!("This file demonstrates macro usage patterns.");
    println!("Macros expand to test functions, so they're typically used in test files.");
    println!();
    println!("Available macros:");
    println!("  - chicago_test!: Synchronous test with AAA pattern");
    println!("  - chicago_async_test!: Async test with AAA pattern");
    println!("  - chicago_fixture_test!: Test with automatic fixture setup");
    println!("  - chicago_performance_test!: Performance test with tick validation");
    println!("  - assert_ok!: Assert Result is Ok");
    println!("  - assert_err!: Assert Result is Err");
    println!("  - assert_within_tick_budget!: Validate tick budget (≤8 ticks)");
    println!("  - assert_in_range!: Assert value is in range");
    println!("  - assert_eq_msg!: Assert equality with custom message");
    println!("  - assert_guard_constraint!: Validate guard constraints");
    println!();
    println!("See README.md for complete usage examples.");
}
