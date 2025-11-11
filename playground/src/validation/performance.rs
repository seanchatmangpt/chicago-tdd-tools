//! Performance Examples
//!
//! Demonstrates RDTSC benchmarking and tick measurement for hot path validation, including type-level validation.

use chicago_tdd_tools::validation::performance::*;
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::ValidatedTickBudget;

/// Example: Basic tick measurement
pub fn example_tick_measurement() {
    // Arrange: Start tick counter
    let counter = TickCounter::start();

    // Act: Perform operation
    let _result: i32 = (0..100).sum();

    // Act: Get elapsed ticks
    let ticks = counter.elapsed_ticks();

    // Assert: Verify ticks measured
    assert!(ticks >= 0);
}

/// Example: Tick budget validation
pub fn example_tick_budget() {
    // Arrange: Measure operation
    let (result, ticks) = measure_ticks(|| {
        // Hot path operation
        1 + 1
    });

    // Act-Assert: Verify within budget (using macro from prelude)
    assert_eq!(result, 2);
    assert_within_tick_budget!(ticks, "Hot path operation");
}

/// Example: Performance validation
pub fn example_performance_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Start counter
    let counter = TickCounter::start();

    // Act: Perform operation
    let _result: i32 = (0..10).sum();

    // Act-Assert: Validate performance
    counter.assert_within_budget(HOT_PATH_TICK_BUDGET)?;

    // Assert: Validation passed
    Ok(())
}

/// Example: ValidatedTickBudget type-level validation
pub fn example_validated_tick_budget() {
    // Arrange: Measure operation
    let (result, ticks) = measure_ticks(|| {
        // Hot path operation
        1 + 1
    });

    // Act: Create validated tick budget (compile-time validated BUDGET <= HOT_PATH_TICK_BUDGET)
    // Valid - BUDGET = 8 <= HOT_PATH_TICK_BUDGET (8)
    let budget = ValidatedTickBudget::<8>::new();

    // Assert: Verify validated budget works
    assert_eq!(result, 2);
    assert_eq!(budget.budget(), 8);
    // Use budget to validate ticks
    let counter = TickCounter::start();
    let _ = measure_ticks(|| 1 + 1);
    assert!(budget.assert_within_budget(&counter).is_ok());
}

/// Example: Function using ValidatedTickBudget
pub fn process_with_budget<const BUDGET: u64>(ticks: u64) -> u64 {
    // BUDGET validation happens at compile time through const generics
    let budget = ValidatedTickBudget::<BUDGET>::new();
    // Return the budget value (not ticks, since budget is compile-time constant)
    budget.budget()
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_tick_measurement, {
        // Arrange-Act-Assert: Run example
        example_tick_measurement();
    });

    test!(test_tick_budget, {
        // Arrange-Act-Assert: Run example
        example_tick_budget();
    });

    test!(test_performance_validation, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_performance_validation());
    });

    test!(test_validated_tick_budget, {
        // Arrange-Act-Assert: Run example
        example_validated_tick_budget();
    });

    test!(test_process_with_budget, {
        // Arrange: Create validated budget
        // Act-Assert: Process with validated budget
        let result = process_with_budget::<8>(100);
        assert_eq!(result, 8); // Budget value, not ticks
    });
}

