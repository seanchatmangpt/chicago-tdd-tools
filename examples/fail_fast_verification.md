# Fail-Fast Verification Example

**Category:** How-To Guide  
**Level:** Intermediate  
**Prerequisites:** Basic understanding of Chicago TDD, test contracts  
**Features Required:** None

---

## Overview

This example demonstrates the 12-phase fail-fast verification pipeline with zero-tolerance invariant checking. Every invariant violation causes immediate test failure - no degradation, no warnings ignored, no partial success.

**What you'll learn:**
- Creating strict execution contexts with zero-tolerance checking
- Executing all 12 phases of the verification pipeline
- Understanding fail-fast semantics and violation handling
- Finalizing execution contexts and verifying phase completion

---

## Quick Start

```bash
cargo run --example fail_fast_verification
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- Understanding of test contracts and invariants

---

## Tutorial: Getting Started

### Step 1: Create Execution Context

Create a strict execution context with a valid contract ID:

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::core::fail_fast::*;

let ctx = StrictExecutionContext::new("contract-001".to_string())?;
```

**Key Points:**
- Contract ID must be non-empty
- Context validates contract ID on creation
- Returns `InvariantResult<StrictExecutionContext>`

### Step 2: Execute Phases

Execute phases in sequence, checking results:

```rust
// Phase 1: Contract Definition
ctx.phase_1_contract_definition(12)?;

// Phase 2: Thermal Testing (τ ≤ 8 enforced)
ctx.phase_2_thermal_testing(5, 8)?;

// Phase 3: Effects Tracking
let declared = vec!["NetworkRead".to_string()];
let observed = vec!["NetworkRead".to_string()];
ctx.phase_3_effects_tracking(declared, &observed)?;
```

**Key Points:**
- Each phase returns `InvariantResult<PhaseResult>`
- `PhaseResult::Ok` indicates success
- `PhaseResult::Violation` indicates invariant violation
- Violations cause immediate failure

### Step 3: Handle Violations

Fail-fast semantics mean violations cause immediate failure:

```rust
match ctx.phase_2_thermal_testing(10, 8) {
    Ok(PhaseResult::Violation(v)) => {
        // Test fails immediately - no degradation
        panic!("Thermal bound exceeded: {}", v);
    }
    Ok(PhaseResult::Ok) => {
        // Success
    }
    Err(e) => {
        // Error (e.g., clock backward)
    }
}
```

### Step 4: Finalize Context

Verify all required phases completed:

```rust
ctx.finalize()?;
```

**Required Phases:**
- Contract Definition
- Thermal Testing
- Receipt Generation
- Verification Pipeline

---

## How-To: Common Tasks

### Execute All 12 Phases

```rust
let mut ctx = StrictExecutionContext::new("contract-002".to_string())?;

// Phase 1: Contract Definition
ctx.phase_1_contract_definition(12)?;

// Phase 2: Thermal Testing
ctx.phase_2_thermal_testing(5, 8)?;

// Phase 3: Effects Tracking
ctx.phase_3_effects_tracking(declared, observed)?;

// Phase 4: State Machine
ctx.phase_4_state_machine(initial_state, all_states)?;

// Phase 5: Receipt Generation
ctx.phase_5_receipt_generation(1, checksum, computed)?;

// Phase 6: Swarm Orchestration
ctx.phase_6_swarm_orchestration(scheduled, executed)?;

// Phase 7: Verification Pipeline
ctx.phase_7_verification_pipeline(&expected_phases)?;

// Phase 8: Continuous Learning
ctx.phase_8_continuous_learning(sample_count, prediction)?;

// Phase 9: Distributed Consensus
ctx.phase_9_distributed_consensus(approval_votes, total_votes)?;

// Phase 10: Time-Travel Debugging
ctx.phase_10_time_travel_debugging(snapshot_version, expected_version)?;

// Phase 11: Performance Prophet
ctx.phase_11_performance_prophet(predicted_tau, confidence)?;

// Phase 12: Quality Dashboard
ctx.phase_12_quality_dashboard(total, passed, failed)?;

// Finalize
ctx.finalize()?;
```

### Handle Thermal Violations

Thermal testing enforces τ (tau) bounds and monotonicity:

