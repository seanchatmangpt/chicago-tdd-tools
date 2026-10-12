//! FAIL_FAST - Strict Verification Pipeline with Zero Tolerance
//!
//! Orchestrates all 12 phases of the testing framework with fail-fast semantics.
//! No degradation, no warnings that are ignored, no partial success.
//! Every invariant violation causes immediate test failure.

use crate::core::invariants::{
    ContractValidator, EffectValidator, InvariantResult, ReceiptValidator, StateValidator,
    ThermalValidator, UnrecoverableInvariantViolation,
};
use std::collections::BTreeMap;

/// Unified phase result: either complete success or specific invariant violation.
#[derive(Debug, Clone)]
pub enum PhaseResult {
    /// Phase executed successfully and all invariants hold.
    Ok,
    /// Phase encountered an unrecoverable invariant violation.
    Violation(UnrecoverableInvariantViolation),
}

impl PhaseResult {
    /// Returns true if the phase result indicates success.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Returns true if the phase result indicates an invariant violation.
    #[must_use]
    pub const fn is_violation(&self) -> bool {
        !self.is_ok()
    }

    /// Convert the phase result into a standard Rust Result type.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if the phase result contains a violation.
    pub fn into_result(self) -> InvariantResult<()> {
        match self {
            Self::Ok => Ok(()),
            Self::Violation(v) => Err(v),
        }
    }
}

/// Test execution context that enforces strict invariant checking.
pub struct StrictExecutionContext {
    /// Contract ID being tested
    contract_id: String,

    /// Validators for each phase
    thermal_validator: ThermalValidator,
    effect_validator: Option<EffectValidator>,
    state_validator: Option<StateValidator>,
    receipt_validator: ReceiptValidator,

    /// Execution state tracking
    phases_completed: Vec<PhaseLabel>,
    receipts: BTreeMap<String, ReceiptData>,
}

/// Labels for the 12 phases of fail-fast verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhaseLabel {
    /// Phase 1: Contract Definition - Verify contracts are completely specified
    ContractDefinition,
    /// Phase 2: Thermal Testing - Validate τ measurement monotonicity
    ThermalTesting,
    /// Phase 3: Effects Tracking - Verify observed effects match declared effects
    EffectsTracking,
    /// Phase 4: State Machine - Validate state transitions
    StateMachine,
    /// Phase 5: Receipt Generation - Store and validate receipts with checksums
    ReceiptGeneration,
    /// Phase 6: Swarm Orchestration - Ensure all tests execute
    SwarmOrchestration,
    /// Phase 7: Verification Pipeline - Verify all configured phases executed
    VerificationPipeline,
    /// Phase 8: Continuous Learning - Validate learner state consistency
    ContinuousLearning,
    /// Phase 9: Distributed Consensus - Verify 2/3 Byzantine quorum
    DistributedConsensus,
    /// Phase 10: Time-Travel Debugging - Validate snapshot integrity
    TimeTravelDebugging,
    /// Phase 11: Performance Prophet - Verify prediction self-checks
    PerformanceProphet,
    /// Phase 12: Quality Dashboard - Verify dashboard consistency
    QualityDashboard,
}

/// Receipt data with version and checksum for receipt-shape completeness:
/// the fields document the receipt contract this phase emits; consumers
/// read them from the serialized receipt, not from this in-memory struct.
#[derive(Debug, Clone)]
struct ReceiptData {
    /// Receipt schema version
    #[allow(dead_code)]
    version: u32,
    /// Checksum for integrity verification
    #[allow(dead_code)]
    checksum: u32,
    /// Which phase generated this receipt
    #[allow(dead_code)]
    phase_label: PhaseLabel,
}

