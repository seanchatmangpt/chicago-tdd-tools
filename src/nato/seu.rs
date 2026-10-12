#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items.
#![allow(dead_code)]
//! NATO N09/N10 — SEU single- and double-bit-fault courts.
//!
//! Ported from strata-protocol tests/nato/{n09_seu_single_bit.rs,
//! n10_seu_double_bit.rs} @ strata 0de13a3 (Apache/MIT per strata LICENSE).
//! Subject transition law: [`crate::conformance::pn64::step_branchless`]
//! (the crate's port of strata's pnabi transition; total fn `u64 -> u64`
//! whose refusal signal is the bit-exact-unchanged word, in place of the
//! donor's typed `PnAbiError`).
//!
//! Classification of every injected fault:
//!
//! - `REFUSED`     — the law returns the corrupted word bit-exact unchanged
//!                   (fail-closed; the law's own refusal signal).
//! - `LAWFUL-DEAD` — the law admits, but the committed word is exactly the
//!                   lawful image of the corrupted word under an independent
//!                   re-derivation of the spec law (see [`lawful_image`]):
//!                   fault effect bounded to the flips themselves. Split into
//!                   dead-equivalent (outcome equals the fault-free baseline)
//!                   and lawful-but-different.
//! - `SDC`         — silent data corruption: the law admits while committing
//!                   something OTHER than the lawful image of the corrupted
//!                   word, a non-marking field drifted, or the evaluation
//!                   panicked (admits-by-crash).
//! - `HALT`        — structurally 0 (`step_branchless` is a total fn with no
//!                   panicking path); disclosed, not assumed.
//!
//! The oracle [`lawful_image`] is an independent re-derivation from the
//! [`crate::conformance::pn64`] getters — it NEVER calls
//! [`crate::conformance::pn64::step_branchless`], so the court does not
//! merely re-run the code under test.
//!
//! Class-boundary note (disclosed): an admitted no-op (lawful image equal to
//! the corrupted word) is indistinguishable from refusal under a
//! bit-exact-unchanged refusal signal; it is counted REFUSED. The corner is
//! conservative — it can never inflate SDC.
//!
//! Tiering (prefix-stream): the fast tier runs the DEFAULT 10,000 flips; the
//! `#[ignore = "nato-tier3"]` full tier runs the donor scale (1,000,000 /
//! 500,000) with the SAME LCG seed, so the full stream's prefix is the fast
//! stream — full subsumes fast.
//!
//! Positive controls (cfg(test) mutants): [`leaky_advance`] (drops the
//! invariant predicate) and [`maskless_advance`] (drops the token predicate)
//! MUST classify SDC, proving the court non-vacuous.

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

// --- Donor constants (verbatim, strata N09/N10 @ 0de13a3) ----------------
// PN64 layout: marking[15:0] pre[31:16] post[47:32] tid[55:48] flags[63:56]
// OCEL64 layout: activity[15:0] src[31:16] tgt[47:32] qual[55:48] tick[63:56]
const PN_BASE: u64 = 0x0002_000C_0002_0082; // m=0x0082 pre=0x0002 post=0x000C tid=0x02
const OCEL_BASE: u64 = 0x41_00_0046_0055_0102; // activity=0x0102 (lo=0x02==tid), tick=0x41
const INVARIANT_MASK: u64 = 0x0000_0000_0000_0010; // bit 4 forbidden in m'

/// Fast-tier flip budget (default): 10,000 flips per court.
const FAST_N: u64 = 10_000;
/// Full-tier budgets: donor scale, verbatim.
const N09_FULL_N: u64 = 1_000_000;
const N10_FULL_ADJ_REPS: u64 = 1_500; // 112 unique pairs x 1,500 = 168,000
const N10_FULL_BYPASS_REPS: u64 = 2_500; // 16 combos x 2,500 = 40,000
const N10_FULL_RANDOM_N: u64 = 292_000;
const N10_FULL_N: u64 = N10_FULL_ADJ_REPS * 112 + N10_FULL_BYPASS_REPS * 16 + N10_FULL_RANDOM_N;

// --- Knuth LCG (donor-faithful; same constants/shape as crate::nato::rng) -
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        // High bits of an LCG have full-period quality; low bits cycle with
        // period 2^k (bit 0 alternates — a naive `% n` never selected the
        // second word in the donor's trial runs). Sample high bits instead.
        (self.next() >> 32) % n
    }
}

