#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests and the court registry, so non-test builds legitimately
// see unused items; the probe/control fns are dead to test builds only and
// referenced by the court registry.
#![allow(dead_code, unused_imports)]
// unused_imports: ConsensusError is doc-link-referenced (`ConsensusError`) but not constructed outside tests
//! N14 — CAST-32A multi-core interference court (NATO lane N14).
//!
//! Port per E-M18 from strata `tests/nato/n14_cast32a_thrash.rs` (Apache/MIT
//! per strata LICENSE), retargeted from the donor's sequencer onto the landed
//! `CasRing` claim/commit/drain surface (claim/commit/abandon at
//! ring.rs:167/:208/:232). Eviction-thread body mirrors N07's storm thread
//! (crate::nato::cache.rs:245 `storm_thread`): LCG-strided mutating writes
//! over a 2x-L2 buffer, 64 touches per stop-flag check.
//!
//! Spec (docs/nato/NATO_N01_N20.md, N14): "16 adversarial threads vs
//! sequencer; bounded max latency, zero deadlock."
//!
//! Three legs, per lane contract, under a 4-consumer / 4-producer MPMC
//! pipeline driving the ring at donor topology:
//!
//! 1. **Typed refusals only (no SDC)** — every protocol failure must surface
//!    as a typed `ConsensusError`, never a bogus word in the drained
//!    stream. Witnessed two ways: producer-side commits are all `Ok` (a
//!    refusal counter must read zero), and consumer-side every drained word
//!    is in the payload domain (never the `u64::MAX`/`u64::MAX-1` sentinels).
//! 2. **Liveness within bound** — the whole court (adversaries + pipeline)
//!    must report inside a generous wall-clock bound, enforced by a watchdog
//!    thread + `mpsc::Receiver::recv_timeout`. A hung pipeline is a typed
//!    [`ThrashCourtError::DeadlockBound`] FAIL, not a harness hang. Every
//!    worker loop checks the shared stop flag, so the post-timeout join is
//!    bounded too (bounded-join liveness law).
//! 3. **Conservation of committed words** — the drained stream contains
//!    every producer word exactly once (sorted disjoint-range law, identical
//!    to `mpmc_4x4_no_double_claim`), under full adversarial load. The
//!    order-sensitive FNV-1a checksum over the sorted stream is the
//!    run-invariant multiset identity for the 3-run determinism leg.
//!
//! Latency honest-scoping (same law as N07/donor): the spec's 4.5 ns
//! per-transition figure is a deterministic single-core evidence-class
//! target and is NOT asserted under contention. The 1 s stall ceiling is a
//! WALL-CLOCK law and is asserted only in the tier-3
//! `#[ignore = "nato-tier3"]` legs (a wall-clock bound under unknown load
//! measures the machine, not the subject — AS2 skeptic; the default tier-1
//! legs assert completion + typed refusals + conservation only); measured
//! max/mean are the honest CAST-32A data point.
//!
//! Tiering (prefix-stream, per E-M18): tier-1 `*_small` variants run a
//! reduced transition budget with the same adversary shape (the
//! interference, not the transition count, is the subject); the
//! `#[ignore = "nato-tier3"]` `*_full` tests run donor-verbatim constants.
//!
//! Positive control: a wedged, watchdog-less shim (every ring slot leaked,
//! producer spinning on `claim()` forever) MUST be deadlock-detected by the
//! bounded watchdog as [`ThrashCourtError::DeadlockBound`] — proving the
//! watchdog load-bearing and the liveness court non-vacuous.
//!
//! Memory contract: payload words are `p * per_producer + i`, max < 10^6 —
//! far from the CasRing's reserved sentinels `u64::MAX` / `u64::MAX - 1`
//! (documented domain reserve).

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
    clippy::manual_is_multiple_of,
    clippy::manual_midpoint,
    clippy::identity_op,
    clippy::format_collect,
    clippy::trivially_copy_pass_by_ref,
    // The producer's bounded spin burst is declared at its single point of
    // use (the claim retry loop); hoisting it to fn top separates it from
    // the only loop that uses it.
    clippy::items_after_statements
)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;
use crate::nato::torn::ring::{CasRing, ConsensusError, SlotId};

