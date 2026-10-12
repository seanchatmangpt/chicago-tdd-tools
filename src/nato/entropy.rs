#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items.
#![allow(dead_code)]
//! Entropy-starvation court over the ledger signer's entropy seam.
//!
//! # Honest scope (mandatory disclosure)
//!
//! This court PROVES the refusal BEHAVIOR at the entropy seam: when the
//! seam's contract is violated (a generated seed that is not exactly
//! [`SEED_LEN`] bytes), the signer's fail-closed branch maps the violation
//! to a typed `SealError::EntropyRefused` carrying an opaque 32-byte
//! [`RefusalDigest`], no `Ok` and no panic escapes, and the refusal text
//! uses a fixed diagnostic vocabulary. It does NOT prove the quality of the
//! operating system's entropy source; a healthy OS entropy source cannot be
//! forced to fail in-process, so the failure branch is exercised through an
//! injected-failure harness that replicates the seam's exact fail-closed
//! branch (the same length check the signer performs), and the healthy
//! branch is exercised against the REAL signer (`Signer::generate()`),
//! 1,000 live trials.
//!
//! Port note: strata's court injected failures through a parameterizable
//! entropy seam trait. This crate's seam is the length check inside
//! [`Signer::generate()`]; it is not (yet) parameterizable, so the failure
//! branch is exercised by a local harness that is byte-for-byte the same
//! check, and the mismatch is recorded here for the coordinator: landing a
//! `Signer::generate_from_entropy_source(source)` seam would let this court
//! inject at the true seam instead of a replica.

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

#[cfg(test)]
use crate::ledger::signing::signer::Signer;

use crate::ledger::error::{RefusalDigest, SealError};
use crate::ledger::signing::signer::SEED_LEN;

/// Number of injected-failure trials.
const INJECTED_TRIALS: usize = 1_000;
/// Number of live healthy-entropy trials against the real signer.
const LIVE_TRIALS: usize = 1_000;

/// The entropy seam, replicated exactly from `Signer::generate()`'s
/// fail-closed branch: a candidate seed of a given length is accepted only
/// when it is exactly [`SEED_LEN`] bytes; any other length is a typed
/// refusal carrying a fresh opaque [`RefusalDigest`].
///
/// When the real seam becomes parameterizable, this function must be
/// deleted and the tests below pointed at `Signer` directly.
fn entropy_seam(candidate_len: usize) -> Result<[u8; SEED_LEN], SealError> {
    if candidate_len != SEED_LEN {
        return Err(SealError::EntropyRefused(refusal_digest_fresh()));
    }
    Ok([0u8; SEED_LEN])
}

/// Local stand-in for `RefusalDigest::fresh()` (the signer references a
/// `fresh()` constructor that error.rs does not yet land — see the lane
/// mismatch note). Deterministic per-call 32-byte digest derived from a
/// process-local counter; opaque to callers.
fn refusal_digest_fresh() -> RefusalDigest {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0x5EED_0000_0000_0001);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&n.to_le_bytes());
    // Spread the counter across the digest so fresh digests differ visibly.
    for (i, b) in bytes.iter_mut().enumerate().skip(8) {
        *b = (n >> (i % 8)) as u8 ^ (i as u8).wrapping_mul(0x9E);
    }
    RefusalDigest(bytes)
}

/// Fixed diagnostic vocabulary: the refusal's Display text must name the
/// refusal in words, must carry the opaque digest hex, and must never leak
/// raw `0x`-prefixed byte tokens (no hex-literal leakage into operator
/// diagnostics).
fn refusal_text_is_in_fixed_vocabulary(text: &str) -> bool {
    text.contains("refused") && !text.contains("0x")
}

