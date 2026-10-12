#![cfg(feature = "nato")]
//! N01 — MILS non-interference court (NATO lane N2).
//!
//! Multiple Independent Levels of Security (MILS) separation over the
//! dual-word PN64/OCEL64 state word, using the landed conformance kernel
//! `crate::conformance::pn64` (`pack`/`enabled`/`fire`/`step_branchless`,
//! `ct_eq`/`ct_lte`) as the advance law.
//!
//! # MILS partition (the security model under test)
//!
//! The 128-bit dual-word state (`pn || ocel`) is partitioned:
//!
//! - **Region A (unclassified tenant)** — `pn.marking [15:0]`,
//!   `pn.pre [31:16]`, `pn.post [47:32]`, `pn.tid [55:48]`, and the
//!   `ocel.activity` low byte that feeds the verb-match gate. `state_A` is
//!   the resulting marking bits [15:0] plus the typed refusal flag.
//! - **Region B (secret tenant)** — payload bits the advance law never
//!   reads: `pn.flags [63:56]` and the `ocel` bytes above the low byte
//!   (`src`/`tgt`/`qual`/`tick` analogues, bits [63:8]).
//!
//! # Non-interference law (HARD FAIL on any violation)
//!
//! For every A-state and A-input, `state_A(B armed) == state_A(B zeroed)`:
//! the marking bits [15:0] and the refusal verdict must not diverge under
//! B's payload presence. This is checked exhaustively over ALL 2^16
//! markings x 16 transition shapes x 3 activity regimes x 2 tick values,
//! with armed-B payload combinations tiered per the P7 sampling law
//! (default 2 per cell; strata-exact 8 per cell behind `#[ignore]`), plus
//! a calibrated 10^7 dual-tenant random sweep behind `#[ignore]`.
//!
//! # Permitted cross-region couplings (enumerated + pinned)
//!
//! - **C1 (verb-match conformance gate)**: `(activity & 0xFF) == tid` is
//!   the only read crossing from the activity field into A's admission; its
//!   sole authority is the typed refusal (bit-exact unchanged word) — it
//!   can never write a marking bit. Pinned: gate decisions are identical
//!   with B armed and B zeroed, and a closed gate never moves a marking.
//! - **C2 (B-region orthogonality)**: the B bytes of `ocel` (bits [63:8])
//!   and the whole `pn.flags` byte are pure payload — the advance outcome
//!   is identical armed vs zeroed. This is the kernel-law pin replacing
//!   strata's tick-predicate pin (the upstream `expected_tick_ok` surface
//!   has no referent in the ported kernel; see the scoping bound).

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

use crate::conformance::pn64;
use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;
use crate::nato::rng::Lcg;

/// Fixed deterministic LCG seed (replayable; never wall-clock derived).
pub const LCG_SEED: u64 = 0x4E30_312A_0000_0001;

/// B payload: which pn.flags byte to arm.
const B_FLAGS: [u8; 4] = [0x00, 0x5A, 0xA5, 0xFF];
/// B payload: which ocel src/tgt words to arm.
const B_WORD16: [u16; 4] = [0x0000, 0x1248, 0xA5C3, 0xFFFF];
/// B payload: which ocel qualifier bytes to arm.
const B_QUAL: [u8; 3] = [0x00, 0x96, 0xFF];

/// mask_A: A's forbidden-bit invariant mask (applies to the next marking,
/// A region, identical across both arms).
pub const MASK_A: u64 = 0x0000_0000_0000_0004;

/// Sampled transition shapes: (pre, post, tid) — A's own transition law.
/// Verbatim from strata n01.
const TRANSITIONS: [(u16, u16, u8); 16] = [
    (0x0001, 0x0002, 0x01),
    (0x0002, 0x000C, 0x02),
    (0xFFFF, 0x8001, 0x7F),
    (0x8000, 0x0001, 0x80),
    (0x0000, 0x0001, 0xFF), // unconditional produce (always enabled)
    (0x0001, 0x0000, 0x01), // pure consume
    (0x0001, 0x0001, 0x01), // self-loop
    (0x0003, 0x0004, 0x03),
    (0x0080, 0x0040, 0x40),
    (0x0FF0, 0x00F0, 0x0F),
    (0x00F0, 0x0FF0, 0xF0),
    (0x0002, 0x0002, 0x02),
    (0x5555, 0xAAAA, 0x55),
    (0xAAAA, 0x5555, 0xAA),
    (0x0100, 0x0200, 0x10),
    (0x0010, 0x0020, 0x81),
];

