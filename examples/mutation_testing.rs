//! # Mutation Testing Example - Comprehensive Guide
//!
//! Demonstrates mutation testing with Chicago TDD tools. Mutation testing validates
//! test quality by introducing mutations (changes) to code and verifying tests catch them.
//!
//! ## Tutorial: Getting Started
//!
//! This example demonstrates mutation testing:
//!
//! 1. **Create Mutation Tester**: Use `MutationTester::new()` to create a tester
//! 2. **Apply Mutations**: Use `apply_mutation()` to introduce mutations
//! 3. **Test Detection**: Use `test_mutation_detection()` to verify tests catch mutations
//! 4. **Calculate Score**: Use `MutationScore::calculate()` to measure test quality
//!
//! ## Explanation: Concepts
//!
//! **Mutation Testing**: Validates test quality by introducing small changes (mutations)
//! to code and verifying that tests fail. If tests don't catch mutations, they may not
//! be testing the right behavior.
//!
//! **Mutation Operators**: Types of mutations applied:
//! - `RemoveKey`: Remove a key from a data structure
//! - `AddKey`: Add a key to a data structure
//! - `ChangeValue`: Change a value in a data structure
//! - `NegateCondition`: Negate a boolean condition
//!
//! **Mutation Score**: Percentage of mutations caught by tests. Higher scores indicate
//! better test quality. Target: >= 80% mutation score.
//!
//! **Mutation Detection**: Tests should fail when mutations are applied. If tests pass
//! after mutation, the mutation wasn't detected (test quality issue).
//!
//! **Test Quality**: Mutation testing reveals:
//! - Tests that don't verify behavior (pass even with mutations)
//! - Dead code (mutations that don't affect behavior)
//! - Missing test coverage (mutations not caught)
//!
//! ## How-to: Common Tasks
//!
//! - Create mutation tester: See `main()` function
//! - Apply mutations: See `apply_mutation()` usage
//! - Test mutation detection: See `test_mutation_detection()` usage
//! - Calculate mutation score: See `MutationScore::calculate()` usage
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `MutationTester<T>`: Mutation tester for type `T`
//! - `MutationOperator`: Type of mutation to apply
//! - `MutationScore`: Mutation score calculation
//!
//! **Key Functions**:
//! - `MutationTester::new(data) -> MutationTester<T>` - Create mutation tester
//! - `MutationTester::apply_mutation(operator)` - Apply mutation
//! - `MutationTester::test_mutation_detection(test)` - Test if mutation is caught
//! - `MutationScore::calculate(caught, total) -> MutationScore` - Calculate score
//! - `MutationScore::score() -> u8` - Get score percentage
//! - `MutationScore::is_acceptable() -> bool` - Check if score >= 80%
//!
//! **Key Concepts**:
//! - **Mutation**: Small change to code or data
//! - **Detection**: Test fails when mutation is applied
//! - **Score**: Percentage of mutations caught
//! - **Acceptable**: Score >= 80%

use chicago_tdd_tools::mutation::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("Mutation Testing Example");
    println!("========================");

    // Arrange: Create data and tester
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    let mut tester = MutationTester::new(data);

    // Apply mutations
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));

    // Act: Test if mutations are caught
    let caught = tester.test_mutation_detection(|data| {
        // Test: Data should have at least one key
        !data.is_empty()
    });

    // Assert: Mutations caught
    println!("Mutation detection: {}", if caught { "CAUGHT" } else { "MISSED" });

    // Test mutation score
    let mut data2 = HashMap::new();
    data2.insert("key1".to_string(), "value1".to_string());
    let mut tester2 = MutationTester::new(data2);

    // Apply mutations
    tester2.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
    tester2.apply_mutation(MutationOperator::AddKey("key2".to_string(), "value2".to_string()));

    // Act: Test mutation detection
    let caught2 = tester2.test_mutation_detection(|data| data.contains_key("key1"));

    // Calculate mutation score
    let total_mutations = 2;
    let caught_mutations = if caught2 { total_mutations } else { 0 };
    let score = MutationScore::calculate(caught_mutations, total_mutations);

    // Assert: Mutation score is acceptable
    println!("Mutation score: {}%", score.score());
    if score.is_acceptable() {
        println!("✓ Mutation score is acceptable (>= 80%)");
    } else {
        println!("✗ Mutation score is too low (< 80%)");
    }
}
