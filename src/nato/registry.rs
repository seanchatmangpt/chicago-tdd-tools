#![cfg(feature = "nato")]
//! Court registry: constructor entries and canonical court names.
//!
//! Real court implementations land in later lanes; this module fixes the
//! registry shape and the canonical `N01`..`N20` naming now so receipts
//! minted by later lanes are stable.

// Lint-scope note: NATO court-harness and ledger wire-format modules use
// domain literals, metric casts, and court-assertion panics by design; the
// style-class lints below are allowed at module scope with this
// justification rather than 300+ inline permits.
#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::use_self,
    clippy::missing_const_for_fn,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::explicit_iter_loop,
    clippy::unusual_byte_groupings,
    clippy::unreadable_literal,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::needless_range_loop,
    clippy::needless_continue,
    clippy::unnecessary_wraps,
    clippy::double_must_use,
    clippy::op_ref,
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::or_fun_call,
    clippy::nonminimal_bool,
    clippy::unnecessary_map_or,
    clippy::manual_is_multiple_of,
    clippy::manual_midpoint,
    clippy::identity_op,
    clippy::format_collect,
    clippy::trivially_copy_pass_by_ref
)]

use crate::nato::bell_lapadula::BellLapadulaCourt;
use crate::nato::cache::CacheCourt;
use crate::nato::court::{Court, CourtId, Pillar};
use crate::nato::disasm::DisasmCourt;
use crate::nato::dossier_court::DossierCourt;
use crate::nato::entropy::EntropyCourt;
use crate::nato::hamming::HammingProfileCourt;
use crate::nato::kat::KatCourt;
use crate::nato::liveness::LivenessCourt;
use crate::nato::mils::MilSeparationCourt;
use crate::nato::orthogonality::OrthogonalityCourt;
use crate::nato::reproducible::ReproducibleCourt;
use crate::nato::seu::{SeuDoubleBitCourt, SeuSingleBitCourt};
use crate::nato::stackbound::StackBoundCourt;
use crate::nato::thrash::Cast32ThrashCourt;
use crate::nato::tick::ClockGlitchCourt;
use crate::nato::timing::TimingCourt;
use crate::nato::torn::TornWriteCourt;
use crate::nato::wcet::WcetCourt;
use crate::nato::zero_ambient::ZeroAmbientCourt;

/// One registered court: its id, pillar, and a constructor.
#[derive(Clone)]
pub struct CourtEntry {
    /// Court identifier.
    pub id: crate::nato::court::CourtId,
    /// Capability pillar.
    pub pillar: crate::nato::court::Pillar,
    /// Boxed constructor for the concrete court.
    pub ctor: fn() -> Box<dyn Court>,
}

/// Canonical court names, indexed by `court_id_number - 1`.
pub static COURT_NAMES: [&str; 20] = [
    "N01", "N02", "N03", "N04", "N05", "N06", "N07", "N08", "N09", "N10", "N11", "N12", "N13",
    "N14", "N15", "N16", "N17", "N18", "N19", "N20",
];

/// Registry entry constructor helper: keeps the 20-entry table literal
/// readable (`entry(CourtId::Nxx, Pillar::X, || Box::new(XCourt))`).
const fn entry(
    id: crate::nato::court::CourtId,
    pillar: crate::nato::court::Pillar,
    ctor: fn() -> Box<dyn Court>,
) -> CourtEntry {
    CourtEntry { id, pillar, ctor }
}

/// The canonical 20-court registry, in `N01`..`N20` order. Every entry's
/// `id`/`pillar` are the landed court's own `ID`/`PILLAR` consts (the consts
/// are the law); the ctor is zero-arg for all 20 courts.
pub static COURT_REGISTRY: [CourtEntry; 20] = [
    entry(CourtId::N01, Pillar::Security, || Box::new(MilSeparationCourt)),
    entry(CourtId::N02, Pillar::Proof, || Box::new(OrthogonalityCourt)),
    entry(CourtId::N03, Pillar::Security, || Box::new(BellLapadulaCourt)),
    entry(CourtId::N04, Pillar::Dossier, || Box::new(ZeroAmbientCourt)),
    entry(CourtId::N05, Pillar::Timing, || Box::new(TimingCourt)),
    entry(CourtId::N06, Pillar::Timing, || Box::new(DisasmCourt)),
    entry(CourtId::N07, Pillar::Timing, || Box::new(CacheCourt)),
    entry(CourtId::N08, Pillar::Security, || Box::new(HammingProfileCourt)),
    entry(CourtId::N09, Pillar::Resilience, || Box::new(SeuSingleBitCourt)),
    entry(CourtId::N10, Pillar::Resilience, || Box::new(SeuDoubleBitCourt)),
    entry(CourtId::N11, Pillar::Resilience, || Box::new(TornWriteCourt)),
    entry(CourtId::N12, Pillar::Resilience, || Box::new(ClockGlitchCourt)),
    entry(CourtId::N13, Pillar::Timing, || Box::new(WcetCourt)),
    entry(CourtId::N14, Pillar::Resilience, || Box::new(Cast32ThrashCourt)),
    entry(CourtId::N15, Pillar::Proof, || Box::new(LivenessCourt)),
    entry(CourtId::N16, Pillar::Dossier, || Box::new(StackBoundCourt)),
    entry(CourtId::N17, Pillar::Security, || Box::new(KatCourt)),
    entry(CourtId::N18, Pillar::Security, || Box::new(EntropyCourt)),
    entry(CourtId::N19, Pillar::Proof, || Box::new(ReproducibleCourt)),
    entry(CourtId::N20, Pillar::Dossier, || Box::new(DossierCourt::control_default())),
];