// ---------------------------------------------------------------------------
// Court constants (donor-verbatim at full tier; reduced tier-1 variant)
// ---------------------------------------------------------------------------

/// Transitions per full run (lane contract / donor: 1M).
const TRANSITIONS_FULL: u64 = 1_000_000;
/// Transitions per tier-1 small run (same adversary shape, reduced budget).
const TRANSITIONS_SMALL: u64 = 50_000;
/// Producers in the MPMC pipeline (donor-verbatim).
const PRODUCERS: u64 = 4;
/// Consumers draining the ring (donor-verbatim).
const CONSUMERS: usize = 4;
/// Ring capacity (power of two, compile-time enforced by CasRing; donor).
const RING_CAP: usize = 4096;
/// Eviction threads, tier-1 small (donor runs 8).
const EVICTORS_SMALL: usize = 4;
/// Eviction threads, full tier (donor-verbatim).
const EVICTORS_FULL: usize = 8;
/// Adjacent-cacheline atomic spinners, tier-1 small (donor runs 8).
const SPINNERS_SMALL: usize = 4;
/// Adjacent-cacheline atomic spinners, full tier (donor-verbatim).
const SPINNERS_FULL: usize = 8;
/// Eviction buffer per evictor thread, full tier: 8 MiB = 2x donor-host L2.
const EVICT_WORDS_FULL: usize = 8 * 1024 * 1024 / 8;
/// Generous deadlock bound, tier-1 small.
const DEADLOCK_BOUND_SMALL: Duration = Duration::from_secs(120);
/// Generous deadlock bound for a full donor-scale court run (donor: 600 s).
const DEADLOCK_BOUND_FULL: Duration = Duration::from_secs(600);
/// Sanity ceiling on observed per-transition latency: anything >= 1 s per
/// single transition is a stall/deadlock-class event, not contention.
const MAX_TRANSITION_NS_CEILING: u64 = 1_000_000_000;

// ---------------------------------------------------------------------------
// Typed court failures
// ---------------------------------------------------------------------------

/// Typed failure of the N14 CAST-32A thrash court. Zero diagnostic entropy:
/// the variant is the whole message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrashCourtError {
    /// The court run did not report within the generous deadlock bound.
    DeadlockBound {
        /// The bound that was exceeded.
        bound: Duration,
    },
    /// The court's result channel closed without a result.
    CourtLost,
}

// ---------------------------------------------------------------------------
// Adversaries
// ---------------------------------------------------------------------------

/// One AtomicU64 per spinner, packed CONSECUTIVELY (adjacent cachelines —
/// deliberately NOT cacheline-padded between neighbors: the coherence
/// traffic between adjacent lines is the load).
#[repr(align(64))]
struct AdjacentLines([AtomicU64; SPINNERS_FULL]);

/// Summary of one full court run.
#[derive(Debug, Clone, Copy)]
pub struct RunSummary {
    /// Max observed per-transition latency, ns (claim -> commit).
    pub max_latency_ns: u64,
    /// Mean per-transition latency, ns.
    pub mean_latency_ns: f64,
    /// Wall time for the pipeline (all transitions, drain to completion).
    pub pipeline_ns: u128,
    /// Order-sensitive checksum over the SORTED drained word stream.
    pub conservation_checksum: u64,
}

/// Scale knobs for one court run.
#[derive(Debug, Clone, Copy)]
struct ThrashConfig {
    /// Uniquely-tagged words committed per run.
    transitions: u64,
    /// Eviction-storm threads.
    evictors: usize,
    /// Adjacent-cacheline spinners.
    spinners: usize,
    /// u64 words in each evictor's buffer.
    evict_buf_words: usize,
    /// Generous watchdog bound.
    deadlock_bound: Duration,
}

impl ThrashConfig {
    /// Donor-verbatim full calibration: 1M transitions, 8+8 adversaries,
    /// 8 MiB eviction buffers, 600 s bound.
    const FULL: Self = Self {
        transitions: TRANSITIONS_FULL,
        evictors: EVICTORS_FULL,
        spinners: SPINNERS_FULL,
        evict_buf_words: EVICT_WORDS_FULL,
        deadlock_bound: DEADLOCK_BOUND_FULL,
    };

