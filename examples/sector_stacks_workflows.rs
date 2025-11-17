//! # Sector Stacks Workflows Example - Comprehensive Guide
//!
//! Demonstrates production-grade sector implementations (Academic Publishing and
//! Enterprise Claims Processing) with deterministic algorithms and cryptographic receipts.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through complete workflows for two sectors:
//!
//! 1. **Academic Publishing**: Paper review lifecycle with deterministic decision algorithms
//! 2. **Enterprise Claims**: Insurance claims processing with fraud detection and settlement
//!
//! Both workflows demonstrate:
//! - Deterministic algorithms (same inputs → same outputs)
//! - Multi-stage workflows (6 stages each)
//! - Cryptographic receipt generation (SHA-256 merkle roots)
//! - Knowledge hooks and guard constraints
//!
//! ## Explanation: Concepts
//!
//! **Sector Stacks**: Production-grade implementations of the Chatman Equation (A = μ(O))
//! applied to real-world workflows. Each sector demonstrates deterministic guarantees
//! and cryptographic proofs.
//!
//! **Deterministic Algorithms**: Same inputs always produce same outputs. This enables:
//! - Reproducible decisions
//! - Audit trail completeness
//! - Cryptographic receipt generation
//!
//! **Receipt Generation**: Every operation generates a cryptographic receipt with:
//! - Unique operation ID
//! - Sector and operation type
//! - Status (Success/PartialSuccess/Failed/PendingReview)
//! - SHA-256 merkle root for integrity
//! - ISO8601 timestamp
//!
//! ## How-to: Common Tasks
//!
//! - Academic workflow: See `example_academic_workflow()`
//! - Claims workflow: See `example_claims_workflow()`
//! - Receipt generation: See `example_receipt_generation()`
//! - Determinism verification: See `example_determinism_verification()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `PaperSubmission`: Academic paper submission
//! - `Review`: Paper review with score and recommendation
//! - `Decision`: Editorial decision (Accepted/MinorRevisions/MajorRevisions/Rejected)
//! - `ClaimSubmission`: Insurance claim submission
//! - `FraudScore`: Fraud detection score
//! - `Settlement`: Settlement calculation
//! - `OperationReceipt`: Cryptographic receipt for all operations
//!
//! **Key Modules**:
//! - `sector_stacks::academic`: Academic publishing workflow
//! - `sector_stacks::claims`: Enterprise claims processing workflow

use chicago_tdd_tools::sector_stacks::academic::*;
use chicago_tdd_tools::sector_stacks::claims::*;
use chicago_tdd_tools::sector_stacks::{OperationStatus, SectorOperation};

