//! Integration Testing
//!
//! External system integration for integration testing with external
//! dependencies, such as Testcontainers for Docker support.
//!
//! **Required Features**:
//! - `testcontainers`: Enable Docker container support (`chicago-tdd-tools = { features = ["testcontainers"] }`)
//!
//! **Usage**:
//! ```rust,ignore
//! // Enable feature in Cargo.toml:
//! // chicago-tdd-tools = { features = ["testcontainers"] }
//!
//! use chicago_tdd_tools::integration::testcontainers::*;
//! ```

#[cfg(feature = "testcontainers")]
pub mod testcontainers;
#[cfg(not(feature = "testcontainers"))]
/// Testcontainers module - requires `testcontainers` feature
///
/// **Error**: If you see "cannot find module 'testcontainers'", enable the feature:
/// ```toml
/// chicago-tdd-tools = { features = ["testcontainers"] }
/// ```
mod testcontainers_placeholder {
    // Placeholder to provide helpful error message
    // When feature is disabled, users get "cannot find module 'testcontainers'" error
    // Documentation above guides them to enable the feature
}

// Re-export commonly used items
#[cfg(feature = "testcontainers")]
pub use testcontainers::*;
