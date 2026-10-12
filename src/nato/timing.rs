#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests and are referenced by the court registry, so non-test
// builds legitimately see unused items.
#![allow(dead_code)] // probes + registry slot (TimingCourt) referenced by the court registry
//! N05 — dudect-style statistical constant-time court (NATO lane).
//!
//! Ported from strata `tests/nato/n05_dudect_timing.rs` (Apache/MIT per
//! strata LICENSE), resurfaced as a library module over the crate's own
//! branchless kernel: [`crate::conformance::pn64::step_branchless`].
//!
//! # Method (identical decision rule to dudect, Reparaz et al. 2017)
//!
//! Alternating fixed valid vs fixed refusal-class testcases; per-batch
//! MEDIAN of burst-timed samples; Welch's two-sample t-test;
//! acceptance `|t| < 4.5`. On `|t| >= 4.5` the measurement is rerun idle up
//! to 4x (machine noise vs genuine divergence); all attempt t-statistics
//! are reported.
//!
//! # Testcase classes (same word, different verb — data effect removed)
//!
//! * Valid class: a fixed enabled PN64 word whose activity low byte matches
//!   `tid`, so [`crate::conformance::pn64::step_branchless`] commits.
//! * Refusal class: the SAME PN64 word paired with an OCEL64 word whose
//!   activity low byte differs — the first refusal gate. Only the verb byte
//!   differs, by construction.
//!
//! # Clock-resolution soundness
//!
//! A ~1-2 ns kernel cannot be resolved by single-execution `Instant` pairs.
//! Each timed sample is a BURST of `reps` executions measured as ONE
//! interval; the fastest raw burst interval must exceed 20x the measured
//! clock resolution and every batch span > 100x, else the measurement is
//! refused as [`TimingCourtError::TimingWindowTooCoarse`] /
//! [`TimingCourtError::BatchSpanTooShort`] instead of grading timer
//! artifacts.
//!
//! # Positive control
//!
//! [`leaky_refusal`] is a deliberately timing-leaky twin of the refusal
//! gate; the court MUST flag it (|t| >= 4.5) or every PASS it ever issues
//! carries no bits.
//!
//! Chicago discipline: real CPU timing of real executions, no mocks; the
//! assertion is on measured state (t-statistic), not call counts.

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
    clippy::trivially_copy_pass_by_ref,
    // The rep-scale computation floors the pilot estimate at a positive
    // value before the cast; the sign can never be lost.
    clippy::cast_sign_loss
)]

use std::time::{Duration, Instant};

use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome as RegistryCourtOutcome, MetricValue,
    Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;

// --- Typed errors (production surface returns Result, never panics) -----

/// Typed failure of the N05 timing court. "CourtError-ish": every failure
/// carries the measured figures needed to diagnose it, no strings-only
/// laundering.
#[derive(Debug, Clone, PartialEq)]
pub enum TimingCourtError {
    /// The measured clock cannot resolve the kernel: the fastest raw burst
    /// interval did not exceed 20x the measured clock resolution (optimizer
    /// elision or timer coarsening). The measurement is vacuous.
    TimingWindowTooCoarse {
        /// Fastest raw burst interval observed, ns.
        min_raw_burst_ns: f64,
        /// Measured clock resolution upper bound, ns.
        resolution_ns: f64,
    },
    /// A batch span did not exceed 100x the clock resolution.
    BatchSpanTooShort {
        /// Shortest batch span observed, ns.
        min_batch_span_ns: f64,
        /// Measured clock resolution upper bound, ns.
        resolution_ns: f64,
    },
    /// The valid and refusal testcases produced identical kernel outputs, so
    /// the timing comparison would be vacuous (the "refusal" class does not
    /// actually refuse, or the kernel ignores the verb).
    BehavioralMismatch {
        /// Kernel output on the valid class.
        valid_out: u64,
        /// Kernel output on the refusal class.
        refusal_out: u64,
    },
    /// A batch produced fewer than 2 samples per class; Welch's t-test is
    /// undefined below that.
    InsufficientSamples {
        /// Samples actually collected per class per batch.
        got: usize,
    },
}

// --- Calibration (disclosed, per honest-scoping law) ---------------------

