//! POWL conformance subsystem: DSL loading, model validation, PN64 replay,
//! structured verdict reporting, and reachability state-space analysis.
//!
//! Canonical type homes (E1 reconcile contract):
//!
//! * [`error::ConformanceError`] — the one error type.
//! * [`loader::PowlModel`] / [`loader::PowlNode`] — the POWL model surface
//!   (loader wins over the retired model.rs variant).
//! * [`report::ConformanceReport`] — the verdict report (deviation closure
//!   laws); the session's replay summary is [`report::ReplaySummary`].
//! * [`state::Marking64`] — the struct marking; [`session::Marking64`] is a
//!   deliberate `u64` alias, distinct on purpose.
//! * [`net::CompiledNet`] — the canonical compiled net; [`session::
//!   CompiledNet`] is the session-local linear net, distinct on purpose.
//!
//! Flat re-exports below give the documented public surface; where a name is
//! homed in two modules by design, the canonical home wins the flat name.

pub mod bcinr;
pub mod error;
pub mod loader;
pub mod model;
pub mod net;
pub mod pn64;
pub mod report;
pub mod session;
pub mod state;
pub mod validate;

// --- v26.10.11 hermetic WASM guest surface (zero-import ABI) ---
#[cfg(feature = "wasm-guest")]
pub mod guest;
#[cfg(all(feature = "wasm-guest", feature = "ledger"))]
pub mod guest_seal;
#[cfg(all(feature = "wasm-guest", feature = "ocel-ingest"))]
pub mod guest_telemetry;
#[cfg(feature = "wasm-guest")]
pub mod guest_yield;

pub use error::ConformanceError;
pub use loader::PowlModel;
pub use net::CompiledNet;
pub use pn64::{
    ct_eq, ct_lte, enabled, fire, flags, marking, pack, post, pre, step_branchless, tid, MASK16,
};
pub use session::{ConformanceSession, Phase, ReplaySummary, ReportOutcome, YieldStatus};
pub use state::Marking64;
pub use validate::validate;

/// Re-exported from the canonical home (moved to report.rs by the E1 item-2
/// reconcile; `session` re-exports them for path stability).
pub use report::{ConformanceReport, Deviation};
