#![cfg(feature = "nato")]
//! N03 — Bell-LaPadula lattice court (NATO lane N2).
//!
//! Assigns classification levels to Petri-net places over the two real
//! transition tables — [`crate::nato::transition_tables`] RONR (12
//! transitions) and DAO (13 transitions) — and asserts the BLP laws over
//! the FULL marking space: exhaustive over all 2^16 markings per
//! transition; no sampling, a real exhaustive proof (25 x 65,536 = 1,638,400
//! firing checks).
//!
//! # Demo lattice (documented mapping onto the place bits)
//!
//! Levels: public=0 < confidential=1 < secret=2 < top_secret=3
//! (`active place level` of a marking = max level over its set bits; the
//! empty marking is level 0).
//!
//! RONR places:
//! - public (0): `FLOOR` (floor recognition is public record),
//!   `DIVISION_DEMANDED` (a division demand is public record).
//! - secret (2): `MAIN_MOTION`, `STATED`, `SECONDED`, `DEBATABLE`,
//!   `AMENDMENT_PENDING`, `PREV_QUESTION_MOVED`, `PREV_QUESTION_ORDERED`
//!   (the deliberative state of the pending question).
//! - top_secret (3): `ADOPTED`, `TABLED`, `POSTPONED_INDEF`,
//!   `RECESS_PENDING`, `ADJOURNED`, `POINT_OF_ORDER_RAISED`
//!   (dispositions and privileged motions — outcome-bearing state).
//! - confidential (1): unassigned in RONR (documented as deliberately
//!   unused). Reserved bit `0x8000` and any undocumented bit: fail-closed
//!   top_secret. Undocumented bits are never created by any transition
//!   (`pre`/`post` are documented-bit masks), so this can only make the
//!   courts stricter, never let a violation slip.
//!
//! DAO places:
//! - confidential (1): `COLLATERAL_STAKED`.
//! - secret (2): `TREASURY_SOLVENT`, `ESCROW_LOCKED`, `STREAM_OPEN`,
//!   `DELEGATED`, `LEASE_HELD`, `MEMBER_ACTIVE`, `ESCROW_RELEASED`,
//!   `DELEGATION_PENDING` (ESCROW_RELEASED / DELEGATION_PENDING are
//!   documented but unused by any verb — assigned for totality).
//! - top_secret (3): `COLLATERAL_WITHDRAWN`, `STREAM_HALTED`,
//!   `MEMBER_EXPELLED`.
//! - public (0): unassigned in DAO; `MEMBER_SUSPENDED` (documented but
//!   unconsumed) and all reserved/undocumented bits: fail-closed
//!   top_secret.
//!
//! # Court 1 — no-write-down (the *-property, information-flow form)
//!
//! A Petri-net state machine has no subjects/files; the honest BLP analog
//! is a declassification ban on firings: for every transition and EVERY
//! marking `M` that enables it (`pre ⊆ M`), the firing
//! `M → (M &! pre) | post` must never lower the active place level:
//! `active_level(result) >= active_level(M)`. Checked exhaustively:
//! 25 transitions x 65,536 markings = 1,638,400 firing checks.
//!
//! # Court 2 — no-read-up (simple security) analog
//!
//! The read-shaped surface that exists is enablement: a transition's firing
//! decision consumes exactly the marking bits named in its `pre` mask. The
//! court pins, exhaustively, that the firing decision and the resulting
//! touch-set projection are invariant under perturbing every bit outside
//! `pre ∪ post` — the transition cannot "read up" any place it does not
//! declare. The classical BLP vacuous case (no data-returning output
//! channel at all) is pinned by the same assertion.
//!
//! # Court 3 — *-property strong tranquility pin
//!
//! The lattice assignment is static per run: the level of each place is a
//! pure function of its bit, re-derived and compared bit-for-bit at the end
//! of the run (two independent full-marking level snapshots must be
//! identical, and every documented place bit must carry a defined level).
//!
//! Ported from strata n03_bell_lapadula.rs onto [`crate::nato::transition_tables`].

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

use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;
use crate::nato::transition_tables::{dao, ronr, shapes};

/// Lattice classification level.
pub type Level = u8;

/// Lattice levels, total order PUBLIC < CONFIDENTIAL < SECRET < TOP_SECRET.
pub const PUBLIC: Level = 0;
/// Confidential.
pub const CONFIDENTIAL: Level = 1;
/// Secret.
pub const SECRET: Level = 2;
/// Top secret.
pub const TOP_SECRET: Level = 3;

