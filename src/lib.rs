//! Chicago TDD Tools
//!
//! A comprehensive testing framework for Chicago TDD (Classicist Test-Driven Development)
//! methodology in Rust. Provides fixtures, builders, helpers, and advanced testing
//! capabilities including property-based testing and mutation testing.
//!
//! ## Features
//!
//! - **Test Fixtures**: Reusable test fixtures with state management and test isolation
//! - **Builders**: Fluent builders for test data and workflows
//! - **Assertion Helpers**: Comprehensive assertion utilities
//! - **Macros**: AAA pattern enforcement and test helpers
//! - **Property-Based Testing**: QuickCheck-style random test generation
//! - **Mutation Testing**: Test quality validation through mutations
//! - **Coverage Analysis**: Test coverage reporting and analysis
//!
//! ## Chicago TDD Principles
//!
//! This framework enforces Chicago TDD principles:
//!
//! 1. **State-Based Testing**: Tests verify outputs and state, not implementation
//! 2. **Real Collaborators**: Uses actual dependencies, not mocks
//! 3. **Behavior Verification**: Tests verify what code does, not how
//! 4. **AAA Pattern**: All tests follow Arrange-Act-Assert structure
//!
//! ## Usage
//!
//! ```rust
//! use chicago_tdd_tools::prelude::*;
//!
//! # #[tokio::test]
//! # async fn test_example() {
//!     // Arrange: Create fixture
//!     let fixture = TestFixture::new().unwrap_or_else(|e| panic!("Failed to create fixture: {}", e));
//!
//!     // Act: Execute test
//!     let counter = fixture.test_counter();
//!
//!     // Assert: Verify state
//!     assert!(counter >= 0);
//! # }
//! ```
//!
//! ## Module Organization
//!
//! Modules are organized into capability groups for better discoverability and maintainability:
//!
//! ### Core Testing Infrastructure (`core`)
//! - `fixture`: Test fixtures and setup utilities
//! - `async_fixture`: Async test fixtures with async traits (requires `async` feature)
//! - `builders`: Fluent builders for test data
//! - `assertions`: Assertion helpers and utilities
//! - `macros`: Test macros for AAA pattern enforcement and assertions
//! - `state`: Type-level AAA enforcement
//! - `poka_yoke`: Error prevention through type-level safety (prevents invalid states)
//! - `const_assert`: Compile-time assertions
//! - `alert`: Alert helpers for visual problem indicators (with optional `log` crate integration)
//!
//! ### Advanced Testing Techniques (`testing`)
//! - `property`: Property-based testing framework
//! - `mutation`: Mutation testing framework
//! - `snapshot`: Snapshot testing (requires `snapshot-testing` feature)
//! - `concurrency`: Concurrency testing (requires `concurrency-testing` feature)
//! - `cli`: CLI testing (requires `cli-testing` feature)
//! - `generator`: Test code generation
//!
//! ### Quality & Validation (`validation`)
//! - `coverage`: Test coverage analysis
//! - `guards`: Guard constraint enforcement (`MAX_RUN_LEN` ≤ 8, `MAX_BATCH_SIZE`)
//! - `jtbd`: Jobs To Be Done validation framework (validates code accomplishes intended purpose)
//! - `performance`: RDTSC benchmarking and tick measurement
//!
//! ### Telemetry & Observability (`observability`)
//! - `otel`: OTEL span/metric validation (requires `otel` feature)
//! - `weaver`: Weaver live validation integration (requires `weaver` feature)
//!
//! ### Integration Testing (`integration`)
//! - `testcontainers`: Docker container support (requires `testcontainers` feature)
//!
//! ## Backward Compatibility
//!
//! All modules are re-exported at the crate root for backward compatibility.
//! Existing code using `chicago_tdd_tools::fixture::*` continues to work.
//! New code is encouraged to use capability group paths: `chicago_tdd_tools::core::fixture::*`
//!
//! ## Macros
//!
//! The crate provides several macros to reduce boilerplate and enforce Chicago TDD principles:
//!
//! ## Procedural Macros
//!
//! - `#[tdd_test]`: Procedural macro for zero-boilerplate tests with AAA validation
//!   - Import: `use chicago_tdd_tools::tdd_test;` (re-exported) or `use chicago_tdd_tools_proc_macros::tdd_test;`
//! - `#[fixture]`: Procedural macro for automatic fixture setup/teardown
//!   - Import: `use chicago_tdd_tools::fixture;` (re-exported) or `use chicago_tdd_tools_proc_macros::fixture;`
//! - `#[derive(TestBuilder)]`: Derive macro for fluent builder generation
//!
//! ## Declarative Macros
//!
//! - `test!`: Enforce AAA pattern for synchronous tests
//! - `async_test!`: Enforce AAA pattern for async tests
//! - `fixture_test!`: Async test with automatic fixture setup/teardown
//! - `performance_test!`: Performance test with tick budget validation
//! - `assert_ok!`: Assert Result is Ok with detailed error messages
//! - `assert_err!`: Assert Result is Err with detailed error messages
//! - `assert_within_tick_budget!`: Validate performance constraints (≤8 ticks)
//! - `assert_in_range!`: Assert value is within range with detailed messages
//! - `assert_eq_msg!`: Assert equality with custom message
//! - `assert_guard_constraint!`: Validate guard constraints
//! - `alert_critical!`: Emit critical alert (🚨) - must stop immediately
//! - `alert_warning!`: Emit warning alert (⚠️) - should stop
//! - `alert_info!`: Emit info alert (ℹ️) - informational
//! - `alert_success!`: Emit success alert (✅) - operation completed
//! - `alert_debug!`: Emit debug alert (🔍) - detailed diagnostics
//! - `alert!`: Emit custom alert with user-defined severity

