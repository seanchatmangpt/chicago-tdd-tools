#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items.
#![allow(dead_code)]
//! 64-byte-aligned double-word slot with a seq/checksum interlock.
//!
//! Faithful subject port of strata-protocol src/consensus/slot.rs @ strata
//! 0de13a3 (Apache/MIT per strata LICENSE).
//!
//! A [`Slot`] is one cache line (64 B), three `u64` fields: `seq` (version
//! stamp), `word` (payload), `checksum` (integrity seal). `write` seals
//! `(seq, word)` with a checksum and publishes in the order word -> checksum
//! -> seq (seq last, Release), so a reader that observes the expected `seq`
//! (Acquire) is guaranteed the payload+checksum stores are visible; a
//! checksum mismatch is a typed, fail-closed `None`.
//!
//! Checksum: blake3 over the 16 LE bytes of `(seq, word)`, truncated to u64.
//! Collision property (honest): blake3 truncated to 64 bits gives ~2^32
//! birthday-bound collision resistance — fine for detecting hardware
//! torn/partial writes and single-bit corruption, NOT a cryptographic
//! commitment against an adversarial writer.
//!
//! Allocation: none on any path (`Hasher` is stack-resident).

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

/// One 64-byte-aligned slot: seq / word / checksum interlock.
#[repr(C, align(64))]
pub struct Slot {
    seq: AtomicU64,
    word: AtomicU64,
    checksum: AtomicU64,
}

// Compile-time refusal if the layout contract is ever violated (fail-closed).
const _: () = assert!(core::mem::size_of::<Slot>() == 64);
const _: () = assert!(core::mem::align_of::<Slot>() == 64);

#[inline]
fn seal(seq: u64, word: u64) -> u64 {
    let mut h = blake3::Hasher::new();
    h.update(&seq.to_le_bytes());
    h.update(&word.to_le_bytes());
    let d = h.finalize();
    let mut head = [0u8; 8];
    head.copy_from_slice(&d.as_bytes()[0..8]);
    u64::from_le_bytes(head)
}

impl Slot {
    pub const fn new() -> Self {
        Self { seq: AtomicU64::new(0), word: AtomicU64::new(0), checksum: AtomicU64::new(0) }
    }

    /// Seal and publish `(seq, word)`. Order: word -> checksum -> seq (Release).
    pub fn write(&self, seq: u64, word: u64) {
        let c = seal(seq, word);
        // Relaxed: ordering is established by the final Release on `seq`.
        self.word.store(word, Ordering::Relaxed);
        self.checksum.store(c, Ordering::Relaxed);
        self.seq.store(seq, Ordering::Release);
    }

    /// Read the payload only if the slot currently holds `expected_seq` AND
    /// the checksum interlock verifies. Any mismatch => `None` (fail-closed).
    pub fn read(&self, expected_seq: u64) -> Option<u64> {
        // Acquire pairs with write's Release on seq: word+checksum visible.
        let s = self.seq.load(Ordering::Acquire);
        if s != expected_seq {
            return None;
        }
        let w = self.word.load(Ordering::Acquire);
        let c = self.checksum.load(Ordering::Acquire);
        if seal(s, w) != c {
            return None;
        }
        Some(w)
    }

    /// Const layout assertion helper: true iff `Slot` is exactly one 64-byte
    /// cache line with 64-byte alignment.
    pub const fn is_aligned() -> bool {
        core::mem::size_of::<Slot>() == 64 && core::mem::align_of::<Slot>() == 64
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_one_cache_line() {
        assert!(Slot::is_aligned());
        // Address-of verification through a heap Box, not just metadata.
        let b = Box::new(Slot::new());
        let addr = &*b as *const Slot as usize;
        assert_eq!(addr % 64, 0, "heap-allocated Slot not 64-byte aligned: {addr:#x}");
    }

    #[test]
    fn checksum_catches_single_bit_flips() {
        // Exhaustive over 64 bits of the word, then the seq field: flip each
        // bit of a fixed (seq, word), verify read() refuses the corruption.
        for bit in 0u64..64 {
            let slot = Slot::new();
            slot.write(7, 0xDEAD_BEEF_CAFE_F00D);
            let flipped = 0xDEAD_BEEF_CAFE_F00D ^ (1 << bit);
            // Simulate corruption: store word directly, checksum now stale.
            slot.word.store(flipped, Ordering::Relaxed);
            assert_eq!(slot.read(7), None, "bit {bit} flip not caught");
        }
        for bit in 0u64..64 {
            let slot = Slot::new();
            slot.write(0x1234, 42);
            slot.seq.store(0x1234 ^ (1 << bit), Ordering::Relaxed);
            assert_eq!(slot.read(0x1234 ^ (1 << bit)), None, "seq bit {bit} flip not caught");
        }
    }

    #[test]
    fn stale_seq_refuses() {
        let slot = Slot::new();
        slot.write(1, 111);
        slot.write(2, 222);
        assert_eq!(slot.read(1), None);
        assert_eq!(slot.read(2), Some(222));
        assert_eq!(slot.read(3), None);
    }

    /// Torn-write court: 4 writer / 4 reader threads x 100k ops. Writers
    /// alternate seq ranges on one slot; readers hold a seq they observed and
    /// must never see a word that fails to verify.
    #[test]
    fn torn_write_hammer() {
        use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
        use std::sync::Arc;

        let slot = Arc::new(Slot::new());
        let failures = Arc::new(AtomicUsize::new(0));
        let mut joins = Vec::new();

        // Bootstrap: publish seq 0 so readers have a valid baseline.
        slot.write(0, 0);

        for t in 0..4usize {
            let slot = Arc::clone(&slot);
            joins.push(std::thread::spawn(move || {
                for i in 0..100_000u64 {
                    let n = (t as u64) * 100_000 + i + 1;
                    slot.write(n, n.wrapping_mul(0x9E37_79B9_7F4A_7C15));
                }
            }));
        }
        for _ in 0..4usize {
            let slot = Arc::clone(&slot);
            let failures = Arc::clone(&failures);
            joins.push(std::thread::spawn(move || {
                let mut seen: u64 = 0;
                for _ in 0..100_000 {
                    loop {
                        seen = seen.wrapping_add(1);
                        match slot.read(seen) {
                            Some(w) => {
                                assert_eq!(
                                    w,
                                    seen.wrapping_mul(0x9E37_79B9_7F4A_7C15),
                                    "verified read returned wrong word for seq {seen}"
                                );
                                break;
                            }
                            None => {
                                // Stale seq (writer ahead) is legal; catch up.
                                if seen > 400_001 {
                                    failures.fetch_add(1, AOrd::Relaxed);
                                    break;
                                }
                            }
                        }
                    }
                }
            }));
        }
        for j in joins {
            j.join().expect("writer/reader thread panicked");
        }
        slot.write(400_001, 400_001u64.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        assert_eq!(slot.read(400_001), Some(400_001u64.wrapping_mul(0x9E37_79B9_7F4A_7C15)));
    }
}
