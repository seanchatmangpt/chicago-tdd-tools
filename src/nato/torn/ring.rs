#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items.
#![allow(dead_code)]
//! Lockless CAS ring over AtomicU64 slots. Every slot padded to its own
//! 64-byte cacheline.
//!
//! Faithful subject port of strata-protocol src/consensus/cas_ring.rs @
//! strata 0de13a3 (Apache/MIT per strata LICENSE). Also carries the typed
//! [`ConsensusError`] (zero diagnostic entropy: the variant is the whole
//! message).
//!
//! Design: [`CasRing<const N: usize>`] holds `N` [`CachePadded`] slots, each
//! initially EMPTY (`u64::MAX`). `claim()` CASes a slot from EMPTY to
//! CLAIMED — the compare_exchange is the only primitive on the claim fast
//! path; contention is expressed as CAS failure (retry at the next slot),
//! never a branch on protocol state. `commit(slot, word)` publishes the
//! payload and transitions the slot to committed via one compare_exchange
//! (Release). `drain_into` takes committed words with one
//! `compare_exchange(committed -> AcqRel EMPTY)`, recycling slots.
//!
//! Memory ordering (explicit):
//! - `claim`: AcqRel on the success CAS. Release publishes the EMPTY->CLAIMED
//!   transition; Acquire pairs with the consumer's drain Release so a recycled
//!   slot's teardown is visible before re-claim. Failure path loads Acquire.
//! - `commit`: Release on the CLAIMED->payload CAS. The payload store is
//!   ordered before the slot-state transition that publishes it; the
//!   consumer's Acquire on the same slot observes the full payload. A refused
//!   commit leaves the slot byte-identical (compare_exchange, never swap).
//! - `drain_into`: AcqRel on the take-CAS. Acquire synchronizes with the
//!   producer's Release commit (payload visible), Release publishes the EMPTY
//!   reset back to producers for safe reuse.
//!
//! Padding: `#[repr(align(64))]` on the struct alone is insufficient — it
//! aligns only the struct start, leaving interior slots sharing cachelines.
//! Each slot is individually wrapped in [`CachePadded`] so concurrent claim/
//! commit on distinct slots never false-shares. The rotating claim cursor is
//! padded too.
//!
//! Allocation: none after construction — fixed array, no Vec/Box. Capacity
//! must be a power of two (compile-time refused, fail-closed). Payload words
//! must avoid the internal sentinels `u64::MAX` and `u64::MAX - 1` (typed
//! domain contract, not a runtime check — zero entropy).

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

use core::sync::atomic::{AtomicU64, Ordering};

/// Sentinel: slot empty (unclaimed, or drained and recycled).
const EMPTY: u64 = u64::MAX;
/// Sentinel: slot claimed, payload not yet committed.
const CLAIMED: u64 = EMPTY - 1;

/// One slot, alone on its own 64-byte cacheline.
#[repr(align(64))]
#[derive(Debug)]
struct CachePadded(AtomicU64);

impl CachePadded {
    const fn new(v: u64) -> Self {
        Self(AtomicU64::new(v))
    }
}

/// A slot index handed out by [`CasRing::claim`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotId(pub usize);

/// Typed consensus failures. Zero diagnostic entropy: the variant is the whole
/// message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusError {
    /// `claim()` found no free slot after one full revolution.
    RingFull,
    /// `commit()`/`abandon()` targeted a slot not in finite CLAIMED state
    /// (never claimed, already drained, or double-commit).
    SlotNotClaimed,
}

impl core::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            ConsensusError::RingFull => "ring full",
            ConsensusError::SlotNotClaimed => "slot not claimed",
        };
        f.write_str(s)
    }
}

impl std::error::Error for ConsensusError {}

/// 64-byte-aligned-per-slot lockless ring of `N` AtomicU64 slots.
#[repr(align(64))]
pub struct CasRing<const N: usize> {
    /// Rotating claim cursor: spreads contention across slots (padded — it is
    /// written by every claim).
    cursor: CachePadded,
    slots: [CachePadded; N],
    /// Lifetime count of explicit [`Self::abandon`] calls.
    abandoned: CachePadded,
}

