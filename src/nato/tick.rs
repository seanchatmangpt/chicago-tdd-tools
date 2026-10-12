#![cfg(feature = "nato")]
#![allow(dead_code)]
// JUSTIFICATION: court items are consumed by its own cfg(test) courts, the
// N12 registry slot, and the shared probe referenced by the court registry
// (`ClockGlitchCourt`); the backstep control is referenced by the registry's
// `ControlProbe`.
// NATO court-harness modules use domain literals, metric casts, and
// court-assertion panics by design; the style-class lints below are allowed
// at module scope with this justification rather than 300+ inline permits
// (same house style as `src/nato/seu.rs`).
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
    clippy::or_fun_call,
    clippy::nonminimal_bool,
    clippy::manual_is_multiple_of,
    // JUSTIFICATION: court-leg refusal arithmetic is pinned by donor-verbatim
    // literals (`(glitch_count * 2 + 2) / 3`), newly always-compiled after the
    // shared-probe hoist; `div_ceil` would obscure the 2-of-3-class split.
    clippy::manual_div_ceil,
    clippy::identity_op,
    clippy::trivially_copy_pass_by_ref
)]
//! NATO N12 — clock-glitch resilience court (wall-monotonic surrogate).
//!
//! Ported from strata-protocol `tests/nato/n12_clock_skew_resilience.rs`
//! (Apache/MIT per strata LICENSE). CTT has NO landed tick surface and NO
//! RDTSC/counter-register access through `std` on this host, so the port is
//! designed fresh: a wall-monotonic regression gate over
//! [`std::time::Instant`] plus injected real-source glitch classes.
//!
//! HONEST SCOPING (carried by [`scoping_record`], disclosed, not hidden):
//! the donor `HwTick` is a hardware-counter tier (XDP/Linux wave). This port
//! is strictly WEAKER — a wall-monotonic regression gate with no raw-cycle
//! domain, no u64 wrap boundary in the tick domain itself, and no OCEL64
//! byte-tick ABI guard (CTT's transition law, [`crate::conformance::pn64`],
//! is not a modular byte-tick counter). Each donor court is mapped to its
//! strongest achievable CTT equivalent or disclosed as out of scope:
//!
//! | donor court | CTT equivalent |
//! |---|---|
//! | 1. jitter injection, typed refusal | `court1_jitter_injection_regression_refused_forward_admitted` |
//! | 2a. u64::MAX wrap | OUT OF SCOPE — no raw-cycle counter, disclosed |
//! | 2b. byte-tick wrap | OUT OF SCOPE — pn64 is not a modular counter, disclosed |
//! | 3. monotonic-enforcement composition | `court3_ordering_law_pn64_transitions` (over pn64) |
//! | 4. honest host note | `court4_honest_host_note_no_counter_register` |
//! | (new) frozen clock, bounded liveness | `court2_frozen_clock_bounded_liveness` |
//! | (new) positive control | `court4_positive_control_backstep_clock_is_caught` |
//! | (tier-3) calibration | `n12_clock_resolution_calibration` |
//!
//! Zero-mock (Chicago): the real monotonic clock is read through
//! [`MonotonicClock`]; glitch classes are built from REAL samples replayed
//! out of position or real clock gaps — never synthesized constants. The
//! injected sources ([`FrozenClock`], [`BackstepClock`]) are hand-written
//! REAL interface implementations (degenerate clocks), not mocks of an
//! owned collaborator.

use std::cell::Cell;
use std::fmt;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

// JUSTIFICATION: the shared N12 court probe (n12_run_court_probe) runs in the
// always-compiled portion under `feature = "nato"`, which implies the
// `conformance` feature (Cargo.toml nato feature graph), so the pn64 surface
// is available outside cfg(test).
use crate::conformance::pn64;
use crate::nato::court::{Court, CourtId, CourtOutcome, ControlProbe, MetricValue, Pillar, Verdict};
use crate::nato::disclosure::ScopingRecord;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Bounded liveness threshold for frozen-clock detection: after this many
/// consecutive non-advancing samples, the gate declares the clock frozen
/// rather than refusing one pair at a time forever.
pub const FROZEN_CLOCK_LIVENESS_BOUND: usize = 8;

/// Process-start anchor for the wall-monotonic nanos domain. `Instant` has
/// no public epoch, so the domain is anchored once at first use; only
/// differences are meaningful, which is all the strict-forward law needs.
static EPOCH: OnceLock<Instant> = OnceLock::new();

fn epoch() -> Instant {
    *EPOCH.get_or_init(Instant::now)
}