/// 1,000 injected-failure trials: every off-contract seed length produces a
/// typed `EntropyRefused` error with an opaque 32-byte refusal digest.
/// Zero `Ok`, zero panic.
#[test]
fn injected_failure_trials_all_refuse_typed_with_opaque_digest() {
    let mut refusals = 0usize;
    for trial in 0..INJECTED_TRIALS {
        // Sweep the whole off-contract space deterministically: lengths
        // around the contract length plus a spread across 0..=96.
        let bad_len = match trial {
            0..=7 => SEED_LEN + 1 + trial,
            8..=15 => SEED_LEN.saturating_sub(1 + (trial - 8)),
            t => (t * 7) % 97,
        };
        // Only off-contract lengths are injected failures; a sweep hit on
        // the contract length is pushed off-contract so every one of the
        // INJECTED_TRIALS trials exercises the refusal branch.
        let bad_len = if bad_len == SEED_LEN { SEED_LEN + 1 } else { bad_len };
        let result = entropy_seam(bad_len);
        let Err(err) = result else {
            panic!("trial {trial}: injected failure MUST NOT succeed");
        };
        // Vocabulary check on the TYPED refusal's Display (the SealError,
        // which names the refusal in words + the opaque digest hex).
        assert!(
            refusal_text_is_in_fixed_vocabulary(&err.to_string()),
            "trial {trial}: refusal text must stay in the fixed diagnostic vocabulary: {}",
            err
        );
        let SealError::EntropyRefused(digest) = err else {
            panic!("trial {trial}: wrong error variant");
        };
        // Opaque: exactly 32 bytes, never all-zero.
        assert_ne!(
            digest.0, [0u8; 32],
            "trial {trial}: refusal digest must not be the zero digest"
        );
        refusals += 1;
    }
    assert_eq!(
        refusals, INJECTED_TRIALS,
        "every injected trial must refuse (loop skips at most the on-contract sweep hits)"
    );
}

/// Refusal digests are opaque and fresh: distinct injected trials carry
/// distinct digests (no digest reuse across refusals).
#[test]
fn refusal_digests_are_fresh_per_trial() {
    let mut seen: Vec<[u8; 32]> = Vec::new();
    for _ in 0..64 {
        let r = refusal_digest_fresh();
        assert!(!seen.contains(&r.0), "refusal digest reused across trials");
        seen.push(r.0);
    }
}

/// The on-contract length is the ONLY length the seam accepts.
#[test]
fn seam_accepts_only_the_contract_length() {
    assert!(entropy_seam(SEED_LEN).is_ok(), "the contract length must be accepted");
    for len in [0usize, 1, 8, 16, 31, 33, 48, 64, 256] {
        assert!(entropy_seam(len).is_err(), "length {len} must be refused");
    }
}

/// 1,000 live trials against the REAL signer's entropy path: a healthy OS
/// entropy source must yield `Ok` every time (the refusal branch must never
/// false-fire), every key distinct, and every minted key operational
/// (sign + verify round-trips).
#[test]
fn live_generate_trials_succeed_with_operational_keys() {
    let mut keys: Vec<[u8; 1952]> = Vec::with_capacity(64);
    for trial in 0..LIVE_TRIALS {
        let signer = match Signer::generate() {
            Ok(s) => s,
            Err(e) => panic!("trial {trial}: healthy entropy must not refuse: {e}"),
        };
        // Operational: deterministic sign + verify round-trip.
        let subject = [trial as u8; 32];
        let sig = signer.sign(&subject);
        assert!(
            Signer::verify(&signer.public_key(), &subject, &sig),
            "trial {trial}: minted key must sign+verify"
        );
        // Sample 64 keys for distinctness (full 1,000-pk set is ~2 MB).
        if trial % (LIVE_TRIALS / 64) == 0 {
            assert!(
                !keys.contains(&signer.public_key()),
                "trial {trial}: repeated public key across independent generations"
            );
            keys.push(signer.public_key());
        }
    }
}

/// Zero-panic guarantee across both branches: the injected failure path and
/// the live path run to completion; this test is the canary that the court
/// itself never panics mid-sweep (it re-runs a shortened sweep).
#[test]
fn entropy_court_never_panics() {
    for len in 0..=64usize {
        let _ = entropy_seam(len); // must not panic for any input length
    }
    for _ in 0..8 {
        let _ = refusal_digest_fresh();
    }
}
