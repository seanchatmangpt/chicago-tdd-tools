//! Generator Examples
//!
//! Demonstrates test code generation with compile-time arrays.

use chicago_tdd_tools::testing::generator::*;
use chicago_tdd_tools::prelude::*;

/// Example: Test generator
pub fn example_test_generator() {
    // Arrange: Create generator
    let mut generator = TestGenerator::new();

    // Act: Generate test code
    let test_code = generator.generate_test("test_name", "test spec");

    // Assert: Verify test code generated
    assert!(test_code.contains("test_name"));
    assert!(test_code.contains("test spec"));
    assert!(test_code.contains("#[test]"));
}

/// Example: Compile-time array generation
pub fn example_compile_time_array() {
    // Arrange: Generate array at compile time
    const TEST_DATA: [u8; 10] = generate_test_array::<10>();

    // Act-Assert: Verify array generated
    assert_eq!(TEST_DATA.len(), 10);
    assert_eq!(TEST_DATA[0], 0);
    assert_eq!(TEST_DATA[1], 1);
}

/// Example: Array pattern generation
pub fn example_array_pattern() {
    // Arrange: Generate array with pattern
    const PATTERN_DATA: [u8; 8] = generate_test_array_pattern::<8>(42);

    // Act-Assert: Verify pattern array generated
    assert_eq!(PATTERN_DATA.len(), 8);
    assert_eq!(PATTERN_DATA[0], 42);
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_test_generator, {
        // Arrange-Act-Assert: Run example
        example_test_generator();
    });

    test!(test_compile_time_array, {
        // Arrange-Act-Assert: Run example
        example_compile_time_array();
    });

    test!(test_array_pattern, {
        // Arrange-Act-Assert: Run example
        example_array_pattern();
    });
}

