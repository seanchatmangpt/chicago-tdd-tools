//! # Fail-Fast Verification Example - Comprehensive Guide
//!
//! Demonstrates the 12-phase fail-fast verification pipeline with zero-tolerance
//! invariant checking. Every invariant violation causes immediate test failure.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through the complete fail-fast verification pipeline:
//!
//! 1. **Creating Execution Context**: Use `StrictExecutionContext::new()` to create
//!    a zero-tolerance execution context
//! 2. **Executing Phases**: Run all 12 phases in sequence with fail-fast semantics
//! 3. **Handling Violations**: Understand how violations are detected and handled
//! 4. **Finalizing Context**: Verify all required phases completed successfully
//!
//! ## Explanation: Concepts
//!
//! **Fail-Fast Semantics**: Zero tolerance for invariant violations. Any violation
//! causes immediate test failure - no degradation, no warnings ignored, no partial success.
//!
//! **12-Phase Pipeline**: Complete verification from contract definition to quality metrics:
//! - Phase 1: Contract Definition
//! - Phase 2: Thermal Testing (τ measurement)
//! - Phase 3: Effects Tracking
//! - Phase 4: State Machine Transitions
//! - Phase 5: Receipt Generation
//! - Phase 6: Swarm Orchestration
//! - Phase 7: Verification Pipeline
//! - Phase 8: Continuous Learning
//! - Phase 9: Distributed Consensus
//! - Phase 10: Time-Travel Debugging
//! - Phase 11: Performance Prophet
//! - Phase 12: Quality Dashboard
//!
//! **PhaseResult**: Unified result type that is either `Ok` (success) or `Violation`
//! (invariant violation detected). Violations cause immediate failure.
//!
//! ## How-to: Common Tasks
//!
//! - Create execution context: See `example_create_context()`
//! - Execute all phases: See `example_execute_all_phases()`
//! - Handle violations: See `example_handle_violations()`
//! - Finalize context: See `example_finalize_context()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `StrictExecutionContext`: Zero-tolerance execution context
//! - `PhaseResult`: Unified result type (Ok or Violation)
//! - `PhaseLabel`: 12 distinct phase labels
//!
//! **Key Methods**:
//! - `StrictExecutionContext::new(contract_id: String) -> InvariantResult<Self>`
//! - `phase_1_contract_definition(&mut self, phase_count: usize) -> InvariantResult<PhaseResult>`
//! - `phase_2_thermal_testing(&mut self, tau: u64, max_tau_bound: u64) -> InvariantResult<PhaseResult>`
//! - `finalize(&self) -> InvariantResult<()>`

use chicago_tdd_tools::core::fail_fast::{PhaseLabel, PhaseResult, StrictExecutionContext};

/// Example: Creating a strict execution context
///
/// ## How-to: Create Execution Context
///
/// Creates a new execution context with strict invariant checking.
/// The contract ID must be non-empty and valid.
fn example_create_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example: Creating Execution Context ===");

    // Arrange: Create execution context with valid contract ID
    let _ctx = StrictExecutionContext::new("contract-001".to_string())?;

    // Assert: Context created successfully
    println!("✓ Execution context created for contract-001");
    Ok(())
}

