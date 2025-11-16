//! Core Testing Infrastructure
//!
//! Foundational testing primitives that all tests use: fixtures, builders,
//! assertions, macros, state management, compile-time assertions, alert helpers,
//! and common test utilities.

pub mod alert;
pub mod assertions;
pub mod async_fixture;
pub mod builders;
pub mod config;
pub mod const_assert;
pub mod contract;
pub mod fixture;
pub mod macros;
pub mod poka_yoke;
pub mod receipt;
pub mod state;
pub mod test_utils;
pub mod type_level;

// Re-export commonly used items
pub use alert::*;
pub use assertions::*;
#[cfg(feature = "async")]
pub use async_fixture::*;
pub use builders::*;
pub use const_assert::*;
pub use contract::*;
pub use fixture::*;
pub use poka_yoke::*;
pub use receipt::*;
pub use state::*;
pub use test_utils::*;
pub use type_level::*;