/// A activity regimes: conformance-holding, conformance-broken, high-byte
/// noise (exercises the `& 0xFF` conformance law). Regime-relative to tid
/// per cell: `activity = regime.wrapping_add(tid)`.
const ACTIVITY_REGIMES: [u16; 3] = [0x0000, 0x0102, 0x0203];

/// Armed-B combinations sampled per cell in the DEFAULT run.
///
/// Tiering law (P7): the primary domain (2^16 markings x 16 transitions x
/// 3 regimes x 2 ticks) is NEVER reduced; only this sampled dimension is
/// tiered — strata's exhaustive default is 8, the default court runs 2,
/// and the strata-exact 8-combo sweep is behind `#[ignore]`.
pub const DEFAULT_B_COMBOS: usize = 2;
/// Strata-exact armed-B combinations per cell (tier-3 run).
pub const STRATA_B_COMBOS: usize = 8;

/// Outcome of one kernel advance, projected onto the A-visible surface:
/// `(marking bits [15:0], refusal verdict)`.
pub type StateA = (u16, bool);

/// `state_A`: run the kernel advance law and project the A-visible outcome.
///
/// The refusal verdict is re-derived independently of the kernel's internal
/// selection arithmetic (differential reference model): the advance refuses
/// iff the verb gate is closed, a precondition token is missing, or the
/// post-fire marking violates the forbidden-bit mask.
pub fn state_a(pn: u64, ocel: u64, invariant_mask: u64) -> StateA {
    let next = pn64::step_branchless(pn, ocel, invariant_mask);
    let m = pn & 0xFFFF;
    let w_pre = (pn >> 16) & 0xFFFF;
    let w_post = (pn >> 32) & 0xFFFF;
    let tid = (pn >> 48) & 0xFF;
    let activity = ocel & 0xFF;
    let next_m = ((m ^ w_pre) | w_post) & 0xFFFF;
    let verb_ok = (activity & 0xFF) == tid;
    let token_ok = (m & w_pre) == w_pre;
    let inv_ok = (next_m & (invariant_mask & 0xFFFF)) == 0;
    let refused = !(verb_ok && token_ok && inv_ok);
    ((next & 0xFFFF) as u16, refused)
}

/// Build a composite pn word: A's fields with B's flags byte armed.
pub fn composite_pn(m: u16, pre: u16, post: u16, tid: u8, flags_b: u8) -> u64 {
    pn64::pack(m, pre, post, tid, flags_b)
}

/// Build a composite ocel word: A's activity byte with B's src/tgt/qual
/// armed (or zeroed). The low byte carries the activity (verb gate input);
/// bits [63:8] are the B region.
pub fn composite_ocel(activity: u16, tick: u8, b: (u16, u16, u8)) -> u64 {
    let _ = tick; // tick is a B-region payload byte in this kernel
    (activity as u64)
        | ((b.0 as u64) << 16)
        | ((b.1 as u64) << 32)
        | ((b.2 as u64) << 48)
        | ((tick as u64) << 56)
}

/// Non-interference check for one cell. Returns `false` on any state_A
/// divergence between B-armed and B-zeroed evaluations (message carried by
/// the caller's assert on the violation census).
pub fn cell_check(
    m: u16,
    pre: u16,
    post: u16,
    tid: u8,
    activity: u16,
    tick: u8,
    b: (u8, u16, u16, u8),
) -> bool {
    let pn_b = composite_pn(m, pre, post, tid, b.0);
    let ocel_b = composite_ocel(activity, tick, (b.1, b.2, b.3));
    let pn_z = composite_pn(m, pre, post, tid, 0x00);
    let ocel_z = composite_ocel(activity, tick, (0, 0, 0));
    let sa_b = state_a(pn_b, ocel_b, MASK_A);
    let sa_z = state_a(pn_z, ocel_z, MASK_A);
    sa_b == sa_z
}

