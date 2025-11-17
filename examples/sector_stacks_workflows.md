# Sector Stacks Workflows Example

**Category:** Tutorial  
**Level:** Intermediate  
**Prerequisites:** Basic understanding of workflows, deterministic algorithms  
**Features Required:** None

---

## Overview

This example demonstrates production-grade sector implementations (Academic Publishing and Enterprise Claims Processing) with deterministic algorithms and cryptographic receipts. Both workflows showcase the Chatman Equation (A = μ(O)) applied to real-world scenarios.

**What you'll learn:**
- Complete workflow execution for Academic Publishing (6 stages)
- Complete workflow execution for Enterprise Claims (6 stages)
- Deterministic algorithms and reproducibility
- Cryptographic receipt generation with SHA-256 merkle roots
- Fraud detection and settlement calculation

---

## Quick Start

```bash
cargo run --example sector_stacks_workflows
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- Understanding of deterministic algorithms

---

## Tutorial: Academic Publishing Workflow

### Complete Workflow Stages

1. **Submission**: Paper submitted with metadata
2. **Desk Review**: Format and scope validation
3. **Reviewer Assignment**: Deterministic reviewer assignment
4. **Review Collection**: Reviews collected from reviewers
5. **Decision**: Deterministic decision algorithm
6. **Notification**: Receipt generation

### Step-by-Step Example

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::sector_stacks::academic::*;

// Stage 1: Create submission
let paper = PaperSubmission {
    paper_id: "paper-001".to_string(),
    title: "Advanced Testing Methodologies".to_string(),
    authors: vec!["Dr. Smith".to_string()],
    abstract_text: "This paper presents...".to_string(),
    file_size_bytes: 500_000,
};

// Stage 3: Assign reviewers (deterministic)
let operation = AcademicOperation::new(paper.clone(), vec![]);
let assignment = operation.assign_reviewers();
// Same paper always gets same reviewers

// Stage 4: Collect reviews
let reviews = vec![
    Review {
        reviewer: assignment.reviewers[0].clone(),
        score: 4.0,
        comments: "Excellent".to_string(),
        recommendation: ReviewRecommendation::Accept,
    },
    // ... more reviews
];

// Stage 5: Make decision (deterministic)
let operation = AcademicOperation::new(paper, reviews);
let decision = operation.decision();
// Decision algorithm: avg_score >= 3.5 = Accepted

// Stage 6: Generate receipt
let receipt = operation.generate_receipt(OperationStatus::Success);
```

---

## Tutorial: Enterprise Claims Processing Workflow

### Complete Workflow Stages

1. **Validation**: Claim structure validation
2. **Fraud Detection**: Deterministic fraud scoring
3. **Entitlements**: Eligibility determination
4. **Settlement**: Settlement calculation
5. **Payment**: Payment processing
6. **Receipt**: Receipt generation

### Step-by-Step Example

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::sector_stacks::claims::*;

// Stage 1: Create claim
let claim = ClaimSubmission {
    claim_id: "CLAIM-001".to_string(),
    claimant_id: "CLT-0001".to_string(),
    claim_amount: 15_000.0,
    claim_date: "2025-01-15".to_string(),
    incident_description: "Vehicle damage...".to_string(),
};

// Process through workflow
let operation = ClaimsOperation::new(claim);

// Stage 2: Validation result
match operation.validation() {
    ValidationResult::Valid => { /* proceed */ }
    ValidationResult::Invalid => { /* reject */ }
}

// Stage 3: Fraud detection
println!("Fraud Score: {}", operation.fraud_score().score);
println!("Is Fraudulent: {}", operation.fraud_score().is_fraudulent);

// Stage 4: Entitlements
match operation.entitlements() {
    EntitlementsResult::Entitled => { /* proceed */ }
    EntitlementsResult::NotEntitled => { /* reject */ }
}

// Stage 5: Settlement calculation
// Formula: final = min(claim - deductible, policy_limit)
println!("Final Settlement: ${:.2}", operation.settlement().final_amount);

// Stage 6: Generate receipt
let receipt = operation.generate_receipt(OperationStatus::Success);
```

---

## How-To: Common Tasks

### Deterministic Reviewer Assignment

Same paper always gets same reviewers (reproducible):

```rust
let paper = PaperSubmission { /* ... */ };
let reviews = vec![];