    /// Build the tier-1 small variant: 50k transitions, 4+4 adversaries,
    /// 2x-probed-L2 eviction buffers, 60 s bound. The storm shape (2x-L2
    /// LCG-strided eviction + adjacent-line spin) is the donor's; only the
    /// transition budget and thread counts shrink.
    fn small() -> Self {
        let l2 = crate::nato::cache::l2_cache_size_bytes();
        let l2_words = (l2 as usize / 8).max(1);
        Self {
            transitions: TRANSITIONS_SMALL,
            evictors: EVICTORS_SMALL,
            spinners: SPINNERS_SMALL,
            evict_buf_words: l2_words * 2,
            deadlock_bound: DEADLOCK_BOUND_SMALL,
        }
    }
}

/// Knuth LCG over 2^64: full period; the stride covers every cache set at
/// every level. Same shape as N07's `storm_thread` (cache.rs:245).
#[inline]
fn lcg(x: u64) -> u64 {
    x.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

/// One eviction-storm thread: LCG-strided mutating writes over a 2x-L2
/// buffer (the N07 eviction-thread body, retargeted; 64 touches per
/// stop-flag check so the post-timeout join is bounded).
fn storm_thread(buf: &mut [u64], seed: u64, stop: &AtomicBool) {
    let n = buf.len();
    let mut x = seed;
    loop {
        for _ in 0..(n / 64).max(1) {
            x = lcg(x);
            let i = ((x >> 24) as usize) % n;
            buf[i] = buf[i].wrapping_add(x | 1);
        }
        if stop.load(Ordering::Relaxed) {
            return;
        }
    }
}

/// One adjacent-cacheline atomic spinner: `fetch_add` on its own line plus a
/// read-modify-write on the ADJACENT line, forcing MESI ownership to bounce.
fn spinner_thread(lines: &AdjacentLines, s: usize, stop: &AtomicBool) {
    let line = &lines.0[s];
    let neighbor = &lines.0[(s + 1) % lines.0.len()];
    let mut local = 0u64;
    while !stop.load(Ordering::Relaxed) {
        local = line.fetch_add(1, Ordering::Relaxed);
        let _ = neighbor.fetch_add(local & 1, Ordering::Relaxed);
        std::hint::spin_loop();
    }
    std::hint::black_box(local);
}

/// Execute one court run under `cfg`: the adversarial thrash plus the
/// CasRing MPMC 4x4 claim/commit/drain pipeline. Returns the run summary,
/// or a typed [`ThrashCourtError::DeadlockBound`] if the watchdog fires.
///
/// Zero SDC leg: producer commits must all be `Ok` (typed-refusal census
/// reads zero) and every drained word must lie in the payload domain (never
/// a reserved sentinel). Conservation leg: sorted disjoint-range law over
/// all committed words, then the FNV-1a order-sensitive checksum.
fn run_court(cfg: ThrashConfig) -> Result<RunSummary, ThrashCourtError> {
    let stop = Arc::new(AtomicBool::new(false));
    let per_producer = cfg.transitions / PRODUCERS;

    // Watchdog: a pipeline that does not report within the generous bound is
    // a typed DEADLOCK finding, not a harness hang. The pipeline sends its
    // RunSummary on completion; on timeout the watchdog sets the shared stop
    // flag so every worker's join is bounded (bounded-join liveness law).
    let (done_tx, done_rx) = mpsc::channel::<RunSummary>();
    let watchdog_stop = Arc::clone(&stop);
    let watchdog = std::thread::spawn(move || {
        done_rx.recv_timeout(cfg.deadlock_bound).map_err(|_| {
            watchdog_stop.store(true, Ordering::Relaxed);
            ThrashCourtError::DeadlockBound { bound: cfg.deadlock_bound }
        })
    });

    // --- Adversarial load ------------------------------------------------
    let mut adversaries = Vec::with_capacity(cfg.evictors + cfg.spinners);
    for e in 0..cfg.evictors {
        let stop = Arc::clone(&stop);
        // Distinct deterministic seed per thread.
        let seed = 0xC0FF_EE00_u64 | (e as u64);
        adversaries.push(std::thread::spawn(move || {
            let mut buf = vec![0u64; cfg.evict_buf_words];
            storm_thread(&mut buf, seed, &stop);
            // Publish a sink so LLVM cannot elide the sweep.
            std::hint::black_box(buf.last().copied());
        }));
    }
    let lines = Arc::new(AdjacentLines([
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ]));
    for s in 0..cfg.spinners {
        let lines = Arc::clone(&lines);
        let stop = Arc::clone(&stop);
        adversaries.push(std::thread::spawn(move || {
            spinner_thread(&lines, s, &stop);
        }));
    }

    // --- Pipeline: CasRing MPMC 4x4 ---------------------------------------
    let ring = Arc::new(CasRing::<RING_CAP>::new());
    let drained_count = Arc::new(AtomicU64::new(0));
    let garbage = Arc::new(AtomicU64::new(0));
    let typed_refusals = Arc::new(AtomicU64::new(0));

    // 4 consumers: drain until the global count reaches the run budget. The
    // stop flag bounds the join on the deadlock path.
    let mut consumers = Vec::with_capacity(CONSUMERS);
    for _ in 0..CONSUMERS {
        let ring = Arc::clone(&ring);
        let drained_count = Arc::clone(&drained_count);
        let garbage = Arc::clone(&garbage);
        let stop = Arc::clone(&stop);
        consumers.push(std::thread::spawn(move || {
            let mut local: Vec<u64> = Vec::new();
            while !stop.load(Ordering::Acquire)
                && drained_count.load(Ordering::Acquire) < cfg.transitions
            {
                let before = drained_count.load(Ordering::Acquire);
                ring.drain_into(&mut |w| {
                    // No SDC: a drained sentinel would be corruption, not a
                    // payload word.
                    if w == u64::MAX || w == u64::MAX - 1 {
                        garbage.fetch_add(1, Ordering::Relaxed);
                    }
                    local.push(w);
                    drained_count.fetch_add(1, Ordering::Relaxed);
                });
                if drained_count.load(Ordering::Acquire) == before {
                    // Idle pass: yield the core so a producer can claim/commit.
                    // Pure hot spinning livelocks under CPU oversubscription
                    // (debug builds): the watchdog then fires spuriously.
                    std::thread::yield_now();
                }
            }
            local
        }));
    }

    // 4 producers: each commits per_producer uniquely-tagged words. One
    // transition = ring claim (spin until a slot frees; consumers drain
    // concurrently so this is liveness-safe) -> commit. Per-transition
    // Instant timing on every transition. The stop flag bounds the join on
    // the deadlock path.
    let mut producers = Vec::with_capacity(PRODUCERS as usize);
    for p in 0..PRODUCERS {
        let ring = Arc::clone(&ring);
        let typed_refusals = Arc::clone(&typed_refusals);
        let stop = Arc::clone(&stop);
        producers.push(std::thread::spawn(move || {
            let mut max_ns = 0u64;
            let mut sum_ns = 0u128;
            let mut committed = 0u64;
            while committed < per_producer {
                let slot: SlotId = loop {
                    if stop.load(Ordering::Acquire) {
                        return (max_ns, sum_ns, committed);
                    }
                    if let Some(s) = ring.claim() {
                        break s;
                    }
                    // Bounded spin burst, then yield: a hot spinner that never
                    // yields the core can livelock against the draining
                    // counterpart under CPU oversubscription (debug builds).
                    static SPIN_BURST: u32 = 64;
                    thread_local!(static SPINS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) });
                    SPINS.with(|c| {
                        let n = c.get();
                        if n < SPIN_BURST {
                            c.set(n + 1);
                            std::hint::spin_loop();
                        } else {
                            c.set(0);
                            std::thread::yield_now();
                        }
                    });
                };
                let t0 = Instant::now();
                match ring.commit(slot, p * per_producer + committed) {
                    Ok(()) => {
                        committed += 1;
                    }
                    Err(_) => {
                        // Typed refusal census: the ring failed CLOSED, never
                        // invented a word.
                        typed_refusals.fetch_add(1, Ordering::Relaxed);
                    }
                }
                let dt = t0.elapsed().as_nanos() as u64;
                max_ns = max_ns.max(dt);
                sum_ns += dt as u128;
            }
            (max_ns, sum_ns, committed)
        }));
    }

    // Join producers, then stop adversaries, then join consumers.
    let t0 = Instant::now();
    let mut max_latency_ns = 0u64;
    let mut total_ns = 0u128;
    let mut committed_total = 0u64;
    for pr in producers {
        let (mx, sum, committed) = pr.join().expect("producer thread");
        max_latency_ns = max_latency_ns.max(mx);
        total_ns += sum;
        committed_total += committed;
    }
    let pipeline_ns = t0.elapsed().as_nanos();

    // Join consumers BEFORE setting stop: a consumer self-terminates only at
    // drained == transitions, and setting stop earlier lets a consumer that
    // is lagging the producers (cross-court contention) exit early and
    // strand committed-but-undrained words in the ring — a spurious typed
    // DeadlockBound. On the true-deadlock path the watchdog has already set
    // stop at the bound, so this join stays bounded either way. The storm
    // now covers the whole commit+drain phase.
    let mut conservation: Vec<u64> = Vec::with_capacity(cfg.transitions as usize);
    for c in consumers {
        conservation.extend(c.join().expect("consumer thread"));
    }
    // Stop the storm only after the pipeline (producers + consumers) is done.
    stop.store(true, Ordering::Release);
    for a in adversaries {
        a.join().expect("adversary thread");
    }

    // The watchdog governs: if the pipeline wedged anywhere above, the
    // recv_timeout has already expired, the stop flag is set, and the partial
    // run is refused typed — no conservation asserts on partial data.
    if drained_count.load(Ordering::Acquire) != cfg.transitions {
        let err = watchdog.join().expect("watchdog thread");
        return match err {
            Err(e) => Err(e),
            // Impossible: the watchdog can only be Ok after receiving a
            // summary, and the summary is only sent below.
            Ok(_) => Err(ThrashCourtError::CourtLost),
        };
    }

    // Drain counter: no loss, no duplication at the counter.
    assert_eq!(
        drained_count.load(Ordering::Acquire),
        cfg.transitions,
        "loss/duplication at the drain counter"
    );
    assert_eq!(committed_total, cfg.transitions, "commit counter divergence");
    assert_eq!(typed_refusals.load(Ordering::Relaxed), 0, "typed ring refusals observed");
    assert_eq!(garbage.load(Ordering::Relaxed), 0, "sentinel-valued word drained (SDC)");

    // Zero loss / zero duplication: sorted disjoint-range law (producer p's
    // word p*per+i lands at exactly one sorted index).
    conservation.sort_unstable();
    for (idx, v) in conservation.iter().enumerate() {
        let expect = (idx as u64 / per_producer) * per_producer + (idx as u64 % per_producer);
        assert_eq!(*v, expect, "CONSERVATION VIOLATION at sorted index {idx}");
    }

    // Order-sensitive checksum over the sorted stream (run-invariant
    // multiset identity for the 3-run determinism leg).
    let mut checksum = 0xcbf2_9ce4_8422_2325u64; // FNV-1a offset basis
    for w in &conservation {
        for byte in w.to_le_bytes() {
            checksum ^= byte as u64;
            checksum = checksum.wrapping_mul(0x0000_0100_0000_01B3);
        }
    }
    debug_assert_eq!(conservation.len() as u64, cfg.transitions);

    let summary = RunSummary {
        max_latency_ns,
        mean_latency_ns: total_ns as f64 / cfg.transitions as f64,
        pipeline_ns,
        conservation_checksum: checksum,
    };

    // Report to the watchdog (donor shape: send on completion, then join).
    // On the healthy path the recv completes as soon as this send lands.
    let _ = done_tx.send(summary);
    // Only reachable as Err if the bound expired between the drained-count
    // check and the send (a >bound assertion gap): still a typed liveness
    // refusal, never a hang.
    watchdog.join().expect("watchdog thread")
}

