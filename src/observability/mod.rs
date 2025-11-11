//! Telemetry & Observability
//!
//! Telemetry validation for OTEL spans/metrics and Weaver live-check.
//! Validates observability correctness and schema conformance.

#[cfg(feature = "otel")]
pub mod otel;
#[cfg(feature = "weaver")]
pub mod weaver;

// Re-export commonly used items
// Note: Both otel and weaver export a `types` module, causing ambiguous glob re-exports.
// This is intentional - users can disambiguate with module paths (otel::types, weaver::types).
#[allow(ambiguous_glob_reexports)]
#[cfg(feature = "otel")]
pub use otel::*;
#[allow(ambiguous_glob_reexports)]
#[cfg(feature = "weaver")]
pub use weaver::*;
