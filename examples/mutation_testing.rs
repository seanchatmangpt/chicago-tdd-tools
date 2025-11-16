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

/// Example: Mutation testing with MutationTester
///
/// ## How-to: Use Mutation Testing
///
/// This example demonstrates mutation testing workflow:
/// 1. Create mutation tester with test data
/// 2. Apply mutations using mutation operators
/// 3. Test if mutations are caught by tests
/// 4. Calculate mutation score to measure test quality
///
/// ## Reference
///
/// - **MutationTester**: `MutationTester::new(data) -> MutationTester<T>`
/// - **Apply Mutation**: `apply_mutation(operator)` - Apply mutation operator
/// - **Test Detection**: `test_mutation_detection(test_fn)` - Test if mutation is caught
/// - **Mutation Score**: `MutationScore::calculate(caught, total) -> MutationScore`
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::mutation::*;
/// use std::collections::HashMap;
///
/// let mut data = HashMap::new();
/// data.insert("key1".to_string(), "value1".to_string());
/// let mut tester = MutationTester::new(data);
/// tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
/// let caught = tester.test_mutation_detection(|data| !data.is_empty());
/// let score = MutationScore::calculate(if caught { 1 } else { 0 }, 1);
/// ```
#[tokio::main]
async fn main() {
    chicago_tdd_tools::alert_info!("Mutation Testing Example");
    chicago_tdd_tools::alert_info!("========================");

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
    chicago_tdd_tools::alert_info!(
        "Mutation detection: {}",
        if caught { "CAUGHT" } else { "MISSED" }
    );

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
    chicago_tdd_tools::alert_info!("Mutation score: {}%", score.score());
    if score.is_acceptable() {
        chicago_tdd_tools::alert_success!("Mutation score is acceptable (>= 80%)");
    } else {
        chicago_tdd_tools::alert_warning!("Mutation score is too low (< 80%)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::test;

    test!(test_mutation_tester_creation, {
        // Arrange
        let data = HashMap::new();

        // Act
        let tester = MutationTester::new(data);

        // Assert: Tester created successfully
        assert!(true); // If we get here, creation succeeded
    });

    test!(test_mutation_score_calculation, {
        // Arrange
        let caught = 8;
        let total = 10;

        // Act
        let score = MutationScore::calculate(caught, total);

        // Assert
        assert_eq!(score.score(), 80);
        assert!(score.is_acceptable());
    });
}
