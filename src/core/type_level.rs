//! Type-Level Arithmetic and Compile-Time Validation
//!
//! Uses const generics and type-level arithmetic for compile-time size/range validation.
//! Provides zero-cost abstractions for compile-time guarantees.
//!
//! # Advanced Rust Features
//!
//! - **Const Generics**: Compile-time constants in type parameters
//! - **Type-Level Arithmetic**: Compile-time calculations using const generics
//! - **Const Trait Bounds**: Compile-time trait bounds for better guarantees
//!
//! # Note on Validation
//!
//! - **`SizeValidatedArray`**: Provides runtime validation with panic if SIZE > `MAX_SIZE`.
//!   Future: Will use Rust 1.79+ const trait bounds for compile-time validation.
//! - **ValidatedSize/ValidatedRange**: Marker types for documentation only, no actual validation.
//!   For actual compile-time validation, see `validation::guards::validated` module.

/// Type-level size marker using const generics
///
/// **Note**: This is a marker type for documentation purposes only.
/// It does not provide actual compile-time validation. For actual compile-time
/// validation, use `SizeValidatedArray` with runtime checks, or implement
/// trait bounds similar to `ValidatedRun` in `validation::guards::validated`.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::core::type_level::ValidatedSize;
///
/// // Type alias for documentation - no actual validation
/// type SmallArray = ValidatedSize<8>;
/// ```
pub struct ValidatedSize<const SIZE: usize>;

/// Const trait for size validation
///
/// This trait is implemented for sizes that meet certain constraints.
/// Uses const generics (stable) for compile-time validation. `MAX_SIZE` is
/// a trait-level const parameter rather than an associated const so the
/// `IS_VALID` default never references `Self::` — associated-const-equality
/// defaults (`SIZE <= Self::MAX_SIZE`) require an unstable feature gate and
/// do not compile under toolchains without `min_generic_const_args`.
pub trait ConstSizeValid<const SIZE: usize, const MAX_SIZE: usize> {
    /// Validate that SIZE <= `MAX_SIZE`
    const IS_VALID: bool = SIZE <= MAX_SIZE;
}

/// Type-level range marker using const generics
///
/// **Note**: This is a marker type for documentation purposes only.
/// It does not provide actual compile-time validation. For actual compile-time
/// validation, implement trait bounds similar to `ValidatedRun` in `validation::guards::validated`.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::core::type_level::ValidatedRange;
///
/// // Type alias for documentation - no actual validation
/// type SmallValue = ValidatedRange<5, 0, 10>;
/// ```
pub struct ValidatedRange<const VALUE: usize, const MIN: usize, const MAX: usize>;

/// Const trait for range validation
///
/// This trait is implemented for ranges that meet certain constraints.
pub trait ConstRangeValid<const VALUE: usize, const MIN: usize, const MAX: usize> {
    /// Validate that MIN <= VALUE <= MAX
    const IS_VALID: bool = MIN <= VALUE && VALUE <= MAX;
}

/// Type-level arithmetic operations
///
/// Provides compile-time arithmetic operations using const generics.
pub mod arithmetic {

    /// Add two const values at compile time
    ///
    /// # Example
    ///
    /// ```rust
    /// use chicago_tdd_tools::core::type_level::arithmetic::Add;
    ///
    /// type Sum = Add<5, 3>;  // Compile-time: 5 + 3 = 8
    /// ```
    pub struct Add<const A: usize, const B: usize>;

    impl<const A: usize, const B: usize> Add<A, B> {
        /// The compile-time sum `A + B`
        pub const VALUE: usize = A + B;
    }

    /// Subtract two const values at compile time
    ///
    /// # Example
    ///
    /// ```rust
    /// use chicago_tdd_tools::core::type_level::arithmetic::Sub;
    ///
    /// type Diff = Sub<10, 3>;  // Compile-time: 10 - 3 = 7
    /// ```
    pub struct Sub<const A: usize, const B: usize>;