/// dudect acceptance threshold (Reparaz et al. 2017): |t| < 4.5.
pub const T_THRESHOLD: f64 = 4.5;

/// Burst-sampling calibration for one measurement.
///
/// Total transitions per class per run = `batches * iters * reps`.
/// The smoke admission gate is 100_000/class
/// ([`TOTAL_PER_CLASS_ADMISSION_MIN`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BurstConfig {
    /// Batches whose medians feed Welch's t-test.
    pub batches: usize,
    /// Timed burst-samples per class per batch.
    pub iters: usize,
    /// Executions per timed sample (burst).
    pub reps: u64,
}

impl BurstConfig {
    /// Full (spec-class) calibration: 100 x 2_000 x 1_024 = 204,800,000
    /// transitions per class per run (nato-tier3; `#[ignore]`d by default).
    pub const FULL: Self = Self { batches: 100, iters: 2_000, reps: 1_024 };
    /// Smoke calibration: 10 x 100 x 256 = 256,000 transitions per class per
    /// run (>= 100k/class lane minimum, default-test scale).
    pub const SMOKE: Self = Self { batches: 10, iters: 100, reps: 256 };
    /// Total timed executions per class per run.
    pub const fn total_per_class(&self) -> u64 {
        self.batches as u64 * self.iters as u64 * self.reps
    }
}

/// Minimum total-per-class for a court result to be admitted at one calibration.
pub const TOTAL_PER_CLASS_ADMISSION_MIN: u64 = 100_000;

/// The burst-interval soundness floor multiplier used by [`measure`]: the
/// fastest raw burst interval must exceed 20x the measured clock resolution.
const RES_FLOOR_MULT: f64 = 20.0;
/// Safety margin applied on top of the resolution floor when scaling the
/// burst rep count ([`scale_reps_for_resolution`]): the scaled burst must
/// clear the 20x floor by at least this factor, so a pilot burst that just
/// grazes the floor (or a busier machine than the pilot's) still measures.
const RES_SCALE_MARGIN: f64 = 4.0;

/// Scale `cfg.reps` up (never down) so the timed burst interval provably
/// clears the 20x-resolution soundness floor — with `RES_SCALE_MARGIN`
/// margin — in BOTH compile profiles.
///
/// Why this exists: the N05 smoke bursts are 256 reps of a ~1 ns/call
/// kernel, a ~200 ns burst. On Apple-Silicon-class hosts the release
/// profile measures a ~40 ns clock-resolution upper bound
/// ([`clock_resolution_ns`]), so the 20x floor is ~820 ns and the 256-rep
/// burst falls BELOW it (`TimingWindowTooCoarse` at timing.rs:535/553).
/// The honest fix is to measure more, not to relax the floor: a pilot
/// burst of `cfg.reps` executions of the kernel under court estimates the
/// per-rep cost, and `reps` is raised to the next power of two that clears
/// `20x * resolution * 4` in one burst interval. Only ever raises reps, so
/// `total_per_class()` only grows and the 100k/class admission gate still
/// holds for the SMOKE calibration.
///
/// The pilot runs the kernel on the VALID class — the CHEAP twin (the
/// refusal twin gates early and is never cheaper): the soundness floor
/// binds on the FASTEST measured burst, which is a valid-class burst, so
/// the scale must be set by the cheap class or the floor check fails on it
/// even when scaled by the expensive twin's cost.
#[must_use]
pub fn scale_reps_for_resolution<F>(cfg: BurstConfig, resolution: f64, mut f: F) -> BurstConfig
where
    F: FnMut(u64, u64) -> u64,
{
    // Pilot: fastest of 8 bursts of cfg.reps executions on the VALID class
    // — the CHEAP twin (refusal gates early). The soundness floor binds on
    // the fastest measured burst, which is always a valid-class burst for
    // this court's testcase family, so the scale must be set by it.
    let mut min_ns = f64::INFINITY;
    let mut acc = 0u64;
    for _ in 0..8 {
        let t0 = Instant::now();
        for _ in 0..cfg.reps {
            acc = std::hint::black_box(
                acc ^ f(std::hint::black_box(PN_VALID), std::hint::black_box(OCEL_VALID)),
            );
        }
        min_ns = min_ns.min(t0.elapsed().as_nanos() as f64);
    }
    let _ = sink(acc);
    let per_rep = (min_ns / cfg.reps as f64).max(0.01);
    // Raise reps to the next power of two clearing 20x * resolution * margin.
    let needed =
        ((RES_FLOOR_MULT * RES_SCALE_MARGIN * resolution) / per_rep).ceil().max(1.0) as u64;
    BurstConfig { reps: cfg.reps.max(needed.next_power_of_two()), ..cfg }
}