let op1 = AcademicOperation::new(paper.clone(), reviews.clone());
let op2 = AcademicOperation::new(paper, reviews);

let assign1 = op1.assign_reviewers();
let assign2 = op2.assign_reviewers();

// Same reviewers assigned
assert_eq!(assign1.reviewers, assign2.reviewers);
```

### Deterministic Decision Algorithm

Decision logic:
- Any rejection → Rejected
- avg_score >= 3.5 → Accepted
- avg_score >= 2.5 → MinorRevisions
- Otherwise → MajorRevisions

```rust
let reviews = vec![
    Review {
        reviewer: "r1".to_string(),
        score: 4.0,
        comments: "Excellent".to_string(),
        recommendation: ReviewRecommendation::Accept,
    },
    // ... more reviews
];

let decision = Decision::from_reviews(&reviews);
// Deterministic: same reviews → same decision
```

### Fraud Detection

Deterministic fraud detection based on indicators:

```rust
let claim = ClaimSubmission {
    claim_id: "CLAIM-002".to_string(),
    claim_amount: 150_000.0, // High amount triggers fraud
    // ... other fields
};

let operation = ClaimsOperation::new(claim);
// Fraud indicators: High amount, duplicate claim, etc.
println!("Fraud Score: {}", operation.fraud_score().score);
println!("Is Fraudulent: {}", operation.fraud_score().is_fraudulent);
```

### Settlement Calculation

Deterministic settlement formula:

```rust
// Formula: final = min(claim - deductible, policy_limit)
let settlement = Settlement::calculate(10_000.0, 500.0, 50_000.0);

// Result: min(10_000 - 500, 50_000) = 9_500.0
assert_eq!(settlement.final_amount, 9_500.0);
```

### Receipt Generation

Cryptographic receipts with SHA-256 merkle roots:

```rust
let operation = AcademicOperation::new(paper, reviews);
let receipt = operation.generate_receipt(OperationStatus::Success);

// Receipt contains:
// - Unique ID
// - Sector and operation type
// - Status
// - SHA-256 merkle root
// - ISO8601 timestamp
```

---

## Explanation: Concepts

### Deterministic Algorithms

**Same Inputs → Same Outputs**: All algorithms are deterministic, meaning:
- Same paper → same reviewers
- Same reviews → same decision
- Same claim → same fraud score and settlement
- Same inputs → same receipt merkle root

**Why Determinism Matters:**
- Reproducible decisions
- Audit trail completeness
- Cryptographic receipt generation
- Test predictability

### Decision Algorithm (Academic)

Deterministic decision logic:

```
if any rejection in reviews:
    return Rejected
else if avg_score >= 3.5:
    return Accepted
else if avg_score >= 2.5:
    return MinorRevisions
else:
    return MajorRevisions