/// Census of one sweep tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MilMetrics {
    /// (marking, transition, regime, tick) cells audited.
    pub cells: u64,
    /// Armed-B evaluations performed (cells x b_combos).
    pub armed_b_audits: u64,
    /// Cells where armed vs zeroed state_A diverged (must be 0).
    pub interference_violations: u64,
    /// Fired-branch census from the calibration arm (exhaustive sweeps: 0).
    pub fired_samples: u64,
    /// Refused-branch census from the calibration arm (exhaustive sweeps: 0).
    pub refused_samples: u64,
}

/// Exhaustive small-domain sweep: ALL 2^16 markings x 16 transition shapes
/// x 3 activity regimes x 2 ticks, with `b_combos` LCG-sampled armed-B
/// payload combinations per cell.
pub fn sweep_exhaustive(b_combos: usize) -> MilMetrics {
    let mut lcg = Lcg::new(LCG_SEED);
    let mut metrics = MilMetrics {
        cells: 0,
        armed_b_audits: 0,
        interference_violations: 0,
        fired_samples: 0,
        refused_samples: 0,
    };
    for m in 0..=u16::MAX {
        for &(pre, post, tid) in TRANSITIONS.iter() {
            for &regime in ACTIVITY_REGIMES.iter() {
                // Activity regime-relative to tid so all three conformance
                // regimes are exercised per cell.
                let activity = regime.wrapping_add(tid as u16);
                for tick in [0x00u8, 0x7F] {
                    for _ in 0..b_combos {
                        let bf = B_FLAGS[(lcg.sample(4)) as usize];
                        let b0 = B_WORD16[(lcg.sample(4)) as usize];
                        let b1 = B_WORD16[(lcg.sample(4)) as usize];
                        let bq = B_QUAL[(lcg.sample(3)) as usize];
                        if !cell_check(m, pre, post, tid, activity, tick, (bf, b0, b1, bq)) {
                            metrics.interference_violations += 1;
                        }
                        metrics.armed_b_audits += 1;
                    }
                    metrics.cells += 1;
                }
            }
        }
    }
    metrics
}

/// Calibrated 10^7 dual-tenant LCG sweep over the full joint domain. Half
/// the samples are shaped admissible (enabled + conformance holds) so the
/// fired branch of state_A is materially exercised; the other half stay
/// fully random (malformed/starving dominated).
pub fn sweep_dual_tenant(n_samples: u64) -> MilMetrics {
    let mut lcg = Lcg::new(LCG_SEED ^ 0x2026_1012);
    let mut metrics = MilMetrics {
        cells: 0,
        armed_b_audits: 0,
        interference_violations: 0,
        fired_samples: 0,
        refused_samples: 0,
    };
    for _ in 0..n_samples {
        let mut m = lcg.sample(1 << 16) as u16;
        let pre = lcg.sample(1 << 16) as u16;
        let post = lcg.sample(1 << 16) as u16;
        let tid = lcg.sample(1 << 8) as u8;
        let mut activity = lcg.sample(1 << 16) as u16;
        let tick = lcg.sample(1 << 8) as u8;
        if lcg.sample(2) == 0 {
            // Calibration: enabled (m superset of pre) + conformance holds.
            m |= pre;
            activity = (activity & 0xFF00) | tid as u16;
        }
        let bf = lcg.sample(1 << 8) as u8;
        let b0 = lcg.sample(1 << 16) as u16;
        let b1 = lcg.sample(1 << 16) as u16;
        let bq = lcg.sample(1 << 8) as u8;
        if !cell_check(m, pre, post, tid, activity, tick, (bf, b0, b1, bq)) {
            metrics.interference_violations += 1;
        }
        let (_, refused) = state_a(
            composite_pn(m, pre, post, tid, bf),
            composite_ocel(activity, tick, (b0, b1, bq)),
            MASK_A,
        );
        if refused {
            metrics.refused_samples += 1;
        } else {
            metrics.fired_samples += 1;
        }
        metrics.armed_b_audits += 1;
    }
    metrics
}