    impl<const A: usize, const B: usize> Sub<A, B> {
        /// The compile-time difference `A - B` (panics at compile time on
        /// underflow)
        pub const VALUE: usize = A - B;
    }

    /// Multiply two const values at compile time
    ///
    /// # Example
    ///
    /// ```rust
    /// use chicago_tdd_tools::core::type_level::arithmetic::Mul;
    ///
    /// type Product = Mul<5, 3>;  // Compile-time: 5 * 3 = 15
    /// ```
    pub struct Mul<const A: usize, const B: usize>;

    impl<const A: usize, const B: usize> Mul<A, B> {
        /// The compile-time product `A * B`
        pub const VALUE: usize = A * B;
    }

    /// Divide two const values at compile time
    ///
    /// # Example
    ///
    /// ```rust
    /// use chicago_tdd_tools::core::type_level::arithmetic::Div;
    ///
    /// type Quotient = Div<15, 3>;  // Compile-time: 15 / 3 = 5
    /// ```
    pub struct Div<const A: usize, const B: usize>;

    impl<const A: usize, const B: usize> Div<A, B> {
        /// The compile-time quotient `A / B` (panics at compile time on
        /// division by zero)
        pub const VALUE: usize = A / B;
    }
}

/// Compile-time size-validated array
///
/// Uses type-level validation to ensure array size meets constraints.
/// **Note**: Actual compile-time validation requires Rust 1.79+ const trait bounds.
/// For now, this provides type-level documentation of constraints.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::core::type_level::SizeValidatedArray;
///
/// // Valid: size 8 <= max 8
/// const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);
/// ```
pub struct SizeValidatedArray<const SIZE: usize, const MAX_SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize, const MAX_SIZE: usize> SizeValidatedArray<SIZE, MAX_SIZE> {
    /// Create a new size-validated array
    ///
    /// # Panics
    ///
    /// Panics at runtime if SIZE > `MAX_SIZE`. This is a temporary limitation
    /// until Rust 1.79+ const trait bounds are available for compile-time validation.
    ///
    /// **Future**: With Rust 1.79+, this will be validated at compile time.
    #[must_use]
    pub const fn new(data: [u8; SIZE]) -> Self {
        // Runtime validation until const trait bounds are stable
        // This ensures safety even if type system doesn't catch it
        // Note: Cannot use formatted panic in const fn, so we use a simple panic
        assert!((SIZE <= MAX_SIZE), "Array size exceeds maximum");
        Self { data }
    }

    /// Get the array data
    #[must_use]
    pub const fn data(&self) -> &[u8; SIZE] {
        &self.data
    }

    /// Get the validated size
    #[must_use]
    #[allow(clippy::unused_self)] // Required for trait consistency - const fn needs self
    pub const fn size(&self) -> usize {
        SIZE
    }
}