/// Independent re-derivation of the spec law from the pn64 getters (NOT a
/// call into [`pn64::step_branchless`]). Admission oracle: `None` iff any
/// validity predicate fails (verb match, token availability, invariant law);
/// `Some(image)` is the only word an admission may commit.
fn lawful_image(pn: u64, ocel: u64, invariant_mask: u64) -> Option<u64> {
    let m = u64::from(pn64::marking(pn));
    let w_pre = u64::from(pn64::pre(pn));
    let w_post = u64::from(pn64::post(pn));
    let tid = u64::from(pn64::tid(pn));
    let activity = ocel & 0xFF;
    let next_m = ((m ^ w_pre) | w_post) & pn64::MASK16;
    let verb_ok = (activity & 0xFF) == tid;
    let token_ok = (m & w_pre) == w_pre;
    let inv_ok = next_m & (invariant_mask & 0xFFFF) == 0;
    if !(verb_ok && token_ok && inv_ok) {
        return None;
    }
    Some((pn & !pn64::MASK16) | next_m)
}

/// Outcome class for one injected evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeuClass {
    /// Law returned the corrupted word bit-exact unchanged (fail-closed).
    Refused,
    /// Admitted; committed word is the lawful image of the corrupted word.
    LawfulDead,
    /// Silent data corruption (admitted a non-lawful word, drifted fields,
    /// or panicked).
    Sdc,
}

/// Classify one injected evaluation against the subject law `adv`.
/// Returns `(class, is_dead_equivalent)`; `is_dead_equivalent` is meaningful
/// only for `LawfulDead` (committed word equals the fault-free baseline).
fn classify_with(
    adv: impl Fn(u64, u64, u64) -> u64,
    pn_in: u64,
    ocel_in: u64,
    invariant_mask: u64,
    baseline_next: u64,
) -> (SeuClass, bool) {
    // A panic under fault is scored SDC-class (admits-by-crash), never
    // credited as fail-closed. HALT is structurally 0: total fn.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        adv(pn_in, ocel_in, invariant_mask)
    }));
    match outcome {
        Err(_panic) => (SeuClass::Sdc, false),
        Ok(next) => {
            if next == pn_in {
                // The law's own refusal signal: bit-exact unchanged.
                (SeuClass::Refused, false)
            } else {
                let non_marking_preserved = (next & !pn64::MASK16) == (pn_in & !pn64::MASK16);
                if lawful_image(pn_in, ocel_in, invariant_mask) != Some(next)
                    || !non_marking_preserved
                {
                    (SeuClass::Sdc, false)
                } else if next == baseline_next {
                    (SeuClass::LawfulDead, true)
                } else {
                    (SeuClass::LawfulDead, false)
                }
            }
        }
    }
}

/// Subject law (production path).
fn advance(pn: u64, ocel: u64, invariant_mask: u64) -> u64 {
    pn64::step_branchless(pn, ocel, invariant_mask)
}

// --- Positive-control mutants (cfg(test), cfg-gated with the file) --------

/// MUTANT: drops the invariant predicate. A fault that pushes a forbidden
/// bit into the post-fire marking must classify SDC under this law.
#[cfg(test)]
fn leaky_advance(pn: u64, ocel: u64, _invariant_mask: u64) -> u64 {
    let m = pn & 0xFFFF;
    let w_pre = (pn >> 16) & 0xFFFF;
    let w_post = (pn >> 32) & 0xFFFF;
    let tid = (pn >> 48) & 0xFF;
    let activity = ocel & 0xFF;
    let next_m = ((m ^ w_pre) | w_post) & 0xFFFF;
    let verb_ok = (activity & 0xFF) == tid;
    let token_ok = (m & w_pre) == w_pre;
    if verb_ok && token_ok {
        (pn & !0xFFFF) | next_m
    } else {
        pn
    }
}

/// MUTANT: drops the token-availability predicate. A fault that kills
/// enablement must classify SDC under this law.
#[cfg(test)]
fn maskless_advance(pn: u64, ocel: u64, invariant_mask: u64) -> u64 {
    let m = pn & 0xFFFF;
    let w_pre = (pn >> 16) & 0xFFFF;
    let w_post = (pn >> 32) & 0xFFFF;
    let tid = (pn >> 48) & 0xFF;
    let activity = ocel & 0xFF;
    let next_m = ((m ^ w_pre) | w_post) & 0xFFFF;
    let verb_ok = (activity & 0xFF) == tid;
    let inv_ok = next_m & (invariant_mask & 0xFFFF) == 0;
    if verb_ok && inv_ok {
        (pn & !0xFFFF) | next_m
    } else {
        pn
    }
}