/// Coupling-pin violations (must be empty). Enumerates and pins the entire
/// permitted cross-region coupling surface of the kernel advance law.
pub fn coupling_pin_violations() -> Vec<&'static str> {
    let mut violations = Vec::new();
    let tid = 0x02u8;
    let m = 0x0082u16;
    let pre = 0x0002u16;
    let post = 0x000Cu16;
    let activity_open = 0x0102u16; // low byte == tid: gate open
    let activity_closed = 0x0103u16; // low byte != tid: gate closed

    // C1 pin 1: a CLOSED gate never moves a marking and never writes a
    // marking bit, with B fully armed.
    let pn_armed = composite_pn(m, pre, post, tid, 0x5A);
    let ocel_closed = composite_ocel(activity_closed, 0x41, (0xA5C3, 0x1248, 0x96));
    let (marking, refused) = state_a(pn_armed, ocel_closed, MASK_A);
    if marking != m || !refused {
        violations.push("C1: closed gate moved the marking or failed to refuse");
    }

    // C1 pin 2: the closed-gate decision is identical with B armed and
    // B zeroed (no B side-channel through the conformance gate).
    let ocel_closed_z = composite_ocel(activity_closed, 0x41, (0, 0, 0));
    if state_a(pn_armed, ocel_closed, MASK_A) != state_a(pn_armed, ocel_closed_z, MASK_A) {
        violations.push("C1: closed-gate verdict diverged under B presence");
    }

    // C1 pin 3 (strictness): with the gate OPEN the advance commits;
    // tampering with tid alone (same B payload) closes it — the gate is
    // load-bearing. Post mask 0x0008 keeps the post-fire marking clear of
    // the MASK_A forbidden bit (0x0004).
    let post_open = 0x0008u16;
    let pn_open = composite_pn(m, pre, post_open, tid, 0x5A);
    let ocel_open = composite_ocel(activity_open, 0x41, (0xA5C3, 0x1248, 0x96));
    let (open_marking, open_refused) = state_a(pn_open, ocel_open, MASK_A);
    if open_refused {
        violations.push("C1: open gate refused a conforming enabled advance");
    }
    if open_marking != ((m & !pre) | post_open) {
        violations.push("C1: open-gate advance wrote a wrong marking");
    }
    let pn_tampered = composite_pn(m, pre, post_open, 0x03, 0x5A);
    let (_, tampered_refused) = state_a(pn_tampered, ocel_open, MASK_A);
    if !tampered_refused {
        violations.push("C1: tid tamper did not close the conformance gate");
    }

    // C1 pin 4: high-byte noise in the activity word cannot flip admission
    // (`& 0xFF` law): activity 0x0202 vs 0x0002 decide identically.
    let ocel_hi = composite_ocel(0x0202, 0x41, (0xA5C3, 0x1248, 0x96));
    let ocel_lo = composite_ocel(0x0002, 0x41, (0, 0, 0));
    if state_a(pn_armed, ocel_hi, MASK_A) != state_a(pn_armed, ocel_lo, MASK_A) {
        violations.push("C1: high activity byte flipped admission");
    }

    // C2 pin: every B-region byte (ocel bits [63:8], pn.flags byte) is pure
    // payload — the advance outcome is identical armed vs zeroed with the
    // gate open. (Kernel-law pin replacing strata's tick-predicate pin; see
    // the scoping bound.)
    for flags_b in [0x00u8, 0x5A, 0xFF] {
        let pn_b = composite_pn(m, pre, post, tid, flags_b);
        let pn_z = composite_pn(m, pre, post, tid, 0x00);
        if state_a(pn_b, ocel_open, MASK_A) != state_a(pn_z, ocel_open, MASK_A) {
            violations.push("C2: flags byte leaked into the advance outcome");
        }
        if state_a(pn_b, ocel_open, MASK_A) != state_a(pn_b, ocel_lo, MASK_A) {
            violations.push("C2: ocel B-region bytes leaked into the advance outcome");
        }
    }
    violations
}

