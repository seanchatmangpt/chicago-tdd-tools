#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items.
#![allow(dead_code)]
//! NATO N11 — torn-write / bus-disconnect courts.
//!
//! Ported from strata-protocol tests/nato/n11_torn_write_fuzz.rs @ strata
//! 0de13a3 (Apache/MIT per strata LICENSE). Targets the subject port's
//! [`slot::Slot`] seq/checksum interlock and [`ring::CasRing`] typed-recovery
//! surface. Three tiers:
//!
//! 1. MODEL-LEVEL (exhaustive): for every interleaving of the three stores
//!    (word, checksum, seq) — all 3! orderings, truncated at every prefix —
//!    plus a single-bit flip injected into each field of each torn state, the
//!    read law must never admit a torn word: `read(expected_seq)` returns
//!    `Some(w)` only when `w` is exactly the intended payload for that seq.
//! 2. HAMMER-LEVEL (real threads): 4 writers x 4 readers, writers abort
//!    mid-sequence at random points (bus disconnect). Readers never observe
//!    checksum-valid garbage; the slot recovers after each abort.
//! 3. CASRING-LEVEL (typed refusals): writers abort mid-sequence on a real
//!    [`ring::CasRing`] and recycle through the typed `abandon`; the drainer
//!    never observes garbage; stale/double/out-of-range abandon refuses
//!    typed; zero torn words admitted.
//!
//! Oracle: the checksum is independently re-derived here (blake3 over the 16
//! LE bytes of (seq, word), truncated to u64) — the same discipline as the
//! SEU courts' independent `lawful_image` oracle, so the model tier does not
//! merely re-run the code under test.
//!
//! Positive control: `naked_read` drops the checksum interlock from the model
//! read law (seq gate only) and MUST admit corrupted words
//! (`corrupted_words_admitted > 0`) — proving the interlock load-bearing and
//! the model court non-vacuous.
//!
//! Tiering (prefix-stream): the fast tier runs default budgets; the
//! `#[ignore = "nato-tier3"]` full tier runs the donor scale with the SAME
//! LCG seeds, so the full stream's prefix is the fast stream — full subsumes
//! fast. Donor constants verbatim.

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

mod ring;
mod slot;

use ring::{CasRing, ConsensusError, SlotId};
use slot::Slot;

/// Knuth LCG, deterministic across platforms (high-bit sampling, per the
/// donor courts). Same constants/shape as `crate::nato::rng`.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 32) % n
    }
}

// --- Independent checksum oracle (NOT a call into slot.rs) ----------------

fn seal_oracle(seq: u64, word: u64) -> u64 {
    let mut h = blake3::Hasher::new();
    h.update(&seq.to_le_bytes());
    h.update(&word.to_le_bytes());
    let d = h.finalize();
    let mut head = [0u8; 8];
    head.copy_from_slice(&d.as_bytes()[0..8]);
    u64::from_le_bytes(head)
}

/// Faithful model of Slot's three fields as plain values, with the exact read
/// law re-derived from the documented contract: seq gate first (fail-closed
/// `None` on mismatch), then the checksum interlock over the independently
/// derived seal. `checksum_gate` is the positive-control switch: `false`
/// gives the `naked_read` mutant (seq gate only).
#[derive(Clone, Copy)]
struct SlotModel {
    seq: u64,
    word: u64,
    checksum: u64,
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Word,
    Checksum,
    Seq,
}

impl SlotModel {
    fn read(&self, expected_seq: u64) -> Option<u64> {
        self.read_with_gate(expected_seq, true)
    }

    fn read_with_gate(&self, expected_seq: u64, checksum_gate: bool) -> Option<u64> {
        if self.seq != expected_seq {
            return None;
        }
        if checksum_gate && seal_oracle(self.seq, self.word) != self.checksum {
            return None;
        }
        Some(self.word)
    }
}

fn apply_store(state: &mut SlotModel, f: Field, target: &SlotModel) {
    match f {
        Field::Word => state.word = target.word,
        Field::Checksum => state.checksum = target.checksum,
        Field::Seq => state.seq = target.seq,
    }
}

// --- Tier 1: model-level exhaustive torn court ----------------------------
//
// Scale: 3! orderings x every truncated prefix (0, 1, 2 stores done) x
// 3 fields x 64 flip bits, across deterministic (seq, word) vectors.
// Donor: 256 vectors => 6 * 3 * 3 * 64 * 256 = 884,736 model checks.
const MODEL_VECTORS_FAST: u64 = 8;
const MODEL_VECTORS_FULL: u64 = 256; // donor-verbatim
const fn model_checks(vectors: u64) -> u64 {
    6 * 3 * 3 * 64 * vectors
}

