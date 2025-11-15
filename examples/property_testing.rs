//! # Property-Based Testing Example - Comprehensive Guide
//!
//! Demonstrates property-based testing with Chicago TDD tools. Shows both the original
//! `PropertyTestGenerator` and the enhanced `ProptestStrategy` (requires `property-testing` feature).
//!
//! ## Tutorial: Getting Started
//!
//! This example demonstrates property-based testing:
//!
//! 1. **PropertyTestGenerator**: Original property test generator (backward compatible)
//! 2. **ProptestStrategy**: Enhanced property testing with `proptest` crate (requires `property-testing` feature)
//!
//! Property-based testing generates random test data and verifies properties hold for all inputs.
//!
//! ## Explanation: Concepts
//!
//! **Property-Based Testing**: Instead of writing specific test cases, define properties that
//! should hold for all inputs. The framework generates random test data and verifies properties.
//!
//! **PropertyTestGenerator**: Original property test generator with configurable size and seed.
//! Generates test data and validates properties across multiple test cases.
//!
//! **ProptestStrategy**: Enhanced property testing using the `proptest` crate. Provides more
//! sophisticated test data generation and shrinking (finding minimal failing cases).
//!
//! **Properties**: Mathematical or logical properties that should hold for all inputs:
//! - **Commutativity**: `a + b == b + a`
//! - **Distributivity**: `a * (b + c) == (a * b) + (a * c)`
//! - **Identity**: `a + 0 == a`
//!
//! **Shrinking**: When a property fails, the framework finds a minimal failing case by
//! systematically reducing the input size. This helps identify root causes.
//!
//! ## How-to: Common Tasks
//!
//! - Use PropertyTestGenerator: See `main()` function
//! - Use ProptestStrategy: See `main()` function (requires `property-testing` feature)
//! - Define properties: See property functions in examples
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `PropertyTestGenerator<const SIZE: usize, const DEPTH: usize>`: Property test generator
//! - `ProptestStrategy`: Enhanced property testing strategy (requires `property-testing` feature)
//!
//! **Key Functions**:
//! - `PropertyTestGenerator::new() -> PropertyTestGenerator` - Create generator
//! - `PropertyTestGenerator::with_seed(seed) -> PropertyTestGenerator` - Set random seed
//! - `PropertyTestGenerator::generate_test_data() -> Vec<TestData>` - Generate test data
//! - `ProptestStrategy::new() -> ProptestStrategy` - Create strategy
//! - `ProptestStrategy::with_cases(count) -> ProptestStrategy` - Set number of test cases
//! - `ProptestStrategy::test(strategy, property)` - Test property with strategy
//!
//! **Key Concepts**:
//! - **Property**: Logical property that should hold for all inputs
//! - **Test Cases**: Number of random inputs to test
//! - **Shrinking**: Finding minimal failing cases
//! - **Seed**: Random seed for reproducible tests

use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::property::*;

/// Example: Property-based testing with PropertyTestGenerator and ProptestStrategy
///
/// ## How-to: Use Property-Based Testing
///
/// This example demonstrates two approaches to property-based testing:
/// 1. **PropertyTestGenerator**: Original generator (backward compatible)
/// 2. **ProptestStrategy**: Enhanced with `proptest` crate (requires `property-testing` feature)
///
/// ## Reference
///
/// - **PropertyTestGenerator**: `PropertyTestGenerator::<SIZE, DEPTH>::new().with_seed(seed)`
/// - **ProptestStrategy**: `ProptestStrategy::new().with_cases(count)`
/// - **Test Property**: `strategy.test(input_strategy, property_function)`
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::property::*;
///
/// // PropertyTestGenerator
/// let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(42);
/// let data = generator.generate_test_data();
///
/// // ProptestStrategy (requires property-testing feature)
/// let strategy = ProptestStrategy::new().with_cases(100);
/// strategy.test(proptest::prelude::any::<(u32, u32)>(), |(x, y)| x + y == y + x);
/// ```
#[tokio::main]
async fn main() {
    chicago_tdd_tools::alert_info!("Property-Based Testing Example");
    chicago_tdd_tools::alert_info!("==============================");

    // Original PropertyTestGenerator (backward compatible)
    chicago_tdd_tools::alert_info!("1. Using PropertyTestGenerator (original):");
    let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(42);
    // Note: property_all_data_valid is a placeholder - in real usage, define your property function
    // let property_valid = property_all_data_valid(&mut generator, 100);
    // chicago_tdd_tools::alert_info!("Property 'all_data_valid': {}", if property_valid { "PASSED" } else { "FAILED" });

    let data = generator.generate_test_data();
    chicago_tdd_tools::alert_info!("Generated {} items", data.len());

    #[cfg(feature = "property-testing")]
    {
        chicago_tdd_tools::alert_info!("2. Using ProptestStrategy (enhanced with proptest):");

        // Enhanced property testing with proptest
        let strategy = ProptestStrategy::new().with_cases(100);

        chicago_tdd_tools::alert_info!("Testing addition commutativity...");
        strategy.test(proptest::prelude::any::<(u32, u32)>(), |(x, y)| x + y == y + x);
        chicago_tdd_tools::alert_success!("Addition is commutative");

        chicago_tdd_tools::alert_info!("Testing multiplication distributivity...");
        strategy.test(proptest::prelude::any::<(u32, u32, u32)>(), |(a, b, c)| {
            a * (b + c) == (a * b) + (a * c)
        });
        chicago_tdd_tools::alert_success!("Multiplication is distributive");
    }

    #[cfg(not(feature = "property-testing"))]
    {
        chicago_tdd_tools::alert_warning!(
            "2. ProptestStrategy requires 'property-testing' feature"
        );
        chicago_tdd_tools::alert_info!("   Enable with: --features property-testing");
    }
}