/// Field histogram bucket for a (word target, bit) injection.
fn field_of(target: usize, bit: u64) -> usize {
    match (target, bit) {
        (0, 0..=15) => 0,
        (0, 16..=31) => 1,
        (0, 32..=47) => 2,
        (0, 48..=55) => 3,
        (0, 56..=63) => 4,
        (1, 0..=15) => 5,
        (1, 16..=31) => 6,
        (1, 32..=47) => 7,
        (1, 48..=55) => 8,
        _ => 9,
    }
}

// --- N09: single-bit-fault census -----------------------------------------

/// Shared N09 driver; `n` is the flip budget. LCG seed is the donor seed at
/// BOTH tiers, so the full run's prefix is the fast run (full subsumes fast).
fn run_n09(n: u64) {
    let mut rng = Lcg(0x4E413039_0000_0001); // "NA09"

    let mut refused: u64 = 0;
    let mut lawful_dead: u64 = 0;
    let mut sdc: u64 = 0;
    let mut dead_equivalent: u64 = 0;
    let mut lawful_different: u64 = 0;
    let mut per_word = [0u64; 2]; // [pn-word flips, ocel-word flips]
    let mut per_field = [0u64; 10];

    let baseline_next = advance(PN_BASE, OCEL_BASE, INVARIANT_MASK);
    assert_ne!(baseline_next, PN_BASE, "baseline fixture must be a lawful firing");

    for _ in 0..n {
        let target = rng.below(2) as usize; // 0 = PN64 word, 1 = OCEL64 word
        let bit = rng.below(64);
        let (word, other) = match target {
            0 => (PN_BASE, OCEL_BASE),
            _ => (OCEL_BASE, PN_BASE),
        };
        let corrupted = word ^ (1u64 << bit);
        let (pn_in, ocel_in) = match target {
            0 => (corrupted, other),
            _ => (other, corrupted),
        };
        let (cls, dead_eq) = classify_with(advance, pn_in, ocel_in, INVARIANT_MASK, baseline_next);
        match cls {
            SeuClass::Refused => refused += 1,
            SeuClass::LawfulDead => {
                lawful_dead += 1;
                if dead_eq {
                    dead_equivalent += 1;
                } else {
                    lawful_different += 1;
                }
            }
            SeuClass::Sdc => sdc += 1,
        }
        per_word[target] += 1;
        per_field[field_of(target, bit)] += 1;
    }

    // Exactly-one-class reconciliation law.
    assert_eq!(refused + lawful_dead + sdc, n, "every fault must land in exactly one class");
    assert_eq!(
        lawful_dead,
        dead_equivalent + lawful_different,
        "LAWFUL-DEAD sub-classes must reconcile"
    );
    // THE invariant: zero silent data corruption over the full run.
    assert_eq!(sdc, 0, "SDC must be 0.000% — silent corruption detected");
    // Fail-closed must actually fire for a meaningful share of faults.
    assert!(refused > 0, "court is vacuous if no fault was ever refused");
}

#[test]
fn n09_seu_single_bit_zero_sdc_fast() {
    run_n09(FAST_N);
}

#[test]
#[ignore = "nato-tier3"]
fn n09_seu_single_bit_zero_sdc_full() {
    run_n09(N09_FULL_N);
}

// --- N10: double-bit-fault census -----------------------------------------

