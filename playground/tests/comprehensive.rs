//! Comprehensive Test Suite
//!
//! Validates all features work correctly in the playground.

use chicago_tdd_tools::prelude::*;

// Import all feature modules
mod core_tests;
mod integration_tests;
mod observability_tests;
mod testing_tests;
mod validation_tests;

// Re-export test modules
pub use core_tests::*;
pub use integration_tests::*;
pub use observability_tests::*;
pub use testing_tests::*;
pub use validation_tests::*;

/// Run all feature validation tests
pub fn run_all_tests() {
    println!("Running comprehensive feature validation tests...");

    // Core features
    core_tests::run_core_tests();

    // Testing features
    testing_tests::run_testing_tests();

    // Validation features
    validation_tests::run_validation_tests();

    // Observability features
    observability_tests::run_observability_tests();

    // Integration features
    integration_tests::run_integration_tests();

    println!("✓ All feature validation tests completed");
}
