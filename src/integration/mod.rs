//! Integration Testing
//!
//! External system integration for integration testing with external
//! dependencies, such as Testcontainers for Docker support.

#[cfg(feature = "testcontainers")]
pub mod testcontainers;

// Re-export commonly used items
#[cfg(feature = "testcontainers")]
pub use testcontainers::*;
