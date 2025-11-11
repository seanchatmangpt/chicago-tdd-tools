//! Type Level Examples
//!
//! Demonstrates type-level programming with const generics.

use chicago_tdd_tools::core::type_level::SizeValidatedArray;
use chicago_tdd_tools::prelude::*;

/// Example: Size-validated array
pub fn example_size_validated_array() {
    // Arrange: Create size-validated array using const generics
    const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);

    // Act & Assert: Verify size validation
    assert_eq!(ARRAY.size(), 8);
    assert_eq!(ARRAY.data().len(), 8);
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_size_validated_array, {
        // Arrange-Act-Assert: Run example
        example_size_validated_array();
    });
}

