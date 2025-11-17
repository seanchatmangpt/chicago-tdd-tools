//! # RDF Validation Example - Comprehensive Guide
//!
//! Demonstrates RDF-driven validation with ontologies as single source of truth.
//! Operations are validated against RDF ontology definitions at runtime.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through RDF-driven validation:
//!
//! 1. **Creating Ontology**: Build sector ontology with stages, guards, and hooks
//! 2. **Defining Workflow Stages**: Specify workflow stages with properties
//! 3. **Adding Guard Constraints**: Define safety constraints (Budget, Chronology, etc.)
//! 4. **Validating Operations**: Validate operations against ontology
//! 5. **Checking Guards**: Verify guard constraints are satisfied
//!
//! ## Explanation: Concepts
//!
//! **RDF as Single Source of Truth**: RDF ontologies define workflow structure,
//! stages, guards, and hooks. Runtime operations are validated against these
//! definitions, ensuring consistency between specification and implementation.
//!
//! **Workflow Stages**: Each stage has:
//! - Unique identifier
//! - Stage number (for ordering)
//! - Determinism flag
//! - Maximum latency budget
//!
//! **Guard Constraints**: Safety constraints that must be satisfied:
//! - **Legality**: Valid state transitions
//! - **Budget**: Resource limits (settlement ≤ policy_limit)
//! - **Chronology**: Temporal ordering
//! - **Causality**: Causal dependencies
//! - **Recursion**: Recursion depth limits
//!
//! **Knowledge Hooks**: Operations within workflows with input/output types.
//!
//! ## How-to: Common Tasks
//!
//! - Create ontology: See `example_create_ontology()`
//! - Define stages: See `example_define_stages()`
//! - Add guards: See `example_add_guards()`
//! - Validate operations: See `example_validate_operations()`
//! - Check stage transitions: See `example_stage_transitions()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `SectorOntology`: Container for workflow definitions
//! - `WorkflowStage`: Individual workflow stages
//! - `GuardConstraint`: Safety constraints
//! - `KnowledgeHook`: Operations within workflows
//! - `RdfOperationValidator`: Runtime validation against ontology
//!
//! **Key Methods**:
//! - `SectorOntology::new(sector: String) -> Self`
//! - `SectorOntology::add_stage(stage: WorkflowStage)`
//! - `RdfOperationValidator::with_ontology(ontology) -> Self`
//! - `RdfOperationValidator::validate_operation_defined(operation: &str) -> RdfValidationResult`

use chicago_tdd_tools::sector_stacks::rdf::*;

/// Example: Creating a sector ontology
///
/// ## How-to: Create Ontology
///
/// Creates a new sector ontology and adds workflow stages, guards, and hooks.
fn example_create_ontology() {
    println!("=== Example: Creating Sector Ontology ===");

    // Arrange: Create new ontology
    let mut ontology = SectorOntology::new("Academic".to_string());

    // Act: Add workflow stages
    ontology.add_stage(WorkflowStage {
        id: "submission".to_string(),
        name: "Submission".to_string(),
        stage_number: 1,
        is_deterministic: true,
        max_latency_seconds: 60,
    });

    ontology.add_stage(WorkflowStage {
        id: "desk_review".to_string(),
        name: "Desk Review".to_string(),
        stage_number: 2,
        is_deterministic: true,
        max_latency_seconds: 300,
    });

    ontology.add_stage(WorkflowStage {
        id: "reviewer_assignment".to_string(),
        name: "Reviewer Assignment".to_string(),
        stage_number: 3,
        is_deterministic: true,
        max_latency_seconds: 120,
    });

    // Assert: Verify ontology structure
    assert_eq!(ontology.sector, "Academic");
    assert_eq!(ontology.stage_count(), 3);
    println!("✓ Ontology created with {} stages", ontology.stage_count());
}

/// Example: Defining workflow stages
///
/// ## How-to: Define Stages
///
/// Demonstrates defining workflow stages with properties.
fn example_define_stages() {
    println!("\n=== Example: Defining Workflow Stages ===");

    let mut ontology = SectorOntology::new("Claims".to_string());

    // Add stages with different properties
    ontology.add_stage(WorkflowStage {
        id: "validation".to_string(),
        name: "Validation".to_string(),
        stage_number: 1,
        is_deterministic: true,
        max_latency_seconds: 30,
    });

    ontology.add_stage(WorkflowStage {
        id: "fraud_detection".to_string(),
        name: "Fraud Detection".to_string(),
        stage_number: 2,
        is_deterministic: true,
        max_latency_seconds: 60,
    });

    ontology.add_stage(WorkflowStage {
        id: "settlement".to_string(),
        name: "Settlement".to_string(),
        stage_number: 3,
        is_deterministic: true,
        max_latency_seconds: 45,
    });

    // Get deterministic stages
    let deterministic = ontology.deterministic_stages();
    println!("✓ Total stages: {}", ontology.stage_count());
    println!("✓ Deterministic stages: {}", deterministic.len());

    // Get specific stage
    if let Some(stage) = ontology.get_stage("validation") {
        println!("✓ Stage '{}' found: {}", stage.id, stage.name);
        println!("  Stage Number: {}", stage.stage_number);
        println!("  Deterministic: {}", stage.is_deterministic);
        println!("  Max Latency: {}s", stage.max_latency_seconds);
    }
}