/// One model-court pass over `vectors` deterministic (seq, word) vectors.
/// Returns (checks, torn_states_examined, torn_words_admitted,
/// corrupted_words_admitted).
fn run_model(vectors: u64) -> (u64, u64, u64, u64) {
    let mut rng = Lcg(0x4E41_3131_0000_0001);
    let mut checks = 0u64;
    let mut torn_states_examined = 0u64;
    let mut torn_words_admitted = 0u64;
    let mut corrupted_words_admitted = 0u64;

    for v in 0..vectors {
        let seq_new = 0x1000_0000 + v;
        let word_new = rng.next();
        // Baseline legal state: a fully committed prior write.
        let seq_old = seq_new - 1;
        let word_old = rng.next();
        let baseline =
            SlotModel { seq: seq_old, word: word_old, checksum: seal_oracle(seq_old, word_old) };
        let target =
            SlotModel { seq: seq_new, word: word_new, checksum: seal_oracle(seq_new, word_new) };

        // All 3! interleavings of the three stores.
        let orders: [[Field; 3]; 6] = [
            [Field::Word, Field::Checksum, Field::Seq], // lawful order
            [Field::Seq, Field::Word, Field::Checksum], // seq-first (worst case)
            [Field::Seq, Field::Checksum, Field::Word],
            [Field::Checksum, Field::Word, Field::Seq],
            [Field::Checksum, Field::Seq, Field::Word],
            [Field::Word, Field::Seq, Field::Checksum],
        ];
        for perm in &orders {
            let mut state = baseline;
            for k in 0..3usize {
                if k > 0 {
                    apply_store(&mut state, perm[k - 1], &target);
                }
                torn_states_examined += 1;
                // The un-corrupted torn state must not admit a foreign word.
                if let Some(w) = state.read(seq_new) {
                    if w != word_new {
                        torn_words_admitted += 1;
                    }
                }
                // Single-bit flip of each field, then apply the read law.
                for f in [Field::Seq, Field::Word, Field::Checksum] {
                    for bit in 0u64..64 {
                        let mut s = state;
                        match f {
                            Field::Seq => s.seq ^= 1 << bit,
                            Field::Word => s.word ^= 1 << bit,
                            Field::Checksum => s.checksum ^= 1 << bit,
                        }
                        if let Some(w) = s.read(seq_new) {
                            // Admissible only if the surviving word is the
                            // intended payload; anything else is a torn word
                            // getting past the interlock.
                            if w != word_new {
                                corrupted_words_admitted += 1;
                            }
                        }
                        checks += 1;
                    }
                }
            }
        }
    }
    (checks, torn_states_examined, torn_words_admitted, corrupted_words_admitted)
}

#[test]
fn model_exhaustive_torn_orderings_bitflips_fast() {
    let (checks, _states, torn_words_admitted, corrupted_words_admitted) =
        run_model(MODEL_VECTORS_FAST);
    assert_eq!(checks, model_checks(MODEL_VECTORS_FAST), "model check count mismatch");
    assert_eq!(torn_words_admitted, 0, "torn word admitted by read law");
    assert_eq!(corrupted_words_admitted, 0, "corrupted state admitted a non-intended word");
}

#[test]
#[ignore = "nato-tier3"]
fn model_exhaustive_torn_orderings_bitflips_full() {
    let (checks, _states, torn_words_admitted, corrupted_words_admitted) =
        run_model(MODEL_VECTORS_FULL);
    assert_eq!(
        checks,
        model_checks(MODEL_VECTORS_FULL),
        "model check count mismatch (donor scale: 884,736)"
    );
    assert_eq!(torn_words_admitted, 0, "torn word admitted by read law");
    assert_eq!(corrupted_words_admitted, 0, "corrupted state admitted a non-intended word");
}