```rust
// Valid: τ within bound and monotonic
ctx.phase_2_thermal_testing(100, 10_000)?; // τ=100 ≤ 10,000
ctx.phase_2_thermal_testing(150, 10_000)?; // τ=150 (monotonic increase)

// Violation: Clock going backward
let result = ctx.phase_2_thermal_testing(50, 10_000); // τ decreased
match result {
    Err(e) => {
        // Clock backward violation
    }
    _ => {}
}
```

### Handle Consensus Deadlock

Distributed consensus requires 2/3 Byzantine quorum:

```rust
// Valid: Sufficient quorum
ctx.phase_9_distributed_consensus(7, 9)?; // 7 >= 7 (2/3 of 9)

// Violation: Insufficient quorum
let result = ctx.phase_9_distributed_consensus(5, 9); // 5 < 7
match result {
    Err(e) => {
        // Consensus deadlock
    }
    _ => {}
}
```

### Verify Dashboard Consistency

Quality dashboard verifies totals add up:

```rust
// Valid: Totals match
ctx.phase_12_quality_dashboard(100, 95, 5)?; // 95 + 5 = 100

// Violation: Totals don't match
let result = ctx.phase_12_quality_dashboard(100, 95, 3); // 95 + 3 ≠ 100
match result {
    Err(e) => {
        // Dashboard inconsistency
    }
    _ => {}
}
```

---

## Explanation: Concepts

### Fail-Fast Semantics

**Zero Tolerance**: Every invariant violation causes immediate test failure. No graceful degradation, no warnings that are ignored, no partial success.

**Why Fail-Fast?**
- Prevents test degradation over time
- Ensures invariants are always enforced
- Catches violations early before they compound
- Maintains framework integrity

### 12-Phase Pipeline

The complete verification pipeline covers:

1. **Contract Definition**: Verify contracts are completely specified
2. **Thermal Testing**: Validate τ measurement monotonicity and bounds
3. **Effects Tracking**: Verify observed effects match declared effects
4. **State Machine**: Validate state transitions
5. **Receipt Generation**: Store and validate receipts with checksums
6. **Swarm Orchestration**: Ensure all tests execute
7. **Verification Pipeline**: Verify all configured phases executed
8. **Continuous Learning**: Validate learner state consistency
9. **Distributed Consensus**: Verify 2/3 Byzantine quorum
10. **Time-Travel Debugging**: Validate snapshot integrity
11. **Performance Prophet**: Verify prediction self-checks
12. **Quality Dashboard**: Verify dashboard consistency

### PhaseResult Type

Unified result type for phase execution:

```rust
pub enum PhaseResult {
    /// Phase executed successfully and all invariants hold
    Ok,
    /// Phase encountered an unrecoverable invariant violation
    Violation(UnrecoverableInvariantViolation),
}
```

**Key Methods:**
- `is_ok()`: Returns true if phase succeeded
- `is_violation()`: Returns true if violation detected
- `into_result()`: Converts to standard `Result` type

### Invariant Violations

47 invariant violation types cover all failure modes across 12 phases:

- **Contract Violations**: Malformed contracts, duplicate IDs, invalid sequences
- **Thermal Violations**: Clock backward, monster jumps, measurement failures
- **Effect Violations**: Unobserved effects, lost effects, composition errors
- **State Violations**: Unhandled events, dead states, invalid transitions
- **Receipt Violations**: Missing receipts, corrupted checksums, version mismatches
- **And more...**

---

## Reference: Quick Lookup

### StrictExecutionContext

**Creation:**
```rust
pub fn new(contract_id: String) -> InvariantResult<Self>
```

**Phase Methods:**
```rust
pub fn phase_1_contract_definition(&mut self, phase_count: usize) -> InvariantResult<PhaseResult>
pub fn phase_2_thermal_testing(&mut self, tau: u64, max_tau_bound: u64) -> InvariantResult<PhaseResult>
pub fn phase_3_effects_tracking(&mut self, declared: Vec<String>, observed: Vec<String>) -> InvariantResult<PhaseResult>
pub fn phase_4_state_machine(&mut self, initial_state: String, all_states: Vec<String>) -> InvariantResult<PhaseResult>
pub fn phase_5_receipt_generation(&mut self, version: u32, checksum: u32, computed: u32) -> InvariantResult<PhaseResult>
pub fn phase_6_swarm_orchestration(&mut self, scheduled: usize, executed: usize) -> InvariantResult<PhaseResult>
pub fn phase_7_verification_pipeline(&mut self, expected_phases: &[PhaseLabel]) -> InvariantResult<PhaseResult>
pub fn phase_8_continuous_learning(&mut self, sample_count: usize, prediction: f64) -> InvariantResult<PhaseResult>
pub fn phase_9_distributed_consensus(&mut self, approval_votes: usize, total_votes: usize) -> InvariantResult<PhaseResult>
pub fn phase_10_time_travel_debugging(&mut self, snapshot_version: u32, expected_version: u32) -> InvariantResult<PhaseResult>
pub fn phase_11_performance_prophet(&mut self, predicted_tau: u64, confidence: f64) -> InvariantResult<PhaseResult>
pub fn phase_12_quality_dashboard(&mut self, total: usize, passed: usize, failed: usize) -> InvariantResult<PhaseResult>
```