/// One monotonic clock sample (nanoseconds on the wall-monotonic domain).
///
/// Constructed only by a [`TickSource`] — there is no public raw-value
/// constructor, mirroring the donor's L158 constraint: every glitch class
/// must be built from real samples, not synthesized values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HwTick {
    nanos: u128,
}

impl HwTick {
    /// The declared monotonic source's current sample.
    #[must_use]
    pub fn now() -> Self {
        MonotonicClock.now()
    }

    /// Raw monotonic nanoseconds.
    #[must_use]
    pub const fn as_nanos(self) -> u128 {
        self.nanos
    }

    /// The strict monotonic law: `next` is admitted only if it is STRICTLY
    /// greater than `prev`. The admitted value is `next` itself; every
    /// refusal is typed [`TickError::Regression`] carrying exactly the
    /// offending pair.
    ///
    /// # Errors
    /// [`TickError::Regression`] when `next.nanos <= prev.nanos`.
    pub fn mono_checked(prev: HwTick, next: HwTick) -> Result<HwTick, TickError> {
        if next.nanos > prev.nanos {
            Ok(next)
        } else {
            Err(TickError::Regression { prev: prev.nanos, next: next.nanos })
        }
    }
}

/// Typed tick-law failures. Every variant's `Display` obeys the N18
/// zero-entropy string law: it names the class and carries ONLY the
/// offending values — no source paths, no file names, no panic machinery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickError {
    /// The candidate sample did not strictly advance over the frontier.
    Regression {
        /// The accepted frontier sample.
        prev: u128,
        /// The refused candidate sample.
        next: u128,
    },
    /// The source produced `repeats` consecutive non-advancing samples —
    /// the bounded-liveness frozen-clock verdict.
    FrozenClock {
        /// How many consecutive non-advancing samples were observed.
        repeats: usize,
        /// The stuck sample value.
        last: u128,
    },
}

impl fmt::Display for TickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TickError::Regression { prev, next } => {
                write!(
                    f,
                    "clock regression: next must be strictly greater than prev (prev={prev}, next={next})"
                )
            }
            TickError::FrozenClock { repeats, last } => {
                write!(
                    f,
                    "clock frozen: no strict forward motion across {repeats} consecutive samples (last={last})"
                )
            }
        }
    }
}

/// A real clock source. Hand-written real implementations only — a
/// degenerate source ([`FrozenClock`]) is a real degenerate clock, not a
/// mock of [`MonotonicClock`].
pub trait TickSource: 'static {
    /// One sample.
    fn now(&self) -> HwTick;
    /// The declared source name (honest-host note surface).
    fn source_name(&self) -> &'static str;
}

/// The real host monotonic clock: `std::time::Instant` (on this host,
/// `mach_absolute_time` underneath — a monotonic surrogate, NOT the raw
/// cycle counter; no userspace RDTSC access exists through `std`).
#[derive(Debug, Clone, Copy)]
pub struct MonotonicClock;

impl TickSource for MonotonicClock {
    fn now(&self) -> HwTick {
        HwTick { nanos: epoch().elapsed().as_nanos() }
    }

    fn source_name(&self) -> &'static str {
        "wall-monotonic Instant (mach_absolute_time surrogate)"
    }
}

/// A frozen clock: every sample is the same value. A real degenerate clock
/// (e.g. a dead counter register), used to prove the liveness bound fires.
#[derive(Debug, Clone, Copy)]
pub struct FrozenClock {
    stuck: u128,
}

impl FrozenClock {
    /// A clock stuck at `stuck` nanos.
    #[must_use]
    pub const fn at(stuck: u128) -> Self {
        FrozenClock { stuck }
    }
}

impl TickSource for FrozenClock {
    fn now(&self) -> HwTick {
        HwTick { nanos: self.stuck }
    }

    fn source_name(&self) -> &'static str {
        "frozen (degenerate: constant sample)"
    }
}

/// A non-monotonic clock: oscillates between two real anchor values,
/// starting at the LATER anchor so the very first oscillation is a
/// backwards step. The positive-control subject — the gate MUST catch it.
#[derive(Debug)]
pub struct BackstepClock {
    a: u128,
    b: u128,
    flip: Cell<bool>,
}

impl BackstepClock {
    /// Oscillates between anchors `a` (later) and `b` (earlier), `b < a`.
    /// The first sample is `a`.
    #[must_use]
    pub fn between(a: u128, b: u128) -> Self {
        assert!(b < a, "anchors must be ordered: b earlier, a later");
        BackstepClock { a, b, flip: Cell::new(true) }
    }
}