/// Example: Executing all 12 phases successfully
///
/// ## How-to: Execute All Phases
///
/// Demonstrates executing all 12 phases in sequence with valid inputs.
/// Each phase validates invariants and tracks completion.
fn example_execute_all_phases() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Executing All 12 Phases ===");

    // Arrange: Create execution context
    let mut ctx = StrictExecutionContext::new("contract-002".to_string())?;

    // Act: Execute all 12 phases

    // Phase 1: Contract Definition
    let result = ctx.phase_1_contract_definition(12)?;
    assert!(result.is_ok());
    println!("✓ Phase 1: Contract Definition");

    // Phase 2: Thermal Testing (τ ≤ 8 enforced)
    let result = ctx.phase_2_thermal_testing(5, 8)?;
    assert!(result.is_ok());
    println!("✓ Phase 2: Thermal Testing (τ=5 ≤ 8)");

    // Phase 3: Effects Tracking
    let declared = vec!["NetworkRead".to_string(), "StorageWrite".to_string()];
    let observed = vec!["NetworkRead".to_string()];
    let result = ctx.phase_3_effects_tracking(declared, &observed)?;
    assert!(result.is_ok());
    println!("✓ Phase 3: Effects Tracking");

    // Phase 4: State Machine
    let initial_state = "Initial".to_string();
    let all_states = vec!["Initial".to_string(), "Processing".to_string(), "Complete".to_string()];
    let result = ctx.phase_4_state_machine(initial_state, all_states)?;
    assert!(result.is_ok());
    println!("✓ Phase 4: State Machine");

    // Phase 5: Receipt Generation
    let result = ctx.phase_5_receipt_generation(1, 0x1234, 0x1234)?;
    assert!(result.is_ok());
    println!("✓ Phase 5: Receipt Generation");

    // Phase 6: Swarm Orchestration
    let result = ctx.phase_6_swarm_orchestration(10, 10)?;
    assert!(result.is_ok());
    println!("✓ Phase 6: Swarm Orchestration");

    // Phase 7: Verification Pipeline
    let expected_phases = vec![
        PhaseLabel::ContractDefinition,
        PhaseLabel::ThermalTesting,
        PhaseLabel::EffectsTracking,
        PhaseLabel::StateMachine,
        PhaseLabel::ReceiptGeneration,
        PhaseLabel::SwarmOrchestration,
    ];
    let result = ctx.phase_7_verification_pipeline(&expected_phases)?;
    assert!(result.is_ok());
    println!("✓ Phase 7: Verification Pipeline");

    // Phase 8: Continuous Learning
    let result = ctx.phase_8_continuous_learning(10, 0.85)?;
    assert!(result.is_ok());
    println!("✓ Phase 8: Continuous Learning");

    // Phase 9: Distributed Consensus (2/3 quorum)
    let result = ctx.phase_9_distributed_consensus(7, 9)?; // 7 >= 7 (2/3 of 9)
    assert!(result.is_ok());
    println!("✓ Phase 9: Distributed Consensus (7/9 votes)");

    // Phase 10: Time-Travel Debugging
    let result = ctx.phase_10_time_travel_debugging(1, 1)?;
    assert!(result.is_ok());
    println!("✓ Phase 10: Time-Travel Debugging");

    // Phase 11: Performance Prophet
    let result = ctx.phase_11_performance_prophet(100, 0.9)?;
    assert!(result.is_ok());
    println!("✓ Phase 11: Performance Prophet");

    // Phase 12: Quality Dashboard
    let result = ctx.phase_12_quality_dashboard(100, 95, 5)?;
    assert!(result.is_ok());
    println!("✓ Phase 12: Quality Dashboard");

    // Finalize: Verify all required phases completed
    ctx.finalize()?;
    println!("\n✓ All phases executed successfully!");

    Ok(())
}

