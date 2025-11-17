# nomrg DFLSS Charter: Machine-Readable Specification

**Version**: 1.0.0  
**Status**: Draft  
**Last Updated**: 2025-01-16  
**Format**: Machine-readable DFLSS charter for agent consumption

---

## 1. Process Identity (R1)

### 1.1 Canonical Merge μ-Kernel

**Identity**: `nomrg` is the **unique, canonical merge μ-kernel** in the AHI stack.

**Scope**:
- Code and configuration repositories (ΔΣ → Σ* transitions)
- Knowledge ontologies and Σ projections
- Capability contracts and policies

**Authority**: All ΔΣ → Σ* transitions are considered valid **only** if routed through nomrg. All other components treat nomrg as the source of truth for "what is integrated".

**Interface Type**: Machine-only (CNV CLI, RPC, event streams, receipts). No human UX requirements.

---

## 2. Goal Surface (R2)

### 2.1 Graph-Level Objectives

DFLSS controllers encode goals as **graph objectives**, not human business KPIs. nomrg must expose these as formal goal objects that AHI can optimize against.

**Goal Types**:

```rust
pub enum MergeGoal {
    /// Minimize merge-induced defect rate in production traces
    MinimizeDefectRate { target_rate: f64, confidence: f64 },
    
    /// Minimize merge latency under fixed risk budget
    MinimizeLatency { max_latency_ms: u64, risk_budget: RiskBudget },
    
    /// Maximize Σ coverage (dark-matter reduction) subject to Q invariants
    MaximizeCoverage { min_q_score: f64, coverage_target: f64 },
    
    /// Bound volatility of mainline (limit concurrent high-risk overlays)
    BoundVolatility { max_concurrent_high_risk: usize, volatility_window: Duration },
}
```

**Decision Constraint**: All merge decisions A must satisfy: `A = μ(O)` where μ respects encoded goals.

---

## 3. Process Observables (R3)

### 3.1 Standardized Observation Types

nomrg must standardize observation types required for DFLSS process control.

**Structural Observables**:

```rust
pub struct StructuralObservable {
    /// Overlap metrics: which regions in Σ are touched by which ΔΣ
    pub overlap_metrics: OverlapMetrics,
    
    /// Topological impact: dependency changes, fan-in/fan-out delta
    pub topological_impact: TopologicalImpact,
}

pub struct OverlapMetrics {
    pub touched_files: Vec<FileId>,
    pub touched_modules: Vec<ModuleId>,
    pub overlap_ratio: f64, // Fraction of Σ touched by ΔΣ
    pub conflict_regions: Vec<ConflictRegion>,
}

pub struct TopologicalImpact {
    pub dependency_added: Vec<Dependency>,
    pub dependency_removed: Vec<Dependency>,
    pub fan_in_delta: i32,
    pub fan_out_delta: i32,
    pub critical_path_changes: Vec<CriticalPathChange>,
}
```

**Behavioral Observables**:

```rust
pub struct BehavioralObservable {
    /// Test outcomes organized by category
    pub test_outcomes: TestOutcomeSet,
    
    /// Timing data for μ-kernel execution under new Σ*
    pub timing_data: TimingMeasurement,
}

pub struct TestOutcomeSet {
    pub safety_tests: TestCategoryResults,
    pub regression_tests: TestCategoryResults,
    pub performance_tests: TestCategoryResults,
    pub total_tests: usize,
    pub pass_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
}

pub struct TimingMeasurement {
    pub merge_latency_ms: u64,
    pub static_analysis_ms: u64,
    pub test_execution_ms: u64,
    pub policy_check_ms: u64,
    pub total_ms: u64,
}
```

**Governance Observables**:

```rust
pub struct GovernanceObservable {
    /// Q violations encountered
    pub q_violations: Vec<QViolation>,
    
    /// Policy decisions (allow, degrade, quarantine, block)
    pub policy_decisions: Vec<PolicyDecision>,
}

pub struct QViolation {
    pub invariant_id: String,
    pub severity: ViolationSeverity,
    pub context: ViolationContext,
    pub suggested_remediation: Option<String>,
}

pub enum PolicyDecision {
    Allow { confidence: f64, conditions: Vec<Condition> },
    Degrade { reason: String, degraded_capabilities: Vec<String> },
    Quarantine { reason: String, quarantine_duration: Duration },
    Block { reason: String, blocking_policy: String },
}
```