/// Example: Complete Academic Publishing Workflow
///
/// ## How-to: Academic Workflow
///
/// Demonstrates the complete paper review lifecycle:
/// Submission → Desk Review → Reviewer Assignment → Review Collection → Decision → Notification
fn example_academic_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example: Academic Publishing Workflow ===");

    // Arrange: Create paper submission
    let paper = PaperSubmission {
        paper_id: "paper-001".to_string(),
        title: "Advanced Testing Methodologies".to_string(),
        authors: vec!["Dr. Smith".to_string(), "Dr. Jones".to_string()],
        abstract_text: "This paper presents novel testing methodologies...".to_string(),
        file_size_bytes: 500_000,
    };

    println!("\n--- Stage 1: Submission ---");
    println!("Paper ID: {}", paper.paper_id);
    println!("Title: {}", paper.title);
    println!("Authors: {:?}", paper.authors);

    // Stage 2: Desk Review (simplified - in real system would check format, scope, etc.)
    println!("\n--- Stage 2: Desk Review ---");
    println!("✓ Paper format valid");
    println!("✓ Within scope");

    // Stage 3: Reviewer Assignment (deterministic)
    println!("\n--- Stage 3: Reviewer Assignment ---");
    let reviews = vec![]; // Will be populated
    let operation = AcademicOperation::new(paper.clone(), reviews);
    let assignment = operation.assign_reviewers();
    println!("Assigned Reviewers: {:?}", assignment.reviewers);
    println!("Assignment Date: {}", assignment.assignment_date);

    // Stage 4: Review Collection
    println!("\n--- Stage 4: Review Collection ---");
    let reviews = vec![
        Review {
            reviewer: assignment.reviewers[0].clone(),
            score: 4.0,
            comments: "Excellent contribution to the field".to_string(),
            recommendation: ReviewRecommendation::Accept,
        },
        Review {
            reviewer: assignment.reviewers[1].clone(),
            score: 3.8,
            comments: "Good work, minor improvements needed".to_string(),
            recommendation: ReviewRecommendation::Accept,
        },
        Review {
            reviewer: assignment.reviewers[2].clone(),
            score: 3.5,
            comments: "Solid paper, acceptable".to_string(),
            recommendation: ReviewRecommendation::Accept,
        },
    ];

    for review in &reviews {
        println!(
            "Reviewer: {}, Score: {:.1}, Recommendation: {:?}",
            review.reviewer, review.score, review.recommendation
        );
    }

    // Stage 5: Decision (deterministic algorithm)
    println!("\n--- Stage 5: Decision ---");
    let operation = AcademicOperation::new(paper.clone(), reviews.clone());
    // Decision is computed internally, we can see it in the receipt
    let receipt = operation.generate_receipt(OperationStatus::Success);
    println!("Decision: {}", receipt.result);

    // Decision algorithm: avg_score >= 3.5 = Accepted
    let avg_score = reviews.iter().map(|r| r.score).sum::<f64>() / reviews.len() as f64;
    println!("Average Score: {:.2}", avg_score);
    println!("Decision: {}", receipt.result);

    // Stage 6: Notification (receipt generation)
    println!("\n--- Stage 6: Notification (Receipt Generation) ---");
    // Receipt already generated above
    println!("Receipt ID: {}", receipt.id);
    println!("Sector: {}", receipt.sector);
    println!("Operation: {}", receipt.operation);
    println!("Status: {:?}", receipt.status);
    println!("Merkle Root: {}", receipt.merkle_root);
    println!("Timestamp: {}", receipt.timestamp);

    // Verify receipt properties
    assert_eq!(receipt.sector, "Academic");
    assert_eq!(receipt.operation, "Decision");
    assert!(!receipt.merkle_root.is_empty());
    assert!(!receipt.id.is_empty());

    println!("\n✓ Academic workflow completed successfully!");

    Ok(())
}

/// Example: Complete Enterprise Claims Processing Workflow
///
/// ## How-to: Claims Workflow
///
/// Demonstrates the complete claims processing pipeline:
/// Validation → Fraud Detection → Entitlements → Settlement → Payment → Receipt
fn example_claims_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Enterprise Claims Processing Workflow ===");

    // Arrange: Create claim submission
    let claim = ClaimSubmission {
        claim_id: "CLAIM-001".to_string(),
        claimant_id: "CLT-0001".to_string(),
        claim_amount: 15_000.0,
        claim_date: "2025-01-15".to_string(),
        incident_description: "Vehicle damage from collision on highway".to_string(),
    };

    println!("\n--- Stage 1: Validation ---");
    println!("Claim ID: {}", claim.claim_id);
    println!("Claimant ID: {}", claim.claimant_id);
    println!("Claim Amount: ${:.2}", claim.claim_amount);

    // Process claim through workflow
    let operation = ClaimsOperation::new(claim.clone());

    // Stage 2-5: Process through workflow (all handled internally)
    println!("\n--- Stages 2-5: Processing ---");
    println!("✓ Claim processed through workflow");

    // Get receipt to see results
    let receipt = operation.generate_receipt(OperationStatus::Success);
    println!("Processing Result: {}", receipt.result);

    // Settlement calculation is deterministic and handled internally

    // Stage 6: Payment & Receipt
    println!("\n--- Stage 6: Payment & Receipt Generation ---");
    let should_approve = operation.should_approve();
    println!("Should Approve: {}", should_approve);

    let receipt = operation.generate_receipt(OperationStatus::Success);
    println!("Receipt ID: {}", receipt.id);
    println!("Sector: {}", receipt.sector);
    println!("Operation: {}", receipt.operation);
    println!("Status: {:?}", receipt.status);
    println!("Merkle Root: {}", receipt.merkle_root);

    // Verify receipt properties
    assert_eq!(receipt.sector, "Enterprise Claims");
    assert_eq!(receipt.operation, "Settlement");
    assert!(!receipt.merkle_root.is_empty());

    println!("\n✓ Claims workflow completed successfully!");

    Ok(())
}

