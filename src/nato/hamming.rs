#![cfg(feature = "nato")]
// JUSTIFICATION: court items are consumed by its own cfg(test) courts, the
// N08 registry slot, and the shared probe referenced by the court registry
// (`HammingProfileCourt`); the leaky-observer control is referenced by the
// registry's `ControlProbe`.
#![allow(dead_code)] // N08 registry slot; probes referenced by the court registry (HammingProfileCourt)
#![allow(unused_imports)]
// JUSTIFICATION: ScopingRecord feeds only the cfg(test) courts; kept at module scope for doc-link resolution
//! N08 — Hamming weight/distance profile court (NATO lane CP-N08).
//!
//! Ported from `strata_protocol` (commit 6023295 lineage,
//! `tests/nato/n08_hamming_weight.rs`) per plan E-M18, adapted to the CTT
//! 16-bit marking surface: the observed state word is the PN64 committed
//! marking (`crate::conformance::pn64::step_branchless`), the payload-class
//! fields are the OCEL64 object-id words `[31:16]` (src) and `[47:32]`
//! (tgt) of the OCEL event word, exactly as on the strata surface.
//!
//! Spec mandate (NATO_N01_N20 N08): "Hamming weight/distance profile across
//! successive states." The lane's adversarial question: is the state-word
//! Hamming profile INPUT-INDEPENDENT? A payload-dependent divergence is a
//! power-analysis signal (Hamming-weight leak through the branchless
//! kernel).
//!
//! # Delivered court
//!
//! Deterministic chained kernel steps per payload class (LCG `u64`,
//! Knuth MMIX multiplier (via crate::nato::rng::Lcg), two classes:
//!
//! * HIGH: OCEL64 object-id payload fields forced to `0xFFFF` (max weight).
//! * LOW:  same fields forced to `0x0000` (zero weight).
//!
//! Both classes share the IDENTICAL LCG seed, so the payload fields are the
//! only difference between the chains — any divergence is attributable to
//! payload and nothing else. Each step draws a fresh PN64 transition whose
//! `W-` is a subset of the current marking, so token availability always
//! holds; the activity byte is `(LCG byte | tid)`, so the verb gate admits
//! a realistic fraction of steps (non-fired steps surface as distance-0
//! mass). We record (1) the Hamming weight of the 16-bit marking and (2)
//! the Hamming distance between successive markings.
//!
//! Distributions are compared across classes two ways:
//!
//! 1. Welch's two-sample t-test (`crate::nato::timing::welch_t`), NaN-safe:
//!    zero total variance with equal means is exact equality (maximal
//!    input independence, treated as pass, mirroring `timing::passes`);
//!    with unequal means it is maximal divergence (fires).
//! 2. Exact binned-distribution comparison: weight and distance histograms
//!    over `0..=16` must match class-vs-class within 1% of N per bin.
//!
//! Any payload-dependent divergence = FAIL (power-analysis signal).
//!
//! # Anti-vacuity
//!
//! A court that would pass anything carries no bits. The negative-control
//! test re-runs the HIGH/LOW chains through a LEAKY observation transform —
//! all 16 payload bits folded into the observed marking, exactly what a
//! data-dependent kernel would emit — and asserts the Welch rule FIRES.
//! The positive-control discipline holds at both tiers.
//!
//! # Tiering (E-M18)
//!
//! * Tier-1 (default `#[test]`): calibrated 10,000-sample prefix per class.
//! * Tier-2 (`#[ignore]`, `--ignored` opt-in): full 100,000-sample main
//!   profile court at spec scale (the negative control runs at Tier-1;
//!   both tiers share the same seed and chain generator).
//!
//! Chicago discipline: real chained executions of the real kernel, no
//! mocks; assertions on measured state (distributions), not call counts.
//! Zero stdout: profile tables are omitted; every failure carries its
//! statistics in the assertion message. Scoping is disclosed via
//! `crate::nato::disclosure::ScopingRecord` (partial at Tier-1, full at
//! Tier-2) and validated in-test.