// --- Fixed testcases ------------------------------------------------------

// --- Fixed testcases ------------------------------------------------------

/// Fixed valid testcase: enabled PN64 word (tid=0x02, pre=0x0002,
/// marking=0x0082, post=0x000C) + OCEL64 activity low byte 0x02 == tid.
/// `step_branchless(PN_VALID, OCEL_VALID, 0x0100)` commits.
pub const PN_VALID: u64 = 0x0002_000C_0002_0082;
/// OCEL64 word matching [`PN_VALID`]: activity low byte 0x02 == tid.
pub const OCEL_VALID: u64 = 0x4100_0046_0055_0102;
/// Refusal-class OCEL64 word: same shape, activity low byte 0x99 != tid 0x02
/// — the verb-match refusal gate. Paired with the SAME [`PN_VALID`] word.
pub const OCEL_REFUSAL: u64 = 0x4100_0046_0055_0199;
/// Invariant mask used across all N05 measurements (forbidden bit 0x0100 is
/// not touched by the valid class's post-fire marking 0x008C).
pub const INVARIANT_MASK: u64 = 0x0100;

/// The kernel under court: [`crate::conformance::pn64::step_branchless`].
///
/// Production measurement paths never panic; the kernel itself is total.
fn kernel(pn: u64, ocel: u64) -> u64 {
    crate::conformance::pn64::step_branchless(pn, ocel, INVARIANT_MASK)
}

// --- Statistics -----------------------------------------------------------

/// Median of `v` (mutates: sorts). `f64::total_cmp` ordering — no unwrap, no
/// partial_cmp panic on NaN.
fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n == 0 {
        return f64::NAN;
    }
    if n % 2 == 0 {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    } else {
        v[n / 2]
    }
}

/// Welch's two-sample t-statistic (unequal variances, unequal n).
///
/// Returns NaN when either sample has zero variance (both classes quantize
/// to the same timer-quantum level — the strongest no-divergence signal this
/// clock can produce) or either slice has fewer than 2 elements.
pub fn welch_t(a: &[f64], b: &[f64]) -> f64 {
    if a.len() < 2 || b.len() < 2 {
        return f64::NAN;
    }
    let n1 = a.len() as f64;
    let n2 = b.len() as f64;
    let m1 = a.iter().sum::<f64>() / n1;
    let m2 = b.iter().sum::<f64>() / n2;
    let v1 = a.iter().map(|x| (x - m1) * (x - m1)).sum::<f64>() / (n1 - 1.0);
    let v2 = b.iter().map(|x| (x - m2) * (x - m2)).sum::<f64>() / (n2 - 1.0);
    (m1 - m2) / ((v1 / n1) + (v2 / n2)).sqrt()
}

/// Pass predicate for a Welch t outcome: NaN (zero variance) or |t| < 4.5.
#[must_use]
pub fn passes(t: f64) -> bool {
    t.is_nan() || t.abs() < T_THRESHOLD
}

/// Minimum observed positive `Instant::now()` delta over a 5 ms busy census
/// — an upper bound on the clock resolution (true resolution <= this).
#[must_use]
pub fn clock_resolution_ns() -> f64 {
    let mut min = f64::INFINITY;
    let mut prev = Instant::now();
    let deadline = prev + Duration::from_millis(5);
    loop {
        let now = Instant::now();
        let d = now.duration_since(prev).as_nanos() as f64;
        if d > 0.0 && d < min {
            min = d;
        }
        prev = now;
        if now >= deadline {
            return min;
        }
    }
}