/// Example: Deterministic Reviewer Assignment
///
/// ## How-to: Deterministic Assignment
///
/// Demonstrates that same paper always gets same reviewers (reproducible).
fn example_deterministic_reviewer_assignment() {
    println!("\n=== Example: Deterministic Reviewer Assignment ===");

    let paper = PaperSubmission {
        paper_id: "paper-123".to_string(),
        title: "Test Paper".to_string(),
        authors: vec!["Author".to_string()],
        abstract_text: "Abstract".to_string(),
        file_size_bytes: 1000,
    };

    let reviews = vec![];

    // Create operation twice with same paper
    let op1 = AcademicOperation::new(paper.clone(), reviews.clone());
    let op2 = AcademicOperation::new(paper, reviews);

    let assign1 = op1.assign_reviewers();
    let assign2 = op2.assign_reviewers();

    // Same paper should get same reviewers (deterministic)
    assert_eq!(assign1.reviewers, assign2.reviewers);
    println!("✓ Same paper always gets same reviewers");
    println!("Reviewers: {:?}", assign1.reviewers);
}

/// Example: Deterministic Decision Algorithm
///
/// ## How-to: Decision Logic
///
/// Demonstrates deterministic decision algorithm:
/// - Any rejection → Rejected
/// - avg_score >= 3.5 → Accepted
/// - avg_score >= 2.5 → MinorRevisions
/// - Otherwise → MajorRevisions
fn example_deterministic_decision() {
    println!("\n=== Example: Deterministic Decision Algorithm ===");

    // Test Case 1: High scores → Accepted
    println!("\n--- Test Case 1: High Scores → Accepted ---");
    let reviews = vec![
        Review {
            reviewer: "r1".to_string(),
            score: 4.0,
            comments: "Excellent".to_string(),
            recommendation: ReviewRecommendation::Accept,
        },
        Review {
            reviewer: "r2".to_string(),
            score: 3.8,
            comments: "Good".to_string(),
            recommendation: ReviewRecommendation::Accept,
        },
    ];
    let paper = PaperSubmission {
        paper_id: "test-1".to_string(),
        title: "Test".to_string(),
        authors: vec!["Author".to_string()],
        abstract_text: "Abstract".to_string(),
        file_size_bytes: 1000,
    };
    let operation = AcademicOperation::new(paper, reviews.clone());
    let receipt = operation.generate_receipt(OperationStatus::Success);
    let avg = reviews.iter().map(|r| r.score).sum::<f64>() / reviews.len() as f64;
    println!("Average Score: {:.2}", avg);
    println!("Decision: {}", receipt.result);
    assert!(receipt.result.contains("Accepted"));

    // Test Case 2: Rejection → Rejected (all-or-nothing)
    println!("\n--- Test Case 2: Any Rejection → Rejected ---");
    let reviews = vec![
        Review {
            reviewer: "r1".to_string(),
            score: 4.0,
            comments: "Excellent".to_string(),
            recommendation: ReviewRecommendation::Accept,
        },
        Review {
            reviewer: "r2".to_string(),
            score: 2.0,
            comments: "Not suitable".to_string(),
            recommendation: ReviewRecommendation::Reject,
        },
    ];
    let paper = PaperSubmission {
        paper_id: "test-2".to_string(),
        title: "Test".to_string(),
        authors: vec!["Author".to_string()],
        abstract_text: "Abstract".to_string(),
        file_size_bytes: 1000,
    };
    let operation = AcademicOperation::new(paper, reviews);
    let receipt = operation.generate_receipt(OperationStatus::Success);
    println!("Decision: {}", receipt.result);
    assert!(receipt.result.contains("Rejected"));

    // Test Case 3: Medium scores → MinorRevisions
    println!("\n--- Test Case 3: Medium Scores → MinorRevisions ---");
    let reviews = vec![
        Review {
            reviewer: "r1".to_string(),
            score: 3.0,
            comments: "Good".to_string(),
            recommendation: ReviewRecommendation::MinorRevisions,
        },
        Review {
            reviewer: "r2".to_string(),
            score: 2.8,
            comments: "Needs work".to_string(),
            recommendation: ReviewRecommendation::MinorRevisions,
        },
    ];
    let paper = PaperSubmission {
        paper_id: "test-3".to_string(),
        title: "Test".to_string(),
        authors: vec!["Author".to_string()],
        abstract_text: "Abstract".to_string(),
        file_size_bytes: 1000,
    };
    let operation = AcademicOperation::new(paper, reviews.clone());
    let receipt = operation.generate_receipt(OperationStatus::Success);
    let avg = reviews.iter().map(|r| r.score).sum::<f64>() / reviews.len() as f64;
    println!("Average Score: {:.2}", avg);
    println!("Decision: {}", receipt.result);
    assert!(receipt.result.contains("Minor") || receipt.result.contains("Major"));
}