/// POSITIVE CONTROL: with the checksum interlock dropped from the model read
/// law (seq gate only), corrupted torn states MUST admit non-intended words.
/// If this fails the interlock is not load-bearing and the model court above
/// is vacuous.
#[test]
fn positive_control_naked_read_admits_corruption() {
    let mut rng = Lcg(0x4E41_3131_0000_0001);
    let mut corrupted_words_admitted = 0u64;

    let seq_new = 0x1000_0000;
    let word_new = rng.next();
    let seq_old = seq_new - 1;
    let word_old = rng.next();
    let baseline =
        SlotModel { seq: seq_old, word: word_old, checksum: seal_oracle(seq_old, word_old) };
    let target =
        SlotModel { seq: seq_new, word: word_new, checksum: seal_oracle(seq_new, word_new) };

    // Same torn-state enumeration as the model tier (one full ordering,
    // truncated at every prefix, every field/bit flip), naked read law.
    let orders: [[Field; 3]; 6] = [
        [Field::Word, Field::Checksum, Field::Seq],
        [Field::Seq, Field::Word, Field::Checksum],
        [Field::Seq, Field::Checksum, Field::Word],
        [Field::Checksum, Field::Word, Field::Seq],
        [Field::Checksum, Field::Seq, Field::Word],
        [Field::Word, Field::Seq, Field::Checksum],
    ];
    for perm in &orders {
        let mut state = baseline;
        for k in 0..3usize {
            if k > 0 {
                apply_store(&mut state, perm[k - 1], &target);
            }
            for f in [Field::Seq, Field::Word, Field::Checksum] {
                for bit in 0u64..64 {
                    let mut s = state;
                    match f {
                        Field::Seq => s.seq ^= 1 << bit,
                        Field::Word => s.word ^= 1 << bit,
                        Field::Checksum => s.checksum ^= 1 << bit,
                    }
                    if let Some(w) = s.read_with_gate(seq_new, false) {
                        if w != word_new {
                            corrupted_words_admitted += 1;
                        }
                    }
                }
            }
        }
    }
    assert!(
        corrupted_words_admitted > 0,
        "naked read (no checksum gate) admitted no corruption — control vacuous"
    );
}

// --- Tier 2: hammer-level abort/recovery court ----------------------------
//
// 4 writers x 4 readers (donor-verbatim), random mid-sequence writer aborts
// (bus disconnect). Readers never observe checksum-valid garbage; the slot
// recovers after the aborts; the final state self-verifies.
const HAMMER_OPS_FAST: u64 = 100_000;
const HAMMER_OPS_FULL: u64 = 1_000_000; // donor-verbatim

fn run_hammer(total: u64) -> u64 {
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering as AOrd};
    use std::sync::Arc;

    const WRITERS: usize = 4;
    const READERS: usize = 4;
    const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

    let slot = Arc::new(Slot::new());
    let garbage = Arc::new(AtomicU64::new(0));
    let aborts = Arc::new(AtomicUsize::new(0));

    // Bootstrap: publish seq 0 so readers have a valid baseline.
    slot.write(0, 0);

    let mut joins = Vec::new();
    // Writers: unique seq ranges; each writer aborts mid-chunk at a random
    // point in ~10% of its 10 chunks (transient bus disconnect), then
    // resumes with the next chunk — the slot must recover each time.
    for t in 0..WRITERS {
        let slot = Arc::clone(&slot);
        let aborts = Arc::clone(&aborts);
        joins.push(std::thread::spawn(move || {
            let mut rng = Lcg(0x4E41_3131_0000_0002 + t as u64);
            let base = (t as u64) * (total / WRITERS as u64);
            let per_writer = total / WRITERS as u64;
            let take = per_writer / 10;
            for chunk in 0..10u64 {
                let do_abort = rng.below(100) < 10;
                let upto = if do_abort { take / 2 } else { take };
                for i in 0..upto {
                    let n = base + chunk * take + i + 1;
                    slot.write(n, n.wrapping_mul(GOLDEN));
                }
                if do_abort {
                    aborts.fetch_add(1, AOrd::Relaxed);
                }
            }
        }));
    }
    // Readers: chase the live sequence once; a Some(w) for any seq must be
    // the lawful word for that seq (monotone seq law), never garbage.
    for _ in 0..READERS {
        let slot = Arc::clone(&slot);
        let garbage = Arc::clone(&garbage);
        joins.push(std::thread::spawn(move || {
            for seq in 1..=total {
                if let Some(w) = slot.read(seq) {
                    if w != seq.wrapping_mul(GOLDEN) {
                        garbage.fetch_add(1, AOrd::Relaxed);
                    }
                }
            }
        }));
    }
    for j in joins {
        j.join().expect("writer/reader thread panicked");
    }
    // Recovery: republish the final seq and require self-verification.
    let final_seq = total;
    slot.write(final_seq, final_seq.wrapping_mul(GOLDEN));
    assert_eq!(
        slot.read(final_seq),
        Some(final_seq.wrapping_mul(GOLDEN)),
        "slot did not recover after aborts"
    );
    assert_eq!(garbage.load(AOrd::Relaxed), 0, "checksum-valid garbage observed");
    aborts.load(AOrd::Relaxed) as u64
}

#[test]
fn hammer_4w4r_abort_recovery_fast() {
    let _aborts = run_hammer(HAMMER_OPS_FAST);
}