/// Sanitize: disable (as much as the process allows) optimizer elision of
/// the measured call. Consumes the result into an atomic sink never folded
/// at -O2/3 in practice.
#[inline(never)]
fn sink(x: u64) -> u64 {
    std::sync::atomic::AtomicU64::new(0).fetch_xor(x, std::sync::atomic::Ordering::Relaxed)
}

/// One completed dudect measurement of one kernel.
#[derive(Debug, Clone)]
pub struct Measurement {
    /// Per-batch medians, valid class (ns/execution).
    pub med_valid: Vec<f64>,
    /// Per-batch medians, refusal class (ns/execution).
    pub med_refusal: Vec<f64>,
    /// XOR of the kernel's outputs on the two fixed testcases (must be
    /// nonzero: valid commits, refusal does not).
    pub behavior_xor: u64,
    /// Fastest raw burst interval observed, ns.
    pub min_raw_sample_ns: f64,
    /// Shortest batch span observed, ns.
    pub min_batch_span_ns: f64,
}

/// One dudect measurement of kernel `f`: warmup, then alternating A/B
/// batches; each batch collects `cfg.iters` burst samples per class and the
/// batch statistic is their MEDIAN (dudect's heavy-tail defense).
///
/// Soundness floors are enforced as typed errors, not asserts: the timing
/// window must provably exceed the clock resolution, else the court refuses
/// to grade a vacuous measurement.
pub fn measure<F>(
    cfg: BurstConfig,
    resolution: f64,
    mut f: F,
) -> Result<Measurement, TimingCourtError>
where
    F: FnMut(u64, u64) -> u64,
{
    if cfg.batches < 1 || cfg.iters < 2 || cfg.reps < 1 {
        return Err(TimingCourtError::InsufficientSamples { got: cfg.iters });
    }

    // Warmup: primes caches, branch predictors, page tables.
    let mut acc = 0u64;
    for _ in 0..10_000u64 {
        acc ^= f(PN_VALID, OCEL_VALID);
        acc ^= f(PN_VALID, OCEL_REFUSAL);
    }
    acc = sink(acc);

    let mut med_valid: Vec<f64> = Vec::with_capacity(cfg.batches);
    let mut med_refusal: Vec<f64> = Vec::with_capacity(cfg.batches);
    let mut min_raw_sample_ns = f64::INFINITY;
    let mut min_batch_span_ns = f64::INFINITY;

    for batch in 0..cfg.batches {
        // Class order alternates per batch (valid-first on even, refusal-
        // first on odd) to cancel linear clock/thermal drift.
        let valid_first = batch % 2 == 0;
        let (class_a, class_b) =
            if valid_first { (OCEL_VALID, OCEL_REFUSAL) } else { (OCEL_REFUSAL, OCEL_VALID) };

        let batch_start = Instant::now();
        let mut t_a: Vec<f64> = Vec::with_capacity(cfg.iters);
        let mut t_b: Vec<f64> = Vec::with_capacity(cfg.iters);
        for ocel in [class_a, class_b] {
            for _ in 0..cfg.iters {
                // One burst sample: `reps` executions in ONE Instant interval.
                let t0 = Instant::now();
                for _ in 0..cfg.reps {
                    // black_box on the ACCUMULATOR forces a serial data
                    // dependency across iterations — without it LLVM folds
                    // the whole burst into closed form.
                    acc = std::hint::black_box(
                        acc ^ f(std::hint::black_box(PN_VALID), std::hint::black_box(ocel)),
                    );
                }
                let t1 = Instant::now();
                let raw = (t1 - t0).as_nanos() as f64;
                min_raw_sample_ns = min_raw_sample_ns.min(raw);
                if ocel == class_a {
                    t_a.push(raw / cfg.reps as f64);
                } else {
                    t_b.push(raw / cfg.reps as f64);
                }
            }
        }
        min_batch_span_ns = min_batch_span_ns.min(batch_start.elapsed().as_nanos() as f64);

        let (mv, mr) = if valid_first {
            (median(&mut t_a), median(&mut t_b))
        } else {
            (median(&mut t_b), median(&mut t_a))
        };
        med_valid.push(mv);
        med_refusal.push(mr);
    }
    let _ = sink(acc);

    // Soundness floors (typed refusal, not assert).
    if min_raw_sample_ns <= 20.0 * resolution {
        return Err(TimingCourtError::TimingWindowTooCoarse {
            min_raw_burst_ns: min_raw_sample_ns,
            resolution_ns: resolution,
        });
    }
    if min_batch_span_ns <= 100.0 * resolution {
        return Err(TimingCourtError::BatchSpanTooShort {
            min_batch_span_ns,
            resolution_ns: resolution,
        });
    }

    // Behavioral sanity: on this testcase pair the kernel MUST behave as
    // documented (valid commits, refusal leaves state unchanged), else the
    // timing comparison is vacuous.
    let out_valid = f(std::hint::black_box(PN_VALID), std::hint::black_box(OCEL_VALID));
    let out_refusal = f(std::hint::black_box(PN_VALID), std::hint::black_box(OCEL_REFUSAL));
    if out_valid == out_refusal {
        return Err(TimingCourtError::BehavioralMismatch {
            valid_out: out_valid,
            refusal_out: out_refusal,
        });
    }

    Ok(Measurement {
        med_valid,
        med_refusal,
        behavior_xor: out_valid ^ out_refusal,
        min_raw_sample_ns,
        min_batch_span_ns,
    })
}

