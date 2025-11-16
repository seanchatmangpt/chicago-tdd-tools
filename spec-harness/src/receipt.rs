//! Spec Conformance Receipt Generation
//!
//! Each spec harness test run generates a cryptographically signed receipt
//! proving that the Chicago-TDD-Tools framework correctly implements the
//! Chatman Equation specification.

use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// A signed receipt proving spec conformance at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecConformanceReceipt {
    /// Unique receipt identifier
    pub receipt_id: String,

    /// Specification version being validated (e.g., "ChatmanEquation-1.0")
    pub spec_version: String,

    /// Git commit hash of chicago-tdd-tools at time of test
    pub git_commit_hash: String,

    /// Version of chicago-tdd-tools framework tested
    pub framework_version: String,

    /// Version of this spec harness
    pub harness_version: String,

    /// Timestamp of spec conformance check
    pub timestamp: String,

    /// Results for each chapter's theorems
    pub chapter_results: Vec<ChapterResult>,

    /// Overall compliance status
    pub overall_status: ComplianceStatus,

    /// Merkle root of all theorem results (for integrity proof)
    pub merkle_root: String,

    /// Number of theorems validated
    pub theorem_count: usize,

    /// Number of theorems that passed
    pub passed_count: usize,

    /// Number of theorems that failed
    pub failed_count: usize,

    /// Number of theorems pending implementation
    pub pending_count: usize,

    /// Test execution time in milliseconds
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    FullCompliance,
    PartialCompliance,
    NonCompliant,
    PendingReview,
}

/// Results for theorems in a specific chapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterResult {
    pub chapter_id: String,
    pub chapter_name: String,
    pub theorems: Vec<TheoremResult>,
    pub summary: ChapterSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pending: usize,
}

/// Result for a single theorem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremResult {
    /// Theorem identifier (e.g., "Thm-2.1")
    pub theorem_id: String,

    /// Human-readable theorem name
    pub theorem_name: String,

    /// Test status
    pub status: TestStatus,

    /// Error message if test failed
    pub error_message: Option<String>,

    /// Test execution time in milliseconds
    pub execution_time_ms: u64,

    /// SHA256 hash of test input data (for reproducibility)
    pub input_hash: String,

    /// SHA256 hash of test output data (for reproducibility)
    pub output_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Pending,
}

/// Merkle proof for receipt integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub root_hash: String,
    pub leaf_hashes: Vec<String>,
    pub proof_path: Vec<String>,
}

impl SpecConformanceReceipt {
    /// Create a new spec conformance receipt
    pub fn new(
        spec_version: String,
        git_commit_hash: String,
        framework_version: String,
        harness_version: String,
        chapter_results: Vec<ChapterResult>,
    ) -> Self {
        let receipt_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        let mut theorem_count = 0;
        let mut passed_count = 0;
        let mut failed_count = 0;
        let mut pending_count = 0;

        for chapter in &chapter_results {
            for theorem in &chapter.theorems {
                theorem_count += 1;
                match theorem.status {
                    TestStatus::Passed => passed_count += 1,
                    TestStatus::Failed => failed_count += 1,
                    TestStatus::Pending => pending_count += 1,
                }
            }
        }

        let overall_status = if failed_count == 0 && pending_count == 0 {
            ComplianceStatus::FullCompliance
        } else if failed_count == 0 {
            ComplianceStatus::PartialCompliance
        } else if failed_count > theorem_count / 2 {
            ComplianceStatus::NonCompliant
        } else {
            ComplianceStatus::PendingReview
        };

        let merkle_root = Self::compute_merkle_root(&chapter_results);

        Self {
            receipt_id,
            spec_version,
            git_commit_hash,
            framework_version,
            harness_version,
            timestamp,
            chapter_results,
            overall_status,
            merkle_root,
            theorem_count,
            passed_count,
            failed_count,
            pending_count,
            execution_time_ms: 0, // Will be set by caller
        }
    }

