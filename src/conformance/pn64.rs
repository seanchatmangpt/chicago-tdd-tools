//! PN64: one Petri-net transition encoded in a single 64-bit word.
//!
//! Ported from strata-protocol src/pn64.rs + src/branchless.rs @ strata
//! fix/kg-42-status-code (0de13a3), Apache/MIT per strata LICENSE.
//!
//! Bit layout (little-endian bit fields), one `u64` = the whole state:
//!   - bits `15:0`  : marking `m` (token presence per place, one bit per place)
//!   - bits `31:16` : precondition mask `W-` (places this transition consumes)
//!   - bits `47:32` : postcondition mask `W+` (places this transition produces)
//!   - bits `55:48` : transition id
//!   - bits `63:56` : flags / invariant byte
//!
//! All accessors are total and branch-light; no heap, no strings — the word is
//! the whole state, suitable for dispatch tables and OCEL-style event streams.
//! Zero dependencies, `no_std`-compatible (no alloc on hot paths).
//!
//! # 1-safe domain precondition
//!
//! The firing arithmetic uses bitwise OR semantics: a deposit into an
//! already-occupied place *coalesces* (the two tokens become one), it does
//! not accumulate. Token conservation (`|m'| == |m| - |W-| + |W+|`) and the
//! standard 1-safe reasoning therefore hold only on the restricted domain of
//! 1-safe nets — at most one token per place — where coalescing can never
//! fire. The `fire_token_conservation` test guard below enumerates the
//! conserved sub-domain explicitly.

/// Low 16-bit field mask (marking, `W-`, `W+`).
pub const MASK16: u64 = 0xFFFF;

/// Extract the 16-bit marking field.
#[inline]
#[must_use]
pub const fn marking(pn: u64) -> u16 {
    (pn & MASK16) as u16
}

/// Extract the 16-bit precondition mask `W-`.
#[inline]
#[must_use]
pub const fn pre(pn: u64) -> u16 {
    ((pn >> 16) & MASK16) as u16
}

/// Extract the 16-bit postcondition mask `W+`.
#[inline]
#[must_use]
pub const fn post(pn: u64) -> u16 {
    ((pn >> 32) & MASK16) as u16
}

/// Extract the 8-bit transition id.
#[inline]
#[must_use]
pub const fn tid(pn: u64) -> u8 {
    // JUSTIFICATION: the PN64 tid field is 8 bits wide ([55:48]); the
    // narrowing cast is the field extraction itself.
    {
        #[allow(clippy::cast_possible_truncation)]
        let t = (pn >> 48) as u8;
        t
    }
}

/// Extract the 8-bit flags/invariant byte.
#[inline]
#[must_use]
pub const fn flags(pn: u64) -> u8 {
    (pn >> 56) as u8
}

/// Pack the five fields into one PN64 word. Out-of-range field values are
/// truncated to their field width (callers treat that as their own invariant).
#[inline]
#[must_use]
pub const fn pack(mark: u16, pre: u16, post: u16, tid: u8, flags: u8) -> u64 {
    (mark as u64)
        | ((pre as u64) << 16)
        | ((post as u64) << 32)
        | ((tid as u64) << 48)
        | ((flags as u64) << 56)
}

/// A transition is enabled iff every precondition place currently holds a
/// token: `(m & W-) == W-`.
#[inline]
#[must_use]
pub const fn enabled(pn: u64) -> bool {
    (marking(pn) & pre(pn)) == pre(pn)
}

/// Fire the transition: `m' = (m ^ W-) | W+`, all non-marking fields
/// preserved bit-exact. Returns `None` when the transition is not enabled.
#[inline]
#[must_use]
pub const fn fire(pn: u64) -> Option<u64> {
    if !enabled(pn) {
        return None;
    }
    let m = marking(pn) as u64;
    let next = ((m ^ (pre(pn) as u64)) | (post(pn) as u64)) & MASK16;
    Some((pn & !MASK16) | next)
}

/// Constant-time equality: returns `1` if `a == b`, `0` otherwise.
///
/// The value difference `a ^ b` is collapsed through the
/// `(x | x.wrapping_neg()) >> 63` idiom and inverted. The sequence (XOR, NEG,
/// OR, shift, XOR) is fixed; no branch, lookup, or early exit depends on
/// operand values, so runtime is independent of equality and magnitude.
#[inline]
#[must_use]
pub const fn ct_eq(a: u64, b: u64) -> u64 {
    let d = a ^ b;
    ((d | d.wrapping_neg()) >> 63) ^ 1
}

