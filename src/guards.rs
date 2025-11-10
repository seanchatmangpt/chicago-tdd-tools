//! Guard Constraint Enforcement
//!
//! Provides guard constraint validation at ingress points (input boundaries).
//! Enforces MAX_RUN_LEN ≤ 8 (Chatman Constant) and MAX_BATCH_SIZE constraints.

use thiserror::Error;

/// Guard constraint error
#[derive(Error, Debug)]
pub enum GuardConstraintError {
    /// Max run length exceeded
    #[error("Run length {} exceeds maximum {} (Chatman Constant violation)", .0, .1)]
    MaxRunLengthExceeded(usize, usize),
    /// Max batch size exceeded
    #[error("Batch size {} exceeds maximum {}", .0, .1)]
    MaxBatchSizeExceeded(usize, usize),
    /// Invalid constraint value
    #[error("Invalid constraint value: {0}")]
    InvalidConstraintValue(String),
}

/// Result type for guard constraint validation
pub type GuardConstraintResult<T> = Result<T, GuardConstraintError>;

/// Maximum run length (Chatman Constant: ≤8)
pub const MAX_RUN_LEN: usize = 8;

/// Maximum batch size
pub const MAX_BATCH_SIZE: usize = 1000;

/// Guard constraint validator
pub struct GuardValidator {
    max_run_len: usize,
    max_batch_size: usize,
}

impl Default for GuardValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl GuardValidator {
    /// Create a new guard validator with default constraints
    pub fn new() -> Self {
        Self {
            max_run_len: MAX_RUN_LEN,
            max_batch_size: MAX_BATCH_SIZE,
        }
    }

    /// Create a guard validator with custom constraints
    pub fn with_constraints(max_run_len: usize, max_batch_size: usize) -> Self {
        Self {
            max_run_len,
            max_batch_size,
        }
    }

    /// Validate run length at ingress
    ///
    /// This should be called at input boundaries before execution paths.
    /// Execution paths (hot path, executor, state) assume pre-validated inputs.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use chicago_tdd_tools::guards::GuardValidator;
    ///
    /// let validator = GuardValidator::new();
    /// validator.validate_run_len(5)?; // OK
    /// validator.validate_run_len(9)?; // Error: exceeds MAX_RUN_LEN
    /// ```
    pub fn validate_run_len(&self, len: usize) -> GuardConstraintResult<()> {
        if len > self.max_run_len {
            return Err(GuardConstraintError::MaxRunLengthExceeded(
                len,
                self.max_run_len,
            ));
        }
        Ok(())
    }

    /// Validate batch size at ingress
    ///
    /// This should be called at input boundaries before execution paths.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use chicago_tdd_tools::guards::GuardValidator;
    ///
    /// let validator = GuardValidator::new();
    /// validator.validate_batch_size(500)?; // OK
    /// validator.validate_batch_size(1500)?; // Error: exceeds MAX_BATCH_SIZE
    /// ```
    pub fn validate_batch_size(&self, size: usize) -> GuardConstraintResult<()> {
        if size > self.max_batch_size {
            return Err(GuardConstraintError::MaxBatchSizeExceeded(
                size,
                self.max_batch_size,
            ));
        }
        Ok(())
    }

    /// Validate run length for a slice/array
    ///
    /// Convenience method for validating collections.
    pub fn validate_run<T>(&self, items: &[T]) -> GuardConstraintResult<()> {
        self.validate_run_len(items.len())
    }

    /// Validate batch for a slice/array
    ///
    /// Convenience method for validating collections.
    pub fn validate_batch<T>(&self, items: &[T]) -> GuardConstraintResult<()> {
        self.validate_batch_size(items.len())
    }
}

/// Assert guard constraint at ingress (for use in tests)
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::guards::assert_guard_run_len;
///
/// let run = vec![1, 2, 3, 4, 5];
/// assert_guard_run_len(&run); // OK
/// ```
pub fn assert_guard_run_len<T>(items: &[T]) {
    let validator = GuardValidator::new();
    validator.validate_run(items).unwrap_or_else(|e| {
        panic!("Guard constraint violation: {}", e);
    });
}

/// Assert batch size constraint at ingress (for use in tests)
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::guards::assert_guard_batch_size;
///
/// let batch = vec![0; 500];
/// assert_guard_batch_size(&batch); // OK
/// ```
pub fn assert_guard_batch_size<T>(items: &[T]) {
    let validator = GuardValidator::new();
    validator.validate_batch(items).unwrap_or_else(|e| {
        panic!("Guard constraint violation: {}", e);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_run_len_valid() {
        let validator = GuardValidator::new();
        assert!(validator.validate_run_len(0).is_ok());
        assert!(validator.validate_run_len(5).is_ok());
        assert!(validator.validate_run_len(8).is_ok());
    }

    #[test]
    fn test_validate_run_len_exceeds() {
        let validator = GuardValidator::new();
        assert!(validator.validate_run_len(9).is_err());
        assert!(validator.validate_run_len(100).is_err());
    }

    #[test]
    fn test_validate_batch_size_valid() {
        let validator = GuardValidator::new();
        assert!(validator.validate_batch_size(0).is_ok());
        assert!(validator.validate_batch_size(500).is_ok());
        assert!(validator.validate_batch_size(1000).is_ok());
    }

    #[test]
    fn test_validate_batch_size_exceeds() {
        let validator = GuardValidator::new();
        assert!(validator.validate_batch_size(1001).is_err());
        assert!(validator.validate_batch_size(10000).is_err());
    }

    #[test]
    fn test_validate_run() {
        let validator = GuardValidator::new();
        let valid_run = vec![1, 2, 3, 4, 5];
        assert!(validator.validate_run(&valid_run).is_ok());

        let invalid_run = vec![0; 9];
        assert!(validator.validate_run(&invalid_run).is_err());
    }

    #[test]
    fn test_validate_batch() {
        let validator = GuardValidator::new();
        let valid_batch = vec![0; 500];
        assert!(validator.validate_batch(&valid_batch).is_ok());

        let invalid_batch = vec![0; 1001];
        assert!(validator.validate_batch(&invalid_batch).is_err());
    }

    #[test]
    fn test_assert_guard_run_len() {
        let valid_run = vec![1, 2, 3, 4, 5];
        assert_guard_run_len(&valid_run); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Guard constraint violation")]
    fn test_assert_guard_run_len_panics() {
        let invalid_run = vec![0; 9];
        assert_guard_run_len(&invalid_run); // Should panic
    }

    #[test]
    fn test_assert_guard_batch_size() {
        let valid_batch = vec![0; 500];
        assert_guard_batch_size(&valid_batch); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Guard constraint violation")]
    fn test_assert_guard_batch_size_panics() {
        let invalid_batch = vec![0; 1001];
        assert_guard_batch_size(&invalid_batch); // Should panic
    }
}
