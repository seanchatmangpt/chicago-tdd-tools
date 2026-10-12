#![cfg(all(feature = "wasm-guest", feature = "ocel-ingest"))]
#![allow(unsafe_code)]
// JUSTIFICATION: ABI boundary writes raw guest-memory pointers; # Safety contract documented at each fn
//! WASM guest OCEL telemetry export (PRD §3: "records every internal state
//! transition as an IEEE OCEL 2.0 causal event graph").
//!
//! This lane converts a replayed conformance trajectory — a session replay
//! summary plus the per-step [`TrajectoryEvent`] slice — into packed OCEL64
//! words and a BLAKE3 digest. It owns NO packing law of its own: every word
//! is minted through the landed `crate::observability::ocel::ocel64_bridge`
//! [`pack`] primitive, so this module inherits the bridge's word layout
//! (activity `[15:0]`, src `[31:16]`, tgt `[47:32]`, qualifier `[55:48]`,
//! tick `[63:56]`) and its lossy-by-design disclosures verbatim:
//!
//! * the 8-bit tick field wraps at 256; a trajectory spanning more than one
//!   256-tick era is caller-checkpoint duty (see the bridge's era-wrap
//!   disclosure), not something this lane saturates or hides;
//! * payloads beyond the fixed word fields (markings here are carried as
//!   16-bit limbs in continuation words, see [`emit_trajectory`]) are a
//!   fixed-layout tier choice, declared up front, not a silent drop.
//!
//! SIBLING CONTRACTS: the yield-side status constants come from the sibling
//! `guest_yield` lane (`super::guest_yield`), which re-exports the canonical
//! ABI success code from `guest.rs`. The canonical ABI errno table itself is
//! owned by `guest.rs` (`ERR_*` 0..=16 at ABI v2). No ambient reads: every byte
//! exported comes from the caller's arguments.

use super::guest::{out_span_oob, ERR_OOB_WRITE};
use super::guest_yield::{STATUS_E_INVALID, STATUS_E_NOMEM, STATUS_OK};
use crate::observability::ocel::ocel64_bridge::pack;

// ── Fixed event layout ──────────────────────────────────────────────────────

/// One internal state transition of a replayed trajectory.
///
/// Fixed layout, fixed width: every field is sized to survive the packed
/// OCEL64 tier unchanged (no narrowing loss on pack — see [`emit_trajectory`]
/// for the word-level placement of each field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrajectoryEvent {
    /// Ordinal of this transition within the replay (0-based, monotonic).
    pub step: u32,
    /// OCEL activity id (16-bit word field, 0 = reserved null).
    pub activity_id: u16,
    /// 8-bit tick at the transition; wraps at 256 (bridge era disclosure).
    pub tick: u8,
    /// Place marking before the transition (one 64-bit marking vector word).
    pub marking_before: u64,
    /// Place marking after the transition.
    pub marking_after: u64,
    /// 8-bit outcome byte (0 = nominal; nonzero = typed deviation class).
    pub outcome: u8,
}

/// Session replay summary bound into the packed stream before the events.
///
/// One word: activity id `0xFFFF` (reserved summary tag, outside the caller's
/// activity id space — ids `0x0000`..=0xFFFE stay caller-owned), src/tgt
/// limbs carry the 32-bit step count, qualifier carries the final outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrajectorySummary {
    /// Total steps replayed (the sum of per-event `step` progress is not
    /// assumed; the caller asserts its own count).
    pub steps: u32,
    /// Final outcome byte of the session (same code space as
    /// [`TrajectoryEvent::outcome`]).
    pub final_outcome: u8,
}

// ── Pack ────────────────────────────────────────────────────────────────────

