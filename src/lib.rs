//! Chicago TDD Tools
//!
//! A comprehensive testing framework for Chicago TDD (Classicist Test-Driven Development)
//! methodology in Rust. Provides fixtures, builders, helpers, and advanced testing
//! capabilities including property-based testing and mutation testing.
//!
//! ## Features
//!
//! - **Test Fixtures**: Reusable test fixtures with automatic cleanup
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
//! - `builders`: Fluent builders for test data
//! - `assertions`: Assertion helpers and utilities
//! - `macros`: Test macros for AAA pattern enforcement and assertions
//! - `state`: Type-level AAA enforcement
//! - `const_assert`: Compile-time assertions
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
//! - `#[chicago_test]`: Procedural macro for zero-boilerplate tests with AAA validation
//!   - Import: `use chicago_tdd_tools_proc_macros::chicago_test;`
//! - `#[chicago_fixture]`: Procedural macro for automatic fixture setup/teardown
//! - `#[derive(TestBuilder)]`: Derive macro for fluent builder generation
//!
//! ## Declarative Macros
//!
//! - `chicago_test!`: Enforce AAA pattern for synchronous tests
//! - `chicago_async_test!`: Enforce AAA pattern for async tests
//! - `chicago_fixture_test!`: Async test with automatic fixture setup/teardown
//! - `chicago_performance_test!`: Performance test with tick budget validation
//! - `assert_ok!`: Assert Result is Ok with detailed error messages
//! - `assert_err!`: Assert Result is Err with detailed error messages
//! - `assert_within_tick_budget!`: Validate performance constraints (≤8 ticks)
//! - `assert_in_range!`: Assert value is within range with detailed messages
//! - `assert_eq_msg!`: Assert equality with custom message
//! - `assert_guard_constraint!`: Validate guard constraints

#![deny(clippy::unwrap_used)]
#![warn(missing_docs)]
// Poka-Yoke: pub use is necessary for procedural macro re-exports
#![allow(clippy::pub_use, reason = "Procedural macros must be re-exported via pub use")]
// Test code - panic is appropriate for test failures
#![cfg_attr(test, allow(clippy::panic))]

// Re-export procedural macros
// Note: #[chicago_test] and #[chicago_fixture] are available via chicago_tdd_tools_proc_macros
// Users can import them explicitly: use chicago_tdd_tools_proc_macros::{chicago_test, chicago_fixture};
pub use chicago_tdd_tools_proc_macros::chicago_fixture;

// Re-export TestBuilder derive macro (users will use #[derive(TestBuilder)])
pub use chicago_tdd_tools_proc_macros::TestBuilder;

// Capability groups - organized by functionality
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
pub use validation::coverage::{CoveredCount, TotalCount};
pub use validation::jtbd::ScenarioIndex;
pub use validation::performance::ValidatedTickBudget;

// Backward compatibility: Re-export modules at crate root for existing code
// New code should use capability group paths: core::fixture, validation::guards, etc.
pub use core::{assertions, builders, const_assert, fixture, state};
#[cfg(feature = "testcontainers")]
pub use integration::testcontainers;
#[cfg(feature = "otel")]
pub use observability::otel;
#[cfg(feature = "weaver")]
pub use observability::weaver;
#[cfg(feature = "cli-testing")]
pub use testing::cli;
#[cfg(feature = "concurrency-testing")]
pub use testing::concurrency;
#[cfg(feature = "snapshot-testing")]
pub use testing::snapshot;
pub use testing::{generator, mutation, property};
pub use validation::{coverage, guards, jtbd, performance};

/// Prelude module - import commonly used items
pub mod prelude {
    pub use crate::core::*;
    pub use crate::validation::*;

    // Macros are automatically exported via #[macro_use] in lib.rs
    // They can be used directly: chicago_test!, assert_ok!, etc.
    // Or explicitly: use chicago_tdd_tools::{chicago_test, assert_ok};

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
    pub use crate::observability::weaver::*;

    #[cfg(feature = "testcontainers")]
    pub use crate::integration::testcontainers::*;
}
