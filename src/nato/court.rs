#![cfg(feature = "nato")]
//! Court definitions: verdicts, pillars, court identifiers, the `Court`
//! trait, and the positive-control-gated `run_court` entry point.

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

use serde::{Deserialize, Serialize};

use crate::nato::receipt::CourtReceipt;
use crate::nato::registry::COURT_NAMES;
use crate::nato::{control, disclosure::ScopingRecord};

/// Verdict of a court run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// The candidate satisfied the court's law.
    Pass,
    /// The candidate violated the court's law.
    Fail,
    /// The court did not evaluate the candidate at all.
    Refused,
}

/// Capability pillar a court adjudicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pillar {
    /// The security pillar.
    Security,
    /// The timing pillar.
    Timing,
    /// The resilience pillar.
    Resilience,
    /// The proof pillar.
    Proof,
    /// The dossier pillar.
    Dossier,
}

impl Pillar {
    /// Stable machine name used in receipts and JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Pillar::Security => "security",
            Pillar::Timing => "timing",
            Pillar::Resilience => "resilience",
            Pillar::Proof => "proof",
            Pillar::Dossier => "dossier",
        }
    }
}

/// Court identifier. 20 courts, N01..N20.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum CourtId {
    /// Court 01.
    N01 = 1,
    /// Court 02.
    N02 = 2,
    /// Court 03.
    N03 = 3,
    /// Court 04.
    N04 = 4,
    /// Court 05.
    N05 = 5,
    /// Court 06.
    N06 = 6,
    /// Court 07.
    N07 = 7,
    /// Court 08.
    N08 = 8,
    /// Court 09.
    N09 = 9,
    /// Court 10.
    N10 = 10,
    /// Court 11.
    N11 = 11,
    /// Court 12.
    N12 = 12,
    /// Court 13.
    N13 = 13,
    /// Court 14.
    N14 = 14,
    /// Court 15.
    N15 = 15,
    /// Court 16.
    N16 = 16,
    /// Court 17.
    N17 = 17,
    /// Court 18.
    N18 = 18,
    /// Court 19.
    N19 = 19,
    /// Court 20.
    N20 = 20,
}

impl CourtId {
    /// Numeric court number (1..=20).
    pub fn id_number(self) -> u8 {
        self as u8
    }

    /// Canonical name (`N01`..`N20`), read from [`COURT_NAMES`].
    pub fn canonical(self) -> &'static str {
        COURT_NAMES[(self.id_number() - 1) as usize]
    }
}

/// A named candidate under adjudication (e.g. a subject SHA or build id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateRef(pub String);

/// A positive control: a `fn() -> bool` that must return `true` for the
/// court to be allowed to run. Implementations should be cheap and
/// deterministic (e.g. verify a known-bad input is actually caught).
pub type ControlProbe = fn() -> bool;

/// A single measured metric emitted by a court.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    /// Unsigned 64-bit metric.
    U64(u64),
    /// Floating-point metric.
    F64(f64),
    /// Boolean metric.
    Bool(bool),
    /// String metric.
    Str(String),
}

impl MetricValue {
    /// Stable machine tag used in canonical receipt strings and JSON.
    pub fn tag(&self) -> &'static str {
        match self {
            MetricValue::U64(_) => "u64",
            MetricValue::F64(_) => "f64",
            MetricValue::Bool(_) => "bool",
            MetricValue::Str(_) => "str",
        }
    }

    /// Canonical scalar rendering used in canonical receipt strings.
    pub fn render(&self) -> String {
        match self {
            MetricValue::U64(v) => v.to_string(),
            MetricValue::F64(v) => {
                // Total, deterministic rendering; no panics.
                let s = format!("{:?}", v);
                s
            }
            MetricValue::Bool(v) => v.to_string(),
            MetricValue::Str(s) => s.clone(),
        }
    }
}

/// Outcome returned by a court's `run` method.
#[derive(Debug, Clone)]
pub struct CourtOutcome {
    /// Byte verdict of the run.
    pub verdict: Verdict,
    /// Named metrics measured by the court.
    pub metrics: Vec<(&'static str, MetricValue)>,
    /// Scoping disclosure for the run.
    pub scoping: ScopingRecord,
}

/// A court: a pillar-classified adversarial evaluator with a mandatory
/// positive control.
pub trait Court {
    /// Dyn-safe court identifier accessor. Concrete courts define the
    /// identity as associated consts (`const ID` / `const PILLAR`) and
    /// forward these methods to them.
    fn id(&self) -> CourtId;

    /// Dyn-safe pillar accessor (see [`Court::id`]).
    fn pillar(&self) -> Pillar;

    /// Canonical court name.
    fn name(&self) -> &'static str;

    /// The mandatory positive-control probe.
    fn positive_control(&self) -> ControlProbe;

    /// Run the court's adversarial evaluation of the candidate.
    fn run(&self, candidate: &CandidateRef) -> CourtOutcome;
}

/// Run `court` against `candidate` on behalf of `subject_sha`.
///
/// The court's positive control runs FIRST. If the probe returns `false`,
/// the court's `run` is never called and a `Verdict::Refused` receipt is
/// returned with `ControlResult::GateRefused`.
pub fn run_court(court: &dyn Court, candidate: &CandidateRef, subject_sha: &str) -> CourtReceipt {
    let probe = court.positive_control();
    let control_result = control::run_probe(probe);

    let outcome = if control::probe_passed(&control_result) {
        court.run(candidate)
    } else {
        CourtOutcome {
            verdict: Verdict::Refused,
            metrics: Vec::new(),
            scoping: ScopingRecord::refused_scope(),
        }
    };

    CourtReceipt::record(
        court.id(),
        court.pillar(),
        court.name(),
        subject_sha,
        outcome,
        control_result,
    )
}
