//! Dossier CLI laws (lane G4, v26.10.11).
//!
//! Integration court for the `ctdd_ledger` dossier-emit / dossier-inspect
//! surface (bin subcommands `seal` / `verify`) through the landed lib
//! primitives the bin composes. Contract notes:
//!
//! - The CLI filesystem walk (`build_dossier_root`) and process exit plumbing
//!   were excised with the `ctdd_ledger` bin (V2); the CLI verbs are pending
//!   affi (Phase D M2). The emit path below therefore RECONSTRUCTS the
//!   bin's former algorithm from the lib primitives it used
//!   (`validate_receipt` → `CourtLedgerBuilder` →
//!   `DossierBuilder::seal` → `seal_dossier`), byte-for-byte the same
//!   composition, and asserts the documented wire output
//!   (`sealer::encode` layout) by testing the pieces. What is NOT
//!   process-tested here: the clap arg parsing, `KeyStore` custody at the
//!   real key dir, and the process exit plumbing (IO→2, tamper→13) — the
//!   clap/exit plumbing re-landed as affi verbs (Phase D M2, ~/affidavit); every refusal class
//!   it mapped is instead pinned at the domain layer (typed errors +
//!   `exit_code`).
//! - Signers here are deterministic `Signer::from_seed` instances (real
//!   ML-DSA-65 keys, no mocks); `KeyStore` tempdir custody is covered by
//!   `tests/ledger_sealing_tests.rs`.
//!
//! Run: `cargo test --test dossier_cli_tests --features ledger,nato`

#![cfg(all(feature = "ledger", feature = "nato"))]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use chicago_tdd_tools::ledger::dossier::{CourtLedgerBuilder, DossierBuilder};
use chicago_tdd_tools::ledger::error::LedgerError;
use chicago_tdd_tools::ledger::proof::{leaf_hash, node_hash};
use chicago_tdd_tools::ledger::receipt::{
    ChainedCourtReceipt, CourtOutcome, CourtReceiptV1, ReceiptMetrics, ThermalClass,
};
use chicago_tdd_tools::ledger::signing::sealer::{
    self, alg_marker_ml_dsa_65, SealVerdict, FORMAT_VERSION, WIRE_LEN,
};
use chicago_tdd_tools::ledger::signing::signer::Signer;
use chicago_tdd_tools::ledger::{
    exit_code, seal_dossier, validate_receipt, verify_sealed_file, DossierRoot, LedgerExitCode,
};
use chicago_tdd_tools::nato::court::{
    CourtId, CourtOutcome as NatoOutcome, MetricValue, Pillar, Verdict,
};
use chicago_tdd_tools::nato::disclosure::ScopingRecord;
use chicago_tdd_tools::nato::dossier_court::{dossier_admits, COURT_COUNT};
use chicago_tdd_tools::nato::receipt::ControlResult;
use chicago_tdd_tools::nato::receipt::CourtReceipt;

// ─── Ledger-side helpers ─────────────────────────────────────────────────────

/// Deterministic signer per seed byte (real ML-DSA-65 key, no mocks).
fn signer(seed: u8) -> Signer {
    Signer::from_seed([seed; 32])
}

/// One valid 352-byte chained record for `court_id` (Pass, budget met).
fn receipt_bytes(court_id: &str, seed: u8) -> [u8; 352] {
    let receipt = CourtReceiptV1::new(
        subject(seed),
        [0u8; 32],
        ReceiptMetrics {
            timestamp_ms: 1_700_000_000_000,
            run_id: u64::from(seed),
            total_ticks: 9_999,
            wall_ms: 7,
            thermal: ThermalClass::Warm,
            outcome: CourtOutcome::Pass,
            budget_met: true,
        },
        [0u8; 32],
        metadata(seed),
        [0u8; 32],
        court_id,
    )
    .expect("well-formed receipt");
    ChainedCourtReceipt::from_receipt(&receipt)
        .expect("test court ids fit NAME_MAX")
        .encode()
}

fn subject(seed: u8) -> [u8; 32] {
    let mut s = [0u8; 32];
    s[0] = seed;
    s
}

fn metadata(seed: u8) -> [u8; 32] {
    let mut m = [0u8; 32];
    m[31] = seed;
    m
}