/// Example: Adding guard constraints
///
/// ## How-to: Add Guards
///
/// Demonstrates adding guard constraints for safety validation.
fn example_add_guards() {
    println!("\n=== Example: Adding Guard Constraints ===");

    let mut ontology = SectorOntology::new("Claims".to_string());

    // Add Budget guard
    ontology.add_guard(GuardConstraint {
        id: "budget_guard".to_string(),
        guard_type: "Budget".to_string(),
        constraints: vec!["settlement <= policy_limit".to_string()],
    });

    // Add Chronology guard
    ontology.add_guard(GuardConstraint {
        id: "chronology_guard".to_string(),
        guard_type: "Chronology".to_string(),
        constraints: vec!["claim_date <= current_date".to_string()],
    });

    // Add Causality guard
    ontology.add_guard(GuardConstraint {
        id: "causality_guard".to_string(),
        guard_type: "Causality".to_string(),
        constraints: vec!["fraud_score determines entitlements".to_string()],
    });

    println!("✓ Total guards: {}", ontology.guard_count());

    // Display guard information
    for (id, guard) in &ontology.guards {
        println!(
            "  Guard '{}': Type={}, Constraints={:?}",
            id, guard.guard_type, guard.constraints
        );
    }
}

/// Example: Adding knowledge hooks
///
/// ## How-to: Add Hooks
///
/// Demonstrates adding knowledge hooks (operations) to ontology.
fn example_add_hooks() {
    println!("\n=== Example: Adding Knowledge Hooks ===");

    let mut ontology = SectorOntology::new("Academic".to_string());

    // Add knowledge hooks
    ontology.add_hook(KnowledgeHook {
        id: "desk_review_hook".to_string(),
        name: "Desk Review".to_string(),
        description: "Initial desk review of paper submission".to_string(),
        input_type: "PaperSubmission".to_string(),
        output_type: "ReviewResult".to_string(),
    });

    ontology.add_hook(KnowledgeHook {
        id: "reviewer_assignment_hook".to_string(),
        name: "Reviewer Assignment".to_string(),
        description: "Assign reviewers to paper".to_string(),
        input_type: "DeskReviewResult".to_string(),
        output_type: "ReviewerAssignment".to_string(),
    });

    ontology.add_hook(KnowledgeHook {
        id: "decision_hook".to_string(),
        name: "Decision".to_string(),
        description: "Make editorial decision based on reviews".to_string(),
        input_type: "ReviewCollection".to_string(),
        output_type: "Decision".to_string(),
    });

    println!("✓ Total hooks: {}", ontology.hook_count());

    // Display hook information
    for (id, hook) in &ontology.hooks {
        println!("  Hook '{}': {} -> {}", id, hook.input_type, hook.output_type);
    }
}

/// Example: Validating operations
///
/// ## How-to: Validate Operations
///
/// Demonstrates validating operations against ontology definitions.
fn example_validate_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Validating Operations ===");

    // Arrange: Create ontology with hooks
    let mut ontology = SectorOntology::new("Academic".to_string());
    ontology.add_hook(KnowledgeHook {
        id: "desk_review".to_string(),
        name: "Desk Review".to_string(),
        description: "Desk review operation".to_string(),
        input_type: "PaperSubmission".to_string(),
        output_type: "ReviewResult".to_string(),
    });

    // Act: Create validator with ontology
    let validator = RdfOperationValidator::new().with_ontology(ontology);

    // Validate defined operation
    let result = validator.validate_operation_defined("desk_review");
    match result {
        Ok(()) => {
            println!("✓ Operation 'desk_review' is defined in ontology");
        }
        Err(e) => {
            println!("✗ Validation failed: {}", e);
        }
    }

    // Validate undefined operation
    let result = validator.validate_operation_defined("undefined_operation");
    match result {
        Ok(()) => {
            println!("✓ Operation defined (unexpected)");
        }
        Err(e) => {
            println!("✗ Operation not defined (expected): {}", e);
        }
    }

    Ok(())
}