/// Pack a replay summary plus event slice into OCEL64 words.
///
/// FIXED LAYOUT (documented contract, deterministic):
///
/// * word 0 — summary header:
///   `pack(0xFFFF, lo16(steps), hi16(steps), final_outcome, 0)`.
/// * then, PER EVENT, exactly 10 words in this order:
///   1. event header: `pack(activity_id, 0, 0, outcome, tick)`
///   2. step limbs:   `pack(0, lo16(step), hi16(step), 0, 0)`
///   3. `marking_before` as four 16-bit limbs, least-significant first:
///      `pack(0, limb_i, 0, 0, 0)` for `i = 0..4`
///   4. `marking_after` as four limbs, same shape.
///
/// Reuses the bridge's [`pack`] for every word — this module mints no
/// second packing law. Because the layout is positional (not separator-
/// driven like [`crate::observability::ocel::ocel64_bridge::to_ocel64`]),
/// the bridge's zero-word ambiguity rules do not arise here; the summary tag
/// `0xFFFF` and the fixed 10-word stride make the stream self-describing.
///
/// Lossy disclosures inherited verbatim from the bridge: the tick byte wraps
/// at 256 (era boundary = caller-checkpoint duty), and only the fields named
/// in [`TrajectoryEvent`] exist at this tier — no timestamps, no attributes,
/// no object names.
///
/// Determinism: pure function of the arguments; the same
/// `(summary, events)` always yields byte-identical words.
#[must_use]
// JUSTIFICATION: summary is taken by reference to keep the call sites
// symmetric with the `events` slice argument; the type is Copy so the cost
// is nil, but the signature is part of the documented API surface.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn emit_trajectory(summary: &TrajectorySummary, events: &[TrajectoryEvent]) -> Vec<u64> {
    let mut words = Vec::with_capacity(1 + events.len() * 10);
    words.push(pack(
        0xFFFF,
        (summary.steps & 0xFFFF) as u16,
        ((summary.steps >> 16) & 0xFFFF) as u16,
        summary.final_outcome,
        0,
    ));
    for ev in events {
        // 1. Event header: activity id, outcome, tick.
        words.push(pack(ev.activity_id, 0, 0, ev.outcome, ev.tick));
        // 2. Step limbs (src = low half, tgt = high half).
        words.push(pack(0, (ev.step & 0xFFFF) as u16, ((ev.step >> 16) & 0xFFFF) as u16, 0, 0));
        // 3./4. Marking limbs, least-significant 16-bit limb first.
        for marking in [ev.marking_before, ev.marking_after] {
            for i in 0..4 {
                let limb = ((marking >> (16 * i)) & 0xFFFF) as u16;
                words.push(pack(0, limb, 0, 0, 0));
            }
        }
    }
    words
}

// ── Digest ──────────────────────────────────────────────────────────────────

/// BLAKE3 digest over the packed words — the digest guest receipts bind.
///
/// Canonical byte form: each word serialized little-endian, concatenated in
/// stream order. Same words -> same digest; any single-word flip changes it.
#[must_use]
pub fn trajectory_digest(words: &[u64]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for w in words {
        hasher.update(&w.to_le_bytes());
    }
    hasher.finalize().into()
}

// ── Bounded buffer export ───────────────────────────────────────────────────

