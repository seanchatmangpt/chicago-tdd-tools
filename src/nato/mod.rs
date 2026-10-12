#![cfg(feature = "nato")]
//! NATO court framework: pillar-classified adversarial courts with
//! mandatory positive controls, receipted outcomes, and scoping disclosure.
//!
//! # Positive control law
//! Every court run executes the court's `positive_control()` probe FIRST.
//! A probe returning `false` short-circuits the run: the receipt records
//! `Verdict::Refused` with `ControlResult::GateRefused` and the court's
//! `run()` is never called. No probe, no verdict.

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

pub mod bell_lapadula;
pub mod cache;
pub mod control;
pub mod court;
pub mod disasm;
pub mod disclosure;
pub mod dossier_court;
pub mod entropy;
pub mod guest_audit;
pub mod hamming;
pub mod kat;
pub mod liveness;
pub mod mils;
pub mod orthogonality;
pub mod receipt;
pub mod receipts;
pub mod registry;
pub mod reproducible;
pub mod rng;
pub mod seu;
pub mod stackbound;
pub mod thrash;
pub mod tick;
pub mod timing;
pub mod torn;
pub mod transition_tables;
pub mod wcet;
pub mod zero_ambient;

pub use control::{build_control_result, run_probe};
pub use court::{
    run_court, CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar,
    Verdict,
};
pub use disclosure::{ScopingRecord, ScopingValidationError};
pub use receipt::{ControlResult, CourtReceipt, ReceiptParseError};
pub use receipts::{load_dir, persist, PersistError};
pub use registry::{CourtEntry, COURT_NAMES, COURT_REGISTRY};
pub use rng::Lcg;