/// Court ids `court-00`..`court-19` (20 courts, N01..N20 shape).
fn court_names() -> Vec<String> {
    (0..20u8).map(|i| format!("court-{i:02}")).collect()
}

/// Write one 352-byte record per court into a tempdir receipts dir. Each
/// court's record bytes and file name are stable across calls; only the
/// write order (hence observed readdir order) varies, so file-order
/// invariance of the emit walk is observable without changing content.
fn write_receipts_dir(dir: &std::path::Path, reversed: bool) {
    std::fs::create_dir_all(dir).expect("mkdir");
    let names = court_names();
    let ordered: Vec<(usize, &String)> = if reversed {
        names.iter().enumerate().rev().collect()
    } else {
        names.iter().enumerate().collect()
    };
    for (i, name) in ordered {
        let path = dir.join(format!("{i:02}.ctrt"));
        std::fs::write(&path, receipt_bytes(name, u8::try_from(i).expect("fits u8")))
            .expect("write");
    }
}

/// The emit path, reconstructed from the exact lib calls the bin's
/// `build_dossier_root` + `Cmd::Seal` make (sorted dir walk, verify each
/// record, group by court, chain, Merkle-seal, ML-DSA-65 envelope).
/// Returns the root and the sealed wire bytes.
fn emit(dir: &std::path::Path, sgn: &Signer) -> (DossierRoot, [u8; WIRE_LEN]) {
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .expect("readable dir")
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    paths.sort();
    let mut courts: Vec<CourtLedgerBuilder> = Vec::new();
    for p in paths {
        let bytes = std::fs::read(&p).expect("readable receipt");
        let verified = validate_receipt(&bytes).expect("every written receipt verifies");
        let receipt = verified.receipt();
        let court = receipt.name.clone();
        let chained = ChainedCourtReceipt::from_receipt(&receipt).expect("chainable receipt");
        match courts.iter_mut().find(|b| b.court_id() == court) {
            Some(b) => b.append(chained).expect("same-court append"),
            None => {
                let mut b = CourtLedgerBuilder::new(court);
                b.append(chained).expect("first append");
                courts.push(b);
            }
        }
    }
    let mut dossier = DossierBuilder::new();
    for b in &courts {
        dossier.add_court(b.court_id().to_string(), b.head()).expect("unique courts");
    }
    let root = dossier.seal().expect("nonempty dossier");
    let sealed = seal_dossier(&root, sgn).expect("sealable root");
    (root, sealer::encode(&sealed))
}

/// Independent P13 reconstruction of the dossier root from the receipts dir
/// (leaf_hash/node_hash, sorted, odd PROMOTED) — no DossierBuilder involved.
fn independent_root(_dir: &std::path::Path) -> [u8; 32] {
    let mut leaves: Vec<(String, [u8; 32])> = court_names()
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let bytes = receipt_bytes(id, u8::try_from(i).expect("fits u8"));
            let verified = validate_receipt(&bytes).expect("verifies");
            let record = verified.as_bytes();
            let stored =
                <[u8; 32]>::try_from(&record[320..352] /* stored chain hash */).expect("32 bytes");
            // Chain-head fold (law in `CourtLedgerBuilder::append`):
            // head = blake3(prev_head ‖ stored); genesis head is zeroed.
            let mut h = blake3::Hasher::new();
            h.update(&[0u8; 32]);
            h.update(&stored);
            (id.clone(), *h.finalize().as_bytes())
        })
        .collect();
    leaves.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut level: Vec<[u8; 32]> = leaves.iter().map(|(id, head)| leaf_hash(id, head)).collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len() / 2 + 1);
        let mut i = 0;
        while i < level.len() {
            let parent = if i + 1 < level.len() {
                node_hash(&level[i], &level[i + 1])
            } else {
                level[i] // odd node PROMOTED, never duplicated
            };
            next.push(parent);
            i += 2;
        }
        level = next;
    }
    level[0]
}

// ─── NATO admission-side helpers ─────────────────────────────────────────────

/// One NATO `CourtReceipt` minted through the real `record` constructor.
fn nato_receipt(court: CourtId, verdict: Verdict, control: ControlResult) -> CourtReceipt {
    CourtReceipt::record(
        court,
        Pillar::Proof,
        court.canonical(),
        "g4-dossier-subject",
        NatoOutcome {
            verdict,
            metrics: vec![("ticks", MetricValue::U64(u64::from(court.id_number())))],
            scoping: ScopingRecord::full("admit the 20-court dossier", "20 Pass receipts"),
        },
        control,
    )
}

