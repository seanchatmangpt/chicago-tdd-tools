//! Mutation Testing Example
//!
//! Demonstrates mutation testing with Chicago TDD tools.

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