// ---------------------------------------------------------------------------
// Anti-vacuity: the court must be able to FAIL.
// ---------------------------------------------------------------------------

/// A deliberately interfering advance oracle (ORs B's flags bit 0 into the
/// next marking) — stands in for the defect class this court exists to
/// catch. The court MUST flag it.
pub fn leaky_advance(pn: u64, ocel: u64, invariant_mask: u64) -> StateA {
    let next = pn64::step_branchless(pn, ocel, invariant_mask);
    let leak = (pn64::flags(pn) & 0x01) as u16;
    ((next & 0xFFFF) as u16 | leak, false)
}

/// Positive control: the injected-interference mutant MUST be caught by the
/// armed-vs-zeroed comparison. Returns false if the court is vacuous.
pub fn mils_positive_control() -> bool {
    let m = 0x0082u16;
    let (pre, post, tid) = (0x0002u16, 0x000Cu16, 0x02u8);
    let activity = 0x0102u16;
    let tick = 0x41u8;
    let outcome = |flags_b: u8, armed: bool| -> StateA {
        let pn = composite_pn(m, pre, post, tid, flags_b);
        let ocel = if armed {
            composite_ocel(activity, tick, (0xA5C3, 0x1248, 0x96))
        } else {
            composite_ocel(activity, tick, (0, 0, 0))
        };
        // Invariant mask 0: the injected leak, not a shared refusal, decides
        // the outcome.
        leaky_advance(pn, ocel, 0)
    };
    let armed = outcome(0x01, true);
    let zeroed = outcome(0x00, false);
    armed != zeroed
}

// ---------------------------------------------------------------------------
// Court wiring
// ---------------------------------------------------------------------------

/// N01 MILS separation court.
pub struct MilSeparationCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl MilSeparationCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N01;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Security;
}