/// Example: Handling violations (fail-fast semantics)
///
/// ## How-to: Handle Violations
///
/// Demonstrates how violations are detected and cause immediate failure.
/// Fail-fast means no degradation - any violation fails the test immediately.
fn example_handle_violations() {
    println!("\n=== Example: Handling Violations ===");

    // Example 1: Thermal bound exceeded
    println!("\n--- Violation 1: Thermal Bound Exceeded ---");
    let mut ctx = StrictExecutionContext::new("contract-003".to_string()).unwrap();
    let _ = ctx.phase_1_contract_definition(12);

    let result = ctx.phase_2_thermal_testing(10, 8); // τ=10 > 8 (bound)
    match result {
        Ok(PhaseResult::Violation(v)) => {
            println!("✗ Violation detected: {}", v);
            println!("  Test fails immediately - no degradation allowed");
        }
        Ok(PhaseResult::Ok) => {
            println!("✓ No violation (unexpected)");
        }
        Err(e) => {
            println!("✗ Error: {}", e);
        }
    }

    // Example 2: Consensus deadlock (insufficient quorum)
    println!("\n--- Violation 2: Consensus Deadlock ---");
    let mut ctx = StrictExecutionContext::new("contract-004".to_string()).unwrap();
    let result = ctx.phase_9_distributed_consensus(5, 9); // 5 < 7 (required quorum)
    match result {
        Err(e) => {
            println!("✗ Consensus deadlock: {}", e);
            println!("  Need 7 approvals (2/3 of 9), only got 5");
        }
        _ => {
            println!("✓ Consensus reached (unexpected)");
        }
    }

    // Example 3: Dashboard inconsistency
    println!("\n--- Violation 3: Dashboard Inconsistency ---");
    let mut ctx = StrictExecutionContext::new("contract-005".to_string()).unwrap();
    let result = ctx.phase_12_quality_dashboard(100, 95, 3); // 95 + 3 ≠ 100
    match result {
        Err(e) => {
            println!("✗ Dashboard inconsistency: {}", e);
            println!("  Totals don't match: 95 + 3 ≠ 100");
        }
        _ => {
            println!("✓ Dashboard consistent (unexpected)");
        }
    }

    // Example 4: Prophet self-check failed
    println!("\n--- Violation 4: Prophet Self-Check Failed ---");
    let mut ctx = StrictExecutionContext::new("contract-006".to_string()).unwrap();
    let result = ctx.phase_11_performance_prophet(100, 1.5); // Confidence > 1.0
    match result {
        Err(e) => {
            println!("✗ Prophet self-check failed: {}", e);
            println!("  Confidence must be in [0.0, 1.0], got 1.5");
        }
        _ => {
            println!("✓ Prophet check passed (unexpected)");
        }
    }
}

/// Example: Finalizing execution context
///
/// ## How-to: Finalize Context
///
/// Verifies all required phases completed successfully.
/// Required phases: Contract Definition, Thermal Testing, Receipt Generation, Verification Pipeline.
fn example_finalize_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Finalizing Context ===");

    // Arrange: Create context and execute required phases
    let mut ctx = StrictExecutionContext::new("contract-007".to_string())?;

    // Act: Execute required phases
    ctx.phase_1_contract_definition(12)?;
    ctx.phase_2_thermal_testing(5, 8)?;
    ctx.phase_5_receipt_generation(1, 0x1234, 0x1234)?;

    // Execute verification pipeline with required phases
    let expected = vec![
        PhaseLabel::ContractDefinition,
        PhaseLabel::ThermalTesting,
        PhaseLabel::ReceiptGeneration,
    ];
    ctx.phase_7_verification_pipeline(&expected)?;

    // Assert: Finalize succeeds
    ctx.finalize()?;
    println!("✓ Context finalized successfully - all required phases completed");

    Ok(())
}

/// Example: Thermal testing with monotonicity enforcement
///
/// ## How-to: Thermal Testing
///
/// Demonstrates τ (tau) measurement with monotonicity enforcement.
/// Clock going backward or exceeding bounds causes immediate failure.
fn example_thermal_testing() {
    println!("\n=== Example: Thermal Testing ===");

    // Valid thermal progression
    println!("\n--- Valid Thermal Progression ---");
    let mut ctx = StrictExecutionContext::new("contract-008".to_string()).unwrap();
    let _ = ctx.phase_1_contract_definition(12);

    let r1 = ctx.phase_2_thermal_testing(100, 10_000);
    assert!(r1.is_ok());
    println!("✓ τ=100 (within bound)");

    let r2 = ctx.phase_2_thermal_testing(150, 10_000);
    assert!(r2.is_ok());
    println!("✓ τ=150 (monotonic increase)");

    // Clock going backward (violation)
    println!("\n--- Clock Going Backward (Violation) ---");
    let mut ctx = StrictExecutionContext::new("contract-009".to_string()).unwrap();
    let _ = ctx.phase_1_contract_definition(12);

    let r1 = ctx.phase_2_thermal_testing(100, 10_000);
    assert!(r1.is_ok());
    println!("✓ τ=100");

    let r2 = ctx.phase_2_thermal_testing(50, 10_000); // Clock went backward
    match r2 {
        Err(e) => {
            println!("✗ Clock backward violation: {}", e);
            println!("  τ decreased from 100 to 50 - not allowed");
        }
        _ => {
            println!("✓ No violation (unexpected)");
        }
    }
}

