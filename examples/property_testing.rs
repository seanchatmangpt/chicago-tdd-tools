//! Property-Based Testing Example
//!
//! Demonstrates property-based testing with Chicago TDD tools.

use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::property::*;

#[tokio::main]
async fn main() {
    println!("Property-Based Testing Example");
    println!("==============================");

    // Arrange: Create generator with const generics (MAX_ITEMS=10, MAX_DEPTH=3)
    let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(42);

    // Act & Assert: Test property
    let property_valid = property_all_data_valid(&mut generator, 100);
    println!(
        "Property 'all_data_valid': {}",
        if property_valid { "PASSED" } else { "FAILED" }
    );

    // Act: Generate data
    let data = generator.generate_test_data();
    println!("Generated {} items", data.len());

    // Assert: Data generated
    if !data.is_empty() {
        println!("✓ Generator creates data successfully");
    } else {
        println!("✗ Generator failed to create data");
    }
}