/// Example: Validating stage transitions
///
/// ## How-to: Stage Transitions
///
/// Demonstrates validating stage transitions (forward progression only).
fn example_stage_transitions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Validating Stage Transitions ===");

    // Arrange: Create ontology with stages
    let mut ontology = SectorOntology::new("Claims".to_string());
    ontology.add_stage(WorkflowStage {
        id: "validation".to_string(),
        name: "Validation".to_string(),
        stage_number: 1,
        is_deterministic: true,
        max_latency_seconds: 30,
    });
    ontology.add_stage(WorkflowStage {
        id: "fraud_detection".to_string(),
        name: "Fraud Detection".to_string(),
        stage_number: 2,
        is_deterministic: true,
        max_latency_seconds: 60,
    });
    ontology.add_stage(WorkflowStage {
        id: "settlement".to_string(),
        name: "Settlement".to_string(),
        stage_number: 3,
        is_deterministic: true,
        max_latency_seconds: 45,
    });

    let validator = RdfOperationValidator::new().with_ontology(ontology);

    // Valid: Forward transition
    println!("\n--- Valid: Forward Transition ---");
    let result = validator.validate_stage_transition("validation", "fraud_detection");
    match result {
        Ok(()) => {
            println!("✓ Forward transition valid: validation → fraud_detection");
        }
        Err(e) => {
            println!("✗ Transition failed: {}", e);
        }
    }

    // Valid: Same stage (no-op)
    println!("\n--- Valid: Same Stage ---");
    let result = validator.validate_stage_transition("validation", "validation");
    match result {
        Ok(()) => {
            println!("✓ Same stage transition valid");
        }
        Err(e) => {
            println!("✗ Transition failed: {}", e);
        }
    }

    // Invalid: Backward transition
    println!("\n--- Invalid: Backward Transition ---");
    let result = validator.validate_stage_transition("fraud_detection", "validation");
    match result {
        Ok(()) => {
            println!("✓ Transition valid (unexpected)");
        }
        Err(e) => {
            println!("✗ Backward transition rejected (expected): {}", e);
        }
    }

    Ok(())
}

/// Example: Validating latency budgets
///
/// ## How-to: Latency Validation
///
/// Demonstrates validating operation latency against stage budgets.
fn example_latency_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Validating Latency Budgets ===");

    // Arrange: Create ontology with latency budgets
    let mut ontology = SectorOntology::new("Academic".to_string());
    ontology.add_stage(WorkflowStage {
        id: "fast_stage".to_string(),
        name: "Fast Stage".to_string(),
        stage_number: 1,
        is_deterministic: true,
        max_latency_seconds: 1, // 1 second = 1000ms
    });
    ontology.add_stage(WorkflowStage {
        id: "slow_stage".to_string(),
        name: "Slow Stage".to_string(),
        stage_number: 2,
        is_deterministic: true,
        max_latency_seconds: 10, // 10 seconds = 10000ms
    });

    let validator = RdfOperationValidator::new().with_ontology(ontology);

    // Valid: Within budget
    println!("\n--- Valid: Within Budget ---");
    let result = validator.validate_latency_budget("fast_stage", 500); // 500ms < 1000ms
    match result {
        Ok(()) => {
            println!("✓ Latency 500ms within budget (1000ms)");
        }
        Err(e) => {
            println!("✗ Validation failed: {}", e);
        }
    }

    // Invalid: Exceeds budget
    println!("\n--- Invalid: Exceeds Budget ---");
    let result = validator.validate_latency_budget("fast_stage", 2000); // 2000ms > 1000ms
    match result {
        Ok(()) => {
            println!("✓ Latency within budget (unexpected)");
        }
        Err(e) => {
            println!("✗ Latency exceeds budget (expected): {}", e);
        }
    }

    // Valid: Within larger budget
    println!("\n--- Valid: Within Larger Budget ---");
    let result = validator.validate_latency_budget("slow_stage", 5000); // 5000ms < 10000ms
    match result {
        Ok(()) => {
            println!("✓ Latency 5000ms within budget (10000ms)");
        }
        Err(e) => {
            println!("✗ Validation failed: {}", e);
        }
    }

    Ok(())
}

/// Example: Checking guard constraints
///
/// ## How-to: Guard Validation
///
/// Demonstrates retrieving and checking guard constraints.
fn example_guard_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Guard Constraint Validation ===");

    // Arrange: Create ontology with guards
    let mut ontology = SectorOntology::new("Claims".to_string());
    ontology.add_guard(GuardConstraint {
        id: "budget_guard".to_string(),
        guard_type: "Budget".to_string(),
        constraints: vec!["settlement <= policy_limit".to_string()],
    });
    ontology.add_guard(GuardConstraint {
        id: "chronology_guard".to_string(),
        guard_type: "Chronology".to_string(),
        constraints: vec!["claim_date <= current_date".to_string()],
    });

    let validator = RdfOperationValidator::new().with_ontology(ontology);

    // Get all guards
    let guards = validator.get_guards()?;
    println!("✓ Retrieved {} guard constraints", guards.len());

    for guard in &guards {
        println!(
            "  Guard '{}': Type={}, Constraints={:?}",
            guard.id, guard.guard_type, guard.constraints
        );
    }

    Ok(())
}

