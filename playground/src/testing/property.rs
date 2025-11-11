//! Property Testing Examples
//!
//! Demonstrates property-based testing with const generics and reproducible seeds.

#[cfg(feature = "property-testing")]
use chicago_tdd_tools::testing::property::*;
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "property-testing")]
/// Example: Property test generator
pub fn example_property_generator() {
    // Arrange: Create generator with seed
    let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(42);

    // Act: Generate test data
    let data = generator.generate_test_data();

    // Assert: Verify data generated
    assert!(!data.is_empty());
    assert!(data.len() <= 10); // MAX_ITEMS constraint
}

#[cfg(feature = "property-testing")]
/// Example: Property validation
pub fn example_property_validation() {
    // Arrange: Create generator
    let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(42);

    // Act: Validate property
    let valid = property_all_data_valid(&mut generator, 100);

    // Assert: Property holds
    assert!(valid);
}

#[cfg(feature = "property-testing")]
/// Example: Proptest strategy
pub fn example_proptest_strategy() {
    // Arrange: Create strategy
    let strategy = ProptestStrategy::new().with_cases(100);

    // Act-Assert: Test addition commutativity
    strategy.test(proptest::prelude::any::<(u32, u32)>(), |(x, y)| x + y == y + x);
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "property-testing")]
    use super::*;

    #[cfg(feature = "property-testing")]
    test!(test_property_generator, {
        // Arrange-Act-Assert: Run example
        example_property_generator();
    });

    #[cfg(feature = "property-testing")]
    test!(test_property_validation, {
        // Arrange-Act-Assert: Run example
        example_property_validation();
    });

    #[cfg(feature = "property-testing")]
    test!(test_proptest_strategy, {
        // Arrange-Act-Assert: Run example
        example_proptest_strategy();
    });
}

