//! Testcontainers module tests
//!
//! This file includes all testcontainers test modules from the testcontainers/ subdirectory.
//! Tests are organized into separate files for better maintainability:
//! - tests.rs - Unit tests (error paths, boundaries, features, stress tests)
//! - integration.rs - Integration tests (cleanup, real containers)
//! - expert.rs - Expert-level scenarios (warmup, etc.)
//! - weaver.rs - Weaver integration tests

#[cfg(all(feature = "testcontainers", test))]
mod testcontainers_tests {
    include!("testcontainers/tests.rs");
}

#[cfg(all(feature = "testcontainers", test))]
mod testcontainers_integration_tests {
    include!("testcontainers/integration.rs");
}

#[cfg(all(feature = "testcontainers", test))]
mod testcontainers_expert_tests {
    include!("testcontainers/expert.rs");
}

#[cfg(all(feature = "testcontainers", feature = "weaver", test))]
mod testcontainers_weaver_tests {
    include!("testcontainers/weaver.rs");
}