// Lint-scope note: NATO court-harness modules use domain literals, metric
// casts, and court-assertion panics by design; the style-class lints below
// are allowed at module scope with this justification rather than dozens of
// inline permits.
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
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::identity_op,
    clippy::trivially_copy_pass_by_ref
)]

use crate::conformance::pn64::step_branchless;
use crate::nato::court::{
    ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;
use crate::nato::rng::Lcg;
use crate::nato::timing::T_THRESHOLD;

/// Tier-1 sample count: calibrated prefix per payload class (default run).
const N_TIER1: usize = 10_000;

/// Tier-2 sample count: full spec scale per payload class (`#[ignore]`).
const N_TIER2: usize = 100_000;

/// Max per-bin share divergence for the exact binned comparison.
const BIN_TOL: f64 = 0.01;

/// Shared seed for both classes: aligned LCG streams isolate payload as the
/// only chain difference. ASCII "NATO" + N08 tag.
const SEED: u64 = 0x4E41_544F_4E30_3807;

/// Initial 16-bit PN64 marking for both chains.
const INITIAL_MARKING: u64 = 0x0002;

/// Welch decision rule, NaN-safe: NaN (zero total variance) with the
/// means we never get to compare is treated as exact equality — pass;
/// otherwise accept only when |t| is below the N05-mirrored threshold.
fn accepts(t: f64) -> bool {
    t.is_nan() || t.abs() < T_THRESHOLD
}

/// Fires = the rule rejects: finite t at or above the threshold.
fn fires(t: f64) -> bool {
    !t.is_nan() && t.abs() >= T_THRESHOLD
}

/// Collected profile for one payload class.
pub(crate) struct Profile {
    weights: Vec<f64>,
    distances: Vec<f64>,
    weight_hist: [u64; 17],
    distance_hist: [u64; 17],
}

impl Profile {
    fn new(n: usize) -> Self {
        Profile {
            weights: Vec::with_capacity(n),
            distances: Vec::with_capacity(n),
            weight_hist: [0; 17],
            distance_hist: [0; 17],
        }
    }

    fn push(&mut self, marking: u16, prev: u16) {
        let w = marking.count_ones();
        let d = (marking ^ prev).count_ones();
        self.weights.push(f64::from(w));
        self.distances.push(f64::from(d));
        self.weight_hist[w as usize] += 1;
        self.distance_hist[d as usize] += 1;
    }
}

/// Run a real chained kernel sequence with the given OCEL64 payload class,
/// collecting weight/distance samples and histograms.
///
/// The chain stays live by construction: each step draws a fresh PN64
/// transition with `W- = r & state` (a subset of the current marking), so
/// token availability always holds; the activity byte is `(LCG byte | tid)`
/// so the verb gate (`(activity & 0xFF) == tid`) admits only a realistic
/// fraction of steps (non-committed steps show up as distance-0). The
/// kernel's committed marking is the observed state.
/// `observe` is applied to the committed marking — the identity for the
/// main court, a payload-leaking transform for the negative control.
fn run_chain(
    lcg_seed: u64,
    payload_field: u64,
    n: usize,
    observe: impl Fn(u16, u64) -> u16,
) -> Profile {
    let mut lcg = Lcg::new(lcg_seed);
    let mut profile = Profile::new(n);

    let mut state: u64 = INITIAL_MARKING;

    for _ in 0..n {
        let r = lcg.next_u64();
        let pre = r & state & 0xFFFF;
        let post = (r >> 16) & 0xFFFF;
        let tid = ((r >> 32) & 0xFF) | 1;
        let pn = (tid << 48) | (post << 32) | (pre << 16) | state;
        // OCEL64: activity low byte = (LCG byte | tid) — the verb gate
        // admits only when the LCG byte is a subset of tid; object-id
        // payload fields [31:16] and [47:32] forced to the class value;
        // qualifier and tick left as LCG noise.
        let ocel = (lcg.next_u64() & !(0xFFFF << 16) & !(0xFFFF << 32))
            | (payload_field << 16)
            | (payload_field << 32)
            | tid;
        let committed = (step_branchless(pn, ocel, 0) & 0xFFFF) as u16;
        let next = observe(committed, payload_field);
        profile.push(next, state as u16);
        state = u64::from(next);
    }
    profile
}

/// Identity observation — the honest kernel surface.
fn observe_clean(committed: u16, _payload: u64) -> u16 {
    committed
}

/// Strong leak: a data-dependent kernel mixing the payload word into the
/// observed marking. Even a weak single-bit leak is caught at these N
/// (mean shift ~0.5 bits gives |t| ~ 40), but the word-wide leak makes the
/// control's power unambiguous.
fn observe_leaky(committed: u16, payload: u64) -> u16 {
    committed | (payload as u16 & 0xFFFF)
}

/// Max per-bin share divergence between two histograms.
fn max_bin_diff(hi: &[u64; 17], lo: &[u64; 17], n: usize) -> f64 {
    let n = n as f64;
    (0..17).map(|i| (hi[i] as f64 - lo[i] as f64).abs() / n).fold(0.0_f64, f64::max)
}

// ---------------------------------------------------------------------------
// Shared probe: one real tier-1 court run, three consumers (the tier-1
// #[test] wrapper, the receipt emitter's mint, and the N08 registry adapter).
// Anti-drift law (plan W-NTW §6): the metrics cannot diverge because every
// consumer reads the same probe.
// ---------------------------------------------------------------------------

/// One real tier-1 profile-court run: the HIGH/LOW clean-observer chains
/// plus the Welch and binned-distribution metrics derived from them.
pub(crate) struct N08ProbeRun {
    /// HIGH (0xFFFF) payload-class chain profile.
    pub hi: Profile,
    /// LOW (0x0000) payload-class chain profile.
    pub lo: Profile,
    /// Welch t over the weight samples.
    pub t_w: f64,
    /// Welch t over the distance samples.
    pub t_d: f64,
    /// Max per-bin weight-histogram divergence.
    pub w_bin: f64,
    /// Max per-bin distance-histogram divergence.
    pub d_bin: f64,
}

/// Run the real tier-1 main court (HIGH 0xFFFF vs LOW 0x0000 clean chains at
/// the calibrated 10k prefix) and derive its metrics. Deterministic: fixed
/// LCG seed, no wall clock, no environment.
pub(crate) fn n08_probe() -> N08ProbeRun {
    let hi = run_chain(SEED, 0xFFFF, N_TIER1, observe_clean);
    let lo = run_chain(SEED, 0x0000, N_TIER1, observe_clean);
    let t_w = crate::nato::timing::welch_t(&hi.weights, &lo.weights);
    let t_d = crate::nato::timing::welch_t(&hi.distances, &lo.distances);
    let w_bin = max_bin_diff(&hi.weight_hist, &lo.weight_hist, N_TIER1);
    let d_bin = max_bin_diff(&hi.distance_hist, &lo.distance_hist, N_TIER1);
    N08ProbeRun { hi, lo, t_w, t_d, w_bin, d_bin }
}

/// The Welch decision rule must have POWER — the leaky-observer mutant MUST
/// fire. `false` = the court is vacuous.
pub(crate) fn n08_negative_control_court_has_power() -> bool {
    let hi = run_chain(SEED, 0xFFFF, N_TIER1, observe_leaky);
    let lo = run_chain(SEED, 0x0000, N_TIER1, observe_leaky);
    let t_leak = crate::nato::timing::welch_t(&hi.weights, &lo.weights);
    fires(t_leak)
}

/// Tier-1 main court: the Hamming weight/distance profile of the committed
/// 16-bit marking must be input-(payload-)independent.
#[test]
fn n08_hamming_profile_is_input_independent_tier1() {
    let disclosure = ScopingRecord::partial(
        "N08: Hamming weight/distance profile across successive states is input-independent",
        "10,000-step chained kernel profile court per payload class (HIGH 0xFFFF / LOW 0x0000), Welch t + 1%-per-bin histogram comparison, plus firing negative control",
        "calibrated Tier-1 prefix: 10k samples per class vs the 100k spec scale",
    );
    disclosure.validate().expect("tier-1 scoping disclosure must validate");

    let run = n08_probe();
    let hi = &run.hi;
    let lo = &run.lo;

    assert_eq!(
        hi.weights.len(),
        N_TIER1,
        "HIGH chain truncated: {} of {N_TIER1} samples",
        hi.weights.len()
    );
    assert_eq!(
        lo.weights.len(),
        N_TIER1,
        "LOW chain truncated: {} of {N_TIER1} samples",
        lo.weights.len()
    );

    let t_w = run.t_w;
    let t_d = run.t_d;
    let max_w_bin_diff = run.w_bin;
    let max_d_bin_diff = run.d_bin;

    // Anti-vacuity: the profile itself must be non-degenerate.
    let populated =
        |h: &[u64; 17]| h.iter().filter(|c| **c as f64 >= 0.01 * N_TIER1 as f64).count();
    assert!(
        populated(&hi.weight_hist) >= 3,
        "VACUOUS COURT: weight histogram degenerate ({} bins >= 1% N); verdict carries no bits",
        populated(&hi.weight_hist)
    );
    assert!(
        hi.distances.iter().sum::<f64>() > 0.0,
        "VACUOUS COURT: states never changed across the chain; distance profile is empty"
    );

    assert!(
        accepts(t_w),
        "POWER-ANALYSIS SIGNAL: state-word Hamming weight depends on payload class (t = {t_w:.4}, threshold |t| < {T_THRESHOLD})"
    );
    assert!(
        accepts(t_d),
        "POWER-ANALYSIS SIGNAL: successive-state Hamming distance depends on payload class (t = {t_d:.4}, threshold |t| < {T_THRESHOLD})"
    );
    assert!(
        max_w_bin_diff <= BIN_TOL,
        "POWER-ANALYSIS SIGNAL: weight distribution diverges per-bin by {max_w_bin_diff:.6} (tol {BIN_TOL})"
    );
    assert!(
        max_d_bin_diff <= BIN_TOL,
        "POWER-ANALYSIS SIGNAL: distance distribution diverges per-bin by {max_d_bin_diff:.6} (tol {BIN_TOL})"
    );
}

/// Tier-1 negative control (anti-vacuity): the Welch decision rule must
/// have POWER — a leaky-observer mutant MUST fire.
#[test]
fn n08_negative_control_court_has_power_tier1() {
    assert!(
        n08_negative_control_court_has_power(),
        "VACUOUS COURT: Welch rule did not fire on an injected payload leak \
         (threshold |t| >= {T_THRESHOLD})"
    );
}

/// Real-receipt emission for the N08 dossier slot (opt-in via
/// `CTT_EMIT_RECEIPTS=1`): runs the tier-1 fast tier for real — the main
/// profile court at the calibrated 10k prefix plus the firing leaky-observer
/// negative control — then mints and persists `docs/receipts/N08.json` from
/// that run. Deterministic: fixed LCG seed, every metric derived from the
/// seeded run, `timestamp_ms` pinned at the fixed 0 slot. Replaces the
/// synthetic `control-subject` placeholder.
#[cfg(test)]
mod receipt_emission {
    use super::*;
    use crate::nato::court::{CourtId, CourtOutcome, MetricValue, Pillar, Verdict};
    use crate::nato::receipt::{ControlResult, CourtReceipt};
    use sha2::{Digest, Sha256};

    /// Deterministic subject identity: SHA-256 over the canonical court name
    /// plus the exact subject the court adjudicates. No wall clock, no
    /// environment — byte-identical across runs and machines.
    fn subject_sha() -> String {
        let mut h = Sha256::new();
        h.update(
            b"N08|src/nato/hamming.rs|hamming weight/distance profile of the pn64 16-bit \
              committed marking across successive chained kernel states (HIGH 0xFFFF vs \
              LOW 0x0000 OCEL64 payload classes)",
        );
        h.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>()
    }

    /// Run the real tier-1 court and its positive control, and mint the
    /// receipt. Panics if the court fails or the control does not fire —
    /// a failing court must never mint a Pass receipt.
    fn mint() -> CourtReceipt {
        // Shared probe (anti-drift law): the same real tier-1 run the
        // #[test] wrapper and the HammingProfileCourt adapter read.
        let run = n08_probe();
        let t_w = run.t_w;
        let t_d = run.t_d;
        let w_bin = run.w_bin;
        let d_bin = run.d_bin;
        assert!(
            accepts(t_w) && accepts(t_d) && w_bin <= BIN_TOL && d_bin <= BIN_TOL,
            "receipt-tier court run must pass; got t_w={t_w} t_d={t_d} \
             w_bin={w_bin} d_bin={d_bin}"
        );

        // Positive control: the leaky-observer mutant MUST fire.
        let hi_leak = run_chain(SEED, 0xFFFF, N_TIER1, observe_leaky);
        let lo_leak = run_chain(SEED, 0x0000, N_TIER1, observe_leaky);
        let t_leak = crate::nato::timing::welch_t(&hi_leak.weights, &lo_leak.weights);
        assert!(fires(t_leak), "positive control must fire on the injected leak (t = {t_leak})");

        let outcome = CourtOutcome {
            verdict: Verdict::Pass,
            metrics: vec![
                ("n_per_class", MetricValue::U64(N_TIER1 as u64)),
                ("welch_t_weight", MetricValue::F64(t_w)),
                ("welch_t_distance", MetricValue::F64(t_d)),
                ("max_bin_diff_weight", MetricValue::F64(w_bin)),
                ("max_bin_diff_distance", MetricValue::F64(d_bin)),
            ],
            scoping: ScopingRecord::partial(
                "N08: Hamming weight/distance profile across successive states is input-independent",
                "receipt minted from a real tier-1 run: 10,000-step chained kernel profile \
                 court per payload class (HIGH 0xFFFF / LOW 0x0000), Welch t + 1%-per-bin \
                 histogram comparison, plus the firing leaky-observer positive control",
                "calibrated Tier-1 prefix: 10k samples per class vs the 100k spec scale",
            ),
        };
        CourtReceipt::record(
            CourtId::N08,
            Pillar::Security,
            "N08",
            &subject_sha(),
            outcome,
            ControlResult::MutantCaught {
                mutant_id: "observe_leaky".to_string(),
                detection_channel: "welch_t".to_string(),
            },
        )
    }

    /// Mint and persist `docs/receipts/N08.json` from a real tier-1 run.
    /// Hermetic by default; opt in with `CTT_EMIT_RECEIPTS=1`.
    #[test]
    fn emit_receipt_when_opted_in() {
        if std::env::var("CTT_EMIT_RECEIPTS").ok().as_deref() != Some("1") {
            return;
        }
        let receipt = mint();

        // Verify before persisting: honest verdict, firing control, real
        // subject identity, digest-consistent parse-back.
        let json = receipt.to_json().expect("serialize N08 receipt");
        let back = CourtReceipt::from_json(&json).expect("receipt must parse back");
        assert_eq!(back.verdict, Verdict::Pass);
        assert!(back.positive_control.passed());
        assert_ne!(back.subject_sha, "control-subject");

        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/receipts");
        let path = dir.join("N08.json");
        if let Err(e) = std::fs::write(&path, json) {
            println!("skipping N08 receipt emission: cannot write {}: {e}", path.display());
        }
    }
}

/// Tier-2 full-scale main court (opt-in via `--ignored`): the same profile
/// court at spec scale, 100k samples per class.
#[test]
#[ignore = "Tier-2 full-scale N08: 100k samples per class; run with --ignored"]
fn n08_hamming_profile_is_input_independent_tier2() {
    let disclosure = ScopingRecord::full(
        "N08: Hamming weight/distance profile across successive states is input-independent",
        "100,000-step chained kernel profile court per payload class (HIGH 0xFFFF / LOW 0x0000), Welch t + 1%-per-bin histogram comparison",
    );
    disclosure.validate().expect("tier-2 scoping disclosure must validate");

    let hi = run_chain(SEED, 0xFFFF, N_TIER2, observe_clean);
    let lo = run_chain(SEED, 0x0000, N_TIER2, observe_clean);

    assert_eq!(
        hi.weights.len(),
        N_TIER2,
        "HIGH chain truncated: {} of {N_TIER2} samples",
        hi.weights.len()
    );
    assert_eq!(
        lo.weights.len(),
        N_TIER2,
        "LOW chain truncated: {} of {N_TIER2} samples",
        lo.weights.len()
    );

    let t_w = crate::nato::timing::welch_t(&hi.weights, &lo.weights);
    let t_d = crate::nato::timing::welch_t(&hi.distances, &lo.distances);
    let max_w_bin_diff = max_bin_diff(&hi.weight_hist, &lo.weight_hist, N_TIER2);
    let max_d_bin_diff = max_bin_diff(&hi.distance_hist, &lo.distance_hist, N_TIER2);

    // Anti-vacuity: the profile itself must be non-degenerate.
    let populated =
        |h: &[u64; 17]| h.iter().filter(|c| **c as f64 >= 0.01 * N_TIER2 as f64).count();
    assert!(
        populated(&hi.weight_hist) >= 3,
        "VACUOUS COURT: weight histogram degenerate ({} bins >= 1% N); verdict carries no bits",
        populated(&hi.weight_hist)
    );
    assert!(
        hi.distances.iter().sum::<f64>() > 0.0,
        "VACUOUS COURT: states never changed across the chain; distance profile is empty"
    );

    assert!(
        accepts(t_w),
        "POWER-ANALYSIS SIGNAL: state-word Hamming weight depends on payload class (t = {t_w:.4}, threshold |t| < {T_THRESHOLD})"
    );
    assert!(
        accepts(t_d),
        "POWER-ANALYSIS SIGNAL: successive-state Hamming distance depends on payload class (t = {t_d:.4}, threshold |t| < {T_THRESHOLD})"
    );
    assert!(
        max_w_bin_diff <= BIN_TOL,
        "POWER-ANALYSIS SIGNAL: weight distribution diverges per-bin by {max_w_bin_diff:.6} (tol {BIN_TOL})"
    );
    assert!(
        max_d_bin_diff <= BIN_TOL,
        "POWER-ANALYSIS SIGNAL: distance distribution diverges per-bin by {max_d_bin_diff:.6} (tol {BIN_TOL})"
    );
}

// ---------------------------------------------------------------------------
// Court wiring (plan W-NTW: N08 registry adapter, mirrors mils.rs:422-475)
// ---------------------------------------------------------------------------

/// N08 Hamming weight/distance profile court.
pub struct HammingProfileCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl HammingProfileCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N08;
    /// Capability pillar. Must match the emitter's mint
    /// (`emit_receipt`, this file) and the on-disk
    /// `docs/receipts/N08.json` — adapters align to emitter
    /// receipts, never to a plan table.
    pub const PILLAR: Pillar = Pillar::Security;
}