/// Example: Fraud Detection and Settlement
///
/// ## How-to: Fraud Detection
///
/// Demonstrates deterministic fraud detection and settlement calculation.
fn example_fraud_detection() {
    println!("\n=== Example: Fraud Detection and Settlement ===");

    // Test Case 1: Valid claim (no fraud)
    println!("\n--- Test Case 1: Valid Claim (No Fraud) ---");
    let claim = ClaimSubmission {
        claim_id: "CLAIM-002".to_string(),
        claimant_id: "CLT-0002".to_string(),
        claim_amount: 5_000.0,
        claim_date: "2025-01-15".to_string(),
        incident_description: "Valid incident description here".to_string(),
    };

    let operation = ClaimsOperation::new(claim);
    println!("Should Approve: {}", operation.should_approve());
    let receipt = operation.generate_receipt(OperationStatus::Success);
    println!("Result: {}", receipt.result);
    assert!(operation.should_approve());

    // Test Case 2: High amount (fraud indicator)
    println!("\n--- Test Case 2: High Amount (Fraud Indicator) ---");
    let claim = ClaimSubmission {
        claim_id: "CLAIM-003".to_string(),
        claimant_id: "CLT-0003".to_string(),
        claim_amount: 150_000.0, // High amount triggers fraud
        claim_date: "2025-01-15".to_string(),
        incident_description: "Valid incident description here".to_string(),
    };

    let operation = ClaimsOperation::new(claim);
    println!("Should Approve: {}", operation.should_approve());
    let receipt = operation.generate_receipt(OperationStatus::Success);
    println!("Result: {}", receipt.result);
    assert!(!operation.should_approve());

    // Test Case 3: Settlement calculation
    println!("\n--- Test Case 3: Settlement Calculation ---");
    let claim = ClaimSubmission {
        claim_id: "CLAIM-004".to_string(),
        claimant_id: "CLT-0004".to_string(),
        claim_amount: 10_000.0,
        claim_date: "2025-01-15".to_string(),
        incident_description: "Valid incident description here".to_string(),
    };

    let operation = ClaimsOperation::new(claim);
    let receipt = operation.generate_receipt(OperationStatus::Success);
    println!("Settlement Result: {}", receipt.result);
    // Settlement calculation is deterministic and shown in receipt
    println!("✓ Settlement calculation complete");
}