// ---------------------------------------------------------------------------
// Positive control: watchdog-less wedged shim must deadlock-detect
// ---------------------------------------------------------------------------

/// Wedged, watchdog-less shim: every ring slot is claimed and DROPPED (no
/// commit, no abandon — the documented leak), so `claim()` returns None
/// forever and the producer spins structurally. The shim itself carries no
/// bound; the caller's bounded watchdog must detect the deadlock as
/// [`ThrashCourtError::DeadlockBound`], stop the producer via the shared
/// flag, and join. Returns the producer's partial commit count (zero: no
/// word was ever committed — no SDC under wedge).
fn run_wedged_shim(bound: Duration) -> Result<u64, ThrashCourtError> {
    let (_done_tx, done_rx) = mpsc::channel::<u64>();
    let watchdog = std::thread::spawn(move || {
        done_rx
            .recv_timeout(bound)
            .map_err(|_| ThrashCourtError::DeadlockBound { bound })
    });

    let ring = Arc::new(CasRing::<8>::new());
    let stop = Arc::new(AtomicBool::new(false));
    // Structural wedge: leak all 8 slots (dropped SlotIds, no abandon).
    for _ in 0..8 {
        let _leaked = ring.claim().expect("slot within capacity");
    }
    assert_eq!(ring.claim(), None, "shim wedge precondition: ring full");

    let producer = {
        let ring = Arc::clone(&ring);
        let stop = Arc::clone(&stop);
        std::thread::spawn(move || {
            let mut committed = 0u64;
            loop {
                if stop.load(Ordering::Acquire) {
                    return committed;
                }
                if let Some(s) = ring.claim() {
                    // Unreachable while wedged; a commit here would be SDC.
                    let _ = ring.commit(s, 0xDEAD_BEEF);
                    committed += 1;
                }
                std::hint::spin_loop();
            }
        })
    };

    // The watchdog (not the shim) provides the only bound.
    match watchdog.join().expect("watchdog thread") {
        Ok(_) => unreachable!("wedged shim reported a result"),
        Err(e) => {
            stop.store(true, Ordering::Relaxed);
            let committed = producer.join().expect("shim producer thread");
            assert_eq!(committed, 0, "wedged shim committed a word (SDC under wedge)");
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Tier-1 (default): one small court run — completion (zero deadlock),
/// typed refusals only / no SDC, conservation. NO wall-clock bound is
/// asserted here: under unknown ambient load a wall-clock reading measures
/// the machine, not the subject (AS2 skeptic, same honest-scoping law as
/// N05/N13). The bounded-latency law moved to [`n14_cast32a_thrash_full`]
/// (tier-3) and the watchdog bound was raised to 120 s so a correct-but-
/// slow-under-compile-load pipeline is never misclassified as a deadlock.
#[test]
fn n14_thrash_zero_deadlock_zero_loss_small() {
    let _ = run_court(ThrashConfig::small()).expect("small court run: pipeline completes");
}

/// Tier-1: 3 independent small runs; the drained multiset is bit-identical
/// across runs (conservation determinism).
#[test]
fn n14_three_runs_conservation_determinism_small() {
    let mut checksums = [0u64; 3];
    for (i, c) in checksums.iter_mut().enumerate() {
        let s = run_court(ThrashConfig::small()).expect("small court run within bound");
        *c = s.conservation_checksum;
    }
    assert_eq!(checksums[0], checksums[1], "run 2 conservation checksum diverged from run 1");
    assert_eq!(checksums[1], checksums[2], "run 3 conservation checksum diverged from run 2");
}

/// Tier-3 (donor-verbatim scale): 1M transitions, 16 adversarial threads,
/// 600 s bound. Same legs as the small variant at donor constants.
#[test]
#[ignore = "nato-tier3"]
fn n14_cast32a_thrash_full() {
    let summary = run_court(ThrashConfig::FULL).expect("full court run within bound");
    assert!(
        summary.max_latency_ns <= MAX_TRANSITION_NS_CEILING,
        "LATENCY STALL: max transition latency {} ns >= {} ns ceiling",
        summary.max_latency_ns,
        MAX_TRANSITION_NS_CEILING
    );
}

/// POSITIVE CONTROL: the watchdog-less wedged shim (all slots leaked,
/// producer structurally stuck on claim) MUST deadlock-detect via the
/// bounded watchdog as a typed DeadlockBound. If this fails, the watchdog
/// is not load-bearing and the liveness leg above is vacuous.
/// POSITIVE CONTROL probe: the watchdog-less wedged shim (all slots leaked,
/// producer structurally stuck on claim) MUST deadlock-detect via the
/// bounded watchdog as a typed `DeadlockBound` at the 250 ms bound. Returns
/// `true` iff the typed deadlock was detected — if `false`, the watchdog is
/// not load-bearing and the liveness court is vacuous. Shared verbatim by
/// the `#[test]` wrapper and the court registry.
pub(crate) fn n14_positive_control() -> bool {
    let wedge_bound = Duration::from_millis(250);
    match run_wedged_shim(wedge_bound) {
        Err(ThrashCourtError::DeadlockBound { bound }) => bound == wedge_bound,
        _ => false,
    }
}

#[test]
fn n14_positive_control_wedged_shim_deadlock_detected() {
    assert!(
        n14_positive_control(),
        "wedged shim did not deadlock-detect as typed DeadlockBound — control vacuous"
    );
}

/// Real-receipt emission for the N14 dossier slot (opt-in via
/// `CTT_EMIT_RECEIPTS=1`): runs the tier-1 small court for real (50k
/// transitions, 4+4 adversaries, MPMC 4x4 — completion + typed-refusal +
/// conservation legs all asserted inside `run_court`), plus the wedged-shim
/// positive control, then mints and persists `docs/receipts/N14.json` from
/// that run. Deterministic: the only per-run values in the receipt are the
/// conservation checksum over the SORTED drained stream (interleaving-
/// independent by the disjoint-range law) and fixed counts; no wall-clock
/// latencies enter the receipt. Replaces the synthetic `control-subject`
/// placeholder.
#[cfg(test)]
mod receipt_emission {
    use super::*;
    use crate::nato::court::{CourtId, CourtOutcome, MetricValue, Pillar, Verdict};
    use crate::nato::disclosure::ScopingRecord;
    use crate::nato::receipt::{ControlResult, CourtReceipt};
    use sha2::{Digest, Sha256};

    /// Deterministic subject identity: SHA-256 over the canonical court name
    /// plus the exact subject the court adjudicates.
    fn subject_sha() -> String {
        let mut h = Sha256::new();
        h.update(
            b"N14|src/nato/thrash.rs|CAST-32A multi-core interference court over the CasRing \
              MPMC 4x4 claim/commit/drain surface (tier-1 small: 50k transitions, 4 evictors \
              + 4 spinners, 120 s watchdog bound)",
        );
        h.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>()
    }

    /// Mint and persist `docs/receipts/N14.json` from a real tier-1 run.
    /// Hermetic by default; opt in with `CTT_EMIT_RECEIPTS=1`.
    #[test]
    fn emit_receipt_when_opted_in() {
        if std::env::var("CTT_EMIT_RECEIPTS").ok().as_deref() != Some("1") {
            return;
        }

        // Positive control FIRST: the watchdog-less wedged shim (all slots
        // leaked, producer structurally stuck) MUST deadlock-detect — it
        // panics otherwise. A surviving control must never mint a Pass.
        let wedge_bound = Duration::from_millis(250);
        let err = run_wedged_shim(wedge_bound).expect_err("wedged shim must deadlock-detect");
        assert_eq!(err, ThrashCourtError::DeadlockBound { bound: wedge_bound });
        let control = ControlResult::MutantCaught {
            mutant_id: "wedged-shim".to_string(),
            detection_channel: "watchdog recv_timeout".to_string(),
        };

        // Real tier-1 court run (completion + typed-refusal + conservation
        // legs asserted inside run_court; panics on any violation).
        let summary = run_court(ThrashConfig::small()).expect("tier-1 court run within bound");
        assert!(
            summary.max_latency_ns < MAX_TRANSITION_NS_CEILING,
            "LATENCY STALL: max transition latency {} ns",
            summary.max_latency_ns
        );

        let outcome = CourtOutcome {
            verdict: Verdict::Pass,
            metrics: vec![
                ("transitions", MetricValue::U64(TRANSITIONS_SMALL)),
                ("producers", MetricValue::U64(PRODUCERS)),
                ("consumers", MetricValue::U64(CONSUMERS as u64)),
                ("conservation_checksum", MetricValue::U64(summary.conservation_checksum)),
            ],
            scoping: ScopingRecord::partial(
                "N14 CAST-32A: 16 adversarial threads vs sequencer; bounded max latency, \
                 zero deadlock",
                "receipt minted from a real tier-1 run: 50k-transition CasRing MPMC 4x4 \
                 pipeline under 4 evictors + 4 spinners; completion, typed-refusals-only \
                 (zero SDC), and conservation legs all asserted; wedged-shim positive \
                 control deadlock-detected",
                "tier-1 small prefix of the donor 1M-transition/16-thread tier-3 court; \
                 no wall-clock latency bound asserted at this tier (measures the machine, \
                 not the subject)",
            ),
        };
        let receipt = CourtReceipt::record(
            CourtId::N14,
            Pillar::Resilience,
            "N14",
            &subject_sha(),
            outcome,
            control,
        );

        let json = receipt.to_json().expect("serialize N14 receipt");
        let back = CourtReceipt::from_json(&json).expect("receipt must parse back");
        assert_eq!(back.verdict, Verdict::Pass);
        assert!(back.positive_control.passed());
        assert_ne!(back.subject_sha, "control-subject");

        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/receipts");
        let path = dir.join("N14.json");
        if let Err(e) = std::fs::write(&path, json) {
            println!("skipping N14 receipt emission: cannot write {}: {e}", path.display());
        }
    }
}

// ---------------------------------------------------------------------------
// Court wiring
// ---------------------------------------------------------------------------

/// N14 CAST-32A multi-core interference court.
pub struct Cast32ThrashCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl Cast32ThrashCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N14;
    /// Capability pillar.
    /// Pillar-alignment law: emitter (`CourtReceipt::record`) and
    /// docs/receipts/N14.json both mint Resilience; the plan table's
    /// "Timing" was wrong for the standing record.
    pub const PILLAR: Pillar = Pillar::Resilience;
}

impl Court for Cast32ThrashCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N14 cast32a_thrash"
    }

    fn positive_control(&self) -> ControlProbe {
        n14_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        // All three legs (completion/zero-deadlock, typed-refusals-only /
        // zero SDC, conservation) are asserted inside run_court; a typed
        // DeadlockBound/CourtLost is a FAIL, not a harness hang.
        let summary = run_court(ThrashConfig::small()).ok();
        CourtOutcome {
            verdict: if summary.is_some() { Verdict::Pass } else { Verdict::Fail },
            metrics: vec![
                ("transitions", MetricValue::U64(TRANSITIONS_SMALL)),
                ("producers", MetricValue::U64(PRODUCERS)),
                ("consumers", MetricValue::U64(CONSUMERS as u64)),
                (
                    "conservation_checksum",
                    MetricValue::U64(summary.map_or(0, |s| s.conservation_checksum)),
                ),
            ],
            scoping: ScopingRecord::partial(
                "N14 CAST-32A: 16 adversarial threads vs sequencer; bounded max latency, \
                 zero deadlock",
                "tier-1 small run: 50k-transition CasRing MPMC 4x4 pipeline under 4 evictors \
                 + 4 spinners; completion, typed-refusals-only (zero SDC), and conservation \
                 legs all asserted inside run_court; wedged-shim positive control proves the \
                 watchdog load-bearing",
                "tier-1 small prefix of the donor 1M-transition/16-thread tier-3 court; \
                 no wall-clock latency bound asserted at this tier (measures the machine, \
                 not the subject)",
            ),
        }
    }
}
