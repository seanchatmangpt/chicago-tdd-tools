//! Mutation Testing Framework
//!
//! Validates test quality by introducing mutations and checking if tests catch them.

use std::collections::HashMap;

/// Mutation operator
#[derive(Debug, Clone)]
pub enum MutationOperator {
    /// Remove a key
    RemoveKey(String),
    /// Add a key-value pair
    AddKey(String, String),
    /// Change a value
    ChangeValue(String, String),
}

/// Mutation tester
pub struct MutationTester {
    /// Original data
    original: HashMap<String, String>,
    /// Mutations applied
    mutations: Vec<MutationOperator>,
}

impl MutationTester {
    /// Create new mutation tester
    pub fn new(original: HashMap<String, String>) -> Self {
        Self {
            original,
            mutations: vec![],
        }
    }

    /// Apply mutation operator
    pub fn apply_mutation(&mut self, mutation: MutationOperator) -> HashMap<String, String> {
        self.mutations.push(mutation.clone());
        self.mutate_data(&self.original.clone(), mutation)
    }

    fn mutate_data(
        &self,
        data: &HashMap<String, String>,
        mutation: MutationOperator,
    ) -> HashMap<String, String> {
        let mut mutated = data.clone();

        match mutation {
            MutationOperator::RemoveKey(key) => {
                mutated.remove(&key);
            }
            MutationOperator::AddKey(key, value) => {
                mutated.insert(key, value);
            }
            MutationOperator::ChangeValue(key, new_value) => {
                if let Some(v) = mutated.get_mut(&key) {
                    *v = new_value;
                }
            }
        }

        mutated
    }

    /// Test if mutation is caught by tests
    pub fn test_mutation_detection<F>(&mut self, test_fn: F) -> bool
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        // Test original (should pass)
        if !test_fn(&self.original) {
            return false; // Original test fails - invalid test
        }

        // Apply each mutation and test
        let mutations = self.mutations.clone();
        for mutation in mutations {
            let mutated = self.apply_mutation(mutation);
            if test_fn(&mutated) {
                // Mutation not detected - test quality issue
                return false;
            }
        }

        true
    }
}

/// Mutation score (percentage of mutations caught)
pub struct MutationScore {
    /// Total mutations tested
    #[allow(dead_code)] // Used in tests and future analysis
    pub total: usize,
    /// Mutations caught by tests
    #[allow(dead_code)] // Used in tests and future analysis
    pub caught: usize,
    /// Score percentage
    pub score: f64,
}

impl MutationScore {
    /// Calculate mutation score
    pub fn calculate(caught: usize, total: usize) -> Self {
        let score = if total > 0 {
            (caught as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total,
            caught,
            score,
        }
    }

    /// Get score percentage
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Is score acceptable? (>= 80%)
    pub fn is_acceptable(&self) -> bool {
        self.score >= 80.0
    }
}
