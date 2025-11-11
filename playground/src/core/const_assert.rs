//! Const Assert Examples
//!
//! Demonstrates compile-time assertions.

use chicago_tdd_tools::prelude::*;

/// Example: Const assertions
pub fn example_const_assertions() {
    // Compile-time assertions are macros, not functions
    // They are evaluated at compile time
    // Example usage would be in const contexts:
    // const _: () = assert!(1 + 1 == 2);
    
    // For runtime examples, we demonstrate the concept
    assert!(1 + 1 == 2);
    assert!(2 + 2 == 4);
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_const_assertions, {
        // Arrange-Act-Assert: Run example
        example_const_assertions();
    });
}

