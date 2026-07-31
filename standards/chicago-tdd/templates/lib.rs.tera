#![forbid(unsafe_code)]
#![deny(warnings)]

pub mod checkpoints;
pub mod refusals;
pub mod standards;

pub use checkpoints::{GallCheckpoint, GALL_CHECKPOINTS};
pub use refusals::{Refusal, REFUSALS};
pub use standards::{Standard, STANDARDS};

pub const MANUFACTURING_LIFECYCLE: &[&str] = &["Resolve", "Enrich", "Extract", "Render", "Write", "Receipt"];
pub const GALL_STATES: &[&str] = &["PARTIAL_ALIVE", "ALIVE", "BLOCKED", "BUILD_BROKEN", "UNKNOWN", "UNSUPPORTED"];
