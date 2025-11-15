//! Shared test utilities for Chicago TDD Tools tests
//!
//! Provides common test helpers used across multiple test files.
//! Consolidates duplicate code to reduce maintenance burden.

// Include common test utilities
include!("../test_common.inc");

/// Integration test helpers with compile-time enforcement
///
/// **Poka-yoke design**: Type-level prevention ensures integration tests
/// cannot be written without Docker checks.
pub mod integration {
    include!("integration.rs");
}