/// Outcome of one full N05 court over one kernel.
#[derive(Debug, Clone)]
pub struct CourtOutcome {
    /// t-statistic of every attempt, in order (idle reruns appended).
    pub attempts: Vec<f64>,
    /// Fastest raw burst interval of the final attempt, ns.
    pub min_raw_sample_ns: f64,
    /// Shortest batch span of the final attempt, ns.
    pub min_batch_span_ns: f64,
    /// Measured clock resolution upper bound, ns.
    pub resolution_ns: f64,
}

impl CourtOutcome {
    /// The t-statistic of the last attempt.
    ///
    /// Returns a typed error rather than panicking if no attempt ran.
    pub fn final_t(&self) -> Result<f64, TimingCourtError> {
        self.attempts
            .last()
            .copied()
            .ok_or(TimingCourtError::InsufficientSamples { got: 0 })
    }
}

/// N05 court over one kernel: full measurement; on |t| >= 4.5 rerun idle
/// (up to 4 attempts total) before concluding — machine noise vs genuine
/// divergence, all attempt t-statistics reported.
pub fn run_court<F>(cfg: BurstConfig, mut f: F) -> Result<CourtOutcome, TimingCourtError>
where
    F: FnMut(u64, u64) -> u64,
{
    let resolution = clock_resolution_ns();
    // Scale the burst rep count so one burst interval clears the 20x
    // resolution floor in BOTH profiles (release optimizes the kernel to
    // ~1 ns/call; a 256-rep smoke burst is ~200 ns, below the ~800 ns floor
    // a ~40 ns-resolution host imposes). Only raises reps.
    let cfg = scale_reps_for_resolution(cfg, resolution, &mut f);
    let mut attempts: Vec<f64> = Vec::new();
    let mut soundness = (f64::NAN, f64::NAN);
    for attempt in 0..4 {
        if attempt > 0 {
            // Idle-cooldown rerun: let caches/interrupts settle.
            std::thread::sleep(Duration::from_millis(500));
        }
        let m = measure(cfg, resolution, &mut f)?;
        soundness = (m.min_raw_sample_ns, m.min_batch_span_ns);
        attempts.push(welch_t(&m.med_valid, &m.med_refusal));
        if passes(*attempts.last().unwrap_or(&f64::NAN)) {
            break;
        }
    }
    Ok(CourtOutcome {
        attempts,
        min_raw_sample_ns: soundness.0,
        min_batch_span_ns: soundness.1,
        resolution_ns: resolution,
    })
}