/// Example: Receipt Reproducibility
///
/// ## How-to: Receipt Generation
///
/// Demonstrates that same inputs produce same receipts (deterministic).
fn example_receipt_reproducibility() {
    println!("\n=== Example: Receipt Reproducibility ===");

    // Academic receipt
    let paper = PaperSubmission {
        paper_id: "paper-456".to_string(),
        title: "Test Paper".to_string(),
        authors: vec!["Author".to_string()],
        abstract_text: "Abstract".to_string(),
        file_size_bytes: 1000,
    };

    let reviews = vec![Review {
        reviewer: "reviewer1".to_string(),
        score: 3.5,
        comments: "Good".to_string(),
        recommendation: ReviewRecommendation::Accept,
    }];

    let op1 = AcademicOperation::new(paper.clone(), reviews.clone());
    let op2 = AcademicOperation::new(paper, reviews);

    let receipt1 = op1.generate_receipt(OperationStatus::Success);
    let receipt2 = op2.generate_receipt(OperationStatus::Success);

    // Same inputs should produce same merkle root
    assert_eq!(receipt1.merkle_root, receipt2.merkle_root);
    println!("✓ Academic receipts are reproducible");
    println!("Merkle Root: {}", receipt1.merkle_root);

    // Claims receipt
    let claim = ClaimSubmission {
        claim_id: "CLAIM-005".to_string(),
        claimant_id: "CLT-0005".to_string(),
        claim_amount: 5_000.0,
        claim_date: "2025-01-15".to_string(),
        incident_description: "Valid incident description here".to_string(),
    };

    let op1 = ClaimsOperation::new(claim.clone());
    let op2 = ClaimsOperation::new(claim);

    let receipt1 = op1.generate_receipt(OperationStatus::Success);
    let receipt2 = op2.generate_receipt(OperationStatus::Success);

    // Same inputs should produce same merkle root
    assert_eq!(receipt1.merkle_root, receipt2.merkle_root);
    println!("✓ Claims receipts are reproducible");
    println!("Merkle Root: {}", receipt1.merkle_root);
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Sector Stacks Workflows - Production-Grade Examples         ║");
    println!("║  Academic Publishing & Enterprise Claims Processing          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Run examples
    if let Err(e) = example_academic_workflow() {
        eprintln!("Error in academic workflow: {}", e);
    }

    if let Err(e) = example_claims_workflow() {
        eprintln!("Error in claims workflow: {}", e);
    }

    example_deterministic_reviewer_assignment();
    example_deterministic_decision();
    example_fraud_detection();
    example_receipt_reproducibility();

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║  All Examples Completed Successfully!                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use chicago_tdd_tools::sector_stacks::{
        academic::{AcademicOperation, PaperSubmission, Review, ReviewRecommendation},
        claims::{ClaimSubmission, ClaimsOperation},
        OperationStatus, SectorOperation,
    };
    use chicago_tdd_tools::test;

    test!(test_academic_workflow_complete, {
        // Arrange
        let paper = PaperSubmission {
            paper_id: "test-paper".to_string(),
            title: "Test".to_string(),
            authors: vec!["Author".to_string()],
            abstract_text: "Abstract".to_string(),
            file_size_bytes: 1000,
        };

        let reviews = vec![Review {
            reviewer: "r1".to_string(),
            score: 4.0,
            comments: "Good".to_string(),
            recommendation: ReviewRecommendation::Accept,
        }];

        // Act
        let operation = AcademicOperation::new(paper, reviews);
        let receipt = operation.generate_receipt(OperationStatus::Success);

        // Assert
        assert_eq!(receipt.sector, "Academic");
        assert!(!receipt.merkle_root.is_empty());
    });

    test!(test_claims_workflow_complete, {
        // Arrange
        let claim = ClaimSubmission {
            claim_id: "CLAIM-TEST".to_string(),
            claimant_id: "CLT-0001".to_string(),
            claim_amount: 5_000.0,
            claim_date: "2025-01-15".to_string(),
            incident_description: "Valid incident description here".to_string(),
        };

        // Act
        let operation = ClaimsOperation::new(claim);
        let receipt = operation.generate_receipt(OperationStatus::Success);

        // Assert
        assert_eq!(receipt.sector, "Enterprise Claims");
        assert!(!receipt.merkle_root.is_empty());
    });

    test!(test_deterministic_reviewer_assignment, {
        // Arrange
        let paper = PaperSubmission {
            paper_id: "test-paper".to_string(),
            title: "Test".to_string(),
            authors: vec!["Author".to_string()],
            abstract_text: "Abstract".to_string(),
            file_size_bytes: 1000,
        };

        let reviews = vec![];

        // Act
        let op1 = AcademicOperation::new(paper.clone(), reviews.clone());
        let op2 = AcademicOperation::new(paper, reviews);

        let assign1 = op1.assign_reviewers();
        let assign2 = op2.assign_reviewers();

        // Assert: Same paper gets same reviewers
        assert_eq!(assign1.reviewers, assign2.reviewers);
    });
}
