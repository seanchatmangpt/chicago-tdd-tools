//! chicago-claims: Chicago TDD claim reconciliation and execution-standing courts.
//!
//! The original MLP closes one reference law by loading a TOML claim, scanning real
//! Rust source with `syn`, activating named mutants, attributing oracle outcomes, and
//! emitting a reconciliation report.
//!
//! `chicago_alive` is deliberately separate from the legacy structural/mutation
//! `reconcile::Standing`: it derives the ecosystem-wide `CHICAGO_ALIVE` boundary from
//! exact-subject executable-world evidence without changing the meaning of existing
//! callers' local claim standing.

pub mod chicago_alive;
pub mod claim;
pub mod mutate;
pub mod reconcile;
pub mod report;
pub mod scan;

pub use chicago_alive::{
    evaluate_chicago_alive, AuthorityEvidence, ChicagoDecision, ChicagoFinding, ChicagoStanding,
    ChicagoTrial, CollaboratorEvidence, CollaboratorKind, CorrespondenceEvidence,
    ExecutionEvidence, FalsifierEvidence, ReplayState, SubjectEvidence, WorldEvidence, WorldKind,
};
pub use claim::{Claim, ClaimError, MutantSpec, RequiredStructure, Scope};
pub use mutate::{
    activate_and_test_mutant, classify_mutant, execute_mutant, resolve_provider, ActivationWitness,
    CargoFeatureProvider, FixtureProvider, MutantExecutionReport, MutantResolutionError,
    MutationCapabilities, MutationProvider, OracleOutcomeKind, OracleResult, PatchOverlayProvider,
    DEFAULT_PATCH_OVERLAY_TIMEOUT_SECS,
};
pub use reconcile::{reconcile, Delta, MutantClassification, Reconciliation, Standing};
pub use report::render_report;
pub use scan::{scan_required_structure, ScanError, ScanResult};