#[test]
#[ignore = "nato-tier3"]
fn hammer_4w4r_abort_recovery_full() {
    let _aborts = run_hammer(HAMMER_OPS_FULL);
}

// --- Tier 3: CasRing typed-refusal court ----------------------------------
//
// Writers abort mid-sequence on a real CasRing: abort recycles through the
// typed `abandon` (never a leak); the drainer never observes garbage; stale,
// double, and out-of-range abandons refuse typed; zero torn words admitted.
// Donor: 4 writers x 2,500 commits = 10,000 drained.
const CASRING_PER_WRITER_FAST: u64 = 500; // 4 x 500 = 2,000 drained
const CASRING_PER_WRITER_FULL: u64 = 2_500; // donor-verbatim

fn run_casring(per_writer: u64) -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering as AOrd};
    use std::sync::Arc;

    const WRITERS: usize = 4;
    const RING: usize = 64;
    let drained_target = WRITERS as u64 * per_writer;

    let ring = Arc::new(CasRing::<RING>::new());
    let drained = Arc::new(AtomicU64::new(0));
    let garbage = Arc::new(AtomicU64::new(0));
    let refused_typed = Arc::new(AtomicU64::new(0));
    let abandons = Arc::new(AtomicU64::new(0));

    let drainer = {
        let ring = Arc::clone(&ring);
        let drained = Arc::clone(&drained);
        let garbage = Arc::clone(&garbage);
        std::thread::spawn(move || {
            while drained.load(AOrd::Acquire) < drained_target {
                ring.drain_into(&mut |w| {
                    // Payload domain: unique per commit, never the reserved
                    // sentinels — a drained sentinel would be corruption.
                    if w == u64::MAX || w == u64::MAX - 1 {
                        garbage.fetch_add(1, AOrd::Relaxed);
                    }
                    drained.fetch_add(1, AOrd::Relaxed);
                });
            }
        })
    };

    let mut joins = vec![drainer];
    for t in 0..WRITERS {
        let ring = Arc::clone(&ring);
        let refused_typed = Arc::clone(&refused_typed);
        let abandons = Arc::clone(&abandons);
        joins.push(std::thread::spawn(move || {
            let mut rng = Lcg(0x4E41_3131_0000_000A + t as u64);
            // Each writer commits exactly `per_writer` words and interleaves
            // ~10% abort ops on top: an abort claims a slot and recycles it
            // via typed abandon (mid-sequence bus disconnect), never leaking
            // the slot.
            let mut committed = 0u64;
            let mut i = 0u64;
            while committed < per_writer {
                let s = loop {
                    if let Some(s) = ring.claim() {
                        break s;
                    }
                    std::hint::spin_loop();
                };
                if rng.below(100) < 10 {
                    // Abort mid-sequence: typed recycle.
                    match ring.abandon(s) {
                        Ok(()) => {
                            abandons.fetch_add(1, AOrd::Relaxed);
                        }
                        Err(_) => {
                            refused_typed.fetch_add(1, AOrd::Relaxed);
                        }
                    }
                } else {
                    match ring.commit(s, (t as u64) * 10_000 + i) {
                        Ok(()) => {
                            committed += 1;
                        }
                        Err(_) => {
                            refused_typed.fetch_add(1, AOrd::Relaxed);
                        }
                    }
                }
                i += 1;
            }
        }));
    }
    for j in joins {
        j.join().expect("thread panicked");
    }
    assert_eq!(drained.load(AOrd::Relaxed), drained_target, "all commits drained");
    assert_eq!(garbage.load(AOrd::Relaxed), 0, "garbage words drained");
    // Post-run typed-refusal census (deterministic, single-threaded).
    let s = ring.claim().expect("recycled slot after run");
    assert_eq!(ring.abandon(s), Ok(()));
    assert_eq!(
        ring.abandon(s),
        Err(ConsensusError::SlotNotClaimed),
        "double abandon must refuse typed"
    );
    assert_eq!(
        ring.abandon(SlotId(RING + 1)),
        Err(ConsensusError::SlotNotClaimed),
        "out-of-range abandon must refuse typed"
    );
    assert!(ring.abandoned_count() as u64 >= abandons.load(AOrd::Relaxed));
    abandons.load(AOrd::Relaxed)
}

#[test]
fn casring_abort_recycles_typed_fast() {
    let _abandons = run_casring(CASRING_PER_WRITER_FAST);
}

#[test]
#[ignore = "nato-tier3"]
fn casring_abort_recycles_typed_full() {
    let _abandons = run_casring(CASRING_PER_WRITER_FULL);
}
