//! Quality & Validation
//!
//! Quality assurance and constraint validation: test coverage analysis,
//! guard constraints (runtime and compile-time), Jobs To Be Done validation,
//! and performance validation.

pub mod coverage;
pub mod guards;
pub mod jtbd;
pub mod performance;

// Re-export commonly used items
pub use coverage::*;
pub use guards::*;
pub use jtbd::*;
pub use performance::*;