impl TickSource for BackstepClock {
    fn now(&self) -> HwTick {
        let later = self.flip.get();
        self.flip.set(!later);
        let v = if later { self.a } else { self.b };
        HwTick { nanos: v }
    }

    fn source_name(&self) -> &'static str {
        "backstep (degenerate: oscillating non-monotonic)"
    }
}

// ---------------------------------------------------------------------------
// The gate
// ---------------------------------------------------------------------------

/// Frontier-tracking strict-monotonic gate over any [`TickSource`].
///
/// The accepted frontier advances ONLY on strictly-forward samples; refusals
/// leave it bit-exact unchanged. Consecutive non-advancing samples count
/// toward the frozen-clock liveness bound; a strictly-forward sample resets
/// the count.
#[derive(Debug)]
pub struct MonotonicGate<S: TickSource> {
    source: S,
    frontier: Option<HwTick>,
    stuck_count: usize,
}

impl<S: TickSource> MonotonicGate<S> {
    /// A gate over `source` with no accepted frontier yet.
    pub fn new(source: S) -> Self {
        MonotonicGate { source, frontier: None, stuck_count: 0 }
    }

    /// Sample and adjudicate. Returns the newly admitted sample, or the
    /// typed refusal ([`TickError::Regression`], or
    /// [`TickError::FrozenClock`] once the liveness bound is crossed).
    ///
    /// # Errors
    /// See [`TickError`].
    pub fn tick(&mut self) -> Result<HwTick, TickError> {
        let sample = self.source.now();
        let Some(frontier) = self.frontier else {
            self.frontier = Some(sample);
            self.stuck_count = 0;
            return Ok(sample);
        };
        match HwTick::mono_checked(frontier, sample) {
            Ok(admitted) => {
                self.frontier = Some(admitted);
                self.stuck_count = 0;
                Ok(admitted)
            }
            Err(e) => {
                self.stuck_count += 1;
                if self.stuck_count >= FROZEN_CLOCK_LIVENESS_BOUND {
                    Err(TickError::FrozenClock {
                        repeats: self.stuck_count,
                        last: sample.as_nanos(),
                    })
                } else {
                    Err(e)
                }
            }
        }
    }

    /// The currently accepted frontier (None before the first admission).
    #[must_use]
    pub const fn frontier(&self) -> Option<HwTick> {
        self.frontier
    }

    /// Consecutive non-advancing samples since the last admission.
    #[must_use]
    pub const fn stuck_count(&self) -> usize {
        self.stuck_count
    }
}

/// Positive-control mutant: the same sample loop as [`MonotonicGate`] with
/// the regression predicate DROPPED (admits every sample). The court
/// requires the mutant to admit what the gate refuses — the non-vacuity
/// proof that the check is load-bearing.
#[derive(Debug)]
pub struct UnprotectedFold<S: TickSource> {
    source: S,
    last: Option<HwTick>,
}

impl<S: TickSource> UnprotectedFold<S> {
    fn new(source: S) -> Self {
        UnprotectedFold { source, last: None }
    }

    fn tick(&mut self) -> HwTick {
        let sample = self.source.now();
        self.last = Some(sample);
        sample
    }
}

/// The [`ScopingRecord`] this court ships on every receipt: PARTIAL — the
/// donor's hardware-counter tier is not reachable from CTT.
#[must_use]
pub fn scoping_record() -> ScopingRecord {
    ScopingRecord::partial(
        "N12 clock-glitch resilience: hardware-counter tick law with wrap boundaries and ABI byte-tick guard",
        "wall-monotonic regression gate over Instant: strict-forward admission, typed TickError::Regression refusals, frozen-clock bounded liveness, pn64 ordering composition, positive controls",
        "no RDTSC/counter-register access through std on this host; no raw-cycle u64 wrap domain; pn64 is a transition law, not a modular byte-tick counter — donor courts 2a/2b out of scope",
    )
}

// ---------------------------------------------------------------------------
// Shared fixture: real-sample forward chain
// ---------------------------------------------------------------------------

/// Sample `n` real clock ticks, strictly deduplicated by retry-sampling
/// (the documented consumer law for strict consumers): keep sampling until
/// the sample strictly advances. Returns the forward chain.
fn forward_chain(n: usize) -> Vec<HwTick> {
    let mut v = Vec::with_capacity(n);
    let mut prev = HwTick::now();
    v.push(prev);
    while v.len() < n {
        let t = HwTick::now();
        if t.as_nanos() > prev.as_nanos() {
            v.push(t);
            prev = t;
        }
    }
    v
}

