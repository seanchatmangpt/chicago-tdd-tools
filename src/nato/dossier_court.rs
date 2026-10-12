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
use crate::ledger::signing::dossier_subject;
use crate::ledger::signing::signer::Signer;
use crate::nato::court::{Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict};
use crate::nato::disclosure::ScopingRecord;
use crate::nato::receipt::{canonical_digest, ControlResult, CourtReceipt};

/// Number of courts in the dossier: N01..N20.
pub const COURT_COUNT: usize = 20;

/// Typed failure of the digest-head guard in [`dossier_root`].
///
/// Defense in depth for AS5 D1: `CourtReceipt::from_json` refuses receipts
/// whose `digest` is not a 64-hex-char SHA-256 string at load time; this
/// guard refuses them again at root-computation time, should a receipt ever
/// reach this module without passing through `from_json` (e.g. a hand-built
/// struct literal with a hand-set digest).
#[derive(Debug)]
pub enum DossierRootError {
    /// A receipt's `digest` is not a 64-hex-char SHA-256 string, so its
    /// first-32-byte head cannot be extracted.
    InvalidDigestHead {
        /// Canonical id of the offending court.
        court_id: String,
        /// What is wrong with the digest (with the offending value).
        reason: String,
    },
    /// A receipt's stored `digest` does not match the digest recomputed over
    /// the receipt's canonical fields (court, subject, verdict, metrics).
    /// F5 law: the NATO root binds the receipt CONTENT, not the self-reported
    /// digest — a metric injected into a receipt after `record()` computed
    /// its digest cannot ride an unmodified root through emit + inspect.
    DigestMismatch {
        /// Canonical id of the offending court.
        court_id: String,
        /// The digest stored on the receipt.
        stored: String,
        /// The digest recomputed from the receipt's canonical content.
        recomputed: String,
    },
}

impl std::fmt::Display for DossierRootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DossierRootError::InvalidDigestHead { court_id, reason } => {
                write!(f, "invalid digest head for court {court_id}: {reason}")
            }
            DossierRootError::DigestMismatch { court_id, stored, recomputed } => {
                write!(
                    f,
                    "digest mismatch for court {court_id}: stored {stored} does not match \
                     recomputed {recomputed} (content tampered after record())"
                )
            }
        }
    }
}

impl std::error::Error for DossierRootError {}

/// Whether `s` is exactly 64 hex chars (either case), i.e. a SHA-256 hex
/// digest as minted by `canonical_digest`.
fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Compute the dossier root over the receipts' canonical digests using the
/// P13 Merkle primitives. Leaves sorted byte-lexicographically by court
/// name; odd node PROMOTED, never duplicated; empty set ->
/// `BLAKE3("P13-EMPTY")` (P13's `empty_root()`).
///
/// Guards each receipt's digest (see [`DossierRootError`]); prefer the
/// checked [`dossier_root_checked`] on admission paths.
pub fn dossier_root(receipts: &[CourtReceipt]) -> [u8; 32] {
    dossier_root_checked(receipts).expect(
        "dossier_root: receipt digest failed the 64-hex guard; \
                 route untrusted receipts through CourtReceipt::from_json",
    )
}