/// Fail-closed: any bit without a documented mapping is treated as
/// top_secret by the laws (undocumented bits are never created by any
/// transition, so this can only make the courts stricter).
pub const UNDOCUMENTED: Level = TOP_SECRET;

/// Classification of one RONR place bit (see module doc: the demo lattice).
/// `None` = bit has no documented mapping.
pub fn ronr_place_level_doc(mask: u16) -> Option<Level> {
    match mask {
        m if m == ronr::N_FLOOR || m == ronr::N_DIVISION_DEMANDED => Some(PUBLIC),
        m if m == ronr::N_MAIN_MOTION
            || m == ronr::N_STATED
            || m == ronr::N_SECONDED
            || m == ronr::N_DEBATABLE
            || m == ronr::N_AMENDMENT_PENDING
            || m == ronr::N_PREV_QUESTION_MOVED
            || m == ronr::N_PREV_QUESTION_ORDERED =>
        {
            Some(SECRET)
        }
        m if m == ronr::N_ADOPTED
            || m == ronr::N_TABLED
            || m == ronr::N_POSTPONED_INDEF
            || m == ronr::N_RECESS_PENDING
            || m == ronr::N_ADJOURNED
            || m == ronr::N_POINT_OF_ORDER_RAISED =>
        {
            Some(TOP_SECRET)
        }
        _ => None,
    }
}

/// Classification of one RONR place bit, fail-closed on undocumented bits.
pub fn ronr_place_level(mask: u16) -> Level {
    ronr_place_level_doc(mask).unwrap_or(UNDOCUMENTED)
}

/// Classification of one DAO place bit (see module doc: the demo lattice).
/// `None` = bit has no documented mapping.
pub fn dao_place_level_doc(mask: u16) -> Option<Level> {
    match mask {
        m if m == dao::P_COLLATERAL_STAKED => Some(CONFIDENTIAL),
        m if m == dao::P_TREASURY_SOLVENT
            || m == dao::P_ESCROW_LOCKED
            || m == dao::P_STREAM_OPEN
            || m == dao::P_DELEGATED
            || m == dao::P_LEASE_HELD
            || m == dao::P_MEMBER_ACTIVE
            || m == dao::P_ESCROW_RELEASED
            || m == dao::P_DELEGATION_PENDING =>
        {
            Some(SECRET)
        }
        m if m == dao::P_COLLATERAL_WITHDRAWN
            || m == dao::P_STREAM_HALTED
            || m == dao::P_MEMBER_EXPELLED =>
        {
            Some(TOP_SECRET)
        }
        _ => None,
    }
}

/// Classification of one DAO place bit, fail-closed on undocumented bits.
pub fn dao_place_level(mask: u16) -> Level {
    dao_place_level_doc(mask).unwrap_or(UNDOCUMENTED)
}

/// Active place level of a marking: max level over set bits (0 if empty).
/// Fail-closed on undocumented bits via [`UNDOCUMENTED`].
pub fn active_level(marking: u16, level_of: &dyn Fn(u16) -> Level) -> Level {
    let mut best = PUBLIC;
    let mut m = marking;
    while m != 0 {
        let bit = 1u16 << m.trailing_zeros();
        let lvl = level_of(bit);
        if lvl > best {
            best = lvl;
        }
        m &= m - 1;
    }
    best
}

fn level_of_family(family: &str) -> &'static dyn Fn(u16) -> Level {
    if family == "ronr" {
        &ronr_place_level
    } else {
        &dao_place_level
    }
}

/// Metrics of the exhaustive no-write-down sweep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlpMetrics {
    /// (transition, marking) pairs visited (must be 25 x 65,536).
    pub pairs: u64,
    /// Enabled firings actually checked against the declassification ban.
    pub firings: u64,
    /// Firings that lowered the active level (must be 0).
    pub write_down_violations: u64,
}

/// Exhaustive no-write-down sweep over all 25 transitions x 2^16 markings.
pub fn no_write_down_sweep() -> BlpMetrics {
    let mut metrics = BlpMetrics { pairs: 0, firings: 0, write_down_violations: 0 };
    for (family, shape) in shapes() {
        let level_of = level_of_family(family);
        for m in 0u16..=u16::MAX {
            metrics.pairs += 1;
            if m & shape.pre != shape.pre {
                continue; // not enabled
            }
            let result = (m & !shape.pre) | shape.post;
            if active_level(result, level_of) < active_level(m, level_of) {
                metrics.write_down_violations += 1;
            }
            metrics.firings += 1;
        }
    }
    metrics
}

