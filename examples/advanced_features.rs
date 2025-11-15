//! # Advanced Rust Features Examples - Comprehensive Guide
//!
//! Demonstrates hyper-advanced Rust features used in Chicago TDD Tools:
//! async traits, type-level arithmetic, const generics, and type state patterns.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through advanced Rust features:
//!
//! 1. **Type-Level Arithmetic**: Use const generics for compile-time size validation
//! 2. **Type State Pattern**: Enforce test phase ordering at compile time
//! 3. **Async Traits**: Use async traits for async fixture management (requires `async` feature)
//!
//! These features maximize developer experience (DX) by providing compile-time guarantees
//! and zero-cost abstractions.
//!
//! ## Explanation: Concepts
//!
//! **Const Generics**: Allow generic parameters to be constant values known at compile time.
//! Enables type-level arithmetic and size validation without runtime overhead.
//!
//! **Type State Pattern**: Use Rust's type system to encode state machines. The compiler
//! enforces valid state transitions, preventing invalid operations at compile time.
//!
//! **Async Traits**: Rust 1.75+ allows async methods in traits. Enables async fixture
//! management with the same ergonomics as sync code.
//!
//! **Sealed Traits**: Prevent external implementations, ensuring only intended types
//! implement a trait. Provides API stability and prevents misuse.
//!
//! **Zero-Cost Abstractions**: Advanced features compile to the same code as manual
//! implementations, with no runtime overhead. You get safety and ergonomics for free.
//!
//! **Compile-Time Guarantees**: Type system catches errors before code runs. Invalid
//! state transitions, wrong method calls, and type mismatches are compile errors, not runtime bugs.
//!
//! ## How-to: Common Tasks
//!
//! - Use type-level arithmetic: See `example_type_level_arithmetic()`
//! - Use type state pattern: See `example_type_state_pattern()`
//! - Use async traits: See `src/core/async_fixture.rs` tests (requires `async` feature)
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `SizeValidatedArray<const SIZE: usize, const VALIDATED_SIZE: usize>`: Size-validated array with const generics
//! - `TestState<Phase>`: Type state pattern for test phases (Arrange, Act, Assert)
//! - `AsyncFixtureProvider`: Async trait for fixture management (requires `async` feature)
//!
//! **Key Functions**:
//! - `SizeValidatedArray::new(data) -> SizeValidatedArray` - Create size-validated array
//! - `TestState::<Arrange>::new() -> TestState<Arrange>` - Create arrange state
//! - `TestState::act() -> TestState<Act>` - Transition to act phase
//! - `TestState::assert() -> TestState<Assert>` - Transition to assert phase
//!
//! **Key Concepts**:
//! - **Const Generics**: Generic parameters that are constant values
//! - **Type State**: Encode state machines in types
//! - **Zero-Cost**: No runtime overhead for abstractions
//! - **Compile-Time**: Errors caught before code runs

#[cfg(feature = "async")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use chicago_tdd_tools::core::async_fixture::{AsyncFixtureManager, AsyncFixtureProvider};
#[cfg(feature = "async")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use chicago_tdd_tools::core::fixture::FixtureError;
use chicago_tdd_tools::core::state::{Arrange, TestState};
use chicago_tdd_tools::core::type_level::SizeValidatedArray;

// ============================================================================
// Example 1: Async Traits (Rust 1.75+)
// ============================================================================

#[cfg(feature = "async")]
#[allow(dead_code)] // Example code - struct shown for demonstration
struct DatabaseFixture {
    connection: String,
}

#[cfg(feature = "async")]
#[allow(dead_code)] // Example code - struct shown for demonstration
struct DatabaseProvider;

// Note: Sealed trait pattern prevents external implementations
// For examples, we can't implement the sealed trait, so we'll skip the async fixture example
// See src/core/async_fixture.rs tests for working examples

// ============================================================================
// Example 2: Type-Level Arithmetic and Const Generics
// ============================================================================