#![deny(clippy::unwrap_used)]
#![deny(warnings)] // Poka-Yoke: Prevent warnings at compile time - compiler enforces correctness
#![warn(missing_docs)]
// Poka-Yoke: pub use is necessary for procedural macro re-exports
#![allow(clippy::pub_use, reason = "Procedural macros must be re-exported via pub use")]
// Test code - panic is appropriate for test failures
#![cfg_attr(test, allow(clippy::panic))]

// Note: When using the `logging` feature (enabled by default), users should initialize
// the AlertLogger at the start of their application:
//   use chicago_tdd_tools::alert::AlertLogger;
//   let _ = AlertLogger::init_default();
// This enables standard log macros (log::error!, log::warn!, etc.) to use the alert format.
// Alert macros (alert_critical!, alert_warning!, etc.) also use log::* when logging is enabled.

// Re-export procedural macros
// Both #[tdd_test] and #[fixture] are re-exported at crate root for convenience
// Users can import from chicago_tdd_tools: use chicago_tdd_tools::{tdd_test, fixture};
// Or directly from chicago_tdd_tools_proc_macros: use chicago_tdd_tools_proc_macros::{tdd_test, fixture};
pub use chicago_tdd_tools_proc_macros::fixture;
pub use chicago_tdd_tools_proc_macros::tdd_test;

// Re-export TestBuilder derive macro (users will use #[derive(TestBuilder)])
pub use chicago_tdd_tools_proc_macros::TestBuilder;

// Capability groups - organized by functionality
//
// **Kaizen improvement**: Module declaration pattern to prevent dead code.
// All modules MUST be declared here (or in parent module's mod.rs).
// Files not declared as modules are dead code and will be removed.
// Pattern: Use `pub mod` for new modules, `pub use` for re-exports.
// **Waste elimination**: Work reports and internal documentation don't belong in docs/.
// Only user-facing documentation should be in docs/ (guides, API refs, architecture).
pub mod core;
pub mod integration;
pub mod observability;
pub mod testing;
pub mod validation;

// Macros are exported via core::macros module
// src/macros.rs re-exports from core::macros for backward compatibility
// Note: #[macro_use] is not needed here - macros are exported via #[macro_export] in macro definitions
pub mod macros;

// Re-export new "go the extra mile" types
pub use core::assertions::{AssertionBuilder, ValidatedAssertion};
pub use core::builders::{GenericTestDataBuilder, ValidatedTestDataBuilder};
pub use validation::coverage::{CoveragePercentage, CoveredCount, TotalCount};
pub use validation::jtbd::ScenarioIndex;
pub use validation::performance::ValidatedTickBudget;

// Backward compatibility: Re-export modules at crate root for existing code
// New code should use capability group paths: core::fixture, validation::guards, etc.
pub use core::{alert, assertions, builders, const_assert, fixture, state};
// Note: async_fixture is separate because it's feature-gated (requires `async` feature)
#[cfg(feature = "async")]
pub use core::async_fixture;
#[cfg(feature = "testcontainers")]
pub use integration::testcontainers;
#[cfg(feature = "otel")]
pub use observability::otel;
#[cfg(feature = "weaver")]
pub use observability::weaver::types::WeaverLiveCheck;
#[cfg(feature = "weaver")]
pub use observability::weaver::{WeaverValidationError, WeaverValidationResult};
// Unified observability API (new)
#[cfg(any(feature = "otel", feature = "weaver"))]
pub use observability::{ObservabilityError, ObservabilityResult, ObservabilityTest};
#[cfg(feature = "cli-testing")]
pub use testing::cli;
#[cfg(feature = "concurrency-testing")]
pub use testing::concurrency;
#[cfg(feature = "snapshot-testing")]
pub use testing::snapshot;
pub use testing::{generator, mutation, property};
pub use validation::{coverage, guards, jtbd, performance};

/// Prelude module - import everything you need with `use chicago_tdd_tools::prelude::*;`
///
/// **Usage**:
/// ```rust,ignore
/// use chicago_tdd_tools::prelude::*;
///
/// test!(my_test, {
///     // All macros and types available
/// });
/// ```
///
/// **What's included**:
/// - All core modules (fixture, builders, assertions, macros)
/// - All validation modules (coverage, guards, jtbd, performance)
/// - Feature-gated modules (when features enabled): property, mutation, snapshot, concurrency, cli
pub mod prelude {
    pub use crate::core::*;
    pub use crate::validation::*;

    // Macros are automatically exported via #[macro_export] in macro definitions
    // They can be used directly: test!, assert_ok!, etc.
    // Or explicitly: use chicago_tdd_tools::{test, assert_ok};

    #[cfg(feature = "property-testing")]
    pub use crate::testing::property::*;

    #[cfg(feature = "mutation-testing")]
    pub use crate::testing::mutation::*;

    #[cfg(feature = "snapshot-testing")]
    pub use crate::testing::snapshot::*;

    #[cfg(feature = "concurrency-testing")]
    pub use crate::testing::concurrency::*;

    #[cfg(feature = "cli-testing")]
    pub use crate::testing::cli::*;

    #[cfg(feature = "otel")]
    pub use crate::observability::otel::*;

    #[cfg(feature = "weaver")]
    pub use crate::observability::weaver::{WeaverValidationError, WeaverValidationResult};

    // Unified observability API (new)
    #[cfg(any(feature = "otel", feature = "weaver"))]
    pub use crate::observability::{ObservabilityError, ObservabilityResult, ObservabilityTest};

    #[cfg(feature = "testcontainers")]
    pub use crate::integration::testcontainers::*;
}