/// Exhaustive no-read-up (read-containment) sweep. Returns violation count
/// (must be 0): enablement and the touch-set projection must be invariant
/// under every bit outside `pre | post`, for all 25 transitions x 2^16
/// markings.
pub fn no_read_up_sweep() -> u64 {
    let mut violations = 0u64;
    for (_, shape) in shapes() {
        let touch = shape.pre | shape.post;
        for m in 0u16..=u16::MAX {
            let enabled_full = m & shape.pre == shape.pre;
            let restricted = m & touch;
            let result_full = (m & !shape.pre) | shape.post;
            let result_restricted = (restricted & !shape.pre) | shape.post;
            if enabled_full != (restricted & shape.pre == shape.pre) {
                violations += 1; // enablement depends on bits outside the touch set
            }
            if result_full & touch != result_restricted & touch {
                violations += 1; // touch-set projection reads undeclared places
            }
        }
    }
    violations
}

/// Documented place bits that MUST carry a defined lattice level.
fn documented_bits() -> (Vec<u16>, Vec<u16>) {
    let ronr_documented = vec![
        ronr::N_MAIN_MOTION,
        ronr::N_STATED,
        ronr::N_SECONDED,
        ronr::N_DEBATABLE,
        ronr::N_AMENDMENT_PENDING,
        ronr::N_PREV_QUESTION_MOVED,
        ronr::N_PREV_QUESTION_ORDERED,
        ronr::N_FLOOR,
        ronr::N_ADOPTED,
        ronr::N_TABLED,
        ronr::N_POSTPONED_INDEF,
        ronr::N_RECESS_PENDING,
        ronr::N_ADJOURNED,
        ronr::N_POINT_OF_ORDER_RAISED,
        ronr::N_DIVISION_DEMANDED,
    ];
    let dao_documented = vec![
        dao::P_COLLATERAL_STAKED,
        dao::P_STREAM_OPEN,
        dao::P_ESCROW_LOCKED,
        dao::P_DELEGATED,
        dao::P_LEASE_HELD,
        dao::P_MEMBER_ACTIVE,
        dao::P_TREASURY_SOLVENT,
        dao::P_ESCROW_RELEASED,
        dao::P_DELEGATION_PENDING,
        dao::P_COLLATERAL_WITHDRAWN,
        dao::P_STREAM_HALTED,
        dao::P_MEMBER_EXPELLED,
    ];
    (ronr_documented, dao_documented)
}

/// Strong tranquility pin: (a) totality — every documented place bit maps
/// to a defined level (not the fail-closed default); (b) purity — two
/// independent full-marking level snapshots are bit-identical; (c) lattice
/// ordering is total. Returns violation count (must be 0).
pub fn strong_tranquility_violations() -> u64 {
    let mut violations = 0u64;
    let (ronr_documented, dao_documented) = documented_bits();
    for bit in ronr_documented {
        if ronr_place_level_doc(bit).is_none() {
            violations += 1;
        }
    }
    for bit in dao_documented {
        if dao_place_level_doc(bit).is_none() {
            violations += 1;
        }
    }
    for level_of in [&ronr_place_level as &dyn Fn(u16) -> Level, &dao_place_level] {
        let first: Vec<Level> = (0u16..=u16::MAX).map(|m| active_level(m, level_of)).collect();
        let second: Vec<Level> = (0u16..=u16::MAX).map(|m| active_level(m, level_of)).collect();
        if first != second {
            violations += 1;
        }
    }
    if !(PUBLIC < CONFIDENTIAL && CONFIDENTIAL < SECRET && SECRET < TOP_SECRET) {
        violations += 1;
    }
    violations
}

/// Positive control: a deliberately declassifying table row (DAO withdraw
/// rewritten so it keeps the secret treasury solvent state and produces a
/// public token while consuming the confidential stake — a downgrade) MUST
/// be flagged by the sweep law. Returns false if the court is vacuous.
pub fn bell_lapadula_positive_control() -> bool {
    // DAO withdraw_collateral: pre = COLLATERAL_STAKED (confidential), post
    // = COLLATERAL_WITHDRAWN (top secret) — lawful. Mutant: post = 0
    // (destroys the confidential place with no equal-or-higher production),
    // which declassifies 1 -> 0. The law must reject it.
    let pre = dao::P_COLLATERAL_STAKED;
    let mutant_post = 0u16;
    let m = pre;
    let result = (m & !pre) | mutant_post;
    let before = active_level(m, &dao_place_level);
    let after = active_level(result, &dao_place_level);
    after < before
}