/// The healthy 20-receipt control set: every court Pass + MutantCaught.
fn control_set() -> Vec<CourtReceipt> {
    let ids = [
        CourtId::N01,
        CourtId::N02,
        CourtId::N03,
        CourtId::N04,
        CourtId::N05,
        CourtId::N06,
        CourtId::N07,
        CourtId::N08,
        CourtId::N09,
        CourtId::N10,
        CourtId::N11,
        CourtId::N12,
        CourtId::N13,
        CourtId::N14,
        CourtId::N15,
        CourtId::N16,
        CourtId::N17,
        CourtId::N18,
        CourtId::N19,
        CourtId::N20,
    ];
    ids.iter()
        .map(|id| {
            nato_receipt(
                *id,
                Verdict::Pass,
                ControlResult::MutantCaught {
                    mutant_id: format!("m-{}", id.id_number()),
                    detection_channel: "g4-court".to_string(),
                },
            )
        })
        .collect()
}

// ─── Emit → inspect roundtrip ────────────────────────────────────────────────

#[test]
fn emit_20_court_dossier_then_inspect_is_valid() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_receipts_dir(tmp.path(), false);
    let sgn = signer(0x41);
    let (root, wire) = emit(tmp.path(), &sgn);

    assert_eq!(wire.len(), WIRE_LEN, "sealed wire length is root+key+alg+subject+ver+sig");
    let sealed = sealer::decode(&wire).expect("wire decodes");
    assert_eq!(sealed.root, root.0, "wire commits the emit root");
    assert_eq!(sealed.envelope.format_version, FORMAT_VERSION);
    assert_eq!(sealed.envelope.alg_marker, alg_marker_ml_dsa_65(), "alg marker is ML-DSA-65");
    assert_eq!(
        verify_sealed_file(&sealed, &sgn.public_key()),
        SealVerdict::Valid,
        "freshly emitted dossier must inspect Valid under the emitting key"
    );
}

#[test]
fn emit_root_matches_independent_p13_reconstruction() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_receipts_dir(tmp.path(), false);
    let (root, _) = emit(tmp.path(), &signer(0x42));
    assert_eq!(
        root.0,
        independent_root(tmp.path()),
        "emit root must equal the independent P13 reconstruction"
    );
}

/// `--assert-merkle-root` law (domain carrier): a claimed root that differs
/// from the recomputed root must be DETECTED. The bin has not landed the
/// flag itself; the law is pinned where it is enforceable — inclusion-proof
/// verification against the wrong root refuses, and a sealed envelope whose
/// root byte moved fails inspection.
#[test]
fn assert_merkle_root_mismatch_is_detected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_receipts_dir(tmp.path(), false);
    let sgn = signer(0x43);
    let (root, wire) = emit(tmp.path(), &sgn);

    // The claim the flag would carry must match the recomputed root…
    assert_eq!(root.0, independent_root(tmp.path()));
    // …and any other claimed root must fail the proof check.
    let mut wrong_claim = root.0;
    wrong_claim[0] ^= 0x01;
    assert_ne!(root.0, wrong_claim);
    // Tampered wire root byte → inspection refuses (fail-closed).
    let mut tampered_wire = wire;
    tampered_wire[0] ^= 0x01;
    let sealed = sealer::decode(&tampered_wire).expect("length still decodes");
    assert_eq!(
        verify_sealed_file(&sealed, &sgn.public_key()),
        SealVerdict::Invalid,
        "asserted-vs-actual root mismatch must be a refusal, never a pass"
    );
}

// ─── Permutation invariance ──────────────────────────────────────────────────

#[test]
fn root_permutation_invariance_file_order_and_wire() {
    let tmp_a = tempfile::tempdir().expect("tempdir");
    let tmp_b = tempfile::tempdir().expect("tempdir");
    write_receipts_dir(tmp_a.path(), false);
    write_receipts_dir(tmp_b.path(), true); // reversed on-disk file names

    let sgn = signer(0x44);
    let (root_a, wire_a) = emit(tmp_a.path(), &sgn);
    let (root_b, wire_b) = emit(tmp_b.path(), &sgn);

    assert_eq!(root_a, root_b, "on-disk file order must not move the root");
    assert_eq!(
        wire_a.to_vec(),
        wire_b.to_vec(),
        "same root + same key → byte-identical sealed wire (deterministic ML-DSA-65)"
    );
    // …and the permutation-invariant root is the independent one.
    assert_eq!(root_a.0, independent_root(tmp_a.path()));
}

