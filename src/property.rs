//! Property-Based Testing Framework
//!
//! Provides QuickCheck-style property-based testing for validating invariants.
//! Uses const generics for compile-time test configuration.
//!
//! # Enhanced with proptest
//!
//! When the `property-testing` feature is enabled, this module provides enhanced
//! property-based testing using proptest, which offers better shrinking strategies
//! and more advanced features. The original `PropertyTestGenerator` remains available
//! for backward compatibility.

use std::collections::HashMap;

#[cfg(feature = "property-testing")]
use proptest::prelude::*;
#[cfg(feature = "property-testing")]
use proptest::test_runner::{Config, TestRunner};

/// Property test generator with const generics for compile-time configuration
///
/// `MAX_ITEMS` and `MAX_DEPTH` are validated at compile time, providing
/// zero runtime overhead for configuration.
pub struct PropertyTestGenerator<const MAX_ITEMS: usize = 10, const MAX_DEPTH: usize = 3> {
    /// Random seed for reproducibility
    seed: u64,
}

impl<const MAX_ITEMS: usize, const MAX_DEPTH: usize> PropertyTestGenerator<MAX_ITEMS, MAX_DEPTH> {
    /// Create new property test generator
    ///
    /// MAX_ITEMS and MAX_DEPTH are compile-time constants, ensuring
    /// type-safe configuration.
    pub fn new() -> Self {
        Self { seed: 0 }
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Generate random test data
    ///
    /// Uses compile-time MAX_ITEMS constant for bounds checking.
    pub fn generate_test_data(&mut self) -> HashMap<String, String> {
        let mut rng = SimpleRng::new(self.seed);
        self.seed = self.seed.wrapping_add(1);

        let mut data = HashMap::new();
        // Use compile-time constant MAX_ITEMS
        let num_items = (rng.next() as usize % MAX_ITEMS) + 1;

        for i in 0..num_items {
            let key = format!("key_{i}");
            let value = format!("value_{}", rng.next());
            data.insert(key, value);
        }

        data
    }

    /// Get compile-time MAX_ITEMS constant
    pub const fn max_items() -> usize {
        MAX_ITEMS
    }

    /// Get compile-time MAX_DEPTH constant
    pub const fn max_depth() -> usize {
        MAX_DEPTH
    }
}

impl<const MAX_ITEMS: usize, const MAX_DEPTH: usize> Default
    for PropertyTestGenerator<MAX_ITEMS, MAX_DEPTH>
{
    fn default() -> Self {
        Self::new()
    }
}

/// Simple RNG for property testing (LCG)
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // Linear Congruential Generator
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        self.state
    }
}

/// Property: All generated data is valid
pub fn property_all_data_valid<const MAX_ITEMS: usize, const MAX_DEPTH: usize>(
    generator: &mut PropertyTestGenerator<MAX_ITEMS, MAX_DEPTH>,
    num_tests: usize,
) -> bool {
    for _ in 0..num_tests {
        let data = generator.generate_test_data();
        if data.is_empty() {
            return false;
        }
    }
    true
}

// ============================================================================
// Enhanced Property Testing with proptest
// ============================================================================

#[cfg(feature = "property-testing")]
/// Enhanced property test strategy using proptest
///
/// Provides advanced property-based testing with better shrinking strategies
/// and more sophisticated test case generation. This is an enhanced alternative
/// to `PropertyTestGenerator` that uses proptest internally.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::property::ProptestStrategy;
///
/// ProptestStrategy::new()
///     .with_cases(1000)
///     .test(|x: u32| {
///         // Property: x * 2 is always even
///         (x * 2) % 2 == 0
///     });
/// ```
pub struct ProptestStrategy {
    config: Config,
}

#[cfg(feature = "property-testing")]
impl ProptestStrategy {
    /// Create a new proptest strategy with default configuration
    pub fn new() -> Self {
        Self { config: Config::default() }
    }

    /// Set the number of test cases to run
    pub fn with_cases(mut self, cases: u32) -> Self {
        self.config.cases = cases;
        self
    }

    /// Set the maximum number of shrink attempts
    pub fn with_max_shrink_iters(mut self, iters: u32) -> Self {
        self.config.max_shrink_iters = iters;
        self
    }

    /// Set the random seed for reproducibility
    ///
    /// Note: Seed configuration is complex in proptest. For now, use default seeding.
    /// Future versions may support custom seed configuration.
    #[allow(dead_code)] // Reserved for future use
    pub fn with_seed(self, _seed: [u8; 32]) -> Self {
        // Proptest seed configuration is complex - using default for now
        // Future: implement proper seed configuration
        self
    }

    /// Run a property test with a strategy
    ///
    /// # Arguments
    ///
    /// * `strategy` - A proptest strategy for generating test values
    /// * `property` - A function that takes a value and returns true if the property holds
    ///
    /// # Panics
    ///
    /// Panics if the property fails for any generated test case.
    pub fn test<S, F>(&self, strategy: S, property: F)
    where
        S: Strategy,
        S::Value: std::fmt::Debug,
        F: Fn(S::Value) -> bool,
    {
        let mut runner = TestRunner::new(self.config.clone());
        runner
            .run(&strategy, |value| {
                prop_assert!(property(value));
                Ok(())
            })
            .unwrap_or_else(|e| panic!("Property test failed: {:?}", e));
    }

    /// Run a property test with a default strategy for a type
    ///
    /// This is a convenience method that uses the default strategy for the type.
    ///
    /// # Arguments
    ///
    /// * `property` - A function that takes a value and returns true if the property holds
    ///
    /// # Panics
    ///
    /// Panics if the property fails for any generated test case.
    pub fn test_default<T, F>(&self, property: F)
    where
        T: Arbitrary + std::fmt::Debug,
        F: Fn(T) -> bool,
    {
        self.test(any::<T>(), property);
    }
}

#[cfg(feature = "property-testing")]
impl Default for ProptestStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "property-testing")]
#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod proptest_tests {
    use super::*;

    #[test]
    fn test_proptest_strategy_addition_commutative() {
        let strategy = ProptestStrategy::new().with_cases(100);
        strategy.test(any::<(u32, u32)>(), |(x, y)| x + y == y + x);
    }

    #[test]
    fn test_proptest_strategy_multiplication_distributive() {
        let strategy = ProptestStrategy::new().with_cases(100);
        strategy.test(any::<(u32, u32, u32)>(), |(a, b, c)| a * (b + c) == (a * b) + (a * c));
    }

    #[test]
    fn test_proptest_strategy_string_length() {
        let strategy = ProptestStrategy::new().with_cases(100);
        strategy.test(any::<String>(), |s| {
            s.len() == s.chars().count() || s.len() >= s.chars().count()
        });
    }
}