/// Export packed words into a guest-provided bounded buffer.
///
/// ABI-style: returns a status code — [`STATUS_OK`] (the canonical ABI
/// success code, `guest::OK`) on success, [`STATUS_E_NOMEM`] when `out_cap`
/// is too small for all `words.len() * 8` bytes, [`STATUS_E_INVALID`] for a
/// null or non-8-byte-aligned `out`, and the canonical `guest::ERR_OOB_WRITE`
/// (15) when the `[out, out+out_cap)` span falls at/past the installed
/// linear-memory bound (same R44 span law as `guest::ctt_session_step` /
/// `ctt_session_seal`, enforced via `guest::out_span_oob`).
///
/// Note these
/// refusal codes are
/// `guest_yield`'s yield-scheduling statuses, NOT the canonical `guest.rs`
/// ABI errno table (`ERR_*` 0..=16); their numeric values do overlap
/// `ERR_*` 2/3, but the surfaces are distinct namespaces — they must not be
/// presented as ABI errnos. NEVER truncates: on any refusal
/// the buffer is left untouched and nothing is written.
///
/// Words are serialized little-endian, matching [`trajectory_digest`]'s
/// canonical byte form, so a host that digests the exported bytes gets the
/// same digest the guest receipts bind.
///
/// # Safety
///
/// `out` must be valid for writes of `words.len() * 8` bytes when the
/// capacity check passes. The caller owns the buffer's lifetime.
pub unsafe fn write_event_words(words: &[u64], out: *mut u8, out_cap: u32) -> i32 {
    let Some(need) = words.len().checked_mul(8) else {
        return STATUS_E_NOMEM; // cannot even size the export
    };
    if need > usize::try_from(out_cap).unwrap_or(usize::MAX) {
        return STATUS_E_NOMEM;
    }
    // Null and alignment refusals happen after the capacity check so the
    // errno is deterministic regardless of argument evaluation order.
    if out.is_null() || !(out as usize).is_multiple_of(8) {
        return STATUS_E_INVALID;
    }
    // R44 linear-memory span law (same helper as `ctt_session_step` /
    // `ctt_session_seal`): a lawful-shaped out-pointer at/past the installed
    // guest linear-memory end is a typed errno, never a store trap.
    if out_span_oob(out, need) {
        return ERR_OOB_WRITE;
    }
    // SAFETY: capacity checked above; caller contract per # Safety.
    let buf = unsafe { std::slice::from_raw_parts_mut(out, need) };
    for (chunk, w) in buf.chunks_exact_mut(8).zip(words.iter()) {
        chunk.copy_from_slice(&w.to_le_bytes());
    }
    STATUS_OK
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical three-event trajectory used across the tests.
    fn sample_events() -> Vec<TrajectoryEvent> {
        vec![
            TrajectoryEvent {
                step: 0,
                activity_id: 1,
                tick: 0,
                marking_before: 0x0000_0000_0000_0000,
                marking_after: 0x0000_0000_0000_0007,
                outcome: 0,
            },
            TrajectoryEvent {
                step: 1,
                activity_id: 2,
                tick: 4,
                marking_before: 0x0000_0000_0000_0007,
                marking_after: 0x0000_0000_0000_001F,
                outcome: 0,
            },
            TrajectoryEvent {
                step: 2,
                activity_id: 3,
                tick: 9,
                marking_before: 0x0000_0000_0000_001F,
                marking_after: 0xDEAD_BEEF_CAFE_F00D,
                outcome: 2,
                // outcome 2 = typed deviation class
            },
        ]
    }

    fn sample_summary() -> TrajectorySummary {
        TrajectorySummary { steps: 3, final_outcome: 2 }
    }

    // ── Pack → digest determinism ───────────────────────────────────────────

    #[test]
    fn pack_then_digest_is_deterministic() {
        let w1 = emit_trajectory(&sample_summary(), &sample_events());
        let w2 = emit_trajectory(&sample_summary(), &sample_events());
        assert_eq!(w1, w2, "same inputs must give byte-identical words");
        assert_eq!(trajectory_digest(&w1), trajectory_digest(&w2));

        assert_eq!(w1.len(), 1 + 3 * 10, "summary word + 10 words per event");
        // Summary header: tag 0xFFFF, steps=3, final_outcome=2.
        assert_eq!(w1[0], pack(0xFFFF, 3, 0, 2, 0), "summary word layout contract");
        // First event header: activity 1, outcome 0, tick 0.
        assert_eq!(w1[1], pack(1, 0, 0, 0, 0));
        // Event 0: marking_before=0 -> limb word 0; marking_after=0x..0007
        // -> first after-limb (word index 1+2+4=7) carries 0x0007.
        assert_eq!(w1[3], pack(0, 0x0000, 0, 0, 0));
        assert_eq!(w1[7], pack(0, 0x0007, 0, 0, 0));
    }

    #[test]
    fn digest_is_bound_to_word_stream_not_rust_object_identity() {
        // Two independently built but field-identical events digest equal.
        let a = emit_trajectory(&sample_summary(), &sample_events());
        let b =
            emit_trajectory(&TrajectorySummary { steps: 3, final_outcome: 2 }, &sample_events());
        assert_eq!(trajectory_digest(&a), trajectory_digest(&b));
    }

    // ── Digest sensitivity ──────────────────────────────────────────────────

    #[test]
    fn digest_changes_on_any_single_word_flip() {
        let base = emit_trajectory(&sample_summary(), &sample_events());
        let base_digest = trajectory_digest(&base);
        for i in 0..base.len() {
            let mut flipped = base.clone();
            flipped[i] ^= 1; // flip one bit of word i
            assert_ne!(
                trajectory_digest(&flipped),
                base_digest,
                "flipping word {i} must change the digest"
            );
        }
        // Also every field of the summary participates.
        for summary in [
            TrajectorySummary { steps: 4, final_outcome: 2 },
            TrajectorySummary { steps: 3, final_outcome: 3 },
        ] {
            let d = trajectory_digest(&emit_trajectory(&summary, &sample_events()));
            assert_ne!(d, base_digest);
        }
    }

    // ── Field roundtrip through the bridge accessors ────────────────────────

    #[test]
    fn every_field_is_recoverable_from_the_packed_words() {
        let ev = TrajectoryEvent {
            step: 0x0001_0002,
            activity_id: 0xABCD,
            tick: 200,
            marking_before: 0x1234_5678_9ABC_DEF0,
            marking_after: 0xFEDC_BA09_8765_4321,
            outcome: 7,
        };
        let words = emit_trajectory(&sample_summary(), &[ev]);
        use crate::observability::ocel::ocel64_bridge as b;

        // Event header.
        assert_eq!(b::activity(words[1]), 0xABCD);
        assert_eq!(b::qualifier(words[1]), 7);
        assert_eq!(b::tick(words[1]), 200);
        // Step limbs.
        assert_eq!(b::activity(words[2]), 0);
        assert_eq!(b::src_obj(words[2]), 0x0002);
        assert_eq!(b::tgt_obj(words[2]), 0x0001);
        // Marking limbs, LSB-first, little-endian semantics.
        let mk_from = |off: usize| -> u64 {
            let mut m = 0u64;
            for i in 0..4 {
                m |= u64::from(b::src_obj(words[off + i])) << (16 * i);
            }
            m
        };
        assert_eq!(mk_from(3), 0x1234_5678_9ABC_DEF0);
        assert_eq!(mk_from(7), 0xFEDC_BA09_8765_4321);
    }

    // ── Bounded-write refusals ──────────────────────────────────────────────

    #[test]
    fn bounded_write_success_roundtrip_matches_digest() {
        let words = emit_trajectory(&sample_summary(), &sample_events());
        let mut buf = vec![0u8; words.len() * 8];
        // SAFETY: `buf` is a live, word-aligned allocation of `words.len() * 8` bytes.
        let st = unsafe { write_event_words(&words, buf.as_mut_ptr(), buf.len() as u32) };
        assert_eq!(st, STATUS_OK);
        // The exported bytes hash to the same digest the guest receipts bind.
        let mut hasher = blake3::Hasher::new();
        hasher.update(&buf);
        assert_eq!(<[u8; 32]>::from(hasher.finalize()), trajectory_digest(&words));
    }

    #[test]
    fn bounded_write_refuses_small_cap_never_truncates() {
        let words = emit_trajectory(&sample_summary(), &sample_events());
        let mut buf = vec![0u8; words.len() * 8];
        for cap in [0u32, 1, 8, (words.len() * 8 - 1) as u32] {
            // Re-zero the buffer so a partial write would be visible.
            buf.iter_mut().for_each(|b| *b = 0);
            // SAFETY: `buf` is live and word-aligned; `cap` <= its byte length.
            let st = unsafe { write_event_words(&words, buf.as_mut_ptr(), cap) };
            assert_eq!(st, STATUS_E_NOMEM, "cap {cap} must refuse E_NOMEM");
            assert!(
                buf.iter().all(|&b| b == 0),
                "cap {cap}: buffer must be untouched, never truncated"
            );
        }
    }

    #[test]
    fn bounded_write_refuses_null_and_misaligned_out() {
        let words = emit_trajectory(&sample_summary(), &sample_events());
        assert_eq!(
            unsafe { write_event_words(&words, std::ptr::null_mut(), u32::MAX) },
            STATUS_E_INVALID
        ); // SAFETY: refusal path, no write performed
           // Deliberately misaligned destination (1 mod 8).
        let mut raw = vec![0u8; words.len() * 8 + 1];
        let misaligned = raw.as_mut_ptr().wrapping_add(1);
        assert_eq!(misaligned as usize % 8, 1);
        assert_eq!(
            unsafe { write_event_words(&words, misaligned, raw.len() as u32) },
            STATUS_E_INVALID
        ); // SAFETY: refusal path, no write performed
    }

    #[test]
    fn empty_trajectory_packs_to_summary_word_only() {
        let words = emit_trajectory(&sample_summary(), &[]);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], pack(0xFFFF, 3, 0, 2, 0));
    }

    // ── R44 linear-memory span law (guest::out_span_oob) ─────────────────────

    use crate::conformance::guest as guest_abi;

    /// RAII guard restoring the installed linear-memory end, so a failing
    /// assertion cannot leak a bound into other tests on the shared static.
    /// The lock SERIALIZES all guard users: LINEAR_MEMORY_END is a
    /// process-global, and parallel tests would otherwise clobber each
    /// other's bound mid-test (Drop restoring a prior captured before a
    /// concurrent install).
    struct MemBoundGuard(
        usize,
        // Held for the guard's lifetime; never read directly.
        std::sync::MutexGuard<'static, ()>,
    );
    fn bound_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }
    impl MemBoundGuard {
        fn install(end: usize) -> Self {
            // Capture the PRIOR bound before overwriting it, so Drop restores
            // the original value (reading after the set leaks the test's own
            // bound into later tests).
            let lock = bound_lock()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let prior = guest_abi::linear_memory_end();
            guest_abi::set_linear_memory_end(end);
            Self(prior, lock)
        }
    }
    impl Drop for MemBoundGuard {
        fn drop(&mut self) {
            guest_abi::set_linear_memory_end(self.0);
        }
    }

    #[test]
    fn bounded_write_refuses_out_span_past_installed_memory_end() {
        let words = emit_trajectory(&sample_summary(), &sample_events());
        let need = words.len() * 8;
        // Real allocation far below the installed bound: the pointer value is
        // synthetic (past `end`), never dereferenced on the refusal path.
        let mut buf = vec![0u8; need];
        let _guard = MemBoundGuard::install(buf.as_ptr() as usize);
        // SAFETY: refusal path — `out_span_oob` fires before any write, so the
        // past-the-bound pointer is never used as a destination.
        let st = unsafe { write_event_words(&words, buf.as_mut_ptr(), need as u32) };
        assert_eq!(
            st, guest_abi::ERR_OOB_WRITE,
            "out-pointer at/past the installed linear-memory end must be a typed errno"
        );
        assert!(
            buf.iter().all(|&b| b == 0),
            "refused call must leave the buffer untouched"
        );
    }

    #[test]
    fn bounded_write_in_bounds_happy_path_still_works_under_installed_bound() {
        let words = emit_trajectory(&sample_summary(), &sample_events());
        let mut buf = vec![0u8; words.len() * 8];
        // Install a bound comfortably above the real allocation: same bytes,
        // same digest, bound installed and enforced but not violated.
        let _guard = MemBoundGuard::install(buf.as_ptr() as usize + 0x1_0000);
        // SAFETY: live, word-aligned allocation of exactly `need` bytes.
        let st = unsafe { write_event_words(&words, buf.as_mut_ptr(), buf.len() as u32) };
        assert_eq!(st, STATUS_OK);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&buf);
        assert_eq!(<[u8; 32]>::from(hasher.finalize()), trajectory_digest(&words));
    }
}