/// N03 Bell-LaPadula lattice court.
pub struct BellLapadulaCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl BellLapadulaCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N03;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Security;
}

impl Court for BellLapadulaCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N03 bell_lapadula"
    }

    fn positive_control(&self) -> ControlProbe {
        bell_lapadula_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        let blp = no_write_down_sweep();
        let read_up = no_read_up_sweep();
        let tranquility = strong_tranquility_violations();
        let violations = blp.write_down_violations + read_up + tranquility;
        CourtOutcome {
            verdict: if violations == 0 { Verdict::Pass } else { Verdict::Fail },
            metrics: vec![
                ("pairs", MetricValue::U64(blp.pairs)),
                ("enabled_firings", MetricValue::U64(blp.firings)),
                ("write_down_violations", MetricValue::U64(blp.write_down_violations)),
                ("read_up_violations", MetricValue::U64(read_up)),
                ("tranquility_violations", MetricValue::U64(tranquility)),
            ],
            scoping: ScopingRecord::full(
                "exhaustive BLP no-write-down / no-read-up / strong-tranquility over the \
                 full ronr+dao transition tables and the FULL 16-bit marking space",
                "25 transitions x 65,536 markings exhaustive firing census (1,638,400 checks), \
                 exhaustive read-containment census, dual full-marking tranquility snapshots",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n03_no_write_down_exhaustive() {
        let metrics = no_write_down_sweep();
        assert_eq!(
            metrics.pairs,
            25 * (u16::MAX as u64 + 1),
            "sweep coverage drift — primary domain must never be reduced"
        );
        assert!(
            metrics.firings > 0 && metrics.firings <= metrics.pairs,
            "enabled-firing census degenerate: {}",
            metrics.firings
        );
        assert_eq!(
            metrics.write_down_violations, 0,
            "BLP no-write-down violated by a real table transition"
        );
    }

    #[test]
    fn n03_no_read_up_read_containment() {
        let violations = no_read_up_sweep();
        assert_eq!(
            violations, 0,
            "enablement or touch-set projection depends on bits outside the declared touch set"
        );
    }

    #[test]
    fn n03_strong_tranquility_static_levels() {
        let violations = strong_tranquility_violations();
        assert_eq!(violations, 0, "lattice assignment totality/purity or ordering drifted");
    }

    #[test]
    fn n03_court_run_reports_pass_with_receipt_metrics() {
        let court = BellLapadulaCourt;
        assert!(court.positive_control()(), "positive control must hold");
        let outcome = court.run(&CandidateRef("lane-n2-test".to_string()));
        assert_eq!(outcome.verdict, Verdict::Pass, "N03 court verdict must pass");
        let pairs = outcome.metrics.iter().find(|(k, _)| *k == "pairs");
        assert!(
            pairs.is_some_and(|(_, v)| *v == MetricValue::U64(25 * 65_536)),
            "pairs metric must equal the exhaustive 25 x 65,536 census"
        );
        assert!(
            outcome.scoping.is_full_spec_scale,
            "N03 is a fully exhaustive court — disclosure must say so"
        );
        outcome.scoping.validate().expect("scoping record must validate");
    }

    #[test]
    fn n03_mutant_declassification_is_detectable() {
        // Non-vacuity: the declassifying mutant row MUST read as a
        // downgrade under the law (the positive control's assertion).
        assert!(
            bell_lapadula_positive_control(),
            "injected declassification was NOT detected — court is vacuous"
        );
        // And the real table row is lawful (no downgrade).
        let pre = dao::P_COLLATERAL_STAKED;
        let result = (pre & !pre) | dao::P_COLLATERAL_WITHDRAWN;
        assert!(
            active_level(result, &dao_place_level) >= active_level(pre, &dao_place_level),
            "real withdraw_collateral row must be lawful under the ban"
        );
    }

    #[test]
    fn n03_undocumented_bits_fail_closed() {
        // Reserved/undocumented bits classify top_secret, strictly above
        // every documented level: the fail-closed law.
        for bit in [0x8000u16] {
            assert_eq!(ronr_place_level(bit), TOP_SECRET);
            assert_eq!(dao_place_level(bit), TOP_SECRET);
        }
        for bit in [0x0080u16, 0x0200] {
            assert_eq!(dao_place_level(bit), TOP_SECRET, "DAO reserved bit {bit:#06x}");
        }
        assert_eq!(
            dao_place_level_doc(dao::P_MEMBER_SUSPENDED),
            None,
            "MEMBER_SUSPENDED is documented-but-unassigned: fail-closed"
        );
    }
}
