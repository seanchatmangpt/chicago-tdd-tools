//! Telemetry & Observability
//!
//! Telemetry validation for OTEL spans/metrics and Weaver live-check.
//! Validates observability correctness and schema conformance.

#[cfg(feature = "otel")]
pub mod otel;
#[cfg(feature = "weaver")]
pub mod weaver;

// Re-export commonly used items
#[cfg(feature = "otel")]
pub use otel::*;
#[cfg(feature = "weaver")]
pub use weaver::*;