impl<const N: usize> CasRing<N> {
    /// Compile-time refusal of non-power-of-two capacity (fail-closed).
    const POWER_OF_TWO: () =
        assert!(N.is_power_of_two(), "CasRing capacity must be a power of two");

    /// Construct with all slots EMPTY. No heap allocation.
    pub fn new() -> Self {
        const { Self::POWER_OF_TWO };
        Self {
            cursor: CachePadded::new(0),
            slots: core::array::from_fn(|_| CachePadded::new(EMPTY)),
            abandoned: CachePadded::new(0),
        }
    }

    /// Claim a slot: CAS EMPTY -> CLAIMED. The CAS is the only primitive on
    /// the fast path; the retry loop is contention handling, not protocol
    /// branching. Returns `None` (RingFull domain) iff one full revolution
    /// finds no EMPTY slot.
    pub fn claim(&self) -> Option<SlotId> {
        let start = self.cursor.0.fetch_add(1, Ordering::Relaxed);
        let mut i = start;
        loop {
            let idx = (i & (N as u64 - 1)) as usize;
            // idx < N by mask; the indexing bounds check is in-bounds by
            // construction (idx = i & (N-1)), and the crate denies unsafe
            // usage, so the checked form is used here.
            let slot = &self.slots[idx];
            if slot
                .0
                .compare_exchange(EMPTY, CLAIMED, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Some(SlotId(idx));
            }
            i = i.wrapping_add(1);
            if i.wrapping_sub(start) == N as u64 {
                return None;
            }
        }
    }

    /// Commit a payload word into a claimed slot.
    ///
    /// Release: the payload store is ordered before the slot transition that
    /// publishes it to the consumer. Fails closed with
    /// [`ConsensusError::SlotNotClaimed`] if the slot was not in CLAIMED state
    /// (never claimed, already drained, or double-commit) — and leaves the
    /// slot byte-identical on refusal.
    ///
    /// Trust model (slot stealing): `commit` verifies slot *state* (CLAIMED),
    /// not *ownership*. A `SlotId` is an unforgeable-in-practice capability
    /// only within a single trusted process — a thread that hands its SlotId
    /// across threads and the original owner both commit, exactly one wins
    /// the CAS and the other refuses typed. A recycled-slot collision (stale
    /// SlotId after drain) fails closed iff the new owner already committed,
    /// but can succeed otherwise; cross-thread SlotId sharing is therefore a
    /// documented contract violation, not a runtime-checked one. 64-bit slots
    /// carry no spare bits for an owner tag (see the slot.rs seq/checksum
    /// interlock for the slot-level analogue).
    pub fn commit(&self, slot: SlotId, word: u64) -> Result<(), ConsensusError> {
        // u64::MAX (EMPTY) is reserved as a drain sentinel — committing it
        // would silently recycle the slot and lose the word. Documented
        // domain reserve; debug builds refuse loudly, release builds treat it
        // as any other payload word (zero-entropy contract, not a runtime
        // branch on the hot path).
        debug_assert!(word != EMPTY, "u64::MAX reserved as drain sentinel");
        let s = self.slots.get(slot.0).ok_or(ConsensusError::SlotNotClaimed)?;
        // compare_exchange, not swap: a refused commit must leave the slot
        // byte-identical (no phantom payload in an unclaimed slot).
        match s.0.compare_exchange(CLAIMED, word, Ordering::Release, Ordering::Acquire) {
            Ok(_) => Ok(()),
            Err(_) => Err(ConsensusError::SlotNotClaimed),
        }
    }

