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
//! ## Modules
//!
//! - `fixture`: Test fixtures and setup utilities
//! - `builders`: Fluent builders for test data
//! - `assertions`: Assertion helpers and utilities
//! - `macros`: Test macros for AAA pattern enforcement and assertions
//! - `property`: Property-based testing framework
//! - `mutation`: Mutation testing framework
//! - `coverage`: Test coverage analysis
//! - `generator`: Test code generation
//! - `otel`: OTEL span/metric validation (requires `otel` feature)
//! - `performance`: RDTSC benchmarking and tick measurement
//! - `guards`: Guard constraint enforcement (`MAX_RUN_LEN` ≤ 8, `MAX_BATCH_SIZE`)
//! - `jtbd`: Jobs To Be Done validation framework (validates code accomplishes intended purpose)
//! - `weaver`: Weaver live validation integration (requires `weaver` feature)
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

pub mod assertions;
pub mod builders;
#[cfg(feature = "cli-testing")]
pub mod cli;
#[cfg(feature = "concurrency-testing")]
pub mod concurrency;
pub mod const_assert;
pub mod coverage;
pub mod fixture;
pub mod generator;
pub mod guards;
pub mod jtbd;
#[macro_use]
pub mod macros;
pub mod mutation;
#[cfg(feature = "otel")]
pub mod otel;
pub mod otel_types;
pub mod performance;
pub mod property;
#[cfg(feature = "snapshot-testing")]
pub mod snapshot;
pub mod state;
#[cfg(feature = "testcontainers")]
pub mod testcontainers;
#[cfg(feature = "weaver")]
pub mod weaver;
pub mod weaver_types;

// Re-export new "go the extra mile" types
pub use assertions::{AssertionBuilder, ValidatedAssertion};
pub use builders::{GenericTestDataBuilder, ValidatedTestDataBuilder};
pub use coverage::{CoveredCount, TotalCount};
pub use jtbd::ScenarioIndex;
pub use performance::ValidatedTickBudget;

/// Prelude module - import commonly used items
pub mod prelude {
    pub use crate::assertions::*;
    pub use crate::builders::*;
    pub use crate::fixture::*;
    pub use crate::guards::*;
    pub use crate::jtbd::*;
    pub use crate::performance::*;
    pub use crate::state::*;

    // Macros are automatically exported via #[macro_use] in lib.rs
    // They can be used directly: chicago_test!, assert_ok!, etc.
    // Or explicitly: use chicago_tdd_tools::{chicago_test, assert_ok};

    #[cfg(feature = "otel")]
    pub use crate::otel::*;

    #[cfg(feature = "property-testing")]
    pub use crate::property::*;

    #[cfg(feature = "mutation-testing")]
    pub use crate::mutation::*;

    #[cfg(feature = "weaver")]
    pub use crate::weaver::*;

    #[cfg(feature = "testcontainers")]
    pub use crate::testcontainers::*;

    #[cfg(feature = "snapshot-testing")]
    pub use crate::snapshot::*;

    #[cfg(feature = "concurrency-testing")]
    pub use crate::concurrency::*;

    #[cfg(feature = "cli-testing")]
    pub use crate::cli::*;
}
