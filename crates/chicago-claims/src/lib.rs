//! chicago-claims: Chicago TDD claim-reconciliation MLP.
//!
//! One complete loop for one reference law: load a TOML claim describing a required
//! Rust structure, scan real source for that structure with `syn`, activate named
//! mutant fixtures, attribute oracle outcomes, and emit a reconciliation report.
//!
//! `claim` (TOML schema), `scan` (syn-based AST scanner), `mutate` (mutant
//! activation + oracle attribution), `reconcile` (typed standing), and `report`
//! (CLI-facing rendering) are all implemented for the MLP's one reference law.

pub mod claim;
pub mod mutate;
pub mod reconcile;
pub mod report;
pub mod scan;

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