    /// Abandon a claimed slot: recycle it without committing a word.
    ///
    /// CLAIMED -> EMPTY via CAS, AcqRel for the same pairing as drain's
    /// reset. Counts into the lifetime [`Self::abandoned_count`]. This is the
    /// honest leak-recovery signal the raw state layout cannot express on its
    /// own: a dropped SlotId without `abandon` pins the slot forever
    /// (observable as `claim() == None` with reduced capacity); an explicit
    /// `abandon` is counted, typed, and recycles the slot immediately.
    pub fn abandon(&self, slot: SlotId) -> Result<(), ConsensusError> {
        let s = self.slots.get(slot.0).ok_or(ConsensusError::SlotNotClaimed)?;
        match s.0.compare_exchange(CLAIMED, EMPTY, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {
                self.abandoned.0.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(_) => Err(ConsensusError::SlotNotClaimed),
        }
    }

    /// Drain every committed word into `f`, recycling slots for reuse.
    ///
    /// AcqRel on the take-CAS: Acquire pairs with each producer's Release
    /// commit (payload observed in full), Release publishes the EMPTY reset
    /// that makes the slot safely re-claimable.
    ///
    /// Delivery contract (drain ABA): the take-CAS compares the *payload
    /// value*, not slot identity. With a single drainer, delivery is
    /// exactly-once per committed word. With CONCURRENT drainers, a word
    /// whose value is re-committed into another slot while drains race can be
    /// observed more than once across drainers (value-based ABA) —
    /// at-least-once per value. Exactly-once across concurrent drainers
    /// requires unique per-commit payload words (e.g. producer-sequence
    /// tags); witnessed by court below.
    pub fn drain_into(&self, f: &mut impl FnMut(u64)) {
        for slot in self.slots.iter() {
            loop {
                let cur = slot.0.load(Ordering::Acquire);
                if cur == EMPTY || cur == CLAIMED {
                    break;
                }
                match slot.0.compare_exchange(cur, EMPTY, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(word) => f(word),
                    Err(_) => continue, // raced with a commit; retry same slot
                }
            }
        }
    }

    /// Number of slots currently CLAIMED (claimed but not yet committed).
    pub fn claimed_count(&self) -> usize {
        self.slots.iter().filter(|s| s.0.load(Ordering::Acquire) == CLAIMED).count()
    }

    /// Total capacity `N`.
    pub fn capacity(&self) -> usize {
        N
    }

    /// Lifetime count of explicit [`Self::abandon`] calls.
    ///
    /// Distinct signal: `claimed_count()` is instantaneous CLAIMED population
    /// (commits and abandons both leave it); `abandoned_count()` is cumulative
    /// explicit recovery. A dropped SlotId with no `abandon` call is invisible
    /// to both and shows up only as permanently reduced claimable capacity —
    /// the API surfaces recovery through the typed `abandon`, not through a
    /// counter indistinguishable from `claimed_count`.
    pub fn abandoned_count(&self) -> usize {
        self.abandoned.0.load(Ordering::Relaxed) as usize
    }
}

impl<const N: usize> Default for CasRing<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPSC 100k handoff: one producer, one consumer, real thread handoff.
    /// Chicago court: real threads, asserts on final state, no mocks.
    #[test]
    fn spsc_100k_handoff() {
        const TOTAL: u64 = 100_000;
        let ring = std::sync::Arc::new(CasRing::<1024>::new());
        let consumer_ring = std::sync::Arc::clone(&ring);
        let consumer = std::thread::spawn(move || {
            let ring = &*consumer_ring;
            let mut seen: Vec<u64> = Vec::with_capacity(TOTAL as usize);
            while seen.len() < TOTAL as usize {
                ring.drain_into(&mut |w| seen.push(w));
            }
            seen
        });
        for i in 0..TOTAL {
            let slot = loop {
                if let Some(s) = ring.claim() {
                    break s;
                }
                std::thread::yield_now();
            };
            ring.commit(slot, i).expect("commit after claim");
        }
        let mut seen = consumer.join().expect("consumer thread");
        seen.sort_unstable();
        assert_eq!(seen.len() as u64, TOTAL);
        for (i, v) in seen.iter().enumerate() {
            assert_eq!(*v, i as u64, "every committed word exactly once");
        }
    }

    /// MPMC 4x4: four producers x 100k claims each under real contention;
    /// four consumers drain. Court asserts: exactly 400k words, each producer's
    /// sequence present exactly once — no double-claim, no lost word.
    #[test]
    fn mpmc_4x4_no_double_claim() {
        let total = mpmc_court_on(&std::sync::Arc::new(CasRing::<4096>::new()));
        assert_eq!(total, 400_000);
    }