#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::arithmetic::{Add, Div, Mul, Sub};
    use super::{ConstRangeValid, ConstSizeValid, SizeValidatedArray, ValidatedRange, ValidatedSize};
    use crate::assert_eq_msg;

    #[test]
    fn test_size_validated_array() {
        // Arrange: Create validated array
        const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);

        // Act & Assert: Verify array created and size is correct
        assert_eq_msg!(&ARRAY.size(), &8, "Array size should be 8");
        assert_eq_msg!(&ARRAY.data().len(), &8, "Array data length should be 8");
    }

    #[test]
    #[should_panic(expected = "Array size exceeds maximum")]
    fn test_size_validated_array_rejects_oversized() {
        // SIZE (16) > MAX_SIZE (8) must panic at construction
        let _ = SizeValidatedArray::<16, 8>::new([0u8; 16]);
    }

    #[test]
    fn test_type_level_arithmetic() {
        // Const assertions: the type-level values must equal hand-computed ints.
        const _: () = assert!(Add::<5, 3>::VALUE == 8);
        const _: () = assert!(Sub::<10, 3>::VALUE == 7);
        const _: () = assert!(Mul::<5, 3>::VALUE == 15);
        const _: () = assert!(Div::<15, 3>::VALUE == 5);

        // Arrays sized by the type-level values: lengths follow the arithmetic.
        let sum: [u8; Add::<5, 3>::VALUE] = [0u8; 8];
        let diff: [u8; Sub::<10, 3>::VALUE] = [0u8; 7];
        let product: [u8; Mul::<5, 3>::VALUE] = [0u8; 15];
        let quotient: [u8; Div::<15, 3>::VALUE] = [0u8; 5];

        assert_eq_msg!(&sum.len(), &8, "Type-level sum should size array to 8");
        assert_eq_msg!(&diff.len(), &7, "Type-level difference should size array to 7");
        assert_eq_msg!(&product.len(), &15, "Type-level product should size array to 15");
        assert_eq_msg!(&quotient.len(), &5, "Type-level quotient should size array to 5");

        // Composition: (5 + 3) * 2 computed entirely at the type level.
        const COMPOSED: usize = Mul::<{ Add::<5, 3>::VALUE }, 2>::VALUE;
        assert_eq_msg!(&COMPOSED, &16, "Composed type-level arithmetic should be 16");
        let composed: [u8; COMPOSED] = [0u8; 16];
        assert_eq_msg!(&composed.len(), &16, "Composed value should size array to 16");
    }

    // --- ConstSizeValid / ConstRangeValid validation behavior ---

    struct TestSizeHolder<const N: usize>;

    impl<const N: usize> ConstSizeValid<N, 8> for TestSizeHolder<N> {}

    #[test]
    fn test_const_size_valid_behaves_as_validation() {
        // Sizes within the bound validate; sizes above it do not.
        assert!(
            <TestSizeHolder<8> as ConstSizeValid<8, 8>>::IS_VALID,
            "size 8 <= MAX_SIZE 8 should be valid"
        );
        assert!(
            <TestSizeHolder<5> as ConstSizeValid<5, 8>>::IS_VALID,
            "size 5 <= MAX_SIZE 8 should be valid"
        );
        assert!(
            !<TestSizeHolder<9> as ConstSizeValid<9, 8>>::IS_VALID,
            "size 9 > MAX_SIZE 8 should be invalid"
        );
    }

    impl<const V: usize, const MIN: usize, const MAX: usize> ConstRangeValid<V, MIN, MAX>
        for ValidatedRange<V, MIN, MAX>
    {
    }

    #[test]
    fn test_const_range_valid_behaves_as_validation() {
        // ConstRangeValid's IS_VALID default: MIN <= VALUE <= MAX.
        type InRange = ValidatedRange<5, 0, 10>;
        type AtMin = ValidatedRange<0, 0, 10>;
        type AtMax = ValidatedRange<10, 0, 10>;
        type BelowMin = ValidatedRange<0, 1, 10>;
        type AboveMax = ValidatedRange<11, 0, 10>;

        assert!(<InRange as ConstRangeValid<5, 0, 10>>::IS_VALID, "5 in [0,10]");
        assert!(<AtMin as ConstRangeValid<0, 0, 10>>::IS_VALID, "min boundary inclusive");
        assert!(<AtMax as ConstRangeValid<10, 0, 10>>::IS_VALID, "max boundary inclusive");
        assert!(!<BelowMin as ConstRangeValid<0, 1, 10>>::IS_VALID, "0 < MIN 1 invalid");
        assert!(!<AboveMax as ConstRangeValid<11, 0, 10>>::IS_VALID, "11 > MAX 10 invalid");
    }

    #[test]
    fn test_validated_markers_pair_with_const_traits() {
        // The marker types compose with the const traits' validation semantics.
        impl<const SIZE: usize> ConstSizeValid<SIZE, 8> for ValidatedSize<SIZE> {}

        assert!(
            <ValidatedSize<8> as ConstSizeValid<8, 8>>::IS_VALID,
            "ValidatedSize<8> within bound 8"
        );
        assert!(
            !<ValidatedSize<9> as ConstSizeValid<9, 8>>::IS_VALID,
            "ValidatedSize<9> exceeds bound 8"
        );
    }
}
