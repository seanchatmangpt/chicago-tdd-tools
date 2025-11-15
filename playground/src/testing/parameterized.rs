//! Parameterized Testing Examples
//!
//! Demonstrates parameterized testing with rstest and param_test! macro.

#[cfg(feature = "parameterized-testing")]
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "parameterized-testing")]
/// Example: Parameterized test pattern using param_test! macro
pub fn example_parameterized_test() {
    // Arrange: Parameterized tests use param_test! macro with #[case] attributes
    // Act-Assert: Tests run with multiple inputs

    // Example pattern:
    // param_test! {
    //     #[case(1, 2, 3)]
    //     #[case(2, 3, 5)]
    //     fn test_addition(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
    //         assert_eq!(a + b, expected);
    //     }
    // }

    assert!(true);
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "parameterized-testing")]
    use super::*;
    #[cfg(feature = "parameterized-testing")]
    use chicago_tdd_tools::prelude::*;

    #[cfg(feature = "parameterized-testing")]
    test!(test_parameterized_example, {
        // Arrange-Act-Assert: Run example
        example_parameterized_test();
    });

    #[cfg(feature = "parameterized-testing")]
    /// Example: Parameterized test using param_test! macro
    param_test! {
        #[case(1, 2, 3)]
        #[case(2, 3, 5)]
        #[case(10, 20, 30)]
        fn test_addition(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
            // Arrange: Parameters provided by param_test!
            // Act: Execute operation
            let result = a + b;
            // Assert: Verify result
            assert_eq!(result, expected);
        }
    }

    #[cfg(feature = "parameterized-testing")]
    /// Example: Parameterized test with string inputs
    param_test! {
        #[case("hello", "world", "hello world")]
        #[case("foo", "bar", "foo bar")]
        fn test_string_concat(#[case] a: &str, #[case] b: &str, #[case] expected: &str) {
            // Arrange: Parameters provided by param_test!
            // Act: Execute operation
            let result = format!("{} {}", a, b);
            // Assert: Verify result
            assert_eq!(result, expected);
        }
    }
}