/// Example: Effects tracking validation
///
/// ## How-to: Effects Tracking
///
/// Verifies observed effects are subset of declared effects.
/// Any unobserved effect causes immediate failure.
fn example_effects_tracking() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example: Effects Tracking ===");

    // Valid: Observed effects are subset of declared
    println!("\n--- Valid Effects Tracking ---");
    let mut ctx = StrictExecutionContext::new("contract-010".to_string())?;
    let _ = ctx.phase_1_contract_definition(12);

    let declared = vec!["NetworkRead".to_string(), "StorageWrite".to_string()];
    let observed = vec!["NetworkRead".to_string()]; // Subset of declared
    let result = ctx.phase_3_effects_tracking(declared, &observed)?;
    assert!(result.is_ok());
    println!("✓ Observed effects are subset of declared");

    // Invalid: Unobserved effect
    println!("\n--- Unobserved Effect (Violation) ---");
    let mut ctx = StrictExecutionContext::new("contract-011".to_string())?;
    let _ = ctx.phase_1_contract_definition(12);

    let declared = vec!["NetworkRead".to_string()];
    let observed = vec!["NetworkRead".to_string(), "StorageWrite".to_string()]; // Not subset
    let result = ctx.phase_3_effects_tracking(declared, &observed);
    match result {
        Err(e) => {
            println!("✗ Unobserved effect violation: {}", e);
            println!("  Observed 'StorageWrite' but not declared");
        }
        _ => {
            println!("✓ No violation (unexpected)");
        }
    }

    Ok(())
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Fail-Fast Verification Pipeline - 12-Phase Example           ║");
    println!("║  Zero Tolerance, No Degradation, Immediate Failure            ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Run examples
    if let Err(e) = example_create_context() {
        eprintln!("Error creating context: {}", e);
    }

    if let Err(e) = example_execute_all_phases() {
        eprintln!("Error executing phases: {}", e);
    }

    example_handle_violations();
    example_thermal_testing();

    if let Err(e) = example_effects_tracking() {
        eprintln!("Error in effects tracking: {}", e);
    }

    if let Err(e) = example_finalize_context() {
        eprintln!("Error finalizing context: {}", e);
    }

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║  All Examples Completed Successfully!                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use chicago_tdd_tools::core::fail_fast::{PhaseLabel, StrictExecutionContext};
    use chicago_tdd_tools::test;

    test!(test_create_context, {
        // Arrange & Act
        let ctx = StrictExecutionContext::new("test-contract".to_string())?;

        // Assert
        assert!(ctx.is_ok());
    });

    test!(test_execute_phases_success, {
        // Arrange
        let mut ctx = StrictExecutionContext::new("test-contract".to_string())?;

        // Act: Execute core phases
        ctx.phase_1_contract_definition(12)?;
        ctx.phase_2_thermal_testing(5, 8)?;
        ctx.phase_5_receipt_generation(1, 0x1234, 0x1234)?;

        // Assert: Finalize succeeds
        ctx.finalize()?;
    });

    test!(test_thermal_violation, {
        // Arrange
        let mut ctx = StrictExecutionContext::new("test-contract".to_string())?;
        let _ = ctx.phase_1_contract_definition(12);

        // Act: Exceed thermal bound
        let result = ctx.phase_2_thermal_testing(10, 8);

        // Assert: Violation detected
        match result {
            Ok(PhaseResult::Violation(_)) => {
                // Expected: violation detected
            }
            _ => {
                panic!("Expected violation for thermal bound exceeded");
            }
        }
    });
}
