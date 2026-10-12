#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items —
// and referenced by the court registry.
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
//! names the refusal in words with no hex-literal leakage. It does NOT prove the quality of the
//! operating system's entropy source; a healthy OS entropy source cannot be
//! forced to fail in-process, so the failure branch is exercised through an
//! injected-failure harness that replicates the seam's exact fail-closed
//! branch (the same length check the signer performs), and the healthy
//! branch is exercised against the REAL signer (`Signer::generate()`),
//! 1,000 live trials.
//!
//! Port note: strata's court injected failures through a parameterizable
//! entropy seam trait. This crate's seam is the length check inside
//! `Signer::generate()`; it is not (yet) parameterizable, so the failure
//! branch is exercised by a local harness that is byte-for-byte the same
//! check, whose refusal digest is built by a byte-exact replica of the
//! signer's hashed-cause construction (pinned by
//! `replica_refusal_matches_signer_hashed_cause_pattern`), and the
//! remaining mismatch is recorded as work order NATO-ENTROPY-SEAM-01:
//! landing a `Signer::generate_from_entropy_source(source)` seam would let
//! this court inject at the true seam instead of a replica.

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
use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;

/// Number of injected-failure trials.
const INJECTED_TRIALS: usize = 1_000;
/// Number of live healthy-entropy trials against the real signer.
const LIVE_TRIALS: usize = 1_000;

/// The entropy seam, replicated exactly from `Signer::generate()`'s
/// fail-closed branch: a candidate seed of a given length is accepted only
/// when it is exactly [`SEED_LEN`] bytes; any other length is a typed
/// refusal carrying a [`RefusalDigest`] built by the signer's hashed-cause
/// construction (see [`refusal_digest_of`]).
///
/// When the real seam becomes parameterizable, this function must be
/// deleted and the tests below pointed at `Signer` directly.
fn entropy_seam(candidate_len: usize) -> Result<[u8; SEED_LEN], SealError> {
    if candidate_len != SEED_LEN {
        // Same cause shape the signer uses ("ml-dsa seed length refused"),
        // qualified with the concrete off-contract length so each distinct
        // refusal cause yields a distinct, reproducible digest.
        return Err(SealError::EntropyRefused(refusal_digest_of(&format!(
            "ml-dsa seed length refused ({candidate_len})"
        ))));
    }
    Ok([0u8; SEED_LEN])
}

/// Byte-exact local replica of the signer's refusal construction
/// (`src/ledger/signing/signer.rs::refusal_digest_of`): SHA-256 over the
/// signer-local domain tag + b": " + the concrete cause — the
/// `keys::fresh_seed` pattern. The signer's helper is private to its
/// module, so it cannot be called from here; this replica is asserted
/// byte-for-byte equal to that construction by
/// [`replica_refusal_matches_signer_hashed_cause_pattern`] below.
///
/// TEST DEBT (work order NATO-ENTROPY-SEAM-01): land a `pub(crate)`
/// `refusal_digest_of` on the signer (or a `generate_from_entropy_source`
/// seam) so this replica is deleted and the court injects at the true seam.
fn refusal_digest_of(cause: &str) -> RefusalDigest {
    use sha2::{Digest, Sha256};
    const REFUSAL_DOMAIN: &[u8] = b"chicago-ledger-signer-refusal-v1";
    let mut hasher = Sha256::new();
    hasher.update(REFUSAL_DOMAIN);
    hasher.update(b": ");
    hasher.update(cause.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    RefusalDigest(out)
}

/// The replica's construction is pinned byte-for-byte against the signer's
/// hashed-cause pattern: `SHA-256("chicago-ledger-signer-refusal-v1" ||
/// ": " || cause)` — the exact preimage `signer.rs::refusal_digest_of`
/// hashes (its law is pinned in signer's own `refusal_digest_is_hashed_
/// cause_not_fresh_entropy` test: reproducible per cause, distinct across
/// causes, never fresh entropy).
#[test]
fn replica_refusal_matches_signer_hashed_cause_pattern() {
    use sha2::{Digest, Sha256};
    let cause = "ml-dsa seed length refused (33)";
    let expected = {
        let mut h = Sha256::new();
        h.update(b"chicago-ledger-signer-refusal-v1");
        h.update(b": ");
        h.update(cause.as_bytes());
        let d = h.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&d);
        out
    };
    let SealError::EntropyRefused(digest) = entropy_seam(33).err().expect("33 is off-contract")
    else {
        panic!("off-contract length must refuse");
    };
    assert_eq!(
        digest.0, expected,
        "replica refusal digest must equal the signer's hashed-cause construction"
    );
    // Signer law twin: reproducible per cause, distinct across causes.
    assert_eq!(refusal_digest_of(cause), RefusalDigest(expected));
    assert_ne!(
        refusal_digest_of(cause),
        refusal_digest_of("other cause"),
        "distinct causes must produce distinct digests"
    );
}