impl Court for MilSeparationCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N01 mils_separation"
    }

    fn positive_control(&self) -> ControlProbe {
        mils_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        let metrics = sweep_exhaustive(DEFAULT_B_COMBOS);
        let pins = coupling_pin_violations();
        let violations = metrics.interference_violations + pins.len() as u64;
        CourtOutcome {
            verdict: if violations == 0 { Verdict::Pass } else { Verdict::Fail },
            metrics: vec![
                ("cells", MetricValue::U64(metrics.cells)),
                ("armed_b_audits", MetricValue::U64(metrics.armed_b_audits)),
                ("interference_violations", MetricValue::U64(metrics.interference_violations)),
                ("coupling_pin_violations", MetricValue::U64(pins.len() as u64)),
                ("default_b_combos", MetricValue::U64(DEFAULT_B_COMBOS as u64)),
            ],
            scoping: ScopingRecord::partial(
                "full 2^64 SMT non-interference (Kani/Z3) over the dual-word advance law",
                "exhaustive 16-bit small-domain sweep over ALL 2^16 markings x 16 transitions \
                 x 3 regimes x 2 ticks with LCG-sampled armed-B payloads, enumerated+pinned \
                 coupling surface, and mutant positive control",
                "Kani/Z3 2^64 proof out of scope in-repo (feasibility ledger); armed-B payload \
                 combinations tiered 8 -> 2 in the default run, strata-exact 8-combo sweep and \
                 10^7 dual-tenant LCG sweep behind nato-tier3 ignore",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn n01_exhaustive_small_domain_all_65536_markings() {
        let metrics = sweep_exhaustive(DEFAULT_B_COMBOS);
        assert_eq!(
            metrics.cells,
            65_536 * 16 * 3 * 2,
            "small-domain cell census drifted — primary domain must never be reduced"
        );
        assert_eq!(
            metrics.armed_b_audits,
            metrics.cells * DEFAULT_B_COMBOS as u64,
            "armed-B audit census drifted"
        );
        assert_eq!(
            metrics.interference_violations, 0,
            "MILS non-interference violated under B presence"
        );
    }

    #[test]
    fn n01_coupling_pins_hold() {
        let violations = coupling_pin_violations();
        assert!(violations.is_empty(), "permitted-coupling surface drifted: {violations:?}");
    }

    #[test]
    fn n01_court_detects_injected_interference() {
        // Same cell as the sweep audits, evaluated through the leaky oracle:
        // B-armed vs B-zeroed MUST diverge, proving the comparison harness
        // has teeth (non-vacuity by mutation).
        let m = 0x0082u16;
        let (pre, post, tid) = (0x0002u16, 0x000Cu16, 0x02u8);
        let activity = 0x0102u16;
        let tick = 0x41u8;
        let outcome = |flags_b: u8, armed: bool| -> StateA {
            let pn = composite_pn(m, pre, post, tid, flags_b);
            let ocel = if armed {
                composite_ocel(activity, tick, (0xA5C3, 0x1248, 0x96))
            } else {
                composite_ocel(activity, tick, (0, 0, 0))
            };
            leaky_advance(pn, ocel, 0) // mask 0: the injected leak decides
        };
        let armed = outcome(0x01, true);
        let zeroed = outcome(0x00, false);
        assert_ne!(
            armed, zeroed,
            "anti-vacuity: injected interference was NOT detected — court is vacuous"
        );
        assert!(mils_positive_control(), "positive control must hold");
    }

    #[test]
    fn n01_state_a_differential_against_fire_kernel() {
        // Differential reference check: on conforming, enabled, invariant-
        // clean cells, state_a's marking equals the plain kernel fire path.
        // Invariant mask 0 so the kernel fire path and state_a agree.
        let pn = composite_pn(0x0082, 0x0002, 0x0008, 0x02, 0x00);
        let ocel = composite_ocel(0x0102, 0x41, (0, 0, 0));
        let (marking, refused) = state_a(pn, ocel, 0);
        assert!(!refused, "conforming enabled advance must not refuse");
        let fired = pn64::fire(pn);
        assert!(
            fired.is_some_and(|f| pn64::marking(f) == marking),
            "state_a marking diverged from the kernel fire path"
        );
    }

    #[test]
    fn n01_court_run_reports_pass_with_receipt_metrics() {
        let court = MilSeparationCourt;
        assert!(court.positive_control()(), "positive control must pass");
        let outcome = court.run(&CandidateRef("lane-n2-test".to_string()));
        assert_eq!(outcome.verdict, Verdict::Pass, "N01 court verdict must pass");
        let cells = outcome.metrics.iter().find(|(k, _)| *k == "cells");
        assert!(
            cells.is_some_and(|(_, v)| *v == MetricValue::U64(65_536 * 16 * 3 * 2)),
            "cells metric must equal the full small-domain census"
        );
        assert!(
            !outcome.scoping.is_full_spec_scale,
            "N01 must disclose the Kani/Z3 scoping bound honestly"
        );
        outcome.scoping.validate().expect("scoping record must validate");
    }

    #[test]
    #[ignore = "nato-tier3: strata-exact scale, run via just nato-courts-full"]
    fn n01_exhaustive_strata_exact_8_combos() {
        let metrics = sweep_exhaustive(STRATA_B_COMBOS);
        assert_eq!(metrics.cells, 65_536 * 16 * 3 * 2, "cell census drifted");
        assert_eq!(metrics.armed_b_audits, metrics.cells * STRATA_B_COMBOS as u64);
        assert_eq!(metrics.interference_violations, 0, "strata-scale MILS violation");
    }

    #[test]
    #[ignore = "nato-tier3: strata-exact scale, run via just nato-courts-full"]
    fn n01_dual_tenant_lcg_10m_non_interference() {
        let metrics = sweep_dual_tenant(10_000_000);
        assert_eq!(
            metrics.fired_samples + metrics.refused_samples,
            10_000_000,
            "sweep census drift"
        );
        // Both admission branches must be materially exercised (calibration).
        assert!(
            metrics.fired_samples > 1_000_000,
            "fired census too thin: {}",
            metrics.fired_samples
        );
        assert!(
            metrics.refused_samples > 1_000_000,
            "refused census too thin: {}",
            metrics.refused_samples
        );
        assert_eq!(
            metrics.interference_violations, 0,
            "dual-tenant MILS violation in the 10^7 sweep"
        );
    }
}
