//! Advanced Rust Features Examples
//!
//! Demonstrates hyper-advanced Rust features used in Chicago TDD Tools:
//! - Async Traits (Rust 1.75+)
//! - Generic Associated Types (GATs)
//! - Sealed Traits
//! - Type-Level Arithmetic
//! - Const Generics
//! - Type State Pattern
//!
//! These features maximize developer experience (DX) and reduce friction
//! by providing compile-time guarantees and zero-cost abstractions.

#[cfg(feature = "async")]
use chicago_tdd_tools::core::async_fixture::{AsyncFixtureManager, AsyncFixtureProvider};
#[cfg(feature = "async")]
use chicago_tdd_tools::core::fixture::FixtureError;
use chicago_tdd_tools::core::state::{Arrange, TestState};
use chicago_tdd_tools::core::type_level::SizeValidatedArray;

// ============================================================================
// Example 1: Async Traits (Rust 1.75+)
// ============================================================================

#[cfg(feature = "async")]
struct DatabaseFixture {
    connection: String,
}

#[cfg(feature = "async")]
struct DatabaseProvider;

// Note: Sealed trait pattern prevents external implementations
// For examples, we can't implement the sealed trait, so we'll skip the async fixture example
// See src/core/async_fixture.rs tests for working examples

// ============================================================================
// Example 2: Type-Level Arithmetic and Const Generics
// ============================================================================

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