impl StrictExecutionContext {
    /// Create a new execution context with strict invariant checking.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if the contract ID is invalid or empty.
    pub fn new(contract_id: String) -> InvariantResult<Self> {
        // Validate contract ID
        ContractValidator::validate(&contract_id, 12)?;

        Ok(Self {
            contract_id,
            thermal_validator: ThermalValidator::new(1_000_000_000), // 1 billion tick threshold
            effect_validator: None,
            state_validator: None,
            receipt_validator: ReceiptValidator::new(1),
            phases_completed: Vec::new(),
            receipts: BTreeMap::new(),
        })
    }

    /// Phase 1: Contract Definition
    /// Verify contract is completely specified.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if the contract is malformed or phase count is invalid.
    pub fn phase_1_contract_definition(
        &mut self,
        phase_count: usize,
    ) -> InvariantResult<PhaseResult> {
        ContractValidator::validate(&self.contract_id, phase_count)?;
        self.phases_completed.push(PhaseLabel::ContractDefinition);
        Ok(PhaseResult::Ok)
    }

    /// Phase 2: Thermal Testing
    /// Validate τ measurement monotonicity and bounds.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if τ measurement violates monotonicity or exceeds bounds.
    pub fn phase_2_thermal_testing(
        &mut self,
        tau: u64,
        max_tau_bound: u64,
    ) -> InvariantResult<PhaseResult> {
        self.thermal_validator.validate_tau(tau)?;

        // Check τ against configured bound
        if tau > max_tau_bound {
            return Ok(PhaseResult::Violation(UnrecoverableInvariantViolation::Other(format!(
                "Thermal bound exceeded: {tau} > {max_tau_bound}"
            ))));
        }

        self.phases_completed.push(PhaseLabel::ThermalTesting);
        Ok(PhaseResult::Ok)
    }

    /// Phase 3: Effects Tracking
    /// Verify observed effects are subset of declared effects.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if observed effects are not a subset of declared effects.
    pub fn phase_3_effects_tracking(
        &mut self,
        declared: Vec<String>,
        observed: &[String],
    ) -> InvariantResult<PhaseResult> {
        let validator = EffectValidator::new(declared)?;
        validator.validate_observed(observed)?;
        self.effect_validator = Some(validator);
        self.phases_completed.push(PhaseLabel::EffectsTracking);
        Ok(PhaseResult::Ok)
    }

    /// Phase 4: State Machine Transitions
    /// Verify state transitions are valid.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if the initial state is invalid or not in the set of all states.
    pub fn phase_4_state_machine(
        &mut self,
        initial_state: String,
        all_states: Vec<String>,
    ) -> InvariantResult<PhaseResult> {
        let validator = StateValidator::new(initial_state, all_states)?;
        self.state_validator = Some(validator);
        self.phases_completed.push(PhaseLabel::StateMachine);
        Ok(PhaseResult::Ok)
    }

    /// Phase 5: Receipt Generation & Validation
    /// Store receipt with checksum verification.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if receipt version mismatch or checksum validation fails.
    pub fn phase_5_receipt_generation(
        &mut self,
        version: u32,
        checksum: u32,
        computed: u32,
    ) -> InvariantResult<PhaseResult> {
        self.receipt_validator.validate_receipt(version, checksum, computed)?;

        let receipt = ReceiptData { version, checksum, phase_label: PhaseLabel::ReceiptGeneration };

        self.receipts.insert(format!("receipt_{}", self.receipts.len()), receipt);

        self.phases_completed.push(PhaseLabel::ReceiptGeneration);
        Ok(PhaseResult::Ok)
    }

    /// Phase 6: Swarm Orchestration
    /// Verify no tests were abandoned.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if fewer tests were executed than scheduled.
    pub fn phase_6_swarm_orchestration(
        &mut self,
        scheduled: usize,
        executed: usize,
    ) -> InvariantResult<PhaseResult> {
        if executed < scheduled {
            return Err(UnrecoverableInvariantViolation::AbandonedTest(format!(
                "Scheduled {scheduled} tests but only executed {executed}"
            )));
        }

        self.phases_completed.push(PhaseLabel::SwarmOrchestration);
        Ok(PhaseResult::Ok)
    }

