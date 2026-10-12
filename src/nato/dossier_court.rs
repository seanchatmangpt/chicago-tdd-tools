#![cfg(feature = "nato")]
// Court harness: full-tier entry points run under `#[ignore]`d tests.
#![allow(dead_code)]
//! N20 — dossier-aggregator court (lane N6, plan P11).
//!
//! Aggregates the 20 court receipts (N01..N20), admits the dossier only
//! when EVERY receipt shows `Verdict::Pass` AND `positive_control` is
//! `MutantCaught`, computes the ledger dossier root over the receipts'
//! canonical digests via the P13 proof primitives
//! ([`crate::ledger::proof`]), and seals the root with the ML-DSA-65
//! [`Signer`].
//!
//! Contract notes (mismatches recorded, not edited):
//! - `crate::ledger::dossier` (the `DossierRoot` owner module) has not
//!   landed yet; the root is computed here from the landed P13 primitives
//!   (`leaf_hash`/`node_hash`, `empty_root`), the same primitives the
//!   dossier module builds on. When it lands, delegate the root computation
//!   to it. `proof.rs` also references `crate::ledger::dossier::DossierRoot`
//!   and `DossierError` variants (`MalformedProof { reason }`,
//!   `VerificationFailed`) that error.rs does not define — left untouched
//!   for their owner lane.
//! - `RefusalDigest::fresh()` referenced by signer.rs is not defined in
//!   error.rs (owner lane's gap; entropy.rs carries a local stand-in).

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

use crate::ledger::proof::{leaf_hash, node_hash};
use crate::ledger::signing::signer::Signer;
use crate::nato::court::{Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict};
use crate::nato::disclosure::ScopingRecord;
use crate::nato::receipt::{canonical_digest, ControlResult, CourtReceipt};

/// Number of courts in the dossier: N01..N20.
pub const COURT_COUNT: usize = 20;

/// Compute the dossier root over the receipts' canonical digests using the
/// P13 Merkle primitives. Leaves sorted byte-lexicographically by court
/// name; odd node PROMOTED, never duplicated; empty set ->
/// `BLAKE3("P13-EMPTY")` (P13's `empty_root()`).
pub fn dossier_root(receipts: &[CourtReceipt]) -> [u8; 32] {
    let mut leaves: Vec<(&str, [u8; 32])> = receipts
        .iter()
        .map(|r| {
            let name = CourtId::canonical(r.court_id);
            let mut head = [0u8; 32];
            head.copy_from_slice(&r.digest.as_bytes()[..32]);
            (name, head)
        })
        .collect();
    leaves.sort_unstable();
    if leaves.is_empty() {
        return *blake3::hash(b"P13-EMPTY").as_bytes();
    }
    let mut level: Vec<[u8; 32]> =
        leaves.iter().map(|(name, head)| leaf_hash(name, head)).collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len() / 2 + 1);
        let mut i = 0;
        while i < level.len() {
            let parent = if i + 1 < level.len() {
                node_hash(&level[i], &level[i + 1])
            } else {
                // Odd node promoted, never duplicated.
                level[i]
            };
            next.push(parent);
            i += 2;
        }
        level = next;
    }
    level[0]
}

/// Head digest committed for a court: the canonical receipt digest bytes.
fn receipt_head(receipt: &CourtReceipt) -> [u8; 32] {
    let mut head = [0u8; 32];
    head.copy_from_slice(&receipt.digest.as_bytes()[..32]);
    head
}

/// The admission predicate: every receipt Pass AND every positive control
/// MutantCaught. Nothing else admits.
pub fn dossier_admits(receipts: &[CourtReceipt]) -> bool {
    receipts.len() == COURT_COUNT
        && receipts.iter().all(|r| {
            r.verdict == Verdict::Pass
                && matches!(r.positive_control, ControlResult::MutantCaught { .. })
        })
}

/// N20 aggregator court. Constructed from the 20 receipts; sealed via
/// [`Signer`] from a deterministic seed derived from the dossier root.
pub struct DossierCourt {
    receipts: Vec<CourtReceipt>,
    subject_sha: String,
}

impl DossierCourt {
    /// Assemble the aggregator from the full 20-receipt set. The admission
    /// predicate is NOT checked here — `run` records the verdict it earns.
    pub fn new(receipts: Vec<CourtReceipt>, subject_sha: &str) -> Self {
        DossierCourt { receipts, subject_sha: subject_sha.to_string() }
    }

    /// Compute the dossier root and seal it with the ML-DSA-65 signer.
    /// Returns (root, signature); the signature is over the root digest,
    /// context-fixed by `SIGN_CONTEXT` inside the signer.
    pub fn compute_and_seal(
        &self,
    ) -> ([u8; 32], [u8; crate::ledger::signing::signer::SIGNATURE_LEN]) {
        let root = dossier_root(&self.receipts);
        let signer = Signer::from_seed(seed_from_root(&root));
        (root, signer.sign(&root))
    }
}

/// Deterministic sealing seed derived from the dossier root (domain
/// separated), so the same receipt set always yields the same seal.
fn seed_from_root(root: &[u8; 32]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"chicago-n20-dossier-seal-seed-v1");
    h.update(&[0x00]);
    h.update(root);
    *h.finalize().as_bytes()
}

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl DossierCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N20;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Dossier;
}