**Process Metrics**:

```rust
pub struct ProcessMetrics {
    /// Merge queue lengths, age distributions
    pub queue_metrics: QueueMetrics,
    
    /// Conflict rates, revert rates, incident correlations
    pub quality_metrics: QualityMetrics,
}

pub struct QueueMetrics {
    pub queue_length: usize,
    pub oldest_pending_age_ms: u64,
    pub queue_age_distribution: AgeDistribution,
    pub priority_distribution: PriorityDistribution,
}

pub struct QualityMetrics {
    pub conflict_rate: f64, // Conflicts per merge attempt
    pub revert_rate: f64,    // Reverts per accepted merge
    pub incident_correlation: IncidentCorrelation,
}
```

### 3.2 Measurement System Discipline (R4)

**Coverage Declaration**: Every check (test suite, static analyzer, policy engine) must:
- Declare coverage (what portion of ΔΣ and Σ it observes)
- Expose reliability metrics (flakiness, historical false-positive/negative rates)

**Confidence Tracking**: nomrg must track:
- Confidence scores for each observable source
- Aggregated confidence on a per-decision basis

**Calibration Requirement**: No human "eyeballing" of flaky tests; AHI needs calibrated signals.

```rust
pub struct ObservableSource {
    pub source_id: String,
    pub coverage: CoverageDeclaration,
    pub reliability: ReliabilityMetrics,
    pub confidence_score: f64,
}

pub struct CoverageDeclaration {
    pub files_covered: Vec<FileId>,
    pub modules_covered: Vec<ModuleId>,
    pub coverage_ratio: f64, // Fraction of ΔΣ observed
}

pub struct ReliabilityMetrics {
    pub flakiness_rate: f64,        // Fraction of non-deterministic results
    pub false_positive_rate: f64,  // Historical false positive rate
    pub false_negative_rate: f64,  // Historical false negative rate
    pub sample_size: usize,        // Number of historical observations
}
```

---

## 4. Policy Experimentation (R5)

### 4.1 Strategy Variants

nomrg must support **policy experimentation** as first-class capability.

**Strategy Types**:

```rust
pub enum MergeStrategy {
    /// Strict Q enforcement: all invariants must pass
    StrictQ { q_threshold: f64 },
    
    /// Permissive under low blast radius
    PermissiveLowBlastRadius { max_blast_radius: usize, q_threshold: f64 },
    
    /// Refactor-aware: allows structural changes with behavior preservation proof
    RefactorAware { behavior_proof_required: bool },
    
    /// Custom strategy defined by DFLSS agents
    Custom { policy_id: String, parameters: HashMap<String, Value> },
}
```

### 4.2 Shadow Decisions

**Capability**: AHI can run alternate strategies in parallel on the same O, without affecting Σ*, to estimate impact.

```rust
pub struct ShadowDecision {
    pub strategy: MergeStrategy,
    pub decision: MergeDecision,
    pub estimated_impact: EstimatedImpact,
    pub confidence: f64,
}

pub struct EstimatedImpact {
    pub predicted_defect_rate: f64,
    pub predicted_latency_ms: u64,
    pub predicted_revert_probability: f64,
    pub predicted_incident_correlation: f64,
}
```

### 4.3 Structural A/B Testing

DFLSS agents can treat merge strategies as factors in a design experiment and collect response metrics automatically.

```rust
pub struct StrategyExperiment {
    pub experiment_id: String,
    pub strategies: Vec<MergeStrategy>,
    pub response_metrics: Vec<ResponseMetric>,
    pub sample_size: usize,
    pub confidence_level: f64,
}

pub enum ResponseMetric {
    DefectRate,
    RevertRate,
    Latency,
    IncidentCorrelation,
    CoverageDelta,
}
```

---