/// Deliberately timing-leaky twin of the refusal gate: spins (~200
/// dependent iterations) only on the refusal class. Positive control for
/// the harness itself — the court MUST detect this (|t| >= 4.5) or every
/// PASS it issues carries no bits.
#[inline(never)]
pub fn leaky_refusal(pn: u64, ocel: u64) -> u64 {
    let mut x = pn ^ ocel;
    if (ocel & 0xFF) != ((pn >> 48) & 0xFF) {
        // Refusal-class only: dependent spin, never optimized across.
        for i in 0..200u64 {
            x = x.rotate_left(7) ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
    }
    x
}

// ---------------------------------------------------------------------------
// Court wiring
// ---------------------------------------------------------------------------

/// Hoisted positive control for the N05 timing court: the leaky twin
/// [`leaky_refusal`] must be class-selective (cheap on the valid class,
/// expensive on the refusal class) — otherwise it controls nothing and every
/// PASS this court issues carries no bits. 10k iterations and min-of-3 per
/// class: the valid-class window is ~1 ns/call, so a 1k-iteration
/// single-shot interval is dominated by ambient load noise in release (the
/// AS2 law — a wall-clock reading under unknown load measures the machine);
/// the min over repeated trials is the load-robust statistic, and 10k iters
/// puts both windows far above timer noise.
#[must_use]
pub(crate) fn timing_positive_control() -> bool {
    let v = (PN_VALID, OCEL_VALID);
    let mut acc = 0u64;
    let mut class_min = |ocel: u64| -> f64 {
        let mut best = f64::INFINITY;
        for _ in 0..3 {
            let t0 = Instant::now();
            for _ in 0..10_000u64 {
                acc ^= leaky_refusal(v.0, ocel);
            }
            best = best.min(t0.elapsed().as_nanos() as f64);
        }
        best
    };
    let valid_ns = class_min(OCEL_VALID);
    let refusal_ns = class_min(OCEL_REFUSAL);
    std::hint::black_box(&mut acc);
    refusal_ns > valid_ns * 4.0
}

/// N05 dudect-style statistical constant-time court.
pub struct TimingCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl TimingCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N05;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Timing;
}