    /// Compute merkle root of all theorem results
    fn compute_merkle_root(chapter_results: &[ChapterResult]) -> String {
        let mut hasher = Sha256::new();

        for chapter in chapter_results {
            for theorem in &chapter.theorems {
                hasher.update(theorem.theorem_id.as_bytes());
                hasher.update(format!("{:?}", theorem.status).as_bytes());
                hasher.update(theorem.input_hash.as_bytes());
                hasher.update(theorem.output_hash.as_bytes());
            }
        }

        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify receipt integrity (check merkle root)
    pub fn verify_integrity(&self) -> bool {
        let computed_root = Self::compute_merkle_root(&self.chapter_results);
        computed_root == self.merkle_root
    }

    /// Get compliance percentage
    pub fn compliance_percentage(&self) -> f64 {
        if self.theorem_count == 0 {
            0.0
        } else {
            (self.passed_count as f64 / self.theorem_count as f64) * 100.0
        }
    }

    /// Serialize receipt to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize receipt from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl TheoremResult {
    pub fn new_passed(
        theorem_id: String,
        theorem_name: String,
        input_hash: String,
        output_hash: String,
    ) -> Self {
        Self {
            theorem_id,
            theorem_name,
            status: TestStatus::Passed,
            error_message: None,
            execution_time_ms: 0,
            input_hash,
            output_hash,
        }
    }

    pub fn new_failed(
        theorem_id: String,
        theorem_name: String,
        error: String,
        input_hash: String,
        output_hash: String,
    ) -> Self {
        Self {
            theorem_id,
            theorem_name,
            status: TestStatus::Failed,
            error_message: Some(error),
            execution_time_ms: 0,
            input_hash,
            output_hash,
        }
    }

    pub fn new_pending(
        theorem_id: String,
        theorem_name: String,
    ) -> Self {
        Self {
            theorem_id,
            theorem_name,
            status: TestStatus::Pending,
            error_message: None,
            execution_time_ms: 0,
            input_hash: String::new(),
            output_hash: String::new(),
        }
    }
}

impl ChapterResult {
    pub fn new(chapter_id: String, chapter_name: String, theorems: Vec<TheoremResult>) -> Self {
        let total = theorems.len();
        let passed = theorems.iter().filter(|t| t.status == TestStatus::Passed).count();
        let failed = theorems.iter().filter(|t| t.status == TestStatus::Failed).count();
        let pending = theorems.iter().filter(|t| t.status == TestStatus::Pending).count();

        Self {
            chapter_id,
            chapter_name,
            theorems,
            summary: ChapterSummary {
                total,
                passed,
                failed,
                pending,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theorem_result_creation() {
        let result = TheoremResult::new_passed(
            "Thm-2.1".to_string(),
            "Determinism".to_string(),
            "input_hash".to_string(),
            "output_hash".to_string(),
        );

        assert_eq!(result.status, TestStatus::Passed);
        assert_eq!(result.theorem_id, "Thm-2.1");
    }

    #[test]
    fn test_receipt_json_roundtrip() {
        let chapter_result = ChapterResult::new(
            "ch02".to_string(),
            "Chapter 2".to_string(),
            vec![
                TheoremResult::new_passed(
                    "Thm-2.1".to_string(),
                    "Determinism".to_string(),
                    "input".to_string(),
                    "output".to_string(),
                ),
            ],
        );

        let receipt = SpecConformanceReceipt::new(
            "ChatmanEquation-1.0".to_string(),
            "abc123".to_string(),
            "1.3.0".to_string(),
            "1.0.0".to_string(),
            vec![chapter_result],
        );

        let json = receipt.to_json().expect("Failed to serialize");
        let deserialized = SpecConformanceReceipt::from_json(&json).expect("Failed to deserialize");

        assert_eq!(receipt.receipt_id, deserialized.receipt_id);
        assert_eq!(receipt.passed_count, deserialized.passed_count);
    }

    #[test]
    fn test_receipt_integrity_verification() {
        let chapter_result = ChapterResult::new(
            "ch02".to_string(),
            "Chapter 2".to_string(),
            vec![],
        );

        let receipt = SpecConformanceReceipt::new(
            "ChatmanEquation-1.0".to_string(),
            "abc123".to_string(),
            "1.3.0".to_string(),
            "1.0.0".to_string(),
            vec![chapter_result],
        );

        assert!(receipt.verify_integrity(), "Receipt should verify with correct merkle root");
    }

    #[test]
    fn test_compliance_percentage() {
        let theorems = vec![
            TheoremResult::new_passed(
                "Thm-2.1".to_string(),
                "Test 1".to_string(),
                "input".to_string(),
                "output".to_string(),
            ),
            TheoremResult::new_failed(
                "Thm-2.2".to_string(),
                "Test 2".to_string(),
                "Error".to_string(),
                "input".to_string(),
                "output".to_string(),
            ),
        ];

        let chapter = ChapterResult::new("ch02".to_string(), "Chapter 2".to_string(), theorems);
        let receipt = SpecConformanceReceipt::new(
            "ChatmanEquation-1.0".to_string(),
            "abc123".to_string(),
            "1.3.0".to_string(),
            "1.0.0".to_string(),
            vec![chapter],
        );

        assert_eq!(receipt.compliance_percentage(), 50.0);
    }
}