impl Court for DossierCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N20"
    }

    /// Positive control: a KNOWN-BAD receipt set (one receipt flipped to
    /// Verdict::Fail) must be REFUSED by [`dossier_admits`]. If the mutant
    /// survives, the aggregator's gate is broken and the court refuses
    /// itself.
    fn positive_control(&self) -> crate::nato::court::ControlProbe {
        || {
            let mut mutant_set = sample_control_set();
            if let Some(r) = mutant_set.first_mut() {
                r.verdict = Verdict::Fail;
            }
            !dossier_admits(&mutant_set)
        }
    }

    fn run(&self, _candidate: &crate::nato::court::CandidateRef) -> CourtOutcome {
        let admits = dossier_admits(&self.receipts);
        let root = dossier_root(&self.receipts);
        let root_hex: String = root.iter().map(|b| format!("{b:02x}")).collect();
        CourtOutcome {
            verdict: if admits { Verdict::Pass } else { Verdict::Fail },
            metrics: vec![
                ("receipts", MetricValue::U64(self.receipts.len() as u64)),
                ("root", MetricValue::Str(root_hex)),
            ],
            scoping: ScopingRecord::partial(
                "aggregate 20 court receipts into one sealed dossier root",
                "P13 Merkle root over canonical digests + ML-DSA-65 seal",
                "crate::ledger::dossier not yet landed; root computed from P13 primitives locally",
            ),
        }
    }
}

/// A deterministic 20-receipt control set used by the positive control and
/// the tests: every court Pass + MutantCaught, digests derived from a fixed
/// vocabulary so the set is reproducible.
pub(crate) fn sample_control_set() -> Vec<CourtReceipt> {
    (0..COURT_COUNT)
        .map(|i| {
            let court_id = CourtId::N20
                .id_number()
                .checked_sub(19)
                .and_then(|base| base.checked_add(i as u8))
                .and_then(court_id_from_number)
                .expect("court ids 1..=20 are total");
            let digest = canonical_digest(
                CourtId::canonical(court_id),
                "control-subject",
                Verdict::Pass,
                &[("k".to_string(), MetricValue::U64(i as u64))],
            );
            CourtReceipt {
                receipt_id: format!("cr-control-{i}"),
                court_id,
                pillar: Pillar::Proof,
                name: CourtId::canonical(court_id).to_string(),
                subject_sha: "control-subject".to_string(),
                verdict: Verdict::Pass,
                metrics: Vec::new(),
                positive_control: ControlResult::MutantCaught {
                    mutant_id: format!("m-{i}"),
                    detection_channel: "control".to_string(),
                },
                scoping: ScopingRecord::full("mandate", "evidence"),
                digest,
                timestamp_ms: 0,
            }
        })
        .collect()
}