    /// MPMC court driver shared by the padded-ring tests. Returns the number
    /// of words drained (400_000 expected).
    fn mpmc_court_on(ring: &std::sync::Arc<CasRing<4096>>) -> usize {
        const PRODUCERS: usize = 4;
        const PER: u64 = 100_000;
        const TOTAL: u64 = (PRODUCERS as u64) * PER;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc;

        let ring = Arc::clone(ring);
        let drained = Arc::new(AtomicU64::new(0));

        let consumers: Vec<_> = (0..4)
            .map(|_| {
                let ring = Arc::clone(&ring);
                let drained = Arc::clone(&drained);
                std::thread::spawn(move || {
                    let mut local: Vec<u64> = Vec::new();
                    while drained.load(Ordering::Acquire) < TOTAL {
                        ring.drain_into(&mut |w| {
                            local.push(w);
                            drained.fetch_add(1, Ordering::Relaxed);
                        });
                    }
                    local
                })
            })
            .collect();

        let producers: Vec<_> = (0..PRODUCERS)
            .map(|p| {
                let ring = Arc::clone(&ring);
                std::thread::spawn(move || {
                    for i in 0..PER {
                        let slot = loop {
                            if let Some(s) = ring.claim() {
                                break s;
                            }
                            std::hint::spin_loop();
                        };
                        ring.commit(slot, (p as u64) * 1_000_000 + i).expect("commit after claim");
                    }
                })
            })
            .collect();

        for p in producers {
            p.join().expect("producer thread");
        }
        let mut total: Vec<u64> = Vec::new();
        for t in consumers {
            total.extend(t.join().expect("consumer thread"));
        }
        total.sort_unstable();
        // Disjoint per-producer ranges, sorted: producer p's words land at
        // exactly one index each — no double-claim, no loss, no duplication.
        for (idx, v) in total.iter().enumerate() {
            let expect = (idx as u64 / PER) * 1_000_000 + (idx as u64 % PER);
            assert_eq!(*v, expect);
        }
        total.len()
    }

    /// RingFull domain: capacity 4, claim all four, next claim is full; commit
    /// + drain recycles and claims succeed again.
    #[test]
    fn ring_full_at_capacity() {
        let ring = CasRing::<4>::new();
        let mut slots = Vec::new();
        for _ in 0..4 {
            slots.push(ring.claim().expect("slot within capacity"));
        }
        assert_eq!(ring.claim(), None, "no slot after one full revolution");
        assert_eq!(ring.capacity(), 4);
        for (i, s) in slots.iter().enumerate() {
            ring.commit(*s, i as u64).expect("commit after claim");
        }
        ring.drain_into(&mut |_| {});
        assert!(ring.claim().is_some(), "slots recycled after drain");
    }

    /// commit-after-claim ordering: unclaimed commit refuses, claim-then-commit
    /// succeeds, double-commit refuses, drained slot recycles.
    #[test]
    fn commit_after_claim_ordering() {
        let ring = CasRing::<8>::new();
        assert_eq!(ring.commit(SlotId(0), 42), Err(ConsensusError::SlotNotClaimed));
        let s = ring.claim().expect("slot");
        ring.commit(s, 42).expect("commit after claim");
        assert_eq!(
            ring.commit(s, 43),
            Err(ConsensusError::SlotNotClaimed),
            "double-commit refuses fail-closed"
        );
        // Refused commits left no phantom words: exactly one committed word.
        let mut got = Vec::new();
        ring.drain_into(&mut |w| got.push(w));
        assert_eq!(got, vec![42]);
        assert_eq!(ring.claimed_count(), 0);
        // Recycled: a fresh claim succeeds (cursor rotation means it may be
        // any slot, not necessarily index 0 — assert liveness, not position).
        assert!(ring.claim().is_some());
    }

