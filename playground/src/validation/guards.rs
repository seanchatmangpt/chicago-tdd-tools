//! Guards Examples
//!
//! Demonstrates guard constraint enforcement at ingress points, including compile-time validated types.

use chicago_tdd_tools::validation::guards::*;
use chicago_tdd_tools::validation::guards::validated::{ValidatedRun, ValidatedBatch, AssertRunLen};
use chicago_tdd_tools::prelude::*;

/// Example: Basic guard validation
pub fn example_guard_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create validator
    let validator = GuardValidator::new();

    // Act: Validate run length
    validator.validate_run_len(5)?;

    // Assert: Validation passed
    Ok(())
}

/// Example: Guard validation failure
pub fn example_guard_failure() {
    // Arrange: Create validator
    let validator = GuardValidator::new();

    // Act-Assert: Validate run length exceeds limit
    let result = validator.validate_run_len(10);
    assert!(result.is_err());
}

/// Example: Batch size validation
pub fn example_batch_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create validator
    let validator = GuardValidator::new();

    // Act: Validate batch size
    validator.validate_batch_size(500)?;

    // Assert: Validation passed
    Ok(())
}

/// Example: Custom constraints
pub fn example_custom_constraints() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create validator with custom constraints
    let validator = GuardValidator::with_constraints(10, 2000);

    // Act: Validate with custom constraints
    validator.validate_run_len(9)?;
    validator.validate_batch_size(1500)?;

    // Assert: Validation passed
    Ok(())
}

/// Example: ValidatedRun compile-time validation
pub fn example_validated_run() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create validated run (compile-time validated LEN <= MAX_RUN_LEN)
    // Valid - LEN = 5 <= MAX_RUN_LEN (8)
    let run = ValidatedRun::<5>::new(vec![1, 2, 3, 4, 5])?;

    // Act-Assert: Verify run length (using len() method)
    assert_eq!(run.len(), 5);
    Ok(())
}

/// Example: ValidatedBatch compile-time validation
pub fn example_validated_batch() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create validated batch (compile-time validated SIZE <= MAX_BATCH_SIZE)
    // Valid - SIZE = 500 <= MAX_BATCH_SIZE (1000)
    let batch = ValidatedBatch::<500>::new(vec![0u8; 500])?;

    // Act-Assert: Verify batch size (using len() method)
    assert_eq!(batch.len(), 500);
    Ok(())
}

/// Example: Function using ValidatedRun with trait bound
pub fn process_validated_run<const LEN: usize>(run: ValidatedRun<LEN>) -> usize
where
    (): AssertRunLen<LEN>,
{
    // LEN validation happens at compile time through trait bounds
    run.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_guard_basic, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_guard_basic());
    });

    test!(test_guard_failure, {
        // Arrange-Act-Assert: Run example
        example_guard_failure();
    });

    test!(test_batch_validation, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_batch_validation());
    });

    test!(test_custom_constraints, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_custom_constraints());
    });

    test!(test_validated_run, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_validated_run());
    });

    test!(test_validated_batch, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_validated_batch());
    });

    test!(test_process_validated_run, {
        // Arrange: Create validated run
        let run = ValidatedRun::<5>::new(vec![1, 2, 3, 4, 5]).unwrap();
        // Act-Assert: Process run
        assert_eq!(process_validated_run(run), 5);
    });
}