/// Checked variant of [`dossier_root`]: returns a typed error instead of
/// panicking when a receipt's digest cannot yield a 32-byte head.
pub fn dossier_root_checked(receipts: &[CourtReceipt]) -> Result<[u8; 32], DossierRootError> {
    let mut leaves: Vec<(&str, [u8; 32])> = receipts
        .iter()
        .map(|r| {
            let head = digest_head(r)?;
            Ok((CourtId::canonical(r.court_id), head))
        })
        .collect::<Result<_, DossierRootError>>()?;
    leaves.sort_unstable();
    if leaves.is_empty() {
        return Ok(*blake3::hash(b"P13-EMPTY").as_bytes());
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
    Ok(level[0])
}

/// Head digest committed for a court: the first 32 bytes of the receipt's
/// 64-hex-char digest, guarded (typed error, never a range panic).
///
/// F5 law (AS1): the root binds the receipt CONTENT, not the self-reported
/// `digest` string. Before the head is taken, the stored digest is re-derived
/// over the receipt's canonical fields (`court|subject|verdict|metrics`) and
/// a mismatch is refused with [`DossierRootError::DigestMismatch`]. A metric
/// injected into a receipt after [`CourtReceipt::record`] computed its digest
/// therefore can never enter a root: emit refuses, it does not exit 0.
fn digest_head(receipt: &CourtReceipt) -> Result<[u8; 32], DossierRootError> {
    let court_id = CourtId::canonical(receipt.court_id).to_string();
    if !is_sha256_hex(&receipt.digest) {
        return Err(DossierRootError::InvalidDigestHead {
            court_id,
            reason: format!(
                "digest must be 64 hex chars, got {} chars {:?}",
                receipt.digest.len(),
                receipt.digest
            ),
        });
    }
    let recomputed = canonical_digest(
        CourtId::canonical(receipt.court_id),
        &receipt.subject_sha,
        receipt.verdict,
        &receipt.metrics,
    );
    if receipt.digest != recomputed {
        return Err(DossierRootError::DigestMismatch {
            court_id,
            stored: receipt.digest.clone(),
            recomputed,
        });
    }
    let mut head = [0u8; 32];
    head.copy_from_slice(&receipt.digest.as_bytes()[..32]);
    Ok(head)
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

    /// Zero-arg control instance for the registry ctor shape: the
    /// deterministic 20-receipt control set (`sample_control_set`), which the
    /// admission predicate admits by construction. Not a real subject
    /// dossier — registry sweeps and courts that need a zero-arg N20 use
    /// this; real dossiers go through [`DossierCourt::new`].
    pub fn control_default() -> Self {
        DossierCourt { receipts: sample_control_set(), subject_sha: "control-subject".to_string() }
    }

    /// Compute the dossier root and seal it with the ML-DSA-65 signer.
    /// Returns (root, signature); the signature is over the domain-separated
    /// subject `dossier_subject(&root)` — the same subject the landed
    /// sealer law (`crate::ledger::signing::sealer::verify_sealed`) requires —
    /// with the signer's `SIGN_CONTEXT` applied inside the signer.
    pub fn compute_and_seal(
        &self,
    ) -> ([u8; 32], [u8; crate::ledger::signing::signer::SIGNATURE_LEN]) {
        let root = dossier_root(&self.receipts);
        let signer = Signer::from_seed(seed_from_root(&root)).expect("non-degenerate");
        let subject = dossier_subject(&root);
        (root, signer.sign(&subject))
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
        "N20 dossier_aggregate"
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
                // Must equal the metric list the digest above was computed
                // over (F5 law): the root re-derives each receipt's digest
                // from its content and refuses a mismatch.
                metrics: vec![("k".to_string(), MetricValue::U64(i as u64))],
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

    /// Heuristic classification of a persist `Io` failure message as
    /// permission-denied (environmental -> skip) vs. other (data error ->
    /// hard fail). `PersistError::Io` carries a formatted message string, so
    /// we match on the OS-level permission markers rather than the error kind.
    fn is_permission_error(message: &str) -> bool {
        let lower = message.to_lowercase();
        lower.contains("denied")
            || lower.contains("permission")
            || lower.contains("os error 13")
            || lower.contains("read-only")
    }

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
        // N08/N12/N14/N15 mint their own real-run receipts via their module
        // emitters (CTT_EMIT_RECEIPTS=1, same recipe); persisting the control
        // set's synthetic stand-ins here would clobber them. Skip those four.
        let synthetic_control = [
            crate::nato::court::CourtId::N08,
            crate::nato::court::CourtId::N12,
            crate::nato::court::CourtId::N14,
            crate::nato::court::CourtId::N15,
        ];
        let to_persist: Vec<crate::nato::receipt::CourtReceipt> = set
            .iter()
            .filter(|r| !synthetic_control.contains(&r.court_id))
            .cloned()
            .collect();
        let written = match crate::nato::receipts::persist(&dir, &to_persist) {
            Ok(written) => written,
            Err(crate::nato::receipts::PersistError::Io { path, message })
                if is_permission_error(&message) =>
            {
                // Environmental (read-only docs/, unwritable parent, sandbox):
                // skip with a printed reason rather than panic — AS2 Finding 2.
                println!(
                    "skipping dossier receipt emission: cannot write {} \
                     (permission denied: {message}); \
                     re-run with CTT_EMIT_RECEIPTS=1 on a writable tree",
                    path.display()
                );
                return;
            }
            Err(e) => panic!(
                "persist dossier receipts to {} failed (data error, not \
                 environmental): {e}; check CTT_EMIT_RECEIPTS wiring",
                dir.display()
            ),
        };
        // 16 persisted here; the other four come from their own court emitters.
        assert_eq!(written.len(), COURT_COUNT - 4);
        let loaded = crate::nato::receipts::load_dir(&dir).expect("docs/receipts must parse back");
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
        // Changing any receipt's canonical digest moves the root. The
        // mutated receipt's digest is re-derived over its (mutated) content
        // — the same law the ledger follows — so the root sees the change.
        let mut set = sample_control_set();
        set[5].subject_sha = "different-subject".to_string();
        set[5].digest =
            canonical_digest("N06", "different-subject", Verdict::Pass, &set[5].metrics);
        let c = dossier_root(&set);
        assert_ne!(a, c, "changed receipt digest must move the root");
    }

    /// F5 regression: a metric injected into a receipt AFTER `record()`
    /// computed its digest must be refused at root-computation time with the
    /// typed `DigestMismatch` error — never silently accepted (emit must not
    /// exit 0 on tampered evidence).
    #[test]
    fn metric_injected_post_record_is_refused_at_root_time() {
        use crate::nato::court::run_court;
        use crate::nato::court::CandidateRef;
        let real = run_court(
            &crate::nato::mils::MilSeparationCourt,
            &CandidateRef("f5-tamper".into()),
            "sha-f5",
        );
        assert_eq!(real.verdict, Verdict::Pass);
        let mut tampered = real.clone();
        // Inject a metric post-record: content changes, stored digest does not.
        tampered.metrics.push(("injected".to_string(), MetricValue::U64(1)));
        let err = dossier_root_checked(&[tampered]).expect_err("tampered receipt must be refused");
        assert!(
            matches!(err, DossierRootError::DigestMismatch { .. }),
            "expected DigestMismatch, got {err:?}"
        );
        // The un-tampered receipt still roots cleanly.
        let _ = dossier_root_checked(&[real]).expect("honest receipt roots");
    }

    /// F5 companion: a digest rewritten WITHOUT a content change is refused
    /// by the same content-binding law (root must follow content, not the
    /// self-reported digest field).
    #[test]
    fn rewritten_digest_without_content_change_is_refused() {
        let mut set = sample_control_set();
        set[9].digest = "0".repeat(64);
        let err = dossier_root_checked(&set).expect_err("forged digest must be refused");
        assert!(matches!(err, DossierRootError::DigestMismatch { .. }));
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
        let signer = Signer::from_seed(seed_from_root(&root_a)).expect("non-degenerate");
        assert!(
            Signer::verify(&signer.public_key(), &dossier_subject(&root_a), &sig_a),
            "sealed dossier must verify under the derived key over the \
             domain-separated subject"
        );
        // And must NOT verify against the raw root (domain separation is
        // load-bearing; the sealer law signs the subject, not the root).
        assert!(
            !Signer::verify(&signer.public_key(), &root_a, &sig_a),
            "raw-root verification must fail: the seal is over dossier_subject(root)"
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
        // Re-derive the mutated receipt's digest over its content (F5 law:
        // honest mutation rewrites the digest; only tampering leaves it stale).
        receipts[3].digest = canonical_digest(
            CourtId::canonical(receipts[3].court_id),
            &receipts[3].subject_sha,
            receipts[3].verdict,
            &receipts[3].metrics,
        );
        let court = DossierCourt::new(receipts, "subject-run");
        let receipt = run_court(&court, &CandidateRef("dossier".into()), "subject-run");
        assert_eq!(receipt.verdict, Verdict::Fail);
        assert!(
            receipt.positive_control.passed(),
            "court fails on evidence, not on a broken control"
        );
    }

    /// Real-evidence leg: the dossier admits a 20-receipt set anchored by
    /// receipts minted from REAL `Court::run` outputs — `run_court` over the
    /// four landed pillar courts (N01-N04), each with its positive control
    /// firing — instead of `sample_control_set()` synthetics. Partial real
    /// sets refuse (COURT_COUNT law); a mutated real receipt refuses.
    #[test]
    fn dossier_admits_real_court_run_receipts() {
        use crate::nato::bell_lapadula::BellLapadulaCourt;
        use crate::nato::court::{run_court, CandidateRef};
        use crate::nato::mils::MilSeparationCourt;
        use crate::nato::orthogonality::OrthogonalityCourt;
        use crate::nato::zero_ambient::ZeroAmbientCourt;

        let subject = "sha-rv2-real-court-set";
        let real: Vec<CourtReceipt> = [
            &MilSeparationCourt as &dyn Court,
            &OrthogonalityCourt as &dyn Court,
            &BellLapadulaCourt as &dyn Court,
            &ZeroAmbientCourt as &dyn Court,
        ]
        .iter()
        .map(|court| run_court(*court, &CandidateRef("rv2-real".into()), subject))
        .collect();

        // Each landed court ran through the gate: positive control fired
        // (MutantCaught) and the real census/scan passed.
        for r in &real {
            assert_eq!(r.verdict, Verdict::Pass, "{} must pass", r.name);
            assert!(
                matches!(r.positive_control, ControlResult::MutantCaught { .. }),
                "{} positive control must fire, got {:?}",
                r.name,
                r.positive_control
            );
            assert!(!r.metrics.is_empty(), "{} must carry measured metrics", r.name);
        }

        // A partial set of only the 4 real receipts must NOT admit (the
        // COURT_COUNT law) — the aggregator is not lenient toward fragments.
        assert!(!dossier_admits(&real), "4 of 20 receipts must not admit");

        // Fill the remaining 16 slots with control receipts for the courts
        // the real runs did not cover, and the full set admits.
        let mut set: Vec<CourtReceipt> = sample_control_set()
            .into_iter()
            .filter(|r| {
                !matches!(r.court_id, CourtId::N01 | CourtId::N02 | CourtId::N03 | CourtId::N04)
            })
            .collect();
        set.extend(real.clone());
        assert_eq!(set.len(), COURT_COUNT, "4 real + 16 control = 20 receipts");
        assert!(
            dossier_admits(&set),
            "a full 20-receipt set anchored by real run_court receipts must admit"
        );

        // The real receipts carry real evidence: flipping one real receipt
        // (with its canonical digest re-derived, as the ledger would) to
        // Verdict::Fail must refuse admission.
        let mut flipped_real = real.clone();
        flipped_real[2].verdict = Verdict::Fail;
        flipped_real[2].digest = canonical_digest(
            CourtId::canonical(flipped_real[2].court_id),
            &flipped_real[2].subject_sha,
            flipped_real[2].verdict,
            &flipped_real[2].metrics,
        );
        let mut mutant_set = set.clone();
        let pos = mutant_set
            .iter()
            .position(|r| r.receipt_id == set[16 + 2].receipt_id)
            .expect("flipped receipt is in the set");
        mutant_set[pos] = flipped_real[2].clone();
        assert!(
            !dossier_admits(&mutant_set),
            "a real receipt flipped to Fail must refuse admission"
        );
    }
}