impl Court for TimingCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N05 dudect_timing"
    }

    fn positive_control(&self) -> ControlProbe {
        timing_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> RegistryCourtOutcome {
        // Smoke-scale dudect court over the crate's own branchless kernel
        // (full spec-class calibration remains behind the nato-tier3
        // `#[ignore]`d test). Err is a soundness refusal (timer too coarse,
        // vacuous samples, behavioral mismatch), never a Pass.
        match run_court(BurstConfig::SMOKE, kernel) {
            Ok(outcome) => {
                let t = outcome.final_t().unwrap_or(f64::NAN);
                RegistryCourtOutcome {
                    verdict: if passes(t) { Verdict::Pass } else { Verdict::Fail },
                    metrics: vec![
                        ("final_t", MetricValue::F64(t)),
                        ("attempts", MetricValue::U64(outcome.attempts.len() as u64)),
                        ("min_raw_sample_ns", MetricValue::F64(outcome.min_raw_sample_ns)),
                        ("min_batch_span_ns", MetricValue::F64(outcome.min_batch_span_ns)),
                        ("resolution_ns", MetricValue::F64(outcome.resolution_ns)),
                    ],
                    scoping: ScopingRecord::partial(
                        "full spec-class dudect measurement (100 x 2_000 x 1_024 = 204.8M \
                         transitions per class) proving |t| < 4.5 for the branchless kernel",
                        "smoke-scale dudect court over `step_branchless` (>= 100k transitions \
                         per class) with 20x-resolution burst-soundness floors and idle-rerun \
                         (up to 4 attempts) noise defense",
                        "smoke calibration only; the 204.8M-transition/class full-scale court \
                         is behind the nato-tier3 `#[ignore]`d test",
                    ),
                }
            }
            Err(e) => RegistryCourtOutcome {
                verdict: Verdict::Fail,
                metrics: vec![("error", MetricValue::Str(format!("{e:?}")))],
                scoping: ScopingRecord::partial(
                    "full spec-class dudect measurement proving |t| < 4.5 for the branchless \
                     kernel",
                    "court measurement was refused by a typed soundness error before any \
                     verdict could be graded",
                    "measurement refused: see the error metric for the typed failure",
                ),
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // JUSTIFICATION: test code only
mod tests {
    use super::*;

    #[test]
    fn n05_admission_gate_holds_for_both_calibrations() {
        assert!(BurstConfig::SMOKE.total_per_class() >= TOTAL_PER_CLASS_ADMISSION_MIN);
        assert!(BurstConfig::FULL.total_per_class() >= TOTAL_PER_CLASS_ADMISSION_MIN);
        assert!(BurstConfig::SMOKE.total_per_class() >= 100_000);
    }

    #[test]
    fn n05_testcase_classes_behave_as_documented() {
        // Valid class commits; refusal class leaves pn bit-exact unchanged.
        let valid = kernel(PN_VALID, OCEL_VALID);
        let refusal = kernel(PN_VALID, OCEL_REFUSAL);
        assert_ne!(valid, refusal);
        assert_eq!(refusal, PN_VALID);
        // Marking advanced by the valid step.
        assert_eq!(valid & 0xFFFF, 0x008C);
    }

    #[test]
    fn n05_welch_t_separates_disjoint_samples() {
        let a = [10.0; 8];
        let b = [20.0; 8];
        let t = welch_t(&a, &b);
        assert!(t < -T_THRESHOLD, "disjoint means must give |t| large: {t}");
        assert!(!passes(t));
        // Zero variance -> NaN -> passes (strongest no-divergence signal).
        assert!(welch_t(&[5.0; 4], &[5.0; 4]).is_nan());
        assert!(passes(f64::NAN));
    }

    #[test]
    fn n05_median_total_cmp_no_panic() {
        let mut v = vec![1.0, f64::NAN, 3.0, -2.0];
        let m = median(&mut v);
        assert!(m.is_finite() || m.is_nan());
    }

    #[test]
    fn n05_leaky_refusal_control_diverges_by_construction() {
        // The class-selectivity law lives in the hoisted control the court
        // registry itself runs; the wrapper asserts it directly.
        assert!(
            timing_positive_control(),
            "leaky control not class-selective: refusal class must cost > 4x the valid class"
        );
    }

    // --- Real dudect courts (real CPU timing, real state assertions) -----

    /// Smoke-scale dudect court over `step_branchless` (default test suite).
    #[test]
    fn n05_dudect_step_branchless_smoke() {
        let cfg = BurstConfig::SMOKE;
        assert!(cfg.total_per_class() >= TOTAL_PER_CLASS_ADMISSION_MIN);

        let outcome = run_court(cfg, kernel).expect("court errored");
        let t = outcome.final_t().expect("no attempts recorded");
        assert!(
            passes(t),
            "N05 FAIL step_branchless: |t| = {t:.3} >= 4.5 on EVERY of {} idle-rerun \
             attempts (genuine valid-vs-refusal timing divergence, not machine noise). \
             Attempt t-statistics: {:?}",
            outcome.attempts.len(),
            outcome.attempts
        );
        // Soundness figures of the final attempt.
        assert!(outcome.min_raw_sample_ns > 20.0 * outcome.resolution_ns);
        assert!(outcome.min_batch_span_ns > 100.0 * outcome.resolution_ns);
    }

    /// Smoke-scale positive control: the harness MUST detect the leaky twin.
    #[test]
    fn n05_positive_control_detects_leak_smoke() {
        let outcome = run_court(BurstConfig::SMOKE, leaky_refusal).expect("court errored");
        let t = outcome.final_t().expect("no attempts recorded");
        assert!(
            t.abs() >= T_THRESHOLD,
            "N05 HARNESS VACUOUS: positive control leaky_refusal NOT detected \
             (|t| = {t:.3} < 4.5) — the timing harness cannot detect a real divergence, \
             so a pass on the targets carries no bits. Attempts: {:?}",
            outcome.attempts
        );
    }

    /// Full spec-class calibration (204.8M transitions/class); run explicitly.
    #[test]
    #[ignore = "nato-tier3"]
    fn n05_dudect_step_branchless_full_scale() {
        let cfg = BurstConfig::FULL;
        assert!(cfg.total_per_class() >= TOTAL_PER_CLASS_ADMISSION_MIN);
        let outcome = run_court(cfg, kernel).expect("court errored");
        let t = outcome.final_t().expect("no attempts recorded");
        assert!(
            passes(t),
            "N05 FAIL step_branchless (full scale): |t| = {t:.3}; attempts: {:?}",
            outcome.attempts
        );
    }
}