**Finalization:**
```rust
pub fn finalize(&self) -> InvariantResult<()>
```

### PhaseLabel

12 distinct phase labels:

```rust
pub enum PhaseLabel {
    ContractDefinition,
    ThermalTesting,
    EffectsTracking,
    StateMachine,
    ReceiptGeneration,
    SwarmOrchestration,
    VerificationPipeline,
    ContinuousLearning,
    DistributedConsensus,
    TimeTravelDebugging,
    PerformanceProphet,
    QualityDashboard,
}
```

### PhaseResult

Unified result type:

```rust
pub enum PhaseResult {
    Ok,
    Violation(UnrecoverableInvariantViolation),
}
```

**Methods:**
- `is_ok() -> bool`
- `is_violation() -> bool`
- `into_result() -> InvariantResult<()>`

---

## Common Patterns

### Successful Execution

```rust
let mut ctx = StrictExecutionContext::new("contract-001".to_string())?;
ctx.phase_1_contract_definition(12)?;
ctx.phase_2_thermal_testing(5, 8)?;
ctx.phase_5_receipt_generation(1, 0x1234, 0x1234)?;
ctx.finalize()?;
```

### Violation Detection

```rust
match ctx.phase_2_thermal_testing(10, 8) {
    Ok(PhaseResult::Violation(v)) => {
        // Handle violation - test fails immediately
        panic!("Violation: {}", v);
    }
    Ok(PhaseResult::Ok) => {
        // Success
    }
    Err(e) => {
        // Error (e.g., clock backward)
    }
}
```

### Phase Verification

```rust
let expected_phases = vec![
    PhaseLabel::ContractDefinition,
    PhaseLabel::ThermalTesting,
    PhaseLabel::ReceiptGeneration,
];
ctx.phase_7_verification_pipeline(&expected_phases)?;
```

---

## Troubleshooting

### Context Creation Fails

**Error**: Contract ID validation fails

**Solution**: Ensure contract ID is non-empty and valid:
```rust
let ctx = StrictExecutionContext::new("valid-contract-id".to_string())?;
```

### Thermal Violation

**Error**: Clock going backward or bound exceeded

**Solution**: Ensure τ values are monotonic and within bounds:
```rust
// Valid progression
ctx.phase_2_thermal_testing(100, 10_000)?;
ctx.phase_2_thermal_testing(150, 10_000)?; // Monotonic increase
```

### Consensus Deadlock

**Error**: Insufficient quorum

**Solution**: Ensure approval votes meet 2/3 threshold:
```rust
// For 9 voters, need 7 approvals (2/3 of 9)
ctx.phase_9_distributed_consensus(7, 9)?;
```

### Finalization Fails

**Error**: Required phases not completed

**Solution**: Execute all required phases before finalizing:
```rust
ctx.phase_1_contract_definition(12)?;
ctx.phase_2_thermal_testing(5, 8)?;
ctx.phase_5_receipt_generation(1, 0x1234, 0x1234)?;
ctx.phase_7_verification_pipeline(&expected_phases)?;
ctx.finalize()?;
```

---

## Related Documentation

- **Fail-Fast Infrastructure**: `src/core/fail_fast.rs`
- **Invariant Violations**: `src/core/invariants.rs`
- **Release Notes**: `docs/releases/RELEASE_NOTES_v1.4.0.md`

---

## See Also

- [Sector Stacks Workflows](sector_stacks_workflows.md) - Production-grade workflows
- [RDF Validation](rdf_validation.md) - RDF-driven validation
- [Operator Registry](operator_registry.md) - Pattern registration

---

**Quality is the default. Prevention beats detection.**

*Version 1.4.0 | Updated 2025-01-XX | Team KNHK | License MIT*