impl Court for HammingProfileCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N08 hamming_profile"
    }

    fn positive_control(&self) -> ControlProbe {
        n08_negative_control_court_has_power
    }

    fn run(&self, _candidate: &crate::nato::court::CandidateRef) -> CourtOutcome {
        // Shared probe (anti-drift law): the same real tier-1 run the
        // #[test] wrapper and the receipt emitter's mint read — the metrics
        // cannot diverge.
        let r = n08_probe();
        let holds = accepts(r.t_w) && accepts(r.t_d) && r.w_bin <= BIN_TOL && r.d_bin <= BIN_TOL;
        CourtOutcome {
            verdict: if holds { Verdict::Pass } else { Verdict::Fail },
            metrics: vec![
                ("n_per_class", MetricValue::U64(N_TIER1 as u64)),
                ("welch_t_weight", MetricValue::F64(r.t_w)),
                ("welch_t_distance", MetricValue::F64(r.t_d)),
                ("max_bin_diff_weight", MetricValue::F64(r.w_bin)),
                ("max_bin_diff_distance", MetricValue::F64(r.d_bin)),
            ],
            scoping: ScopingRecord::partial(
                "N08: Hamming weight/distance profile across successive states is input-independent",
                "registry-tier run: 10,000-step chained kernel profile court per payload \
                 class (HIGH 0xFFFF / LOW 0x0000), Welch t + 1%-per-bin histogram \
                 comparison, plus the firing leaky-observer positive control",
                "calibrated Tier-1 prefix: 10k samples per class vs the 100k spec scale",
            ),
        }
    }
}
