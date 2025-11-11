//! Unified Observability Testing
//!
//! Ground-up TRIZ redesign combining OTEL and Weaver testing into a single,
//! ergonomic API with automatic resource management and zero-cost abstractions.
//!
//! **Key Features**:
//! - Unified API for OTEL and Weaver testing
//! - RAII-based automatic lifecycle management
//! - Auto-detection of Weaver binary and registry
//! - Compile-time validation where possible (zero-cost)
//! - Runtime validation when needed (real collaborators)
//! - Type-safe API (invalid states unrepresentable)
//!
//! **Usage**:
//! ```rust
//! use chicago_tdd_tools::observability::ObservabilityTest;
//!
//! // Simple usage - zero configuration for 80% of cases
//! let test = ObservabilityTest::new()?;
//! test.validate_span(&span)?;
//! // Automatic cleanup via Drop trait
//! ```
//!
//! **Required Features**:
//! - `otel`: Enable OTEL span/metric validation (`chicago-tdd-tools = { features = ["otel"] }`)
//! - `weaver`: Enable Weaver live validation (`chicago-tdd-tools = { features = ["weaver"] }`)

// Unified API (new implementation)
pub mod unified;

// Re-export unified API as main API
pub use unified::{ObservabilityError, ObservabilityResult, ObservabilityTest, TestConfig};

// Keep old modules temporarily for types (will be removed)
// Types are still needed for the unified API
#[cfg(feature = "otel")]
pub mod otel;
#[cfg(feature = "weaver")]
pub mod weaver;

#[cfg(all(feature = "weaver", feature = "otel"))]
pub mod fixtures;
