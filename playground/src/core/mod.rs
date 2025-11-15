//! Core Features Examples
//!
//! Demonstrates all core features: fixtures, async_fixture, builders,
//! assertions, macros, state, type_level, const_assert, alert
//!
//! v1.3.0 additions:
//! - assertions_v130: New assertion macros (assert_contains, assert_json_eq, assert_approx_eq)
//! - builders_presets: Builder presets and validation hooks

pub mod fixtures;
pub mod async_fixtures;
pub mod builders;
pub mod assertions;
pub mod assertions_v130; // v1.3.0: New assertion macros
pub mod macros;
pub mod state;
pub mod type_level;
pub mod const_assert;
pub mod alert;
pub mod builders_presets; // v1.3.0: Builder presets and validation

