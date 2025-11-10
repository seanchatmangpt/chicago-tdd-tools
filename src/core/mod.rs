//! Core Testing Infrastructure
//!
//! Foundational testing primitives that all tests use: fixtures, builders,
//! assertions, macros, state management, and compile-time assertions.

pub mod assertions;
pub mod builders;
pub mod const_assert;
pub mod fixture;
pub mod macros;
pub mod state;

// Re-export commonly used items
pub use assertions::*;
pub use builders::*;
pub use const_assert::*;
pub use fixture::*;
pub use state::*;