```

**All-or-Nothing**: Any single rejection causes overall rejection.

### Fraud Detection (Claims)

Deterministic fraud indicators:
- High amount (> 100,000)
- Duplicate claim ID
- Other indicators

**Any Indicator = Fraudulent**: Single indicator triggers fraud flag.

### Settlement Calculation

Deterministic formula:

```
final_amount = min(max(claim_amount - deductible, 0), policy_limit)
```

**Guards**:
- Budget: settlement ≤ policy_limit
- Legality: claim_amount > 0
- Chronology: claim_date validation

### Receipt Generation

Cryptographic receipts ensure:
- **Integrity**: SHA-256 merkle root
- **Provenance**: Unique operation ID
- **Auditability**: Complete operation details
- **Reproducibility**: Same inputs → same merkle root

---

## Reference: Quick Lookup

### Academic Types

**PaperSubmission**:
```rust
pub struct PaperSubmission {
    pub paper_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub file_size_bytes: usize,
}
```

**Review**:
```rust
pub struct Review {
    pub reviewer: String,
    pub score: f64, // 0-5
    pub comments: String,
    pub recommendation: ReviewRecommendation,
}
```

**Decision**:
```rust
pub enum Decision {
    Accepted,
    MinorRevisions,
    MajorRevisions,
    Rejected,
}
```

**AcademicOperation**:
```rust
impl AcademicOperation {
    pub fn new(paper: PaperSubmission, reviews: Vec<Review>) -> Self;
    pub fn assign_reviewers(&self) -> ReviewerAssignment;
    pub fn generate_decision_receipt(&self) -> OperationReceipt;
}
```

### Claims Types

**ClaimSubmission**:
```rust
pub struct ClaimSubmission {
    pub claim_id: String,
    pub claimant_id: String,
    pub claim_amount: f64,
    pub claim_date: String,
    pub incident_description: String,
}
```

**FraudScore**:
```rust
pub struct FraudScore {
    pub score: u32, // 0-100
    pub indicators: Vec<String>,
    pub is_fraudulent: bool,
}
```

**Settlement**:
```rust
pub struct Settlement {
    pub amount: f64,
    pub deductible: f64,
    pub policy_limit: f64,
    pub final_amount: f64,
}
```

**ClaimsOperation**:
```rust
impl ClaimsOperation {
    pub fn new(claim: ClaimSubmission) -> Self;
    pub fn should_approve(&self) -> bool;
    pub fn generate_settlement_receipt(&self) -> OperationReceipt;
}
```

### OperationReceipt

Generic receipt for all sector operations:

```rust
pub struct OperationReceipt {
    pub id: String,
    pub sector: String,
    pub operation: String,
    pub status: OperationStatus,
    pub result: String,
    pub merkle_root: String, // SHA-256
    pub timestamp: String,   // ISO8601
}
```

---

## Common Patterns

### Complete Academic Workflow

```rust
let paper = PaperSubmission { /* ... */ };
let operation = AcademicOperation::new(paper.clone(), vec![]);
let assignment = operation.assign_reviewers();

let reviews = vec![/* collect reviews */];
let operation = AcademicOperation::new(paper, reviews);
let receipt = operation.generate_receipt(OperationStatus::Success);
```

### Complete Claims Workflow

```rust
let claim = ClaimSubmission { /* ... */ };
let operation = ClaimsOperation::new(claim);

if operation.should_approve() {
    let receipt = operation.generate_receipt(OperationStatus::Success);
    // Process payment
}
```

### Receipt Reproducibility

```rust
let op1 = AcademicOperation::new(paper.clone(), reviews.clone());
let op2 = AcademicOperation::new(paper, reviews);

let receipt1 = op1.generate_receipt(OperationStatus::Success);
let receipt2 = op2.generate_receipt(OperationStatus::Success);

// Same merkle root
assert_eq!(receipt1.merkle_root, receipt2.merkle_root);
```

---

## Troubleshooting

### Reviewer Assignment Not Deterministic

**Issue**: Different reviewers assigned for same paper

**Solution**: Ensure paper ID is identical:
```rust
let paper1 = PaperSubmission { paper_id: "paper-123".to_string(), /* ... */ };
let paper2 = PaperSubmission { paper_id: "paper-123".to_string(), /* ... */ };
// Same ID → same reviewers
```

### Decision Not Deterministic

**Issue**: Different decisions for same reviews

**Solution**: Ensure reviews are identical (same scores, recommendations):
```rust
let reviews1 = vec![Review { score: 4.0, /* ... */ }];
let reviews2 = vec![Review { score: 4.0, /* ... */ }];
// Same reviews → same decision
```

### Settlement Calculation Incorrect

**Issue**: Settlement doesn't match expected formula

**Solution**: Verify formula application:
```rust
let expected = ((claim_amount - deductible).max(0.0)).min(policy_limit);
assert_eq!(settlement.final_amount, expected);
```

---

## Related Documentation

- **Academic Workflow**: `src/sector_stacks/academic.rs`
- **Claims Workflow**: `src/sector_stacks/claims.rs`
- **Sector Stacks**: `src/sector_stacks.rs`
- **Release Notes**: `docs/releases/RELEASE_NOTES_v1.4.0.md`

---

## See Also

- [Fail-Fast Verification](fail_fast_verification.md) - 12-phase verification pipeline
- [RDF Validation](rdf_validation.md) - RDF-driven validation
- [Operator Registry](operator_registry.md) - Pattern registration

---

**Quality is the default. Prevention beats detection.**

*Version 1.4.0 | Updated 2025-01-16 | Team KNHK | License MIT*