/// Example: Type-level arithmetic with const generics
///
/// ## How-to: Use Const Generics for Size Validation
///
/// Use `SizeValidatedArray` with const generics to create arrays with compile-time
/// size validation. The type system ensures size constraints are met.
///
/// **Note**: Current implementation does runtime validation, but const generics
/// enable future compile-time validation.
///
/// ## Reference
///
/// - **Type**: `SizeValidatedArray<const SIZE: usize, const VALIDATED_SIZE: usize>`
/// - **Function**: `SizeValidatedArray::new(data) -> SizeValidatedArray`
/// - **Methods**:
///   - `size() -> usize` - Get array size
///   - `data() -> &[T]` - Get array data
/// - **Const Generics**: `SIZE` and `VALIDATED_SIZE` are compile-time constants
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::core::type_level::SizeValidatedArray;
///
/// const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);
/// assert_eq!(ARRAY.size(), 8);
/// ```
fn example_type_level_arithmetic() {
    // Arrange: Create size-validated array using const generics
    // Note: SizeValidatedArray does runtime validation, not compile-time
    const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);

    // Act & Assert: Verify size validation (runtime check)
    assert_eq!(ARRAY.size(), 8);
    assert_eq!(ARRAY.data().len(), 8);
}

// ============================================================================
// Example 3: Type State Pattern with Sealed Traits
// ============================================================================

/// Example: Type state pattern for test phases
///
/// ## How-to: Use Type State Pattern
///
/// Use `TestState<Phase>` to enforce test phase ordering at compile time. The type
/// system prevents calling methods in the wrong order (e.g., calling `assert()` before `act()`).
///
/// **Phases**: `Arrange` → `Act` → `Assert`
/// - Start with `TestState::<Arrange>::new()`
/// - Transition to `Act` with `.act()`
/// - Transition to `Assert` with `.assert()`
///
/// ## Reference
///
/// - **Type**: `TestState<Phase>` where `Phase` is `Arrange`, `Act`, or `Assert`
/// - **Function**: `TestState::<Arrange>::new() -> TestState<Arrange>`
/// - **Methods**:
///   - `with_arrange_data(data)` - Set arrange data
///   - `act() -> TestState<Act>` - Transition to act phase
///   - `execute(f)` - Execute action in act phase
///   - `assert() -> TestState<Assert>` - Transition to assert phase
///   - `assert_that(f)` - Assert condition in assert phase
/// - **Compile-Time Safety**: Type system prevents invalid transitions
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::core::state::{Arrange, TestState};
///
/// let arrange = TestState::<Arrange>::new().with_arrange_data(vec![1, 2, 3]);
/// let act = arrange.act().execute(|data| {
///     let mut result = data.unwrap_or_default();
///     result.push(4);
///     result
/// });
/// let assert = act.assert();
/// assert!(assert.assert_that(|result| result.map(|r| r.len() == 4).unwrap_or(false)));
/// ```
fn example_type_state_pattern() {
    // Arrange: Start with Arrange phase (type system enforces order)
    let arrange_state = TestState::<Arrange>::new().with_arrange_data(vec![1, 2, 3]);

    // Act: Transition to Act phase (only possible from Arrange)
    let act_state = arrange_state.act();
    let act_state = act_state.execute(|data| {
        let mut result = data.unwrap_or_default();
        result.push(4);
        result
    });

    // Assert: Transition to Assert phase (only possible from Act)
    let assert_state = act_state.assert();
    assert!(assert_state.assert_that(|result| { result.map(|r| r.len() == 4).unwrap_or(false) }));

    // Type system prevents calling methods in wrong order:
    // - Cannot call `act()` on `TestState<Assert>`
    // - Cannot call `assert()` on `TestState<Arrange>`
    // - Cannot create `TestState<Act>` directly
}

// ============================================================================
// Main Function
// ============================================================================

fn main() {
    println!("Advanced Rust Features Examples");
    println!("================================");
    println!();
    println!("1. Type-Level Arithmetic and Const Generics");
    example_type_level_arithmetic();
    println!("   ✓ Size-validated array created successfully");
    println!();
    println!("2. Type State Pattern");
    example_type_state_pattern();
    println!("   ✓ Type state pattern enforced compile-time guarantees");
    println!();
    #[cfg(feature = "async")]
    {
        println!("3. Async Traits (requires async feature)");
        println!("   Note: Async fixture examples are in src/core/async_fixture.rs tests");
        println!("   Run with: cargo test --features async --lib async_fixture");
    }
    #[cfg(not(feature = "async"))]
    {
        println!("3. Async Traits");
        println!("   Enable async feature to see async traits example");
    }
    println!();
    println!("Summary: Advanced features maximize DX by:");
    println!("  - Providing compile-time guarantees (catch errors early)");
    println!("  - Reducing boilerplate (less code to write)");
    println!("  - Improving error messages (type system guides correct usage)");
    println!("  - Zero-cost abstractions (no runtime overhead)");
}