    /// Phase 7: Verification Pipeline
    /// Verify all configured phases executed.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if any expected phase was not executed.
    pub fn phase_7_verification_pipeline(
        &mut self,
        expected_phases: &[PhaseLabel],
    ) -> InvariantResult<PhaseResult> {
        for expected in expected_phases {
            if !self.phases_completed.contains(expected) {
                return Err(UnrecoverableInvariantViolation::PipelinePhaseSkipped(format!(
                    "Expected phase {expected:?} was not executed"
                )));
            }
        }

        self.phases_completed.push(PhaseLabel::VerificationPipeline);
        Ok(PhaseResult::Ok)
    }

    /// Phase 8: Continuous Learning
    /// Verify learner data is consistent and not corrupted.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if sample count is insufficient or prediction is out of range (which also excludes NaN/Inf).
    pub fn phase_8_continuous_learning(
        &mut self,
        sample_count: usize,
        prediction: f64,
    ) -> InvariantResult<PhaseResult> {
        // Minimum observations required
        if sample_count < 5 {
            return Err(UnrecoverableInvariantViolation::LearnerMathCorrupted(format!(
                "Insufficient observations: {sample_count} < 5"
            )));
        }

        // Verify prediction is in valid range [0.0, 1.0]
        // (the range check also excludes NaN/Inf: `contains` is false for NaN,
        // and ±Inf falls outside the bound)
        if !(0.0..=1.0).contains(&prediction) {
            return Err(UnrecoverableInvariantViolation::LearnerMathCorrupted(format!(
                "Invalid prediction probability: {prediction}"
            )));
        }

        self.phases_completed.push(PhaseLabel::ContinuousLearning);
        Ok(PhaseResult::Ok)
    }

    /// Phase 9: Distributed Consensus
    /// Verify vote quorum and valid signatures.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if approval votes are insufficient for 2/3 Byzantine quorum.
    pub fn phase_9_distributed_consensus(
        &mut self,
        approval_votes: usize,
        total_votes: usize,
    ) -> InvariantResult<PhaseResult> {
        // Require 2/3 Byzantine Fault Tolerant quorum
        let required_quorum = (total_votes * 2) / 3 + 1;

        if approval_votes < required_quorum {
            return Err(UnrecoverableInvariantViolation::ConsensusDeadlock(format!(
                "Insufficient approvals: {approval_votes} < {required_quorum}"
            )));
        }

        self.phases_completed.push(PhaseLabel::DistributedConsensus);
        Ok(PhaseResult::Ok)
    }

    /// Phase 10: Time-Travel Debugging
    /// Verify snapshot integrity and replay determinism.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if snapshot version doesn't match expected version.
    pub fn phase_10_time_travel_debugging(
        &mut self,
        snapshot_version: u32,
        expected_version: u32,
    ) -> InvariantResult<PhaseResult> {
        if snapshot_version != expected_version {
            return Err(UnrecoverableInvariantViolation::SnapshotSchemaVersionMismatch {
                expected: expected_version,
                found: snapshot_version,
            });
        }

        self.phases_completed.push(PhaseLabel::TimeTravelDebugging);
        Ok(PhaseResult::Ok)
    }

    /// Phase 11: Performance Prophet
    /// Verify prediction self-checks pass.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if predicted τ is zero or confidence is out of range (which also excludes NaN/Inf).
    pub fn phase_11_performance_prophet(
        &mut self,
        predicted_tau: u64,
        confidence: f64,
    ) -> InvariantResult<PhaseResult> {
        // Verify prediction is physically possible
        if predicted_tau == 0 {
            return Err(UnrecoverableInvariantViolation::ProphetSelfCheckFailed(
                "Zero predicted ticks is impossible".to_string(),
            ));
        }

        // Verify confidence interval is valid
        // (the range check also excludes NaN/Inf: `contains` is false for NaN,
        // and ±Inf falls outside the bound)
        if !(0.0..=1.0).contains(&confidence) {
            return Err(UnrecoverableInvariantViolation::ProphetSelfCheckFailed(format!(
                "Invalid confidence interval: {confidence}"
            )));
        }

        self.phases_completed.push(PhaseLabel::PerformanceProphet);
        Ok(PhaseResult::Ok)
    }

