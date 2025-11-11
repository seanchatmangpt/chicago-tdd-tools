//! Telemetry & Observability
//!
//! Telemetry validation for OTEL spans/metrics and Weaver live-check.
//! Validates observability correctness and schema conformance.
//!
//! **Required Features**:
//! - `otel`: Enable OTEL span/metric validation (`chicago-tdd-tools = { features = ["otel"] }`)
//! - `weaver`: Enable Weaver live validation (`chicago-tdd-tools = { features = ["weaver"] }`)
//!
//! **Usage**:
//! ```rust
//! // Enable features in Cargo.toml:
//! // chicago-tdd-tools = { features = ["otel", "weaver"] }
//!
//! use chicago_tdd_tools::observability::otel::SpanValidator;
//! use chicago_tdd_tools::observability::weaver::WeaverValidator;
//! ```

#[cfg(feature = "otel")]
pub mod otel;
#[cfg(not(feature = "otel"))]
/// OTEL module - requires `otel` feature
///
/// **Error**: If you see "cannot find module 'otel'", enable the feature:
/// ```toml
/// chicago-tdd-tools = { features = ["otel"] }
/// ```
mod otel_placeholder {
    // Placeholder to provide helpful error message
    // When feature is disabled, users get "cannot find module 'otel'" error
    // Documentation above guides them to enable the feature
}

#[cfg(feature = "weaver")]
pub mod weaver;
#[cfg(not(feature = "weaver"))]
/// Weaver module - requires `weaver` feature
///
/// **Error**: If you see "cannot find module 'weaver'", enable the feature:
/// ```toml
/// chicago-tdd-tools = { features = ["weaver"] }
/// ```
mod weaver_placeholder {
    // Placeholder to provide helpful error message
    // When feature is disabled, users get "cannot find module 'weaver'" error
    // Documentation above guides them to enable the feature
}

// Re-export commonly used items
// Note: Both otel and weaver export a `types` module, causing ambiguous glob re-exports.
// This is intentional - users can disambiguate with module paths (otel::types, weaver::types).
#[allow(ambiguous_glob_reexports)]
#[cfg(feature = "otel")]
pub use otel::*;
#[allow(ambiguous_glob_reexports)]
#[cfg(feature = "weaver")]
pub use weaver::*;