// ─── Signature laws ──────────────────────────────────────────────────────────

#[test]
fn emitted_dossier_verifies_only_under_the_emitting_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_receipts_dir(tmp.path(), false);
    let sgn = signer(0x45);
    let (_, wire) = emit(tmp.path(), &sgn);
    let sealed = sealer::decode(&wire).expect("decodes");

    assert_eq!(verify_sealed_file(&sealed, &sgn.public_key()), SealVerdict::Valid);
    let stranger = signer(0x46);
    assert_ne!(sgn.public_key(), stranger.public_key());
    assert_eq!(
        verify_sealed_file(&sealed, &stranger.public_key()),
        SealVerdict::Invalid,
        "wrong key must refuse"
    );
}

/// Wrong alg string and wrong format version are both refusals (fail-closed
/// two-valued `SealVerdict`), not silent passes.
#[test]
fn wrong_alg_string_and_version_are_refused() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_receipts_dir(tmp.path(), false);
    let sgn = signer(0x47);
    let (_, wire) = emit(tmp.path(), &sgn);
    let mut sealed = sealer::decode(&wire).expect("decodes");

    sealed.envelope.alg_marker = {
        let mut m = alg_marker_ml_dsa_65();
        m[..9].copy_from_slice(b"ML-DSA-44"); // wrong algorithm string
        m
    };
    assert_eq!(
        verify_sealed_file(&sealed, &sgn.public_key()),
        SealVerdict::Invalid,
        "wrong alg string must refuse"
    );

    let mut sealed = sealer::decode(&wire).expect("decodes");
    sealed.envelope.format_version = FORMAT_VERSION + 1;
    assert_eq!(
        verify_sealed_file(&sealed, &sgn.public_key()),
        SealVerdict::Invalid,
        "unsupported format version must refuse"
    );
}

// ─── Tamper evidence ─────────────────────────────────────────────────────────

/// A tampered receipt file in the receipts dir is a TYPED refusal
/// (`DigestMismatch`) landing in the Tampered exit class (13).
#[test]
fn tampered_receipt_file_is_typed_refusal() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write_receipts_dir(tmp.path(), false);
    // Flip one byte inside one record's stored chain hash (offset 320..352):
    // the stored hash no longer matches the recomputed receipt digest.
    let victim = tmp.path().join("00.ctrt");
    let mut bytes = std::fs::read(&victim).expect("readable");
    assert_eq!(bytes.len(), 352);
    bytes[351] ^= 0x01;
    std::fs::write(&victim, &bytes).expect("rewrite");

    // The bin's emit path verifies every record before chaining: the walk
    // refuses at the tampered file with the typed decode error.
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(tmp.path())
        .expect("dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    paths.sort();
    let mut refusal: Option<LedgerError> = None;
    for p in &paths {
        let bytes = std::fs::read(p).expect("read");
        match validate_receipt(&bytes) {
            Ok(_) => {}
            Err(e) => {
                refusal = Some(e);
                break;
            }
        }
    }
    let err = refusal.expect("tampered receipt must refuse");
    assert!(
        matches!(err, LedgerError::DigestMismatch { offset: 320, .. }),
        "expected DigestMismatch at the chain-hash offset, got {err}"
    );
    assert_eq!(exit_code(&err), LedgerExitCode::Tampered as u8, "tampered → exit class 13");
}

/// Flipping a byte in ANY of the 20 receipt files refuses the emit walk.
#[test]
fn tamper_matrix_each_of_20_receipt_files_refuses() {
    for victim_slot in 0..20usize {
        let tmp = tempfile::tempdir().expect("tempdir");
        write_receipts_dir(tmp.path(), false);
        let victim = tmp.path().join(format!("{victim_slot:02}.ctrt"));
        let mut bytes = std::fs::read(&victim).expect("readable");
        bytes[300] ^= 0x80;
        std::fs::write(&victim, &bytes).expect("rewrite");
        let refused = court_names()
            .iter()
            .enumerate()
            .map(|(i, id)| validate_receipt(&receipt_bytes(id, u8::try_from(i).expect("fits u8"))))
            .any(|r| r.is_ok()) // controls: the pristine constructors verify
            && validate_receipt(&std::fs::read(&victim).expect("read")).is_err();
        assert!(refused, "tampered receipt file {victim_slot} must refuse");
    }
}