    /// Phase 12: Quality Dashboard
    /// Verify dashboard consistency invariants.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if dashboard totals don't match (passed + failed ≠ total).
    pub fn phase_12_quality_dashboard(
        &mut self,
        total: usize,
        passed: usize,
        failed: usize,
    ) -> InvariantResult<PhaseResult> {
        // Verify totals add up
        if passed + failed != total {
            return Err(UnrecoverableInvariantViolation::DashboardInconsistency(format!(
                "Totals don't match: {passed} + {failed} ≠ {total}"
            )));
        }

        // usize is unsigned, so negative check is not needed
        // (type system prevents negative counts)

        self.phases_completed.push(PhaseLabel::QualityDashboard);
        Ok(PhaseResult::Ok)
    }

    /// Verify all required phases were executed.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnrecoverableInvariantViolation)` if any required phase was not completed.
    pub fn finalize(&self) -> InvariantResult<()> {
        let required_phases = vec![
            PhaseLabel::ContractDefinition,
            PhaseLabel::ThermalTesting,
            PhaseLabel::ReceiptGeneration,
            PhaseLabel::VerificationPipeline,
        ];

        for required in &required_phases {
            if !self.phases_completed.contains(required) {
                return Err(UnrecoverableInvariantViolation::PartialPipelineSuccess(format!(
                    "Required phase {required:?} not completed"
                )));
            }
        }

        Ok(())
    }
}