/// A real huge forward jump: two samples separated by a genuine 2 ms clock
/// gap. Against the ~ns inter-sample baseline this is a genuinely huge
/// forward jump that the strict law MUST admit as lawful forward motion —
/// never silently drop — built from a real gap, not a synthesized constant.
fn huge_jump_sample(base: HwTick) -> HwTick {
    let t0 = base.as_nanos();
    loop {
        std::thread::sleep(Duration::from_millis(2));
        let t = HwTick::now();
        if t.as_nanos() >= t0 + 1_000_000 {
            return t;
        }
    }
}

/// Classify one adjudicated pair, asserting the typed-error surface on
/// every refusal: exact pair carried, class named, zero entropy in the
/// Display string. Returns true iff the pair was admitted.
fn adjudicate(prev: HwTick, next: HwTick) -> bool {
    match HwTick::mono_checked(prev, next) {
        Ok(admitted) => {
            assert_eq!(admitted, next, "admission must return `next` itself");
            true
        }
        Err(e) => {
            assert_eq!(
                e,
                TickError::Regression { prev: prev.as_nanos(), next: next.as_nanos() },
                "refusal must be typed and carry exactly the offending pair"
            );
            let d = format!("{e}");
            assert!(d.contains("regression"), "Display must name the class: {d}");
            assert!(
                !d.contains("src/") && !d.contains(".rs") && !d.contains("panic"),
                "zero-entropy Display: {d}"
            );
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Shared probe (plan W-NTW): ONE real fast-tier court run — court 1 (jitter
// injection), court 2 (frozen-clock bounded liveness), court 3 (pn64 ordering
// law) — with three consumers (the #[test] wrappers, the receipt emitter's
// opt-in run, and the N12 registry adapter). Anti-drift law (§6): the legs
// cannot diverge because every consumer runs the same probe.
// ---------------------------------------------------------------------------

/// Counts pinned by the court legs' own assertions, returned from one shared
/// probe run for the registry adapter's metrics.
pub(crate) struct N12ProbeRun {
    /// Court 3: firings committed under the accepted-frontier order.
    pub court3_committed: usize,
    /// Court 3: regression-stamped positions that were typed-refused.
    pub court3_refused_stamps: usize,
}

/// Run the real fast-tier N12 court legs for real — court 1, court 2, and
/// court 3 — panicking on any violation. Every leg's assertions are the
/// verbatim court assertions; reaching the end means all three legs hold.
/// Court probe: panics on law violation are the court's verdict law
    /// (donor-verbatim from the tier tests this was hoisted from).
    pub(crate) fn n12_run_court_probe() -> N12ProbeRun {
    // Court 1: feed a real forward chain through the strict law with
    // glitches injected at ~1% (every 100th position, cycling the classes:
    // backwards replay, 0-jump, huge real jump). Every fed pair is
    // adjudicated exactly once; the accepted frontier advances ONLY on
    // lawful forward motion.
    const N: usize = 1_000;
    const GLITCH_PERIOD: usize = 100;

    let chain = forward_chain(N);

    // Baseline: the clean real chain admits end-to-end.
    for w in chain.windows(2) {
        assert!(adjudicate(w[0], w[1]), "clean chain pair must admit");
    }

    let mut fed = 0usize;
    let mut typed_refusals = 0usize;
    let mut huge_jumps_admitted = 0usize;
    let mut frontier = chain[0];
    for i in 1..N {
        let next = if i % GLITCH_PERIOD == 0 {
            match (i / GLITCH_PERIOD) % 3 {
                // backwards step: replay a buffered earlier real sample
                0 => chain[i / 2],
                // 0-jump: replay the current frontier
                1 => frontier,
                // huge jump: a real 2 ms clock gap (lawful forward)
                _ => huge_jump_sample(frontier),
            }
        } else {
            chain[i]
        };
        // Expected class derived from construction, independently of what
        // the law returns — the law must agree.
        let raw_prev = frontier.as_nanos();
        let raw_next = next.as_nanos();
        let expect_refusal = raw_next <= raw_prev;
        if adjudicate(frontier, next) {
            assert!(!expect_refusal, "law admitted a constructed glitch");
            if raw_next - raw_prev >= 1_000_000 {
                huge_jumps_admitted += 1;
                // The frontier deliberately does NOT adopt the 2 ms jump:
                // doing so would lawfully refuse every remaining real
                // sample for 2 ms (the strict law working — see donor
                // court 1's one-shot probe note).
            } else {
                frontier = next;
            }
        } else {
            assert!(expect_refusal, "law refused a lawful forward pair");
            typed_refusals += 1;
        }
        fed += 1;
    }

    let glitch_count = (N - 1) / GLITCH_PERIOD;
    assert_eq!(fed, N - 1, "every fed pair adjudicated exactly once — zero silent acceptance");
    assert_eq!(
        typed_refusals,
        (glitch_count * 2 + 2) / 3,
        "backwards/0-jump classes (2/3 of glitches) must refuse"
    );
    assert_eq!(
        huge_jumps_admitted,
        glitch_count / 3,
        "every huge-jump glitch must be LAWFULLY ADMITTED (ordered), never dropped"
    );

    // Court 2: frozen-clock detection with bounded liveness. A frozen source
    // is refused one pair at a time (`Regression`) inside the first liveness
    // window; from the bound onward the typed `FrozenClock` verdict fires
    // and keeps firing (bounded liveness: the verdict is bounded, the
    // refusal is permanent). A healthy real source NEVER trips the bound.
    let anchor = forward_chain(2);
    let stuck = anchor[0].as_nanos();
    let later = anchor[1].as_nanos();
    assert!(later > stuck, "fixture anchors must be ordered");

    // A gate seeded at a value strictly above the frozen sample refuses
    // every sample: Regression inside the first window, FrozenClock from
    // the bound onward, forever.
    let mut gate = MonotonicGate::new(FrozenClock::at(stuck));
    gate.frontier = Some(HwTick { nanos: later });
    for step in 1..=(FROZEN_CLOCK_LIVENESS_BOUND * 2) {
        match gate.tick() {
            Err(TickError::Regression { prev, next }) => {
                assert_eq!(prev, later, "refusal carries the true frontier");
                assert_eq!(next, stuck, "refusal carries the frozen sample");
                assert!(
                    step < FROZEN_CLOCK_LIVENESS_BOUND,
                    "Regression must fire only inside the first liveness window"
                );
            }
            Err(TickError::FrozenClock { repeats, last }) => {
                assert!(
                    step >= FROZEN_CLOCK_LIVENESS_BOUND && repeats >= FROZEN_CLOCK_LIVENESS_BOUND,
                    "FrozenClock fires only at/after the bound"
                );
                assert_eq!(repeats, step, "one stuck sample per refused tick");
                assert_eq!(last, stuck, "FrozenClock carries the stuck sample");
            }
            Ok(t) => panic!("frozen clock sample {t:?} must never be admitted"),
        }
    }
    assert_eq!(gate.frontier().unwrap().as_nanos(), later);

    // Positive half: a healthy real source crosses thousands of ticks
    // without ever tripping the liveness bound.
    let mut live = MonotonicGate::new(MonotonicClock);
    for _ in 0..256 {
        // Strict-consumer retry law: re-sample on Regression (equal
        // samples are a sampling artifact, not a clock fault).
        let mut s = live.tick();
        while matches!(s, Err(TickError::Regression { .. })) {
            s = live.tick();
        }
        assert!(s.is_ok(), "healthy clock must never yield FrozenClock: {s:?}");
        assert!(live.stuck_count() < FROZEN_CLOCK_LIVENESS_BOUND);
    }

    // Court 3: the ordering law over [`crate::conformance::pn64`]
    // transitions. PN64 firings are stamped with clock samples and applied
    // through [`pn64::step_branchless`]; a transition stamped at a REFUSED
    // tick is not applied (word bit-exact unchanged), and the committed
    // transition stream is deterministic under the accepted-frontier order.
    // The enabled transition alone commits; every regression-stamped
    // position refuses.
    let pn = pn64::pack(0x0002, 0x0002, 0x0004, 0x07, 0x00);
    assert!(pn64::enabled(pn), "fixture transition must start enabled");
    let ocel = 0x0107u64;

    let chain = forward_chain(64);
    let stamp = |i: usize| if i % 4 == 0 { chain[i / 2] } else { chain[i] };

    let run = |chain: &[HwTick]| {
        let mut frontier = chain[0];
        let mut word = pn;
        let mut committed = 0usize;
        let mut refused_stamps = 0usize;
        for i in 1..chain.len() {
            let stamped = stamp(i);
            match HwTick::mono_checked(frontier, stamped) {
                Ok(admitted) => {
                    frontier = admitted;
                    let before = word;
                    word = pn64::step_branchless(word, ocel, 0x0000);
                    if word != before {
                        committed += 1;
                    }
                    // Transition identity survives every committed step.
                    assert_eq!(pn64::tid(word), pn64::tid(pn));
                    assert_eq!(pn64::flags(word), pn64::flags(pn));
                }
                Err(e) => {
                    assert_eq!(
                        e,
                        TickError::Regression {
                            prev: frontier.as_nanos(),
                            next: stamped.as_nanos(),
                        },
                        "regression-stamped transition must be typed-refused with the exact pair"
                    );
                    // REFUSED stamp: the transition is NOT applied — the
                    // word is left bit-exact unchanged.
                    refused_stamps += 1;
                }
            }
        }
        (word, committed, refused_stamps)
    };

    let (word, committed, refused_stamps) = run(&chain);
    assert!(committed >= 1, "the lawful stream must commit at least one firing");
    assert!(
        refused_stamps > 0,
        "the glitch stream must exercise the refusal path (replays are many-to-one)"
    );

    // Determinism under the ordering law: the same stamps, replayed, commit
    // the identical word sequence — ordering is a function of the frontier.
    let (word2, committed2, refused2) = run(&chain);
    assert_eq!(word, word2, "committed stream deterministic under the ordering law");
    assert_eq!(pn64::tid(word), pn64::tid(word2), "transition identity survives replay");
    assert_eq!(committed, committed2);
    assert_eq!(refused_stamps, refused2);

    N12ProbeRun { court3_committed: committed, court3_refused_stamps: refused_stamps }
}

/// Court 1 (default tier): the shared probe's jitter-injection leg plus the
/// frozen-clock and ordering legs — one shared fast-tier run, so the test
/// surface and the registry adapter can never drift.
#[test]
fn n12_shared_probe_court1_leg() {
    n12_run_court_probe();
}

/// Court 2 (default tier): frozen-clock bounded liveness — the shared
/// probe's court-2 leg (see [`n12_run_court_probe`]).
#[test]
fn n12_shared_probe_court2_leg() {
    n12_run_court_probe();
}

/// Court 3 (default tier): the pn64 ordering law — the shared probe's
/// court-3 leg (see [`n12_run_court_probe`]); its committed/refused counts
/// feed the registry adapter's metrics.
#[test]
fn n12_shared_probe_court3_leg() {
    let r = n12_run_court_probe();
    assert!(
        r.court3_committed >= 1 && r.court3_refused_stamps > 0,
        "the ordering-law leg must commit at least one firing and exercise refusals"
    );
}


/// Court 4 (positive control): the injected non-monotonic source MUST be
/// caught. [`BackstepClock`] oscillates between two real anchor values
/// starting at the later one; every sample after the seed is either a
/// backwards step or a 0-jump, and the gate must refuse ALL of them with
/// the exact typed pair, frontier never moving. The unprotected mutant
/// (regression predicate dropped) must admit every one of them — proving
/// the check is load-bearing. Returns `true` iff every refusal/mutant
/// assertion held (panics on any violation otherwise).
pub(crate) fn n12_positive_control() -> bool {
    let anchor = forward_chain(2);
    let (b, a) = (anchor[0].as_nanos(), anchor[1].as_nanos());
    assert!(b < a, "anchors must be ordered (b earlier, a later)");

    let mut gate = MonotonicGate::new(BackstepClock::between(a, b));
    let first = gate.tick().expect("first sample seeds the frontier");
    assert_eq!(first.as_nanos(), a, "the clock starts at the later anchor");
    let mut caught = 0usize;
    let mut frozen = 0usize;
    for step in 1..=32 {
        match gate.tick() {
            Err(TickError::Regression { prev, next }) => {
                assert_eq!(prev, a, "frontier pinned at the later anchor");
                assert!(next == b || next == a, "only real anchor values are fed");
                assert!(
                    step < FROZEN_CLOCK_LIVENESS_BOUND,
                    "Regression fires only inside the first liveness window"
                );
                caught += 1;
            }
            Err(TickError::FrozenClock { repeats, last }) => {
                assert!(step >= FROZEN_CLOCK_LIVENESS_BOUND);
                assert_eq!(repeats, step, "one stuck sample per refused tick");
                assert!(last == b || last == a, "only real anchor values are fed");
                frozen += 1;
            }
            Ok(t) => panic!("backstep sample {t:?} must never be admitted"),
        }
    }
    assert_eq!(caught, 7, "the first window refuses pair-wise; then the frozen verdict takes over");
    assert_eq!(
        frozen, 25,
        "every oscillating sample after the seed is refused, one way or the other"
    );
    assert_eq!(gate.frontier(), Some(first), "frontier never moves on refusals");
    assert_eq!(gate.stuck_count(), 32);

    // Non-vacuity: the unprotected mutant ADMITS the identical stream.
    let mut mutant = UnprotectedFold::new(BackstepClock::between(a, b));
    let mut mutant_admits = 0usize;
    let _ = mutant.tick();
    for _ in 1..=32 {
        let s = mutant.tick();
        assert!(s.as_nanos() > 0, "mutant admits unconditionally");
        mutant_admits += 1;
    }
    assert_eq!(
        mutant_admits, 32,
        "the mutant must admit every sample the gate refuses — check is load-bearing"
    );
    true
}

/// Court 4 (default-tier wrapper): the shared positive control must hold.
#[test]
fn court4_positive_control_backstep_clock_is_caught() {
    assert!(
        n12_positive_control(),
        "VACUOUS COURT: the BackstepClock mutant survived the strict-forward gate"
    );
}

/// Court 4b (honest host note): the declared source is the wall-monotonic
/// surrogate, NOT a raw counter register — and the declaration stands in
/// the compiled source, pinned here (compile-time audit of the real file).
#[test]
fn court4_honest_host_note_no_counter_register() {
    let name = MonotonicClock.source_name();
    assert!(
        name.contains("monotonic") && name.contains("surrogate"),
        "host note: declared source must name the monotonic surrogate, got {name:?}"
    );
    assert!(
        !name.to_lowercase().contains("rdtsc") && !name.to_lowercase().contains("cycle"),
        "no raw-cycle source may be declared on this host"
    );

    let src = include_str!("tick.rs");
    // The surrogate declaration stands in the source the court compiled.
    assert!(
        src.contains("mach_absolute_time surrogate"),
        "surrogate declaration drifted from tick.rs"
    );
    // The honest scoping line: the weaker-than-donor port is declared.
    assert!(src.contains("NO RDTSC/counter-register access"), "honest-host scoping line drifted");
    assert!(src.contains("OUT OF SCOPE"), "donor court 2a/2b disclosure drifted");
    // The scoping record itself validates and names the bound.
    let sr = scoping_record();
    sr.validate()
        .expect("partial scoping record must name delivered evidence and bound");
    assert!(!sr.is_full_spec_scale, "port must disclose as partial, not full scale");
    assert!(sr.bound.contains("RDTSC"), "bound must name the counter-register limitation");
}

/// Tier-3 calibration (donor scale): measure the real clock's forward-step
/// distribution and minimum observable resolution. Not a pass/fail law —
/// pins that the surrogate clock can actually resolve strictly-forward
/// samples at the rate the fast courts assume.
#[test]
#[ignore = "nato-tier3"]
fn n12_clock_resolution_calibration() {
    const N: usize = 100_000;
    let chain = forward_chain(N);
    let mut min_step = u128::MAX;
    let mut zero_steps = 0usize;
    for w in chain.windows(2) {
        let d = w[1].as_nanos() - w[0].as_nanos();
        if d == 0 {
            zero_steps += 1;
        } else if d < min_step {
            min_step = d;
        }
    }
    assert_eq!(zero_steps, 0, "chain is strictly deduplicated by construction");
    assert!(
        min_step > 0 && min_step < 1_000_000,
        "surrogate must resolve sub-millisecond forward steps, got min {min_step} ns"
    );
}

/// Real-receipt emission for the N12 dossier slot (opt-in via
/// `CTT_EMIT_RECEIPTS=1`): runs the fast tier for real — court 1 (jitter
/// injection over a real forward chain), court 2 (frozen-clock bounded
/// liveness), and the court-4 positive control (BackstepClock caught) — then
/// mints and persists `docs/receipts/N12.json` from that run. Deterministic:
/// the receipt carries only counts pinned by the courts' own assertions, no
/// wall-clock values; `timestamp_ms` is the fixed 0 slot. Replaces the
/// synthetic `control-subject` placeholder. Donor court 2b (byte-tick wrap)
/// stays OUT OF SCOPE per the module header — it is not adjudicated here.
#[cfg(test)]
mod receipt_emission {
    use super::*;
    use crate::nato::court::{CourtId, CourtOutcome, MetricValue, Pillar, Verdict};
    use crate::nato::receipt::{ControlResult, CourtReceipt};
    use sha2::{Digest, Sha256};

    /// Deterministic subject identity: SHA-256 over the canonical court name
    /// plus the exact subject the court adjudicates.
    fn subject_sha() -> String {
        let mut h = Sha256::new();
        h.update(
            b"N12|src/nato/tick.rs|wall-monotonic strict-forward clock-glitch resilience gate \
              (MonotonicGate over Instant; courts 1 jitter-injection, 2 frozen-clock bounded \
              liveness, 4 backstep positive control)",
        );
        h.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>()
    }

    /// Mint and persist `docs/receipts/N12.json` from a real fast-tier run.
    /// Hermetic by default; opt in with `CTT_EMIT_RECEIPTS=1`.
    #[test]
    fn emit_receipt_when_opted_in() {
        if std::env::var("CTT_EMIT_RECEIPTS").ok().as_deref() != Some("1") {
            return;
        }

        // Positive control FIRST: the injected non-monotonic BackstepClock
        // MUST be caught (court 4 asserts the gate refuses every sample and
        // the unprotected mutant admits all of them). It panics otherwise —
        // a surviving mutant must never mint a Pass receipt.
        super::court4_positive_control_backstep_clock_is_caught();
        let control = ControlResult::MutantCaught {
            mutant_id: "BackstepClock".to_string(),
            detection_channel: "MonotonicGate strict-forward refusal".to_string(),
        };

        // Real fast-tier courts: panics on any violation.
        super::n12_shared_probe_court1_leg();
        super::n12_shared_probe_court2_leg();

        // Metrics: counts pinned by the courts' own assertions, derived from
        // this real run — never wall-clock readings.
        let outcome = CourtOutcome {
            verdict: Verdict::Pass,
            metrics: vec![
                ("court1_chain_len", MetricValue::U64(1_000)),
                ("court1_glitch_period", MetricValue::U64(100)),
                (
                    "frozen_clock_liveness_bound",
                    MetricValue::U64(FROZEN_CLOCK_LIVENESS_BOUND as u64),
                ),
                ("court4_backsteps_caught", MetricValue::U64(7)),
                ("court4_frozen_verdicts", MetricValue::U64(25)),
            ],
            scoping: crate::nato::tick::scoping_record(),
        };
        let receipt = CourtReceipt::record(
            CourtId::N12,
            Pillar::Resilience,
            "N12",
            &subject_sha(),
            outcome,
            control,
        );

        let json = receipt.to_json().expect("serialize N12 receipt");
        let back = CourtReceipt::from_json(&json).expect("receipt must parse back");
        assert_eq!(back.verdict, Verdict::Pass);
        assert!(back.positive_control.passed());
        assert_ne!(back.subject_sha, "control-subject");

        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/receipts");
        let path = dir.join("N12.json");
        if let Err(e) = std::fs::write(&path, json) {
            println!("skipping N12 receipt emission: cannot write {}: {e}", path.display());
        }
    }
}

// ---------------------------------------------------------------------------
// Court wiring (plan W-NTW: N12 registry adapter, mirrors mils.rs:422-475)
// ---------------------------------------------------------------------------

/// N12 clock-glitch resilience court.
pub struct ClockGlitchCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl ClockGlitchCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N12;
    /// Capability pillar. Must match the emitter's mint
    /// (`emit_receipt`, this file) and the on-disk
    /// `docs/receipts/N12.json` — adapters align to emitter
    /// receipts, never to a plan table.
    pub const PILLAR: Pillar = Pillar::Resilience;
}

// Metrics are informational and deliberately diverge from the
        // opt-in emitter's set (see emit_receipt_when_opted_in): this
        // adapter reports live court-3 probe counts; the emitter reports
        // court-1/court-4 constants. Verdict agreement is carried by the
        // shared probe (n12_run_court_probe) + the court_id/pillar/verdict/
        // name field legs, not by metric-key parity (W-NTW §6).
impl Court for ClockGlitchCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N12 clock_glitch"
    }

    fn positive_control(&self) -> ControlProbe {
        n12_positive_control
    }

    fn run(&self, _candidate: &crate::nato::court::CandidateRef) -> CourtOutcome {
        // Shared probe (anti-drift law): the same real fast-tier run the
        // #[test] wrappers and the opt-in receipt emitter read. All three
        // legs assert internally; reaching here means they hold.
        let r = n12_run_court_probe();
        CourtOutcome {
            verdict: Verdict::Pass,
            metrics: vec![
                ("court1_chain_len", MetricValue::U64(1_000)),
                ("court1_glitch_period", MetricValue::U64(100)),
                (
                    "frozen_clock_liveness_bound",
                    MetricValue::U64(FROZEN_CLOCK_LIVENESS_BOUND as u64),
                ),
                ("court3_committed", MetricValue::U64(r.court3_committed as u64)),
                ("court3_refused_stamps", MetricValue::U64(r.court3_refused_stamps as u64)),
            ],
            // The host-note leg (no counter register on this host) stays a
            // disclosure, not a verdict input: the partial scoping record
            // names the RDTSC/raw-cycle bound.
            scoping: crate::nato::tick::scoping_record(),
        }
    }
}
