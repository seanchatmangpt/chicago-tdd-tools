//! Compile-Time Validated Guard Types
//!
//! Provides compile-time validated types for guard constraints using const generics.
//! Use `ValidatedRun<const LEN: usize>` and `ValidatedBatch<const SIZE: usize>` when
//! the values are known at compile time.
//!
//! # Poka-Yoke: Type-Level Validation
//!
//! These types enforce MAX_RUN_LEN ≤ 8 and MAX_BATCH_SIZE ≤ 1000 at compile time
//! through trait bounds. Invalid values fail to compile.
//!
//! ## Examples
//!
//! ### ValidatedRun
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use chicago_tdd_tools::guards::validated::{ValidatedRun, AssertRunLen};
//!
//! // Valid - LEN = 5 <= MAX_RUN_LEN (8) - compiles successfully
//! let run = ValidatedRun::<5>::new(vec![1, 2, 3, 4, 5])?;
//! assert_eq!(run.len(), 5);
//!
//! // Invalid - LEN = 9 > MAX_RUN_LEN (8) - compile error!
//! // let run = ValidatedRun::<9>::new(vec![0; 9])?; // Compile error: AssertRunLen<9> not implemented
//! # Ok(())
//! # }
//! ```
//!
//! ### ValidatedBatch
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use chicago_tdd_tools::guards::validated::{ValidatedBatch, AssertBatchSize};
//!
//! // Valid - SIZE = 500 <= MAX_BATCH_SIZE (1000)
//! let batch = ValidatedBatch::<500>::new(vec![0; 500])?;
//! assert_eq!(batch.len(), 500);
//!
//! // Invalid - SIZE = 1500 > MAX_BATCH_SIZE (1000) - compile error!
//! // let batch = ValidatedBatch::<1500>::new(vec![0; 1500])?; // Compile error!
//! # Ok(())
//! # }
//! ```

use crate::core::const_assert::Validated;
use crate::validation::guards::GuardConstraintError;

// Re-export constants from guards module
pub use super::{MAX_BATCH_SIZE, MAX_RUN_LEN};

// ============================================================================
// Poka-Yoke: Compile-Time Validated Types
// ============================================================================

/// Compile-time validated run length
///
/// **Poka-Yoke**: This type enforces MAX_RUN_LEN ≤ 8 at compile time using const generics.
/// Use this for known run lengths to prevent errors at compile time.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use chicago_tdd_tools::guards::validated::{ValidatedRun, AssertRunLen};
///
/// // Compile-time validated - LEN must be <= MAX_RUN_LEN
/// fn process_run<const LEN: usize>(run: ValidatedRun<LEN>)
/// where
///     (): AssertRunLen<LEN>,
/// {
///     // LEN is guaranteed to be <= MAX_RUN_LEN at compile time
///     let len = run.len();
///     assert_eq!(len, LEN);
///     // Process run...
/// }
///
/// // Valid - LEN = 5 <= MAX_RUN_LEN (8)
/// let run = ValidatedRun::<5>::new(vec![1, 2, 3, 4, 5])?;
/// assert_eq!(run.len(), 5);
/// process_run(run);
///
/// // Invalid - LEN = 9 > MAX_RUN_LEN (8) - compile error!
/// // let run = ValidatedRun::<9>::new(vec![0; 9])?; // Compile error!
/// # Ok(())
/// # }
/// ```
pub struct ValidatedRun<const LEN: usize> {
    /// Validated run data
    inner: Validated<Vec<u8>>,
}

/// Helper trait for compile-time run length validation
///
/// This trait is only implemented when LEN <= MAX_RUN_LEN.
/// **Poka-Yoke**: Use this trait bound to enforce compile-time validation.
pub trait AssertRunLen<const LEN: usize> {}

/// Type-level marker for valid run lengths
pub trait Valid {}

/// Implementation of Valid for unit type
impl Valid for () {}

/// Manual implementations for valid run lengths (0-8)
/// **Poka-Yoke**: Only valid run lengths (<= MAX_RUN_LEN) are implemented.
impl AssertRunLen<0> for () {}
impl AssertRunLen<1> for () {}
impl AssertRunLen<2> for () {}
impl AssertRunLen<3> for () {}
impl AssertRunLen<4> for () {}
impl AssertRunLen<5> for () {}
impl AssertRunLen<6> for () {}
impl AssertRunLen<7> for () {}
impl AssertRunLen<8> for () {}