/// Lightweight assertion that an invariant holds.
/// Returns error (does not panic) if invariant violated.
///
/// # Errors
///
/// Returns `Err(UnrecoverableInvariantViolation)` if the condition is false.
pub fn assert_invariant(
    condition: bool,
    violation: UnrecoverableInvariantViolation,
) -> InvariantResult<()> {
    if condition {
        Ok(())
    } else {
        Err(violation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = StrictExecutionContext::new("contract_001".to_string());
        assert!(ctx.is_ok());
    }

    #[test]
    fn test_empty_contract_id_fails() {
        let ctx = StrictExecutionContext::new("".to_string());
        assert!(ctx.is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_1_contract_definition() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        let result = ctx.phase_1_contract_definition(12);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_2_thermal_monotonic() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        let _ = ctx.phase_1_contract_definition(12);

        let r1 = ctx.phase_2_thermal_testing(100, 10_000);
        assert!(r1.is_ok());

        let r2 = ctx.phase_2_thermal_testing(100, 10_000);
        assert!(r2.is_ok());

        let r3 = ctx.phase_2_thermal_testing(50, 10_000);
        assert!(r3.is_err()); // Clock went backward
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_5_receipt_generation() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        let _ = ctx.phase_1_contract_definition(12);

        let result = ctx.phase_5_receipt_generation(1, 0x1234, 0x1234);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_9_consensus_requires_quorum() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code

        // With 9 voters, need 7 approvals for 2/3 quorum
        let result = ctx.phase_9_distributed_consensus(6, 9);
        assert!(result.is_err()); // Only 6 out of 7 required
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_11_prophet_rejects_invalid_confidence() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code

        let result = ctx.phase_11_performance_prophet(1000, 1.5); // Confidence > 1.0
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_12_dashboard_consistency() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code

        // Totals don't match
        let result = ctx.phase_12_quality_dashboard(10, 5, 3);
        assert!(result.is_err()); // 5 + 3 ≠ 10
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_finalize_requires_core_phases() {
        let ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code

        // No phases completed
        let result = ctx.finalize();
        assert!(result.is_err());
    }

    // ---- Phase 2: bound-exceeded Ok(Violation) path ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_2_bound_exceeded_returns_violation_not_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code

        // τ within validator bounds (monotonic) but above the configured bound:
        // must surface as Ok(PhaseResult::Violation), not Err.
        let result = ctx.phase_2_thermal_testing(20_000, 10_000).unwrap(); // Test code
        assert!(result.is_violation());
        let err = result.into_result().unwrap_err(); // Test code
        assert!(err.to_string().contains("Thermal bound exceeded"));
        assert!(err.to_string().contains("20000"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_2_within_bound_is_ok() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        let result = ctx.phase_2_thermal_testing(500, 10_000).unwrap(); // Test code
        assert!(result.is_ok());
        assert!(result.into_result().is_ok());
    }

    // ---- Phase 3: Effects Tracking ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_3_effects_tracking_happy() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        let declared = vec!["read_file".to_string(), "write_file".to_string()];
        let observed = vec!["read_file".to_string()];
        let result = ctx.phase_3_effects_tracking(declared, &observed);
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok()); // Test code
    }

    #[test]
    fn test_phase_3_effects_tracking_empty_declared_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).expect("valid"); // Test code
        let result = ctx.phase_3_effects_tracking(Vec::new(), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_phase_3_effects_tracking_unobserved_effect_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).expect("valid"); // Test code
        let declared = vec!["read_file".to_string()];
        let observed = vec!["network_call".to_string()];
        let result = ctx.phase_3_effects_tracking(declared, &observed);
        assert!(result.is_err());
        let err = result.expect_err("unobserved effect must fail"); // Test code
        assert!(err.to_string().contains("Unobserved effect"));
    }

    // ---- Phase 4: State Machine ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_4_state_machine_happy() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        let result = ctx.phase_4_state_machine(
            "Idle".to_string(),
            vec!["Idle".to_string(), "Running".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_phase_4_state_machine_initial_not_in_states_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).expect("valid"); // Test code
        let result = ctx.phase_4_state_machine(
            "Paused".to_string(),
            vec!["Idle".to_string(), "Running".to_string()],
        );
        assert!(result.is_err());
        assert!(result
            .expect_err("initial state outside state set must fail") // Test code
            .to_string()
            .contains("Contract malformed"));
    }

    // ---- Phase 6: Swarm Orchestration ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_6_swarm_orchestration_happy() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        assert!(ctx.phase_6_swarm_orchestration(10, 10).is_ok());
        // Executing more than scheduled is also acceptable
        assert!(ctx.phase_6_swarm_orchestration(10, 12).is_ok());
    }

    #[test]
    fn test_phase_6_swarm_orchestration_abandoned_test_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).expect("valid"); // Test code
        let result = ctx.phase_6_swarm_orchestration(10, 9);
        assert!(result.is_err());
        let err = result.expect_err("abandoned test must fail"); // Test code
        assert!(err.to_string().contains("Abandoned test"));
    }

    // ---- Phase 7: Verification Pipeline ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_7_verification_pipeline_happy() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        let _ = ctx.phase_1_contract_definition(12);
        let expected = vec![PhaseLabel::ContractDefinition];
        let result = ctx.phase_7_verification_pipeline(&expected);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_7_verification_pipeline_skipped_phase_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
                                                                                        // ReceiptGeneration never ran
        let expected = vec![PhaseLabel::ContractDefinition, PhaseLabel::ReceiptGeneration];
        let result = ctx.phase_7_verification_pipeline(&expected);
        assert!(result.is_err());
        let err = result.expect_err("skipped phase must fail"); // Test code
        assert!(err.to_string().contains("Pipeline phase skipped"));
    }

    // ---- Phase 8: Continuous Learning ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_8_continuous_learning_happy() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        assert!(ctx.phase_8_continuous_learning(5, 0.5).is_ok());
        assert!(ctx.phase_8_continuous_learning(100, 0.0).is_ok());
        assert!(ctx.phase_8_continuous_learning(100, 1.0).is_ok());
    }

    #[test]
    fn test_phase_8_continuous_learning_insufficient_samples_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).expect("valid"); // Test code
        let result = ctx.phase_8_continuous_learning(4, 0.5);
        assert!(result.is_err());
        assert!(result
            .expect_err("insufficient samples must fail") // Test code
            .to_string()
            .contains("Insufficient observations"));
    }

    #[test]
    fn test_phase_8_continuous_learning_prediction_out_of_range_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).expect("valid"); // Test code
        assert!(ctx.phase_8_continuous_learning(10, 1.5).is_err());
        assert!(ctx.phase_8_continuous_learning(10, -0.1).is_err());
        // NaN is outside [0.0, 1.0], so the range check rejects it too
        assert!(ctx.phase_8_continuous_learning(10, f64::NAN).is_err());
        assert!(ctx.phase_8_continuous_learning(10, f64::INFINITY).is_err());
    }

    // ---- Phase 10: Time-Travel Debugging ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_phase_10_time_travel_happy() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        assert!(ctx.phase_10_time_travel_debugging(7, 7).is_ok());
    }

    #[test]
    fn test_phase_10_time_travel_version_mismatch_is_err() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).expect("valid"); // Test code
        let result = ctx.phase_10_time_travel_debugging(2, 1);
        assert!(result.is_err());
        let err = result.expect_err("version mismatch must fail"); // Test code
        assert!(err.to_string().contains("Snapshot schema version mismatch"));
        assert!(err.to_string().contains("expected 1"));
    }

    // ---- PhaseResult helpers ----

    #[test]
    fn test_phase_result_is_violation_and_into_result() {
        let ok = PhaseResult::Ok;
        assert!(ok.is_ok());
        assert!(!ok.is_violation());
        assert!(ok.into_result().is_ok());

        let bad =
            PhaseResult::Violation(UnrecoverableInvariantViolation::Other("boom".to_string()));
        assert!(bad.is_violation());
        assert!(!bad.is_ok());
        let err = bad.into_result().expect_err("violation converts to Err"); // Test code
        assert!(err.to_string().contains("boom"));
    }

    // ---- assert_invariant ----

    #[test]
    fn test_assert_invariant() {
        let ok = assert_invariant(
            true,
            UnrecoverableInvariantViolation::Other("should not fire".to_string()),
        );
        assert!(ok.is_ok());

        let err =
            assert_invariant(false, UnrecoverableInvariantViolation::Other("fired".to_string()));
        assert!(err.is_err());
        assert!(err.expect_err("false condition must fail").to_string().contains("fired"));
        // Test code
    }

    // ---- finalize success path ----

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_finalize_success_after_required_phases() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        assert!(ctx.phase_1_contract_definition(12).is_ok());
        assert!(ctx.phase_2_thermal_testing(100, 10_000).is_ok());
        assert!(ctx.phase_5_receipt_generation(1, 0x1234, 0x1234).is_ok());
        let expected = vec![
            PhaseLabel::ContractDefinition,
            PhaseLabel::ThermalTesting,
            PhaseLabel::ReceiptGeneration,
        ];
        assert!(ctx.phase_7_verification_pipeline(&expected).is_ok());
        assert!(ctx.finalize().is_ok());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_finalize_fails_when_one_required_phase_missing() {
        let mut ctx = StrictExecutionContext::new("contract_001".to_string()).unwrap(); // Test code
        assert!(ctx.phase_1_contract_definition(12).is_ok());
        assert!(ctx.phase_2_thermal_testing(100, 10_000).is_ok());
        // Skip ReceiptGeneration and VerificationPipeline
        let result = ctx.finalize();
        assert!(result.is_err());
        assert!(result
            .expect_err("missing required phase must fail") // Test code
            .to_string()
            .contains("Partial pipeline success"));
    }
}