/// Shared N10 driver. Injection classes (donor-verbatim structure):
/// 1. EXHAUSTIVE-ADJACENT: every adjacent-bit pair within each byte lane of
///    each word (2 words x 8 bytes x 7 pairs = 112 unique pairs), fully
///    enumerated at BOTH tiers.
/// 2. BYPASS: one flip in the `pre` field (bits 16..=31) x one flip of an
///    invariant-masked bit (16 combos, forced SET on the masked bit), fully
///    enumerated at BOTH tiers.
/// 3. RANDOMIZED NON-ADJACENT: two distinct non-adjacent bits, LCG-driven,
///    filling the budget; same seed at both tiers so full subsumes fast.
fn run_n10(adj_reps: u64, bypass_reps: u64, random_n: u64) {
    let total = adj_reps * 112 + bypass_reps * 16 + random_n;
    let mut rng = Lcg(0x4E413130_0000_0001); // "NA10"

    let mut refused: u64 = 0;
    let mut lawful_dead: u64 = 0;
    let mut sdc: u64 = 0;
    let mut dead_equivalent: u64 = 0;
    let mut lawful_different: u64 = 0;
    let mut per_word = [0u64; 2];
    let mut class_hist = [[0u64; 3]; 3]; // [injection class][outcome class]
    let mut bypass_refused: u64 = 0;
    let mut _bypass_admitted: u64 = 0;

    let baseline_next = advance(PN_BASE, OCEL_BASE, INVARIANT_MASK);
    assert_ne!(baseline_next, PN_BASE, "baseline fixture must be a lawful firing");

    // Class 1: exhaustive adjacent-bit pairs within each byte lane.
    for word_idx in 0..2u64 {
        let word = if word_idx == 0 { PN_BASE } else { OCEL_BASE };
        for byte in 0..8u64 {
            for lo in 0..7u64 {
                let b0 = byte * 8 + lo;
                let b1 = b0 + 1;
                let flips = (1u64 << b0) | (1u64 << b1);
                for _ in 0..adj_reps {
                    let (pn_in, ocel_in, target) = if word_idx == 0 {
                        (word ^ flips, OCEL_BASE, 0usize)
                    } else {
                        (PN_BASE, word ^ flips, 1usize)
                    };
                    let (cls, dead_eq) =
                        classify_with(advance, pn_in, ocel_in, INVARIANT_MASK, baseline_next);
                    record(
                        &mut refused,
                        &mut lawful_dead,
                        &mut sdc,
                        &mut dead_equivalent,
                        &mut lawful_different,
                        &mut per_word,
                        &mut class_hist,
                        0,
                        cls,
                        dead_eq,
                        target,
                    );
                }
            }
        }
    }

    // Class 2: bypass — pre-field flip (LCG-chosen direction) + forced-SET
    // invariant-masked-bit flip. 16 pre x mask combos, fully enumerated.
    for pre_bit in 16u64..32 {
        for mask_bit in 0u64..64 {
            if INVARIANT_MASK >> mask_bit & 1 == 0 {
                continue;
            }
            for _ in 0..bypass_reps {
                let set_pre = rng.below(2) == 1;
                let flip_pre = if set_pre { 1u64 << pre_bit } else { 0u64 };
                let pn_corrupt = PN_BASE ^ flip_pre ^ (1u64 << mask_bit);
                let (cls, dead_eq) =
                    classify_with(advance, pn_corrupt, OCEL_BASE, INVARIANT_MASK, baseline_next);
                if cls == SeuClass::Refused {
                    bypass_refused += 1;
                } else if cls == SeuClass::LawfulDead {
                    _bypass_admitted += 1;
                }
                record(
                    &mut refused,
                    &mut lawful_dead,
                    &mut sdc,
                    &mut dead_equivalent,
                    &mut lawful_different,
                    &mut per_word,
                    &mut class_hist,
                    1,
                    cls,
                    dead_eq,
                    0,
                );
            }
        }
    }

    // Class 3: randomized non-adjacent pairs.
    for _ in 0..random_n {
        let target = rng.below(2) as usize;
        let b0 = rng.below(64);
        let mut b1 = rng.below(64);
        while b1 == b0 || (b1 as i64 - b0 as i64).abs() == 1 {
            b1 = rng.below(64);
        }
        let flips = (1u64 << b0) | (1u64 << b1);
        let (word, other) = if target == 0 { (PN_BASE, OCEL_BASE) } else { (OCEL_BASE, PN_BASE) };
        let (pn_in, ocel_in) =
            if target == 0 { (word ^ flips, other) } else { (other, word ^ flips) };
        let (cls, dead_eq) = classify_with(advance, pn_in, ocel_in, INVARIANT_MASK, baseline_next);
        record(
            &mut refused,
            &mut lawful_dead,
            &mut sdc,
            &mut dead_equivalent,
            &mut lawful_different,
            &mut per_word,
            &mut class_hist,
            2,
            cls,
            dead_eq,
            target,
        );
    }

    // Exactly-one-class reconciliation law, overall and per injection class.
    assert_eq!(refused + lawful_dead + sdc, total, "every fault must land in exactly one class");
    assert_eq!(class_hist[0][0] + class_hist[0][1] + class_hist[0][2], adj_reps * 112);
    assert_eq!(class_hist[1][0] + class_hist[1][1] + class_hist[1][2], bypass_reps * 16);
    assert_eq!(class_hist[2][0] + class_hist[2][1] + class_hist[2][2], random_n);
    assert_eq!(
        lawful_dead,
        dead_equivalent + lawful_different,
        "LAWFUL-DEAD sub-classes must reconcile"
    );
    assert_eq!(sdc, 0, "SDC must be 0.000% — silent corruption detected");
    assert!(refused > 0, "court is vacuous if no fault was ever refused");
    // The bypass attempt must never amplify into SDC and must be refused a
    // meaningful share of the time via the surviving check.
    assert_eq!(class_hist[1][2], 0, "bypass class produced SDC");
    assert!(bypass_refused > 0, "bypass class never refused — court vacuous");
}

