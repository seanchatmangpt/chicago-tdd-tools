//! Macros Module
//!
//! Re-exports macros from core::macros for backward compatibility.
//! Macros are exported at crate root via #[macro_export] in their definitions.

// Re-export macro submodules - macros use #[macro_export] so they're already at crate root
pub use crate::core::macros::assert;
pub use crate::core::macros::test;