    /// Leak court: a dropped SlotId pins its slot (reduced capacity); explicit
    /// `abandon` recycles it and is counted by `abandoned_count` — a signal
    /// distinct from `claimed_count`.
    #[test]
    fn dropped_slot_pins_capacity_and_abandon_recovers_typed() {
        let ring = CasRing::<4>::new();
        let leaked = ring.claim().expect("slot 1");
        let s2 = ring.claim().expect("slot 2");
        ring.commit(s2, 7).expect("commit");
        drop(leaked); // leak: no commit, no abandon
        ring.drain_into(&mut |_| {}); // takes 7; leaked slot stays CLAIMED
        assert_eq!(ring.claimed_count(), 1, "leaked slot pinned in CLAIMED");
        assert_eq!(ring.abandoned_count(), 0);
        // Capacity reduced by exactly the leak: 3 more claims, then RingFull.
        let mut spare: Vec<SlotId> = Vec::new();
        while let Some(s) = ring.claim() {
            spare.push(s);
        }
        assert_eq!(spare.len(), 3, "4 slots minus the 1 leak");
        assert_eq!(ring.claim(), None, "RingFull after leak");
        // Typed recovery: abandon refuses a stale/unclaimed slot...
        assert_eq!(ring.abandon(SlotId(ring.capacity())), Err(ConsensusError::SlotNotClaimed));
        // Release the spares so the recycle liveness check below is clean.
        for s in &spare {
            ring.abandon(*s).expect("abandon spare");
        }
        assert_eq!(ring.abandoned_count(), 3, "distinct lifetime signal");
        // ...and the recovery path is claim -> abandon, recycling immediately.
        let s = ring.claim().expect("recycled slot");
        ring.abandon(s).expect("abandon claimed slot");
        assert_eq!(ring.abandoned_count(), 4, "distinct lifetime signal");
        assert_eq!(ring.claimed_count(), 1, "abandon leaves CLAIMED set");
        assert!(ring.claim().is_some(), "abandoned slot recycled");
        // Double abandon refuses: slot no longer CLAIMED.
        assert_eq!(ring.abandon(s), Err(ConsensusError::SlotNotClaimed));
    }

    /// Drain ABA, deterministic court: a SINGLE drainer delivers duplicate
    /// payload values exactly-once each — value comparison cannot lose words
    /// when only one taker races.
    #[test]
    fn duplicate_payload_single_drainer_exactly_once() {
        let ring = CasRing::<16>::new();
        for _ in 0..8 {
            let s = ring.claim().expect("slot");
            ring.commit(s, 42).expect("commit");
        }
        let mut count = 0u32;
        ring.drain_into(&mut |w| {
            assert_eq!(w, 42);
            count += 1;
        });
        assert_eq!(count, 8, "exactly-once per committed word, dupes alike");
    }

    /// Drain ABA, concurrent court: with duplicate payload values and racing
    /// drainers, delivery is at-least-once per value. Asserts the documented
    /// lower bound.
    #[test]
    fn concurrent_drainers_duplicate_payload_at_least_once() {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc;
        let ring = Arc::new(CasRing::<1024>::new());
        // Commit 1000 words, all the same value 7.
        for _ in 0..1000 {
            let s = ring.claim().expect("slot");
            ring.commit(s, 7).expect("commit");
        }
        let total = Arc::new(AtomicU64::new(0));
        let drainers: Vec<_> = (0..4)
            .map(|_| {
                let ring = Arc::clone(&ring);
                let total = Arc::clone(&total);
                std::thread::spawn(move || {
                    while total.load(Ordering::Acquire) < 1000 {
                        ring.drain_into(&mut |_| {
                            total.fetch_add(1, Ordering::Relaxed);
                        });
                    }
                })
            })
            .collect();
        for d in drainers {
            d.join().expect("drainer");
        }
        assert!(total.load(Ordering::Acquire) >= 1000, "at-least-once delivery violated");
    }

    /// Sentinel reserve contract: a committed word equal to CLAIMED
    /// (u64::MAX-1) is domain-reserved; drain treats it as uncommitted.
    #[test]
    fn sentinel_words_are_domain_reserved() {
        let ring = CasRing::<4>::new();
        let s = ring.claim().expect("slot");
        ring.commit(s, CLAIMED).expect("stores, but domain-reserved");
        let mut got = Vec::new();
        ring.drain_into(&mut |w| got.push(w));
        assert_eq!(got, Vec::<u64>::new(), "CLAIMED-valued word is reserved");
        assert_eq!(EMPTY, u64::MAX);
    }
}