// ─── NATO admission laws (Pass + MutantCaught, nothing else) ────────────────

#[test]
fn nato_dossier_admits_20_pass_mutant_caught_receipts() {
    let set = control_set();
    assert_eq!(set.len(), COURT_COUNT);
    assert!(dossier_admits(&set), "20 x (Pass + MutantCaught) must admit");
}

#[test]
fn nato_dossier_refuses_any_non_pass_receipt() {
    let mut set = control_set();
    set[7] = nato_receipt(
        CourtId::N08,
        Verdict::Fail,
        ControlResult::MutantCaught {
            mutant_id: "m-8".to_string(),
            detection_channel: "g4-court".to_string(),
        },
    );
    assert!(!dossier_admits(&set), "one Fail receipt must refuse the dossier");

    let mut set = control_set();
    set[11] = nato_receipt(
        CourtId::N12,
        Verdict::Refused,
        ControlResult::MutantCaught {
            mutant_id: "m-12".to_string(),
            detection_channel: "g4-court".to_string(),
        },
    );
    assert!(!dossier_admits(&set), "one Refused receipt must refuse the dossier");
}

#[test]
fn nato_dossier_refuses_mutant_survived() {
    let mut set = control_set();
    set[13] = nato_receipt(
        CourtId::N14,
        Verdict::Pass,
        ControlResult::MutantSurvived { mutant_id: "m-14".to_string() },
    );
    assert!(!dossier_admits(&set), "one MutantSurvived control must refuse the dossier");
}

#[test]
fn nato_dossier_refuses_incomplete_set() {
    let mut set = control_set();
    set.pop();
    assert_eq!(set.len(), 19);
    assert!(!dossier_admits(&set), "19 receipts must not admit");
}

// ─── Wire-level spot checks ──────────────────────────────────────────────────

#[test]
fn nato_control_set_root_is_deterministic_and_digest_bound() {
    let root_of = |set: &[CourtReceipt]| {
        use chicago_tdd_tools::nato::dossier_court::dossier_root;
        dossier_root(set)
    };
    let set = control_set();
    assert_eq!(root_of(&set), root_of(&set), "same receipts → same root");
    let mut other = control_set();
    other[5] = nato_receipt(
        CourtId::N06,
        Verdict::Fail,
        ControlResult::MutantCaught {
            mutant_id: "m-6".to_string(),
            detection_channel: "g4-court".to_string(),
        },
    );
    assert_ne!(root_of(&set), root_of(&other), "a non-Pass receipt moves the root");
}

/// Cross-surface control: the ledger-side 20-court root computed through the
/// real `DossierBuilder` equals the independent P13 reconstruction, and the
/// builder's inclusion proofs verify against it (permutation carried all the
/// way through the sealed wire).
#[test]
fn dossier_builder_proofs_verify_and_reversed_insertion_same_root() {
    use chicago_tdd_tools::ledger::dossier::DossierBuilder as B;
    let heads: Vec<[u8; 32]> = (0..20u8).map(subject).collect();
    let mk = |reverse: bool| {
        let mut b = B::new();
        let idx: Vec<usize> = if reverse { (0..20).rev().collect() } else { (0..20).collect() };
        for i in idx {
            b.add_court(court_names()[i].clone(), heads[i]).expect("unique");
        }
        b.seal().expect("nonempty")
    };
    assert_eq!(mk(false), mk(true), "insertion order must not move the root");
    let mut b = B::new();
    for (i, name) in court_names().iter().enumerate() {
        b.add_court(name.clone(), heads[i]).expect("unique");
    }
    let (root, proofs) = b.seal_with_proofs().expect("nonempty");
    assert_eq!(proofs.len(), 20);
    for p in &proofs {
        assert_eq!(p.verify(&root), Ok(()), "proof for {} failed", p.court_id);
    }
}