## 5. Contractization (R6)

### 5.1 Capability Contracts

When AHI/DFLSS agents identify effective merge patterns, nomrg must be able to turn them into capability contracts.

```rust
pub struct MergeCapabilityContract {
    pub contract_id: String,
    pub version: String,
    pub pattern: MergePattern,
    pub guarantees: Vec<Guarantee>,
    pub preconditions: Vec<Precondition>,
    pub postconditions: Vec<Postcondition>,
}

pub struct MergePattern {
    pub pattern_type: PatternType,
    pub examples: Vec<ExampleMerge>,
    pub success_criteria: SuccessCriteria,
}

pub enum PatternType {
    LowRiskRename { behavior_change: bool },
    RefactorOnly { structural_change: bool },
    FeatureAddition { new_capabilities: Vec<String> },
    BugFix { fixed_issues: Vec<String> },
}
```

### 5.2 Guard and Policy Rules

Effective patterns become guards or policy rules applied in the hot path.

```rust
pub struct MergeGuard {
    pub guard_id: String,
    pub version: String,
    pub contract_reference: String,
    pub enforcement_level: EnforcementLevel,
    pub condition: GuardCondition,
}

pub enum EnforcementLevel {
    Hard,      // Block merge if condition fails
    Soft,      // Warn but allow merge
    Advisory,  // Log only
}

pub enum GuardCondition {
    QScoreAbove { threshold: f64 },
    BlastRadiusBelow { max_files: usize },
    TestCoverageAbove { threshold: f64 },
    PatternMatches { pattern_id: String },
}
```

### 5.3 Versioning and Declaration

Each contract and guard is versioned and referenced in receipts. ΔΣ proposals declare which contracts they intend to satisfy.

```rust
pub struct MergeProposal {
    pub proposal_id: String,
    pub delta_sigma: DeltaSigma,
    pub intended_contracts: Vec<ContractReference>,
    pub risk_assessment: RiskAssessment,
}

pub struct ContractReference {
    pub contract_id: String,
    pub version: String,
    pub confidence: f64, // Confidence that proposal satisfies contract
}
```

---

## 6. Capability Indices (R7)

### 6.1 Process Capability Metrics

nomrg must compute capability indicators for itself.

```rust
pub struct MergeProcessCapability {
    /// Probability that a ΔΣ which passes all checks still results in downstream incidents
    pub false_negative_rate: f64,
    
    /// Distribution of merge latency under different risk tiers
    pub latency_distribution: LatencyDistribution,
    
    /// Process capability index (Cpk) for merge quality
    pub cpk: f64,
    
    /// Process performance index (Ppk) for merge quality
    pub ppk: f64,
}

pub struct LatencyDistribution {
    pub low_risk_p50: u64,
    pub low_risk_p95: u64,
    pub low_risk_p99: u64,
    pub high_risk_p50: u64,
    pub high_risk_p95: u64,
    pub high_risk_p99: u64,
}
```

### 6.2 Capacity Planning

AHI uses capability indices to:
- Decide how much load to route through nomrg
- Decide where to tighten or relax Q for certain tenants or code regions

```rust
pub struct CapacityPlan {
    pub max_concurrent_merges: usize,
    pub risk_tier_limits: HashMap<RiskTier, usize>,
    pub q_thresholds_by_tenant: HashMap<TenantId, f64>,
    pub q_thresholds_by_region: HashMap<CodeRegion, f64>,
}
```

---

## 7. Agent-Grade Interface (R8)

### 7.1 CNV CLI Surface

nomrg must present a stable CNV-based CLI surface oriented to DFLSS agents.

**Nouns**:
- `merge`: Merge operations and proposals
- `policy`: Merge policies and strategies
- `observable`: Process observables and metrics
- `contract`: Capability contracts
- `capability`: Process capability indices

**Verbs**:
- `merge propose`: Submit a ΔΣ proposal
- `merge evaluate`: Evaluate a proposal without committing
- `merge explain`: Explain a merge decision
- `merge replay`: Replay a historical merge decision
- `policy status`: Get current policy configuration
- `policy experiment start`: Start a policy experiment
- `policy experiment result`: Get experiment results
- `observable collect`: Collect process observables
- `contract declare`: Declare a new capability contract
- `capability measure`: Measure process capability indices