impl<const LEN: usize> ValidatedRun<LEN>
where
    (): AssertRunLen<LEN>,
{
    /// Create a new validated run
    ///
    /// This constructor validates that the data length matches the const generic LEN.
    /// Returns an error if the length doesn't match.
    ///
    /// # Errors
    ///
    /// Returns `GuardConstraintError::InvalidConstraintValue` if the data length
    /// doesn't match the const generic LEN.
    pub fn new(data: Vec<u8>) -> Result<Self, GuardConstraintError> {
        if data.len() != LEN {
            return Err(GuardConstraintError::InvalidConstraintValue(format!(
                "Data length {} doesn't match const generic LEN {}",
                data.len(),
                LEN
            )));
        }
        Ok(Self { inner: Validated::new(data) })
    }

    /// Get the run length
    ///
    /// This is guaranteed to be LEN at compile time.
    pub const fn len(&self) -> usize {
        LEN
    }

    /// Get a reference to the run data
    pub fn data(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Consume the validated run and return the data
    pub fn into_data(self) -> Vec<u8> {
        self.inner.into_inner()
    }
}

/// Compile-time validated batch size
///
/// **Poka-Yoke**: This type enforces MAX_BATCH_SIZE ≤ 1000 at compile time using const generics.
/// Use this for known batch sizes to prevent errors at compile time.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use chicago_tdd_tools::guards::validated::{ValidatedBatch, AssertBatchSize};
///
/// // Compile-time validated - SIZE must be <= MAX_BATCH_SIZE
/// fn process_batch<const SIZE: usize>(batch: ValidatedBatch<SIZE>)
/// where
///     (): AssertBatchSize<SIZE>,
/// {
///     // SIZE is guaranteed to be <= MAX_BATCH_SIZE at compile time
///     let size = batch.len();
///     assert_eq!(size, SIZE);
///     // Process batch...
/// }
///
/// // Valid - SIZE = 500 <= MAX_BATCH_SIZE (1000)
/// let batch = ValidatedBatch::<500>::new(vec![0; 500])?;
/// assert_eq!(batch.len(), 500);
/// process_batch(batch);
///
/// // Invalid - SIZE = 1500 > MAX_BATCH_SIZE (1000) - compile error!
/// // let batch = ValidatedBatch::<1500>::new(vec![0; 1500])?; // Compile error!
/// # Ok(())
/// # }
/// ```
pub struct ValidatedBatch<const SIZE: usize> {
    /// Validated batch data
    inner: Validated<Vec<u8>>,
}

/// Helper trait for compile-time batch size validation
///
/// This trait is only implemented when SIZE <= MAX_BATCH_SIZE.
/// **Poka-Yoke**: Use this trait bound to enforce compile-time validation.
pub trait AssertBatchSize<const SIZE: usize> {}

/// Manual implementations for valid batch sizes (0-1000, in increments of 100)
/// **Poka-Yoke**: Only valid batch sizes (<= MAX_BATCH_SIZE) are implemented.
/// Note: For practical use, implement specific sizes as needed
impl AssertBatchSize<0> for () {}
impl AssertBatchSize<100> for () {}
impl AssertBatchSize<200> for () {}
impl AssertBatchSize<300> for () {}
impl AssertBatchSize<400> for () {}
impl AssertBatchSize<500> for () {}
impl AssertBatchSize<600> for () {}
impl AssertBatchSize<700> for () {}
impl AssertBatchSize<800> for () {}
impl AssertBatchSize<900> for () {}
impl AssertBatchSize<1000> for () {}

impl<const SIZE: usize> ValidatedBatch<SIZE>
where
    (): AssertBatchSize<SIZE>,
{
    /// Create a new validated batch
    ///
    /// This constructor validates that the data length matches the const generic SIZE.
    /// Returns an error if the length doesn't match.
    ///
    /// # Errors
    ///
    /// Returns `GuardConstraintError::InvalidConstraintValue` if the data length
    /// doesn't match the const generic SIZE.
    pub fn new(data: Vec<u8>) -> Result<Self, GuardConstraintError> {
        if data.len() != SIZE {
            return Err(GuardConstraintError::InvalidConstraintValue(format!(
                "Data length {} doesn't match const generic SIZE {}",
                data.len(),
                SIZE
            )));
        }
        Ok(Self { inner: Validated::new(data) })
    }

    /// Get the batch size
    ///
    /// This is guaranteed to be SIZE at compile time.
    pub const fn len(&self) -> usize {
        SIZE
    }

    /// Get a reference to the batch data
    pub fn data(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// Consume the validated batch and return the data
    pub fn into_data(self) -> Vec<u8> {
        self.inner.into_inner()
    }
}

#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::*;
    use crate::validation::guards::GuardConstraintError;

    // Poka-Yoke: Compile-time validated types tests
    #[test]
    fn test_validated_run_valid() {
        // Valid - LEN = 5 <= MAX_RUN_LEN (8)
        #[allow(clippy::expect_used)] // Test code - expected to succeed
        let run = ValidatedRun::<5>::new(vec![1, 2, 3, 4, 5]).expect("Should create validated run");
        assert_eq!(run.len(), 5);
        assert_eq!(run.data(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_validated_run_invalid_length() {
        // Invalid - data length doesn't match const generic LEN
        let result = ValidatedRun::<5>::new(vec![1, 2, 3]); // Length 3, not 5
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_batch_valid() {
        // Valid - SIZE = 500 <= MAX_BATCH_SIZE (1000)
        #[allow(clippy::expect_used)] // Test code - expected to succeed
        let batch =
            ValidatedBatch::<500>::new(vec![0; 500]).expect("Should create validated batch");
        assert_eq!(batch.len(), 500);
        assert_eq!(batch.data().len(), 500);
    }

    #[test]
    fn test_validated_batch_invalid_length() {
        // Invalid - data length doesn't match const generic SIZE
        let result = ValidatedBatch::<500>::new(vec![0; 300]); // Length 300, not 500
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_run_invalid_constraint_value() {
        // Test InvalidConstraintValue error
        let result = ValidatedRun::<5>::new(vec![1, 2, 3]); // Length 3, not 5
        assert!(result.is_err());
        match result {
            Err(GuardConstraintError::InvalidConstraintValue(msg)) => {
                assert!(msg.contains("Data length"));
                assert!(msg.contains("doesn't match"));
            }
            _ => panic!("Expected InvalidConstraintValue error"),
        }
    }

    #[test]
    fn test_validated_batch_invalid_constraint_value() {
        // Test InvalidConstraintValue error
        let result = ValidatedBatch::<500>::new(vec![0; 300]); // Length 300, not 500
        assert!(result.is_err());
        match result {
            Err(GuardConstraintError::InvalidConstraintValue(msg)) => {
                assert!(msg.contains("Data length"));
                assert!(msg.contains("doesn't match"));
            }
            _ => panic!("Expected InvalidConstraintValue error"),
        }
    }

    #[test]
    fn test_validated_run_all_valid_lengths() {
        // Test all valid run lengths (0-8) compile and work
        // This test verifies that all valid lengths work correctly
        //
        // Note: ValidatedRun::<9> should fail to compile (compile-fail test)
        // To verify this, try to compile:
        //   let _run = ValidatedRun::<9>::new(vec![0; 9]);
        // This should fail with: "trait bound `(): AssertRunLen<9>` is not satisfied"

        // Test each length separately (each ValidatedRun<LEN> is a different type)
        let data0 = vec![0u8; 0];
        assert!(ValidatedRun::<0>::new(data0).is_ok());
        assert_eq!(ValidatedRun::<0>::new(vec![0u8; 0]).unwrap().len(), 0);

        let data1 = vec![0u8; 1];
        assert!(ValidatedRun::<1>::new(data1).is_ok());
        assert_eq!(ValidatedRun::<1>::new(vec![0u8; 1]).unwrap().len(), 1);

        let data2 = vec![0u8; 2];
        assert!(ValidatedRun::<2>::new(data2).is_ok());
        assert_eq!(ValidatedRun::<2>::new(vec![0u8; 2]).unwrap().len(), 2);

        let data3 = vec![0u8; 3];
        assert!(ValidatedRun::<3>::new(data3).is_ok());
        assert_eq!(ValidatedRun::<3>::new(vec![0u8; 3]).unwrap().len(), 3);

        let data4 = vec![0u8; 4];
        assert!(ValidatedRun::<4>::new(data4).is_ok());
        assert_eq!(ValidatedRun::<4>::new(vec![0u8; 4]).unwrap().len(), 4);

        let data5 = vec![0u8; 5];
        assert!(ValidatedRun::<5>::new(data5).is_ok());
        assert_eq!(ValidatedRun::<5>::new(vec![0u8; 5]).unwrap().len(), 5);

        let data6 = vec![0u8; 6];
        assert!(ValidatedRun::<6>::new(data6).is_ok());
        assert_eq!(ValidatedRun::<6>::new(vec![0u8; 6]).unwrap().len(), 6);

        let data7 = vec![0u8; 7];
        assert!(ValidatedRun::<7>::new(data7).is_ok());
        assert_eq!(ValidatedRun::<7>::new(vec![0u8; 7]).unwrap().len(), 7);

        let data8 = vec![0u8; 8];
        assert!(ValidatedRun::<8>::new(data8).is_ok());
        assert_eq!(ValidatedRun::<8>::new(vec![0u8; 8]).unwrap().len(), 8);
    }

    #[test]
    fn test_validated_batch_all_valid_sizes() {
        // Test all valid batch sizes compile and work
        // Note: ValidatedBatch::<1500> should fail to compile (compile-fail test)
        // This test verifies that all valid sizes work correctly

        // Test each size separately (each ValidatedBatch<SIZE> is a different type)
        assert!(ValidatedBatch::<0>::new(vec![0u8; 0]).is_ok());
        assert_eq!(ValidatedBatch::<0>::new(vec![0u8; 0]).unwrap().len(), 0);

        assert!(ValidatedBatch::<100>::new(vec![0u8; 100]).is_ok());
        assert_eq!(ValidatedBatch::<100>::new(vec![0u8; 100]).unwrap().len(), 100);

        assert!(ValidatedBatch::<500>::new(vec![0u8; 500]).is_ok());
        assert_eq!(ValidatedBatch::<500>::new(vec![0u8; 500]).unwrap().len(), 500);

        assert!(ValidatedBatch::<1000>::new(vec![0u8; 1000]).is_ok());
        assert_eq!(ValidatedBatch::<1000>::new(vec![0u8; 1000]).unwrap().len(), 1000);
    }
}