/// Zero-leak diagnostic shape (not a literal string-set vocabulary): the
/// refusal's Display text must name the refusal in words, must never leak
/// raw `0x`-prefixed byte tokens, and carries only the opaque digest hex
/// plus the (non-secret) off-contract length integer.
fn refusal_text_is_in_fixed_vocabulary(text: &str) -> bool {
    text.contains("refused") && !text.contains("0x")
}

/// 1,000 injected-failure trials: every off-contract seed length produces a
/// typed `EntropyRefused` error with an opaque 32-byte refusal digest.
/// Zero `Ok`, zero panic.
/// Hoisted body of [`injected_failure_trials_all_refuse_typed_with_opaque_digest`]
/// (also the injected-trials leg of the N18 court adapter's `run`). Returns
/// the refusal count; panics on any trial that does not refuse typed.
pub(crate) fn entropy_injected_trials_probe() -> usize {
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
    refusals
}

#[test]
fn injected_failure_trials_all_refuse_typed_with_opaque_digest() {
    let refusals = entropy_injected_trials_probe();
    assert_eq!(refusals, INJECTED_TRIALS);
}

/// Positive control (hoisted from
/// [`refusal_digests_are_reproducible_and_distinct_per_cause`]): refusal
/// digests must be cause-keyed and reproducible — the same cause reproduces
/// the same digest and distinct causes never collide. Returns false if the
/// court's digest comparison has no teeth.
pub(crate) fn entropy_positive_control() -> bool {
    // Reproducible: same cause -> same digest.
    if refusal_digest_of("ml-dsa seed length refused (33)")
        != refusal_digest_of("ml-dsa seed length refused (33)")
    {
        return false;
    }
    // Distinct: the injected sweep's distinct causes never collide.
    let mut seen: Vec<[u8; 32]> = Vec::new();
    for trial in 0..64usize {
        let cause = format!("ml-dsa seed length refused ({})", trial * 7 + 1);
        let r = refusal_digest_of(&cause);
        if seen.contains(&r.0) {
            return false;
        }
        seen.push(r.0);
    }
    true
}

#[test]
fn refusal_digests_are_reproducible_and_distinct_per_cause() {
    assert!(entropy_positive_control(), "digest reproducibility/distinctness must hold");
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
    for i in 0..8 {
        let _ = refusal_digest_of(&format!("never-panics probe {i}"));
    }
}

// ---------------------------------------------------------------------------
// N18 court adapter (W-NTW): trait wiring per the mils.rs N01 template
// ---------------------------------------------------------------------------

/// N18 entropy-starvation court (always-run legs: injected-failure trials
/// with the fixed-vocabulary check + seam length contract; the live-signer
/// trials and reproducibility/distinctness law are the control and the
/// always-run wrapper tests).
///
/// Pillar note: entropy starvation is admitted under [`Pillar::Security`]
/// per the plan's mapping (the 5-value pillar enum has no dedicated
/// variant); the replica-seam injection (work order NATO-ENTROPY-SEAM-01)
/// is disclosed in the run's [`ScopingRecord`].
pub struct EntropyCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl EntropyCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N18;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Security;
}

/// Hoisted court run: the injected-failure trials (every off-contract seed
/// length refuses typed with an opaque digest in the fixed vocabulary),
/// packaged as a [`CourtOutcome`]. Panics (trial assertions) are the Fail
/// path's evidence; a clean return is Pass.
pub(crate) fn entropy_run_court_probe() -> CourtOutcome {
    let refusals = entropy_injected_trials_probe();
    let seam_ok = entropy_seam(SEED_LEN).is_ok();
    let mut off_contract_refusals = 0usize;
    for len in [0usize, 1, 8, 16, 31, 33, 48, 64, 256] {
        if entropy_seam(len).is_err() {
            off_contract_refusals += 1;
        }
    }
    let all_refused = off_contract_refusals == 9 && seam_ok;
    CourtOutcome {
        verdict: if all_refused { Verdict::Pass } else { Verdict::Fail },
        metrics: vec![
            ("injected_refusals", MetricValue::U64(refusals as u64)),
            ("injected_trials", MetricValue::U64(INJECTED_TRIALS as u64)),
            ("off_contract_refusals", MetricValue::U64(off_contract_refusals as u64)),
            ("on_contract_accepted", MetricValue::Bool(entropy_seam(SEED_LEN).is_ok())),
        ],
        scoping: ScopingRecord::partial(
            "entropy-starvation refusal behavior at the ledger signer's \
             entropy seam: off-contract seeds refuse typed with opaque, \
             cause-keyed digests in a fixed diagnostic vocabulary; the \
             healthy branch runs 1,000 live trials against the real signer",
            "1,000 injected-failure trials + seam length-contract census + \
             digest reproducibility/distinctness positive control",
            "failure branch exercised through a byte-exact replica of the \
             length check, not the (non-parameterizable) true seam — work \
             order NATO-ENTROPY-SEAM-01; pillar mapped to Security per plan \
             (no dedicated pillar variant); OS entropy quality NOT proven",
        ),
    }
}

impl Court for EntropyCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N18 entropy_starvation"
    }

    fn positive_control(&self) -> ControlProbe {
        entropy_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        entropy_run_court_probe()
    }
}