/// Constant-time less-than-or-equal: returns `1` if `a <= b`, `0` otherwise.
///
/// Uses the `(b - a)` sign-bit trick: `((b.wrapping_sub(a)) as i64) >> 63`
/// is all-ones when `a > b`, 0 otherwise; wrapping-add 1 maps that to 0 / 1.
/// Sequence (SUB, shift, wrapping ADD) is operand-independent.
///
/// Domain: exact whenever `b.wrapping_sub(a)` does not wrap across 2^63
/// (`b >= a` with `b - a < 2^63`, or `a > b` with `a - b < 2^63`).
#[inline]
#[must_use]
pub const fn ct_lte(a: u64, b: u64) -> u64 {
    // JUSTIFICATION: the sign-bit trick is the point of the function —
    // `wrapping_sub` reinterpreted through i64 and the arithmetic shift
    // `(d >> 63)` are deliberate, not accidental casts.
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    {
        let d = b.wrapping_sub(a) as i64;
        // all-ones + 1 wraps to 0; 0 + 1 = 1.
        ((d >> 63) as u64).wrapping_add(1)
    }
}

/// One branchless PN64 firing step.
///
/// Field unpack is inline (self-contained; no cross-module imports):
///   * `pn[15:0]`    marking `m`
///   * `pn[31:16]`   precondition mask `W-`
///   * `pn[47:32]`   postcondition mask `W+`
///   * `pn[55:48]`   transition id `tid`
///   * `ocel[15:0]`  activity id (verb match on its low byte:
///     `(activity & 0xFF) == tid`)
///
/// Validity predicates, each folded to 1/0 via the `x | x.wrapping_neg()`
/// then `>> 63` idiom:
///   1. **Verb match**: `(activity & 0xFF) == tid`.
///   2. **Token availability**: `(m & W-) == W-`.
///   3. **Invariant law (FORBIDDEN-BIT, PRD FR-03.3)**: no bit of
///      `invariant_mask & 0xFFFF` is set on the post-fire marking —
///      `(M(t+1) & InvariantMask) == 0`.
///
/// Returns the fired word (marking field replaced by `next_m`, all
/// non-marking fields preserved bit-exact) when all three predicates hold,
/// else `pn` bit-exact unchanged. Selection is arithmetic, never a branch.
#[inline]
#[must_use]
pub const fn step_branchless(pn: u64, ocel: u64, invariant_mask: u64) -> u64 {
    // (d | d.wrapping_neg()) >> 63 == 1 iff d != 0, else 0.
    const fn nonzero_bit(d: u64) -> u64 {
        (d | d.wrapping_neg()) >> 63
    }

    // Unpack fields inline (self-contained; no cross-module imports).
    let m: u64 = pn & 0xFFFF;
    let w_pre: u64 = (pn >> 16) & 0xFFFF;
    let w_post: u64 = (pn >> 32) & 0xFFFF;
    let tid: u64 = (pn >> 48) & 0xFF;
    let activity: u64 = ocel & 0xFF;

    // Post-fire marking.
    let next_m: u64 = ((m ^ w_pre) | w_post) & 0xFFFF;

    // 1. Verb match: (activity & 0xFF) == tid.
    let verb_ok: u64 = nonzero_bit(activity ^ tid) ^ 1;

    // 2. Token availability: (m & W-) == W-.
    let token_ok: u64 = nonzero_bit((m & w_pre) ^ w_pre) ^ 1;

    // 3. Invariant law (FORBIDDEN-BIT, PRD FR-03.3): next marking must be
    //    disjoint from the invariant mask.
    let inv_ok: u64 = nonzero_bit(next_m & (invariant_mask & 0xFFFF)) ^ 1;

    // Aggregate valid mask (all three predicates hold).
    let valid: u64 = verb_ok & token_ok & inv_ok;

    // Branch-free select: committed if valid, else pn unchanged.
    let committed: u64 = (pn & !0xFFFF) | next_m;
    let mask: u64 = valid.wrapping_neg(); // all-ones iff valid
    (pn & !mask) | (committed & mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Upstream strata test vectors use .unwrap() on fire()'s Option.
    // Production code in this module is unwrap-free; the three fns below
    // carry the crate's documented test-only exception pattern.

    #[test]
    fn roundtrip_pack_unpack() {
        for mark in [0u16, 1, 0x8001, u16::MAX] {
            for p in [0u16, 0x00FF, u16::MAX] {
                for q in [0u16, 0xF0F0, u16::MAX] {
                    for t in [0u8, 7, u8::MAX] {
                        for f in [0u8, 0xA5, u8::MAX] {
                            let w = pack(mark, p, q, t, f);
                            assert_eq!(marking(w), mark);
                            assert_eq!(pre(w), p);
                            assert_eq!(post(w), q);
                            assert_eq!(tid(w), t);
                            assert_eq!(flags(w), f);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn enabled_true_and_false() {
        // m = 0b0110, W- = 0b0010 -> enabled
        let w = pack(0b0110, 0b0010, 0b1000, 1, 0);
        assert!(enabled(w));
        // m = 0b0110, W- = 0b0001 -> not enabled
        let w2 = pack(0b0110, 0b0001, 0b1000, 1, 0);
        assert!(!enabled(w2));
        // empty precondition is always enabled
        assert!(enabled(pack(0, 0, 0, 0, 0)));
    }

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn fire_consumes_and_produces() {
        // m=0b0110, W-=0b0010, W+=0b1000 -> m' = (0b0110 ^ 0b0010) | 0b1000 = 0b1100
        let w = pack(0b0110, 0b0010, 0b1000, 3, 0xC0);
        let fired = fire(w).unwrap();
        assert_eq!(marking(fired), 0b1100);
        // non-marking fields preserved bit-exact
        assert_eq!(pre(fired), 0b0010);
        assert_eq!(post(fired), 0b1000);
        assert_eq!(tid(fired), 3);
        assert_eq!(flags(fired), 0xC0);
    }

    #[test]
    fn fire_not_enabled_is_none() {
        let w = pack(0b0000, 0b0001, 0b0010, 1, 0);
        assert!(fire(w).is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn fire_token_conservation() {
        // Well-formed loop (W- subseteq m, W+ superseteq W-): firing must
        // conserve tokens as |m'| == |m| - |W-| + |W+|. Also: bits of the
        // marking outside (W-|W+) are untouched by fire.
        for m in 0u16..=0xFF {
            for wpre in 0u16..=0xFF {
                for wpost in 0u16..=0xFF {
                    if (m & wpre) != wpre {
                        continue; // not enabled; fire() returns None
                    }
                    if (wpost & wpre) != wpre {
                        continue; // not a well-formed loop
                    }
                    if (wpost & m) != wpre {
                        continue; // post places must be consumed-or-empty for exact counting
                    }
                    let w = pack(m, wpre, wpost, 0, 0);
                    let fired = fire(w).unwrap();
                    assert_eq!(
                        marking(fired).count_ones(),
                        m.count_ones() - wpre.count_ones() + wpost.count_ones()
                    );
                    let untouched = !(wpre | wpost);
                    assert_eq!(marking(fired) & untouched, m & untouched);
                }
            }
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn flags_preserved_across_fire() {
        let w = pack(0xFFFF, 0xFFFF, 0xFFFF, 0xAB, 0x5A);
        let fired = fire(w).unwrap();
        assert_eq!(flags(fired), 0x5A);
        assert_eq!(tid(fired), 0xAB);
        assert_eq!(pre(fired), 0xFFFF);
        assert_eq!(post(fired), 0xFFFF);
        // m' = (0xFFFF ^ 0xFFFF) | 0xFFFF = 0xFFFF
        assert_eq!(marking(fired), 0xFFFF);
    }

    // ---- ct_eq / ct_lte vectors (ported from branchless.rs surface) ----

    #[test]
    fn ct_eq_matches_rust_equality() {
        for a in [0u64, 1, 0x8000_0000_0000_0000, u64::MAX, 0xDEAD_BEEF] {
            for b in [0u64, 1, 0x8000_0000_0000_0000, u64::MAX, 0xDEAD_BEEF] {
                assert_eq!(ct_eq(a, b), (a == b) as u64);
            }
        }
    }

    #[test]
    fn ct_lte_exact_in_domain_and_documented_out_of_domain() {
        // In-domain pairs (difference < 2^63 on either side).
        for a in [0u64, 1, 0x4000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF] {
            for b in [0u64, 1, 0x4000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF] {
                let exact = if a > b { a - b < (1u64 << 63) } else { b - a < (1u64 << 63) };
                if exact {
                    assert_eq!(ct_lte(a, b), (a <= b) as u64, "ct_lte({a}, {b})");
                }
            }
        }
        // Documented out-of-domain: pair straddling 2^63 by a wide gap.
        let a = 0xFFFF_FFFF_FFFF_FFFF;
        let b = 0;
        assert_eq!(ct_lte(a, b), 1); // wraps; documented inexactness
    }

    // ---- differential: branchless step vs naive branched reference ----

    /// Naive branched reference (pn64/ocel64 semantics restated inline for
    /// independence). Plain `if` control flow.
    fn step_reference(pn: u64, ocel: u64, invariant_mask: u64) -> u64 {
        let m = pn & 0xFFFF;
        let w_pre = (pn >> 16) & 0xFFFF;
        let w_post = (pn >> 32) & 0xFFFF;
        let tid = (pn >> 48) & 0xFF;
        let activity = ocel & 0xFF;
        let next_m = ((m ^ w_pre) | w_post) & 0xFFFF;

        let verb_ok = (activity & 0xFF) == tid;
        let token_ok = (m & w_pre) == w_pre;
        let inv_ok = next_m & (invariant_mask & 0xFFFF) == 0;

        if verb_ok && token_ok && inv_ok {
            (pn & !0xFFFF) | next_m
        } else {
            pn
        }
    }

    /// Deterministic xorshift64* PRNG — fixed seeds, reproducible corpus.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
    }

    fn structured_word(r: &mut Rng) -> u64 {
        // Bias toward structured fields so valid firings occur often.
        let m = (r.next() & 0xFFFF) as u64;
        let pre = (r.next() & 0x00FF) as u64;
        let post = (r.next() & 0x00FF) as u64;
        let tid = (r.next() & 0x0F) as u64;
        let flags = (r.next() & 0xFF) as u64;
        m | (pre << 16) | (post << 32) | (tid << 48) | (flags << 56)
    }

    #[test]
    fn branchless_matches_branched_reference_pseudorandom() {
        let mut r = Rng(0x5EED_D00D_C0FF_EE01);
        for _ in 0..200_000 {
            let pn = structured_word(&mut r);
            let ocel = structured_word(&mut r);
            let inv = r.next() & 0x00FF;
            assert_eq!(
                step_branchless(pn, ocel, inv),
                step_reference(pn, ocel, inv),
                "pn={pn:#018x} ocel={ocel:#018x} inv={inv:#x}"
            );
        }
    }

    #[test]
    fn branchless_matches_branched_reference_structured_corner_cases() {
        // Hand-picked corners: empty pre (always enabled), full-width fields,
        // invariant-forced refusal, verb mismatch.
        let words: [u64; 6] = [
            pack(0, 0, 0xFFFF, 0, 0),
            pack(0xFFFF, 0xFFFF, 0xFFFF, 0xFF, 0xFF),
            pack(0b0110, 0b0010, 0b1000, 3, 0xC0),
            pack(0b0000, 0b0001, 0b0010, 1, 0),
            pack(0x00FF, 0x00FF, 0x00FF, 7, 0),
            pack(0x8001, 0x0001, 0x8000, 9, 0),
        ];
        let ocels: [u64; 6] = [0, 0xFF, 3, 1, (7 << 8) | 7, 0x0109];
        for &pn in &words {
            for &ocel in &ocels {
                for &inv in &[0u64, 0xFF, 0xFFFF] {
                    assert_eq!(
                        step_branchless(pn, ocel, inv),
                        step_reference(pn, ocel, inv),
                        "pn={pn:#018x} ocel={ocel:#018x} inv={inv:#x}"
                    );
                }
            }
        }
    }

    #[test]
    fn branchless_preserves_non_marking_fields_on_commit_and_refusal() {
        // Refusal path must be bit-exact unchanged; commit path preserves
        // bits [63:16] exactly.
        let pn = pack(0b0110, 0b0010, 0b1000, 3, 0xC0);
        // Commit: verb matches tid=3, tokens available, no invariant bits.
        let committed = step_branchless(pn, 3, 0);
        assert_eq!(marking(committed), 0b1100);
        assert_eq!(committed & !MASK16, pn & !MASK16);
        // Refusal: verb mismatch -> bit-exact unchanged.
        assert_eq!(step_branchless(pn, 4, 0), pn);
        // Refusal: invariant bit in post-fire marking -> bit-exact unchanged.
        assert_eq!(step_branchless(pn, 3, 0b1100), pn);
    }
}
