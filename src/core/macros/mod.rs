//! Macros Module
//!
//! Re-exports all macro modules for convenient access.
//! Note: `#[macro_use]` is used here to re-export macros from submodules

#[macro_use]
pub mod test;
#[macro_use]
pub mod assert;

// config_test! / config_refusal_test! are macro_rules!, expanded textually
// at the call site -- they reference `star_toml::` but never require
// star_toml as a dependency of THIS crate, only of whichever crate expands
// them (examples/star-toml, or a downstream consumer that itself depends on
// star_toml). No feature gate needed: an unused macro_rules! definition
// costs nothing to compile.
#[macro_use]
pub mod config_test;

#[cfg(all(feature = "weaver", feature = "otel"))]
#[macro_use]
pub mod weaver_test;