#[test]
fn n10_seu_double_bit_zero_sdc_fast() {
    // Fast tier: 10,000 total, same ratios as the donor budget, all 112
    // adjacent pairs and all 16 bypass combos still fully enumerated.
    const ADJ_REPS: u64 = 15; // 112 x 15 = 1,680
    const BYPASS_REPS: u64 = 25; // 16 x 25 = 400
    const RANDOM_N: u64 = 7_920; // total = 10,000
    run_n10(ADJ_REPS, BYPASS_REPS, RANDOM_N);
}

#[test]
#[ignore = "nato-tier3"]
fn n10_seu_double_bit_zero_sdc_full() {
    // Full tier: donor scale, verbatim budget (500,000).
    run_n10(N10_FULL_ADJ_REPS, N10_FULL_BYPASS_REPS, N10_FULL_RANDOM_N);
}

#[allow(clippy::too_many_arguments)]
fn record(
    refused: &mut u64,
    lawful_dead: &mut u64,
    sdc: &mut u64,
    dead_equivalent: &mut u64,
    lawful_different: &mut u64,
    per_word: &mut [u64; 2],
    class_hist: &mut [[u64; 3]; 3],
    inj_class: usize,
    cls: SeuClass,
    dead_eq: bool,
    word_target: usize,
) {
    match cls {
        SeuClass::Refused => {
            *refused += 1;
            class_hist[inj_class][0] += 1;
        }
        SeuClass::LawfulDead => {
            *lawful_dead += 1;
            class_hist[inj_class][1] += 1;
            if dead_eq {
                *dead_equivalent += 1;
            } else {
                *lawful_different += 1;
            }
        }
        SeuClass::Sdc => {
            *sdc += 1;
            class_hist[inj_class][2] += 1;
        }
    }
    per_word[word_target] += 1;
}

// --- Positive controls: mutants MUST classify SDC -------------------------

/// Leak control: under [`leaky_advance`] a fault pushing a forbidden bit into
/// the post-fire marking is ADMITTED (the surviving checks all pass on the
/// corrupted word), while the independent oracle refuses it — SDC, proving
/// the invariant check is load-bearing for this court.
#[test]
fn positive_control_leaky_advance_is_sdc() {
    let baseline_next = advance(PN_BASE, OCEL_BASE, INVARIANT_MASK);
    // Flip post bit 4 (bit 36): post 0x000C -> 0x001C, so next_m gains the
    // forbidden bit 4 (next_m 0x008C -> 0x009C). Verb and token still hold.
    let pn_corrupt = PN_BASE ^ (1u64 << 36);
    // Sanity: the oracle refuses exactly this fault.
    assert_eq!(lawful_image(pn_corrupt, OCEL_BASE, INVARIANT_MASK), None);
    let (cls, _dead_eq) =
        classify_with(leaky_advance, pn_corrupt, OCEL_BASE, INVARIANT_MASK, baseline_next);
    assert_eq!(cls, SeuClass::Sdc, "leaky mutant must classify SDC");
}

/// Maskless control: under [`maskless_advance`] a fault that kills enablement
/// (clears the only required token from `m`) is ADMITTED, while the
/// independent oracle refuses it — SDC, proving the token check is
/// load-bearing for this court.
#[test]
fn positive_control_maskless_advance_is_sdc() {
    let baseline_next = advance(PN_BASE, OCEL_BASE, INVARIANT_MASK);
    // Clear marking bit 1 (bit 1): m 0x0082 -> 0x0080, m & pre != pre.
    let pn_corrupt = PN_BASE ^ (1u64 << 1);
    assert_eq!(lawful_image(pn_corrupt, OCEL_BASE, INVARIANT_MASK), None);
    let (cls, _dead_eq) =
        classify_with(maskless_advance, pn_corrupt, OCEL_BASE, INVARIANT_MASK, baseline_next);
    assert_eq!(cls, SeuClass::Sdc, "maskless mutant must classify SDC");
}
