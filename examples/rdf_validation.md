# RDF Validation Example

**Category:** How-To Guide  
**Level:** Intermediate  
**Prerequisites:** Understanding of RDF, ontologies, workflow validation  
**Features Required:** None (RDF structures are core, loading is optional)

---

## Overview

This example demonstrates RDF-driven validation with ontologies as single source of truth. Operations are validated against RDF ontology definitions at runtime, ensuring consistency between specification and implementation.

**What you'll learn:**
- Creating sector ontologies with stages, guards, and hooks
- Validating operations against ontology definitions
- Checking stage transitions and latency budgets
- Using guard constraints for safety validation
- Verifying determinism properties

---

## Quick Start

```bash
cargo run --example rdf_validation
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- Understanding of workflow validation

---

## Tutorial: Getting Started

### Step 1: Create Ontology

Create a new sector ontology:

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::sector_stacks::rdf::*;

let mut ontology = SectorOntology::new("Academic".to_string());
```

### Step 2: Define Workflow Stages

Add workflow stages with properties:

```rust
ontology.add_stage(WorkflowStage {
    id: "submission".to_string(),
    name: "Submission".to_string(),
    stage_number: 1,
    is_deterministic: true,
    max_latency_seconds: 60,
});
```

### Step 3: Add Guard Constraints

Add safety constraints:

```rust
ontology.add_guard(GuardConstraint {
    id: "budget_guard".to_string(),
    guard_type: "Budget".to_string(),
    constraints: vec!["settlement <= policy_limit".to_string()],
});
```

### Step 4: Add Knowledge Hooks

Add operations (hooks):

```rust
ontology.add_hook(KnowledgeHook {
    id: "desk_review".to_string(),
    name: "Desk Review".to_string(),
    description: "Initial desk review".to_string(),
    input_type: "PaperSubmission".to_string(),
    output_type: "ReviewResult".to_string(),
});
```

### Step 5: Validate Operations

Create validator and validate:

```rust
let validator = RdfOperationValidator::new().with_ontology(ontology);
validator.validate_operation_defined("desk_review")?;
```

---

## How-To: Common Tasks

### Create Complete Ontology

```rust
let mut ontology = SectorOntology::new("Claims".to_string());

// Add stages
ontology.add_stage(WorkflowStage {
    id: "validation".to_string(),
    name: "Validation".to_string(),
    stage_number: 1,
    is_deterministic: true,
    max_latency_seconds: 30,
});

// Add guards
ontology.add_guard(GuardConstraint {
    id: "budget".to_string(),
    guard_type: "Budget".to_string(),
    constraints: vec!["settlement <= policy_limit".to_string()],
});

// Add hooks
ontology.add_hook(KnowledgeHook {
    id: "validate_claim".to_string(),
    name: "Validate Claim".to_string(),
    description: "Validate claim structure".to_string(),
    input_type: "ClaimSubmission".to_string(),
    output_type: "ValidationResult".to_string(),
});
```

### Validate Stage Transitions

Ensure forward progression only:

```rust
let validator = RdfOperationValidator::new().with_ontology(ontology);

// Valid: Forward transition
validator.validate_stage_transition("validation", "fraud_detection")?;

// Invalid: Backward transition
let result = validator.validate_stage_transition("fraud_detection", "validation");
assert!(result.is_err());
```

### Validate Latency Budgets

Check operation latency against stage budgets:

```rust
// Stage has max_latency_seconds: 1 (1000ms)
validator.validate_latency_budget("fast_stage", 500)?; // OK: 500ms < 1000ms

// Exceeds budget
let result = validator.validate_latency_budget("fast_stage", 2000);
assert!(result.is_err()); // 2000ms > 1000ms
```

### Check Guard Constraints

Retrieve and validate guard constraints:

```rust
let guards = validator.get_guards()?;
for guard in guards {
    println!("Guard: {} ({})", guard.id, guard.guard_type);
    println!("Constraints: {:?}", guard.constraints);
}
```

### Verify Determinism

Check if all stages are deterministic:

```rust
let all_deterministic = validator.all_stages_deterministic()?;
if all_deterministic {
    println!("✓ All stages are deterministic");
}
```

---

## Explanation: Concepts

### RDF as Single Source of Truth

**Ontology-Driven Development**: RDF ontologies define workflow structure, stages, guards, and hooks. Runtime operations are validated against these definitions, ensuring:

- **Consistency**: Specification matches implementation
- **Validation**: Operations checked against ontology
- **Type Safety**: Input/output types validated
- **Guard Enforcement**: Safety constraints checked

### Workflow Stages

Each stage has:
- **ID**: Unique identifier (URI)
- **Name**: Human-readable name
- **Stage Number**: Ordering in workflow
- **Determinism**: Whether stage is deterministic
- **Max Latency**: Maximum allowed latency in seconds

### Guard Constraints

Five guard types:

1. **Legality**: Valid state transitions
2. **Budget**: Resource limits (e.g., settlement ≤ policy_limit)
3. **Chronology**: Temporal ordering
4. **Causality**: Causal dependencies
5. **Recursion**: Recursion depth limits

### Knowledge Hooks

Operations within workflows with:
- **ID**: Unique identifier
- **Name**: Human-readable name
- **Description**: Operation description
- **Input Type**: Expected input type
- **Output Type**: Expected output type

