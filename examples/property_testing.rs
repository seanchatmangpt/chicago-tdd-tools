//! Property-Based Testing Example
//!
//! Demonstrates property-based testing with Chicago TDD tools.
//! Shows both the original PropertyTestGenerator and the enhanced ProptestStrategy.

#[cfg(feature = "property-testing")]
use chicago_tdd_tools::property::*;

#[cfg(not(feature = "property-testing"))]
use chicago_tdd_tools::property::*;

#[tokio::main]
async fn main() {
    println!("Property-Based Testing Example");
    println!("==============================");

    // Original PropertyTestGenerator (backward compatible)
    println!("\n1. Using PropertyTestGenerator (original):");
    let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(42);
    let property_valid = property_all_data_valid(&mut generator, 100);
    println!("Property 'all_data_valid': {}", if property_valid { "PASSED" } else { "FAILED" });

    let data = generator.generate_test_data();
    println!("Generated {} items", data.len());

    #[cfg(feature = "property-testing")]
    {
        println!("\n2. Using ProptestStrategy (enhanced with proptest):");

        // Enhanced property testing with proptest
        let strategy = ProptestStrategy::new().with_cases(100);

        println!("Testing addition commutativity...");
        strategy.test(proptest::prelude::any::<(u32, u32)>(), |(x, y)| x + y == y + x);
        println!("✓ Addition is commutative");

        println!("Testing multiplication distributivity...");
        strategy.test(proptest::prelude::any::<(u32, u32, u32)>(), |(a, b, c)| {
            a * (b + c) == (a * b) + (a * c)
        });
        println!("✓ Multiplication is distributive");
    }

    #[cfg(not(feature = "property-testing"))]
    {
        println!("\n2. ProptestStrategy requires 'property-testing' feature");
        println!("   Enable with: --features property-testing");
    }
}