/// Example: Checking determinism
///
/// ## How-to: Determinism Check
///
/// Demonstrates checking if all stages are deterministic.
fn example_determinism_check() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Determinism Check ===");

    // Test Case 1: All deterministic
    println!("\n--- Test Case 1: All Stages Deterministic ---");
    let mut ontology = SectorOntology::new("Academic".to_string());
    ontology.add_stage(WorkflowStage {
        id: "stage1".to_string(),
        name: "Stage 1".to_string(),
        stage_number: 1,
        is_deterministic: true,
        max_latency_seconds: 30,
    });
    ontology.add_stage(WorkflowStage {
        id: "stage2".to_string(),
        name: "Stage 2".to_string(),
        stage_number: 2,
        is_deterministic: true,
        max_latency_seconds: 60,
    });

    let validator = RdfOperationValidator::new().with_ontology(ontology);
    let all_deterministic = validator.all_stages_deterministic()?;
    println!("✓ All stages deterministic: {}", all_deterministic);

    // Test Case 2: Mixed deterministic/non-deterministic
    println!("\n--- Test Case 2: Mixed Determinism ---");
    let mut ontology = SectorOntology::new("Claims".to_string());
    ontology.add_stage(WorkflowStage {
        id: "stage1".to_string(),
        name: "Stage 1".to_string(),
        stage_number: 1,
        is_deterministic: true,
        max_latency_seconds: 30,
    });
    ontology.add_stage(WorkflowStage {
        id: "stage2".to_string(),
        name: "Stage 2".to_string(),
        stage_number: 2,
        is_deterministic: false, // Non-deterministic
        max_latency_seconds: 60,
    });

    let validator = RdfOperationValidator::new().with_ontology(ontology);
    let all_deterministic = validator.all_stages_deterministic()?;
    println!("✓ All stages deterministic: {}", all_deterministic);

    Ok(())
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  RDF Validation - Ontology-Driven Operation Validation       ║");
    println!("║  Single Source of Truth for Workflow Definitions             ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Run examples
    example_create_ontology();
    example_define_stages();
    example_add_guards();
    example_add_hooks();

    if let Err(e) = example_validate_operations() {
        eprintln!("Error in operation validation: {}", e);
    }

    if let Err(e) = example_stage_transitions() {
        eprintln!("Error in stage transitions: {}", e);
    }

    if let Err(e) = example_latency_validation() {
        eprintln!("Error in latency validation: {}", e);
    }

    if let Err(e) = example_guard_validation() {
        eprintln!("Error in guard validation: {}", e);
    }

    if let Err(e) = example_determinism_check() {
        eprintln!("Error in determinism check: {}", e);
    }

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║  All Examples Completed Successfully!                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use chicago_tdd_tools::sector_stacks::rdf::*;
    use chicago_tdd_tools::test;

    test!(test_ontology_creation, {
        // Arrange & Act
        let ontology = SectorOntology::new("Test".to_string());

        // Assert
        assert_eq!(ontology.sector, "Test");
        assert_eq!(ontology.stage_count(), 0);
    });

    test!(test_add_stages_and_guards, {
        // Arrange
        let mut ontology = SectorOntology::new("Test".to_string());

        // Act
        ontology.add_stage(WorkflowStage {
            id: "stage1".to_string(),
            name: "Stage 1".to_string(),
            stage_number: 1,
            is_deterministic: true,
            max_latency_seconds: 30,
        });

        ontology.add_guard(GuardConstraint {
            id: "guard1".to_string(),
            guard_type: "Budget".to_string(),
            constraints: vec!["x <= 100".to_string()],
        });

        // Assert
        assert_eq!(ontology.stage_count(), 1);
        assert_eq!(ontology.guard_count(), 1);
    });

    test!(test_operation_validation, {
        // Arrange
        let mut ontology = SectorOntology::new("Test".to_string());
        ontology.add_hook(KnowledgeHook {
            id: "test_operation".to_string(),
            name: "Test Operation".to_string(),
            description: "Test".to_string(),
            input_type: "Input".to_string(),
            output_type: "Output".to_string(),
        });

        let validator = RdfOperationValidator::new().with_ontology(ontology);

        // Act & Assert
        assert!(validator.validate_operation_defined("test_operation").is_ok());
        assert!(validator.validate_operation_defined("undefined").is_err());
    });
}