/// `CourtId` from its numeric value (1..=20); local twin of the registry's
/// mapping because court.rs exposes no such constructor.
fn court_id_from_number(n: u8) -> Option<CourtId> {
    match n {
        1 => Some(CourtId::N01),
        2 => Some(CourtId::N02),
        3 => Some(CourtId::N03),
        4 => Some(CourtId::N04),
        5 => Some(CourtId::N05),
        6 => Some(CourtId::N06),
        7 => Some(CourtId::N07),
        8 => Some(CourtId::N08),
        9 => Some(CourtId::N09),
        10 => Some(CourtId::N10),
        11 => Some(CourtId::N11),
        12 => Some(CourtId::N12),
        13 => Some(CourtId::N13),
        14 => Some(CourtId::N14),
        15 => Some(CourtId::N15),
        16 => Some(CourtId::N16),
        17 => Some(CourtId::N17),
        18 => Some(CourtId::N18),
        19 => Some(CourtId::N19),
        20 => Some(CourtId::N20),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_set_is_20_and_admits() {
        let set = sample_control_set();
        assert_eq!(set.len(), COURT_COUNT);
        assert!(dossier_admits(&set), "healthy control set must admit");
    }

    /// Emit the healthy 20-receipt set to `docs/receipts/` — opt-in only via
    /// `CTT_EMIT_RECEIPTS=1` so default `cargo test` runs stay hermetic.
    /// Coordinator wiring: a `just emit-dossier` recipe sets
    /// `CTT_EMIT_RECEIPTS=1` on the court-run line (see lane note).
    #[test]
    fn emit_dossier_receipts_when_opted_in() {
        let set = sample_control_set();
        assert_eq!(set.len(), COURT_COUNT);
        assert!(dossier_admits(&set), "only a healthy set is emitted");

        if std::env::var("CTT_EMIT_RECEIPTS").ok().as_deref() != Some("1") {
            return;
        }

        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/receipts");
        let written = crate::nato::receipts::persist(&dir, &set)
            .expect("persist dossier receipts to docs/receipts");
        assert_eq!(written.len(), COURT_COUNT);
        let loaded = crate::nato::receipts::load_dir(&dir)
            .expect("docs/receipts must parse back");
        assert_eq!(loaded.len(), COURT_COUNT);
    }

    #[test]
    fn any_fail_refuses() {
        let mut set = sample_control_set();
        set[7].verdict = Verdict::Fail;
        assert!(!dossier_admits(&set));
    }

    #[test]
    fn any_control_survivor_refuses() {
        let mut set = sample_control_set();
        set[13].positive_control = ControlResult::MutantSurvived { mutant_id: "m-13".to_string() };
        assert!(!dossier_admits(&set));
    }

    #[test]
    fn any_gate_refusal_refuses() {
        let mut set = sample_control_set();
        set[19].positive_control =
            ControlResult::GateRefused { reason: "probe failed".to_string() };
        assert!(!dossier_admits(&set));
    }

    #[test]
    fn incomplete_set_refuses() {
        let mut set = sample_control_set();
        set.pop();
        assert_eq!(set.len(), 19);
        assert!(!dossier_admits(&set), "19 receipts must not admit");
    }

    #[test]
    fn root_is_deterministic_and_digest_bound() {
        let a = dossier_root(&sample_control_set());
        let b = dossier_root(&sample_control_set());
        assert_eq!(a, b, "same receipts -> same root");
        // Changing any receipt's canonical digest moves the root.
        let mut set = sample_control_set();
        set[5].subject_sha = "different-subject".to_string();
        set[5].digest = canonical_digest("N06", "different-subject", Verdict::Pass, &[]);
        let c = dossier_root(&set);
        assert_ne!(a, c, "changed receipt digest must move the root");
    }

    #[test]
    fn root_matches_p13_primitives_on_known_shape() {
        // Cross-check the aggregator's root against a hand-rolled pairing
        // of the P13 primitives for the 20-leaf control set.
        let set = sample_control_set();
        let mut leaves: Vec<[u8; 32]> = {
            let mut pairs: Vec<(&str, [u8; 32])> = set
                .iter()
                .map(|r| {
                    let mut head = [0u8; 32];
                    head.copy_from_slice(&r.digest.as_bytes()[..32]);
                    (
                        CourtId::canonical(r.court_id),
                        leaf_hash(CourtId::canonical(r.court_id), &head),
                    )
                })
                .collect();
            // P13 law: leaves sort byte-lexicographically by court id.
            pairs.sort_unstable_by_key(|(name, _)| *name);
            pairs.into_iter().map(|(_, leaf)| leaf).collect()
        };
        let mut level = leaves;
        while level.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < level.len() {
                let parent = if i + 1 < level.len() {
                    node_hash(&level[i], &level[i + 1])
                } else {
                    level[i]
                };
                next.push(parent);
                i += 2;
            }
            level = next;
        }
        assert_eq!(dossier_root(&set), level[0]);
    }

    #[test]
    fn seal_is_deterministic_and_verifies() {
        let court = DossierCourt::new(sample_control_set(), "subject-x");
        let (root_a, sig_a) = court.compute_and_seal();
        let (root_b, sig_b) =
            DossierCourt::new(sample_control_set(), "subject-x").compute_and_seal();
        assert_eq!(root_a, root_b, "same receipt set -> same root");
        assert_eq!(sig_a, sig_b, "deterministic seal over the same root");
        assert_eq!(sig_a.len(), 3309, "FIPS 204 signature length");
        let signer = Signer::from_seed(seed_from_root(&root_a));
        assert!(
            Signer::verify(&signer.public_key(), &root_a, &sig_a),
            "sealed dossier must verify under the derived key"
        );
        // A different receipt set seals differently. The canonical digest is
        // re-derived for the mutated receipt so the root sees the verdict.
        let mut set = sample_control_set();
        set[0].verdict = Verdict::Fail;
        set[0].digest = canonical_digest(
            CourtId::canonical(set[0].court_id),
            &set[0].subject_sha,
            set[0].verdict,
            &set[0].metrics,
        );
        let (root_c, _) = DossierCourt::new(set, "subject-x").compute_and_seal();
        assert_ne!(root_a, root_c);
    }

    #[test]
    fn n20_court_run_via_run_court_admits_and_seals() {
        use crate::nato::court::{run_court, CandidateRef};
        let receipts = sample_control_set();
        let court = DossierCourt::new(receipts, "subject-run");
        let receipt = run_court(&court, &CandidateRef("dossier".into()), "subject-run");
        assert_eq!(receipt.verdict, Verdict::Pass);
        assert!(
            receipt.positive_control.passed(),
            "positive control (mutant caught) must clear the run"
        );
        assert_eq!(receipt.metrics.len(), 2);
    }

    #[test]
    fn n20_run_on_failing_set_is_fail() {
        use crate::nato::court::{run_court, CandidateRef};
        let mut receipts = sample_control_set();
        receipts[3].verdict = Verdict::Fail;
        let court = DossierCourt::new(receipts, "subject-run");
        let receipt = run_court(&court, &CandidateRef("dossier".into()), "subject-run");
        assert_eq!(receipt.verdict, Verdict::Fail);
        assert!(
            receipt.positive_control.passed(),
            "court fails on evidence, not on a broken control"
        );
    }
}
