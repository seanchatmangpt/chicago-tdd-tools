//! # Chatman Spec Harness
//!
//! Theorem-to-test mapping and spec conformance receipt generation.
//!
//! This crate provides the infrastructure for verifying that chicago-tdd-tools
//! correctly implements the Chatman Equation principles as documented in the
//! LaTeX specification (docs/latex/).
//!
//! ## Architecture
//!
//! Each chapter in the LaTeX spec maps to a test module:
//! - `chapter02`: Core Chatman Equation properties (Determinism, Idempotence)
//! - `chapter03`: Knowledge hooks and workflow patterns (43 YAWL patterns)
//! - `chapter07`: Chatman Equation realization via type system
//!
//! Each module contains:
//! - Theorem definitions (matching LaTeX theorem numbers)
//! - Property-based tests proving each theorem
//! - Snapshot tests for reproducibility
//! - Receipt generation with merkle proof
//!
//! ## Spec Conformance Receipts
//!
//! Every test run generates a `SpecConformanceReceipt` containing:
//! - Spec version (from LaTeX document)
//! - Git commit hash (from CI environment)
//! - Test suite version (Rust crate version)
//! - Per-theorem pass/fail status
//! - Merkle root of all theorems (for integrity verification)
//!
//! Example usage:
//! ```ignore
//! cargo make spec              # Run all spec harness tests + generate receipt
//! cargo make spec-check        # Verify 100% theorem coverage (CI gate)
//! ```

use serde::{Deserialize, Serialize};

pub mod chapter02;  // Core Chatman Equation properties
pub mod chapter03;  // Knowledge hooks and YAWL patterns
pub mod chapter07;  // Chatman Equation realization
pub mod receipt;     // Receipt generation and merkle proofs

pub use receipt::{SpecConformanceReceipt, TheoremResult, MerkleProof};

/// Specification version this harness validates against
pub const SPEC_VERSION: &str = "ChatmanEquation-1.0";

/// Chicago-tdd-tools framework version
pub const FRAMEWORK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Spec harness version
pub const HARNESS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get current git commit hash if available (requires git-integration feature)
#[cfg(feature = "git-integration")]
pub fn get_git_commit() -> Result<String, String> {
    use git2::Repository;

    let repo = Repository::open(".")
        .map_err(|e| format!("Failed to open git repo: {}", e))?;

    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;

    let oid = head.target()
        .ok_or("HEAD does not point to a valid commit")?;

    Ok(format!("{}", oid))
}

/// Fallback when git-integration feature is disabled
#[cfg(not(feature = "git-integration"))]
pub fn get_git_commit() -> Result<String, String> {
    std::env::var("GITHUB_SHA")
        .or_else(|_| std::env::var("GIT_COMMIT"))
        .map_err(|_| "Git commit unavailable (enable git-integration feature or set GITHUB_SHA/GIT_COMMIT)".to_string())
}

/// Theorem registry for tracking all theorems in the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremRegistry {
    /// Chapter 2 theorems (Core Chatman Equation)
    pub chapter02_theorems: Vec<TheoremMetadata>,

    /// Chapter 3 theorems (Knowledge Hooks)
    pub chapter03_theorems: Vec<TheoremMetadata>,

    /// Chapter 7 theorems (Realization)
    pub chapter07_theorems: Vec<TheoremMetadata>,
}

/// Metadata about a single theorem from the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremMetadata {
    /// Theorem ID (e.g., "Thm-2.1")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// LaTeX line range (for cross-reference)
    pub latex_lines: (usize, usize),

    /// Test path in harness
    pub test_path: String,

    /// Expected test result (Pass/Fail/Pending)
    pub expected_result: TestResultType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestResultType {
    Pass,
    Fail,
    Pending,
}

impl TheoremRegistry {
    /// Create the complete theorem registry matching the LaTeX spec
    pub fn new() -> Self {
        Self {
            chapter02_theorems: chapter02::theorems(),
            chapter03_theorems: chapter03::theorems(),
            chapter07_theorems: chapter07::theorems(),
        }
    }

    /// Total number of theorems across all chapters
    pub fn total_theorems(&self) -> usize {
        self.chapter02_theorems.len()
            + self.chapter03_theorems.len()
            + self.chapter07_theorems.len()
    }

    /// Get all theorems as a flat list
    pub fn all_theorems(&self) -> Vec<&TheoremMetadata> {
        let mut all = Vec::new();
        all.extend(self.chapter02_theorems.iter());
        all.extend(self.chapter03_theorems.iter());
        all.extend(self.chapter07_theorems.iter());
        all
    }
}

impl Default for TheoremRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theorem_registry_has_theorems() {
        let registry = TheoremRegistry::new();
        assert!(registry.total_theorems() > 0, "Registry must contain theorems");
    }

    #[test]
    fn test_spec_version_is_set() {
        assert!(!SPEC_VERSION.is_empty(), "Spec version must be set");
        assert_eq!(SPEC_VERSION, "ChatmanEquation-1.0");
    }
}
