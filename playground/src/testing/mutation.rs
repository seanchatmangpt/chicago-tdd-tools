//! Mutation Testing Examples
//!
//! Demonstrates mutation testing for test quality validation.

use chicago_tdd_tools::testing::mutation::*;
use chicago_tdd_tools::prelude::*;
use std::collections::HashMap;

/// Example: Basic mutation testing
pub fn example_mutation_basic() {
    // Arrange: Create data and tester
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    let mut tester = MutationTester::new(data);

    // Act: Apply mutation
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));

    // Assert: Test mutation detection
    let caught = tester.test_mutation_detection(|data| !data.is_empty());
    assert!(caught); // Mutation should be caught
}

/// Example: Mutation score calculation
pub fn example_mutation_score() {
    // Arrange: Create data
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    let mut tester = MutationTester::new(data);

    // Act: Apply multiple mutations
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
    tester.apply_mutation(MutationOperator::AddKey("key2".to_string(), "value2".to_string()));

    // Act: Test mutation detection
    let caught = tester.test_mutation_detection(|data| data.contains_key("key1"));

    // Act: Calculate score
    let total = 2;
    let caught_count = if caught { total } else { 0 };
    let score = MutationScore::calculate(caught_count, total);

    // Assert: Score is acceptable (>= 80%)
    assert!(score.is_acceptable());
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_mutation_basic, {
        // Arrange-Act-Assert: Run example
        example_mutation_basic();
    });

    test!(test_mutation_score, {
        // Arrange-Act-Assert: Run example
        example_mutation_score();
    });
}