### Validation Types

**Operation Validation**: Check if operation is defined in ontology

**Stage Transition Validation**: Ensure forward progression only

**Latency Validation**: Verify operation latency within budget

**Guard Validation**: Check guard constraints are satisfied

**Determinism Validation**: Verify all stages are deterministic

---

## Reference: Quick Lookup

### SectorOntology

**Creation:**
```rust
pub fn new(sector: String) -> Self
```

**Methods:**
```rust
pub fn add_stage(&mut self, stage: WorkflowStage)
pub fn add_guard(&mut self, guard: GuardConstraint)
pub fn add_hook(&mut self, hook: KnowledgeHook)
pub fn get_stage(&self, id: &str) -> Option<&WorkflowStage>
pub fn deterministic_stages(&self) -> Vec<&WorkflowStage>
pub fn stage_count(&self) -> usize
pub fn guard_count(&self) -> usize
pub fn hook_count(&self) -> usize
```

### WorkflowStage

```rust
pub struct WorkflowStage {
    pub id: String,
    pub name: String,
    pub stage_number: u32,
    pub is_deterministic: bool,
    pub max_latency_seconds: u32,
}
```

### GuardConstraint

```rust
pub struct GuardConstraint {
    pub id: String,
    pub guard_type: String, // "Budget", "Chronology", etc.
    pub constraints: Vec<String>,
}
```

### KnowledgeHook

```rust
pub struct KnowledgeHook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_type: String,
    pub output_type: String,
}
```

### RdfOperationValidator

**Creation:**
```rust
pub fn new() -> Self
pub fn with_ontology(self, ontology: SectorOntology) -> Self
```

**Validation Methods:**
```rust
pub fn validate_operation_defined(&self, operation: &str) -> RdfValidationResult
pub fn validate_stage_transition(&self, from: &str, to: &str) -> RdfValidationResult
pub fn validate_latency_budget(&self, stage: &str, latency_ms: u32) -> RdfValidationResult
pub fn get_guards(&self) -> Result<Vec<GuardConstraint>, RdfValidationError>
pub fn all_stages_deterministic(&self) -> Result<bool, RdfValidationError>
```

### RdfValidationError

```rust
pub enum RdfValidationError {
    OperationNotDefined { operation: String, sector: String },
    GuardViolation { guard: String, operation: String },
    InvalidStageTransition { from: u32, to: u32 },
    LatencyBudgetExceeded { stage: String, actual: u32, budgeted: u32 },
    OntologyNotLoaded,
}
```

---

## Common Patterns

### Complete Ontology Setup

```rust
let mut ontology = SectorOntology::new("Academic".to_string());

// Add stages
ontology.add_stage(WorkflowStage { /* ... */ });

// Add guards
ontology.add_guard(GuardConstraint { /* ... */ });

// Add hooks
ontology.add_hook(KnowledgeHook { /* ... */ });

// Create validator
let validator = RdfOperationValidator::new().with_ontology(ontology);
```

### Operation Validation

```rust
// Validate operation exists
validator.validate_operation_defined("desk_review")?;

// Validate stage transition
validator.validate_stage_transition("submission", "desk_review")?;

// Validate latency
validator.validate_latency_budget("desk_review", 500)?;
```

### Guard Checking

```rust
let guards = validator.get_guards()?;
for guard in guards {
    if guard.guard_type == "Budget" {
        // Check budget constraints
    }
}
```

---

## Troubleshooting

### Ontology Not Loaded

**Error**: `OntologyNotLoaded`

**Solution**: Set ontology before validation:
```rust
let validator = RdfOperationValidator::new().with_ontology(ontology);
```

### Operation Not Defined

**Error**: `OperationNotDefined`

**Solution**: Add operation as hook:
```rust
ontology.add_hook(KnowledgeHook {
    id: "operation_id".to_string(),
    // ... other fields
});
```

### Invalid Stage Transition

**Error**: `InvalidStageTransition`

**Solution**: Ensure forward progression:
```rust
// Valid: stage 1 → stage 2
validator.validate_stage_transition("stage1", "stage2")?;

// Invalid: stage 2 → stage 1
let result = validator.validate_stage_transition("stage2", "stage1");
assert!(result.is_err());
```

### Latency Budget Exceeded

**Error**: `LatencyBudgetExceeded`

**Solution**: Ensure latency within budget:
```rust
// Stage max_latency_seconds: 1 (1000ms)
validator.validate_latency_budget("stage", 500)?; // OK
```

---

## Related Documentation

- **RDF Module**: `src/sector_stacks/rdf/`
- **Ontology Structures**: `src/sector_stacks/rdf/ontology.rs`
- **Validation**: `src/sector_stacks/rdf/validation.rs`
- **Release Notes**: `docs/releases/RELEASE_NOTES_v1.4.0.md`

---

## See Also

- [Sector Stacks Workflows](sector_stacks_workflows.md) - Production-grade workflows
- [Fail-Fast Verification](fail_fast_verification.md) - 12-phase verification
- [Operator Registry](operator_registry.md) - Pattern registration

---

**Quality is the default. Prevention beats detection.**

*Version 1.4.0 | Updated 2025-01-16 | Team KNHK | License MIT*