**Output Format**: JSON-first, deterministic output envelopes for machine calling.

### 7.2 RPC/API Surface

Same capability set as CLI, mirrored through RPC/API with strong schemas for all requests and receipts.

```rust
pub struct MergeProposeRequest {
    pub proposal: MergeProposal,
    pub observables: Vec<Observable>,
    pub context: MergeContext,
}

pub struct MergeProposeResponse {
    pub decision: MergeDecision,
    pub receipt: MergeReceipt,
    pub confidence: f64,
    pub explanation: Option<String>,
}

pub enum MergeDecision {
    Accept { snapshot_id: String },
    Reject { reason: String, blocking_policies: Vec<String> },
    Quarantine { reason: String, duration: Duration },
    DeferPendingMoreO { required_observables: Vec<String> },
}
```

---

## 8. Closed-World Guarantees (R9)

### 8.1 External State Governance

nomrg must:
- Never pull external state except through governed, declared channels (Σ registries, O telemetry feeds)
- Represent all external influences as Δ or O, so that `A = μ(O)` remains auditable and deterministic
- Forbid ad-hoc agent-authored "scripts" that bypass μ_nomrg

### 8.2 Declared Channels

```rust
pub enum ExternalChannel {
    /// Σ registry: source of truth for current state
    SigmaRegistry { registry_id: String, version: String },
    
    /// O telemetry feed: source of observations
    TelemetryFeed { feed_id: String, schema_version: String },
    
    /// Policy registry: source of merge policies
    PolicyRegistry { registry_id: String, version: String },
    
    /// Contract registry: source of capability contracts
    ContractRegistry { registry_id: String, version: String },
}
```

### 8.3 Operating Law

nomrg is not a toolkit; it is an **operating law** agents must route through. All ΔΣ → Σ* transitions must go through nomrg.

---

## 9. Zero-Ambiguity Failure Modes (R10)

### 9.1 Explicit Decision Outcomes

Every decision path must end in one of a small, explicit set of outcomes:

```rust
pub enum MergeDecision {
    /// Merge accepted, receipt generated
    AcceptWithReceipt { 
        snapshot_id: String,
        receipt: MergeReceipt,
    },
    
    /// Merge rejected, receipt explains why
    RejectWithReceipt { 
        reason: String,
        blocking_policies: Vec<String>,
        receipt: MergeReceipt,
    },
    
    /// Merge quarantined, receipt explains conditions
    QuarantineWithReceipt { 
        reason: String,
        duration: Duration,
        conditions: Vec<Condition>,
        receipt: MergeReceipt,
    },
    
    /// Decision deferred, more observations needed
    DeferPendingMoreO { 
        required_observables: Vec<String>,
        current_observables: Vec<String>,
        receipt: MergeReceipt,
    },
}
```

**Constraint**: No implicit "silent success" or "silent drop". All outcomes must be explicit and receipt-carrying.

---

## 10. Degraded Behavior Handling (R11)

### 10.1 Explicit Degradation Signaling

If measurement systems are degraded (missing telemetry, flaky tests detected, partial Σ), nomrg must:
- Surface that explicitly in decision receipts
- Optionally auto-tighten risk posture (e.g., default to reject or quarantine) according to DFLSS policy

```rust
pub struct MergeReceipt {
    pub receipt_id: String,
    pub timestamp: u64,
    pub decision: MergeDecision,
    pub observables: Vec<Observable>,
    pub confidence: f64,
    pub degradation_flags: Vec<DegradationFlag>,
    pub policy_set: Vec<PolicyReference>,
}

pub enum DegradationFlag {
    MissingTelemetry { source: String, impact: String },
    FlakyTestsDetected { test_ids: Vec<String>, flakiness_rate: f64 },
    PartialSigma { missing_regions: Vec<CodeRegion> },
    LowConfidenceObservables { sources: Vec<String>, confidence_scores: Vec<f64> },
}

pub struct PolicyReference {
    pub policy_id: String,
    pub version: String,
    pub decision: PolicyDecision,
}
```

### 10.2 Auto-Tightening Policy

```rust
pub struct DegradationPolicy {
    pub degradation_type: DegradationType,
    pub action: DegradationAction,
    pub severity: Severity,
}

pub enum DegradationAction {
    RejectAll,
    QuarantineAll,
    TightenQThreshold { new_threshold: f64 },
    RequireAdditionalObservables { required: Vec<String> },
}
```

---

## 11. Replay and Auditability (R12)

### 11.1 Decision Reconstruction

For any merge decision, DFLSS controllers must be able to:
- Reconstruct O, Σ, Q, and policy set used
- Recompute μ_nomrg and recover the same A

### 11.2 Receipt Contents

```rust
pub struct MergeReceipt {
    pub receipt_id: String,
    pub timestamp: u64,
    pub decision: MergeDecision,
    
    /// Complete O used in decision
    pub observables: Vec<Observable>,
    
    /// Σ snapshot used in decision
    pub sigma_snapshot: SigmaSnapshot,
    
    /// Q invariants checked
    pub q_invariants: Vec<QInvariant>,
    
    /// Policy set active at decision time
    pub policy_set: Vec<PolicyReference>,
    
    /// Deterministic decision function identifier
    pub decision_function_id: String,
    pub decision_function_version: String,
    
    /// Merkle root of all inputs for verification
    pub input_merkle_root: String,
    
    /// Merkle root of decision for verification
    pub decision_merkle_root: String,
}
```

### 11.3 Replay Interface

```rust
pub struct ReplayRequest {
    pub receipt_id: String,
    pub observables_override: Option<Vec<Observable>>,
    pub policy_set_override: Option<Vec<PolicyReference>>,
}

pub struct ReplayResponse {
    pub original_decision: MergeDecision,
    pub replayed_decision: MergeDecision,
    pub decisions_match: bool,
    pub divergence_explanation: Option<String>,
}
```

---

## 12. Implementation Requirements

### 12.1 μ-Kernel Integration

nomrg must be deployable as a μ-kernel in the AHI stack:
- Executable as a standalone process
- Integratable with KNHK μ-kernels, CNV kernels, clnrm engines
- Exposes observations O and accepts promoted Σ* snapshots

### 12.2 Receipt Schema

All merge operations must produce receipts conforming to the schema defined in section 11.2.

### 12.3 Versioning

All interfaces, schemas, and policies must be versioned. Breaking changes require major version increments.

---

## 13. Success Criteria

### 13.1 Functional Requirements

- [ ] All ΔΣ → Σ* transitions route through nomrg
- [ ] All decisions produce explicit, receipt-carrying outcomes
- [ ] All decisions are replayable and auditable
- [ ] Policy experimentation supported
- [ ] Capability contracts supported
- [ ] Process capability indices computed
- [ ] CNV CLI and RPC interfaces implemented
- [ ] Closed-world guarantees enforced

### 13.2 Quality Requirements

- [ ] Zero-ambiguity failure modes
- [ ] Explicit degradation handling
- [ ] Deterministic decision function
- [ ] Complete observability
- [ ] Calibrated confidence scores

### 13.3 Performance Requirements

- [ ] Merge latency < 5s for low-risk proposals (p95)
- [ ] Merge latency < 30s for high-risk proposals (p95)
- [ ] Support 100+ concurrent merge proposals
- [ ] Replay latency < 1s for historical decisions

---

## 14. Machine-Readable Format

This charter is designed for machine consumption. Key sections can be extracted as:
- **Rust types**: Sections 2-11 define Rust structs/enums
- **JSON schemas**: Receipt and request/response schemas
- **RDF ontology**: Process identity, observables, contracts
- **CNV command definitions**: CLI surface definitions

**Next Steps**:
1. Generate Rust types from this charter
2. Generate JSON schemas for receipts
3. Generate RDF ontology entries
4. Generate CNV command definitions
5. Implement nomrg μ-kernel

---

**End of Charter**

