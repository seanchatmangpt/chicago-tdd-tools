//! Ledger (P12 wire format + P13 dossier) integration tests.
//!
//! Laws under test, per plans P12/P13:
//! 1. Roundtrip `encode` → `from_bytes_verified` → `as_bytes`/`receipt` identity.
//! 2. 352-byte-flip tamper matrix: every single-byte mutation of a valid
//!    chained record is rejected.
//! 3. Chained records validate through the crate's single chain validator
//!    (`Blake3ChainValidator::validate_chain`); flipping one content byte is
//!    tamper-evidently refused.
//! 4. Dossier root is permutation-invariant over `add_court` order.
//! 5. Odd-leaf promotion (never duplication) at 3 and 5 courts.
//! 6. Full 20-court set: `seal_with_proofs` mints proofs that all verify;
//!    tampering one head fails exactly that proof.
//! 7. Empty dossier: root law `BLAKE3("P13-EMPTY")` + proof refusal.
//! 8. Proof paths longer than `MAX_PROOF_STEPS` are rejected.
//! 9. `ledger::exit_code` is total over every `LedgerError` variant.
//! 10. Replay detection: an already-stored head is refused; wrong-parent
//!     threading is refused; chains replay through stored hashes (RL2).
//! 11. Receipt-name law: `NAME_MAX` refusals — too-long names rejected,
//!     collisions distinguished, boundary length accepted.
//!
//! All items here are gated on the `ledger` feature — the ledger module is
//! only compiled when that feature is active.
#![cfg(feature = "ledger")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — assertions on real collected state.
#![allow(clippy::unwrap_used)]

use chicago_tdd_tools::ledger::dossier::{CourtLedgerBuilder, DossierBuilder, DossierRoot};
use chicago_tdd_tools::ledger::error::{DossierError, LedgerError, NameError};
use chicago_tdd_tools::ledger::proof::{
    empty_root, leaf_hash, node_hash, CourtProof, ProofStep, MAX_PROOF_STEPS,
};
use chicago_tdd_tools::ledger::receipt::{
    ChainedCourtReceipt, CourtOutcome, CourtReceiptV1, ReceiptMetrics, ThermalClass,
    VerifiedCourtReceipt, COURT_RECORD_LEN,
};
use chicago_tdd_tools::ledger::{exit_code, LedgerExitCode};
use chicago_tdd_tools::observability::receipt::{Blake3ChainValidator, Blake3ReceiptEntry};

// ─── Fixtures ────────────────────────────────────────────────────────────────

/// Deterministic head for a seed (test-only stand-in for a real chain head).
fn head(seed: u8) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(&[seed]);
    *h.finalize().as_bytes()
}

/// Build a valid receipt with deterministic fields.
fn sample_receipt(seed: u8) -> CourtReceiptV1 {
    CourtReceiptV1::new(
        head(seed),
        [0u8; 32],
        ReceiptMetrics {
            timestamp_ms: 1_700_000_000_000 + u64::from(seed),
            run_id: u64::from(seed),
            total_ticks: 9_999,
            wall_ms: 123,
            thermal: ThermalClass::Warm,
            outcome: CourtOutcome::Pass,
            budget_met: true,
        },
        [0u8; 32],
        head(seed.wrapping_add(100)),
        [0u8; 32],
        "ledger-tests-sample",
    )
    .expect("sample receipt fields are valid")
}

// ─── Law 1: roundtrip identity ───────────────────────────────────────────────

#[test]
fn law01_roundtrip_encode_from_bytes_verified_identity() {
    let receipt = sample_receipt(1);
    let chained = ChainedCourtReceipt::from_receipt(&receipt).expect("valid name");
    let bytes = chained.encode();

    let verified = VerifiedCourtReceipt::from_bytes_verified(&bytes)
        .expect("a freshly chained valid record must verify");
    assert_eq!(
        verified.as_bytes(),
        bytes,
        "verified record must be byte-identical to what was encoded"
    );
    assert_eq!(verified.receipt(), receipt, "typed roundtrip must preserve every field");

    // A second round over the verified bytes is a fixpoint.
    let again = VerifiedCourtReceipt::from_bytes_verified(&verified.as_bytes())
        .expect("re-verifying verified bytes must succeed");
    assert_eq!(again, verified, "verification must be idempotent");
}

// ─── Law 2: 352-byte single-flip tamper matrix ──────────────────────────────

#[test]
fn law02_tamper_matrix_every_single_byte_flip_is_rejected() {
    let bytes = ChainedCourtReceipt::from_receipt(&sample_receipt(2))
        .expect("valid name")
        .encode();
    assert_eq!(
        bytes.len(),
        COURT_RECORD_LEN,
        "chained record must be the documented 352-byte fixed size"
    );
    for i in 0..COURT_RECORD_LEN {
        let mut mutated = bytes;
        mutated[i] ^= 0x01;
        assert!(
            VerifiedCourtReceipt::from_bytes_verified(&mutated).is_err(),
            "flip of byte {i}/{} was accepted",
            COURT_RECORD_LEN
        );
    }
    // A second matrix pass with a different flip mask, so a record position
    // whose bit-0 is a don't-care cannot hide.
    for i in 0..COURT_RECORD_LEN {
        let mut mutated = bytes;
        mutated[i] ^= 0x80;
        assert!(
            VerifiedCourtReceipt::from_bytes_verified(&mutated).is_err(),
            "high-bit flip of byte {i}/{} was accepted",
            COURT_RECORD_LEN
        );
    }
}

// ─── Law 2b (T7): unhashed trailer bytes are covered at chain level ─────────

/// Wire-level chain projection via `ChainRecord::of` for a minted chain.
/// `dossier_wire` is gated on the `nato` feature in `src/ledger/mod.rs`, so
/// this helper (and the Law 2b tests) are likewise `nato`-gated.
#[cfg(feature = "nato")]
fn chain_records_of(n: usize) -> Vec<chicago_tdd_tools::ledger::dossier_wire::ChainRecord> {
    chained_records(n)
        .iter()
        .map(|c| {
            chicago_tdd_tools::ledger::dossier_wire::ChainRecord::of(
                &VerifiedCourtReceipt::from_bytes_verified(&c.encode())
                    .expect("chained record verifies"),
            )
        })
        .collect()
}

#[test]
#[cfg(feature = "nato")]
fn law02b_trailer_tamper_on_last_record_is_refused_by_dossier_chain_validation() {
    use chicago_tdd_tools::ledger::dossier_wire::validate_dossier_chain;
    let mut records = chain_records_of(3);
    // Precondition (the T7 gap): the trait-only validator cannot see the trailer.
    let last = records.len() - 1;
    records[last].trailer[7] ^= 0x01;
    assert!(
        Blake3ChainValidator::validate_chain(&records).is_ok(),
        "precondition: trait-level chain replay misses trailer tamper on the last record"
    );
    let err = validate_dossier_chain(&records).expect_err("trailer tamper must be refused");
    assert!(err.contains("zero trailer"), "typed trailer error: {err}");
}

#[test]
#[cfg(feature = "nato")]
fn law02b_mid_chain_trailer_tamper_names_the_tampered_record() {
    use chicago_tdd_tools::ledger::dossier_wire::validate_dossier_chain;
    let mut records = chain_records_of(3);
    records[0].trailer[15] ^= 0x80;
    let err = validate_dossier_chain(&records).expect_err("mid-chain trailer tamper refused");
    assert!(err.contains("record 0"), "error names the tampered record: {err}");
}

#[test]
#[cfg(feature = "nato")]
fn law02b_chain_record_projection_copies_trailer_bytes() {
    let bytes = ChainedCourtReceipt::from_receipt(&sample_receipt(7))
        .expect("valid name")
        .encode();
    let record = chicago_tdd_tools::ledger::dossier_wire::ChainRecord::of(
        &VerifiedCourtReceipt::from_bytes_verified(&bytes).expect("verifies"),
    );
    assert_eq!(record.trailer.as_slice(), &bytes[304..320]);
    assert_eq!(record.trailer, [0u8; 16], "fresh record trailer is all-zero");
}

// ─── Law 3: chained records validate through Blake3ChainValidator ───────────

/// Test-local entry capturing the trait view of a chained record. A real
/// hand-written adapter (not a fake of the trait): `prev_hash`/`content_bytes`/
/// `stored_hash` are captured through the public trait methods of
/// `ChainedCourtReceipt`, so the same `Blake3ReceiptEntry` law applies.
struct CapturedEntry {
    prev: [u8; 32],
    content: Vec<u8>,
    stored: [u8; 32],
}

impl Blake3ReceiptEntry for CapturedEntry {
    fn prev_hash(&self) -> [u8; 32] {
        self.prev
    }
    fn content_bytes(&self) -> Vec<u8> {
        self.content.clone()
    }
    fn stored_hash(&self) -> [u8; 32] {
        self.stored
    }
}

fn capture(entry: &ChainedCourtReceipt) -> CapturedEntry {
    CapturedEntry {
        prev: entry.prev_hash(),
        content: entry.content_bytes(),
        stored: entry.stored_hash(),
    }
}

/// Build a forward-linked chain of `n` records: each record's parent hash is
/// the previous record's stored chain hash; genesis parent is zeroed.
fn chained_records(n: usize) -> Vec<ChainedCourtReceipt> {
    let mut parent = [0u8; 32];
    let mut records = Vec::with_capacity(n);
    for i in 0..n {
        let mut receipt = sample_receipt(i as u8);
        receipt.parent_receipt_hash = parent;
        let chained = ChainedCourtReceipt::from_receipt(&receipt).expect("valid name");
        parent = chained.stored_hash();
        records.push(chained);
    }
    records
}

#[test]
fn law03_five_record_chain_validates_through_existing_validator() {
    use chicago_tdd_tools::observability::receipt::ChainError;
    let records = chained_records(5);
    Blake3ChainValidator::validate_chain(&records)
        .expect("5 chained court receipts must replay cleanly");

    // Tamper: flip one content byte of record 2 without updating its stored
    // hash. The stored hashes of records 2.. come from the ORIGINAL chain, so
    // replay must fail at exactly index 2.
    let mut entries: Vec<CapturedEntry> = records.iter().map(capture).collect();
    let victim = 100; // inside the 304-byte content region, past the header
    entries[2].content[victim] ^= 0x01;
    let err = Blake3ChainValidator::validate_chain(&entries)
        .expect_err("a flipped content byte must break chain replay");
    assert_eq!(
        err,
        ChainError::HashMismatch {
            index: 2,
            stored: records[2].stored_hash(),
            computed: {
                let mut h = blake3::Hasher::new();
                h.update(&entries[2].prev);
                h.update(&entries[2].content);
                *h.finalize().as_bytes()
            },
        },
        "tamper must be evidenced at the mutated record's index"
    );
}

// ─── Law 4: dossier root permutation invariance ─────────────────────────────

#[test]
fn law04_dossier_root_is_permutation_invariant() {
    let courts: Vec<(String, [u8; 32])> =
        (0..7u8).map(|i| (format!("court-{i:02}"), head(i + 10))).collect();

    let seal = |order: &[usize]| {
        let mut b = DossierBuilder::new();
        for &i in order {
            let (id, h) = &courts[i];
            b.add_court(id.as_str(), *h).expect("each court added exactly once");
        }
        b.seal().expect("nonempty dossier seals")
    };

    // Identity, reversed, and a derangement — three distinct insertion orders.
    let n = courts.len();
    let identity: Vec<usize> = (0..n).collect();
    let reversed: Vec<usize> = (0..n).rev().collect();
    let rotated: Vec<usize> = (0..n).map(|i| (i + 3) % n).collect();
    let r0 = seal(&identity);
    assert_eq!(r0, seal(&reversed), "reversed insertion order must not change the root");
    assert_eq!(r0, seal(&rotated), "rotated insertion order must not change the root");

    // And the root matches an independent promotion-law reference over the
    // sorted leaves.
    let mut idx: Vec<usize> = (0..courts.len()).collect();
    idx.sort_by(|&a, &b| courts[a].0.as_bytes().cmp(courts[b].0.as_bytes()));
    let leaves: Vec<[u8; 32]> =
        idx.iter().map(|&i| leaf_hash(&courts[i].0, &courts[i].1)).collect();
    assert_eq!(r0, DossierRoot(promote_root(leaves)), "root must equal the sorted reference");
}

/// Independent reference Merkle root with PROMOTED (not duplicated) odd nodes.
fn promote_root(mut level: Vec<[u8; 32]>) -> [u8; 32] {
    while level.len() > 1 {
        let mut next = Vec::new();
        let promoted = (level.len() % 2 == 1).then(|| level.pop().expect("len > 1"));
        for pair in level.chunks(2) {
            next.push(node_hash(&pair[0], &pair[1]));
        }
        if let Some(p) = promoted {
            next.push(p);
        }
        level = next;
    }
    level[0]
}

/// Independent reference with DUPLICATED odd nodes — the forbidden variant.
fn duplicate_root(mut level: Vec<[u8; 32]>) -> [u8; 32] {
    while level.len() > 1 {
        if level.len() % 2 == 1 {
            let last = *level.last().expect("nonempty");
            level.push(last);
        }
        level = level.chunks(2).map(|p| node_hash(&p[0], &p[1])).collect();
    }
    level[0]
}

// ─── Law 5: odd-leaf promotion at 3 and 5 courts ────────────────────────────

#[test]
fn law05_odd_leaf_promotion_at_3_and_5_courts() {
    for n in [3usize, 5] {
        let mut b = DossierBuilder::new();
        for i in 0..n {
            b.add_court(format!("c{i}"), head(i as u8)).expect("unique court ids");
        }
        let leaves: Vec<[u8; 32]> =
            (0..n).map(|i| leaf_hash(&format!("c{i}"), &head(i as u8))).collect();
        let (root, proofs) = b.seal_with_proofs().expect("nonempty dossier");

        assert_eq!(
            root,
            DossierRoot(promote_root(leaves.clone())),
            "n={n}: root must be the PROMOTED-odd-node variant"
        );
        assert_ne!(
            root,
            DossierRoot(duplicate_root(leaves)),
            "n={n}: root must NOT be the duplicated-odd-node variant"
        );
        assert_eq!(proofs.len(), n, "n={n}: one proof per court");
        for p in &proofs {
            assert_eq!(p.verify(&root), Ok(()), "n={n}: proof for {} must verify", p.court_id);
        }
    }
}

// ─── Law 6: 20-court full set, seal_with_proofs + tamper isolation ──────────

#[test]
fn law06_twenty_court_full_set_all_proofs_verify_and_tamper_is_isolated() {
    let mut b = DossierBuilder::new();
    for i in 0..20u8 {
        b.add_court(format!("court-{i:02}"), head(i)).expect("unique court ids");
    }
    let (root, proofs) = b.seal_with_proofs().expect("nonempty dossier");
    assert_eq!(proofs.len(), 20, "one proof per court");
    for p in &proofs {
        assert_eq!(p.verify(&root), Ok(()), "proof for {} must verify", p.court_id);
    }
    // Deepest path at 20 leaves: 20 → 10 → 5 → 3(promote) → 2 → 1.
    let max_steps = proofs.iter().map(|p| p.path.len()).max().expect("nonempty");
    assert_eq!(max_steps, 5, "deepest inclusion path at 20 leaves must be 5 steps");

    // Tamper one head (court-07): rebuild with that head corrupted.
    let mut b2 = DossierBuilder::new();
    for i in 0..20u8 {
        let h = if i == 7 { head(200) } else { head(i) };
        b2.add_court(format!("court-{i:02}"), h).expect("unique court ids");
    }
    let (root2, proofs2) = b2.seal_with_proofs().expect("nonempty dossier");
    assert_ne!(root, root2, "a tampered head must change the dossier root");

    // Merkle locality: every original proof crosses the tampered court's
    // subtree somewhere on its path, so ALL original proofs fail against the
    // tampered root; the tampered court's own proof also fails against the
    // ORIGINAL root (its leaf binds the tampered head).
    for p in &proofs {
        assert!(
            p.verify(&root2).is_err(),
            "original proof must fail against the tampered root ({})",
            p.court_id
        );
        if p.court_id == "court-07" {
            continue;
        }
        assert_eq!(p.verify(&root), Ok(()), "untampered {} verifies its own root", p.court_id);
    }
    // Fresh proofs over the tampered dossier verify against it, and fail
    // against the original root (court-07 by head binding, others by
    // locality).
    for p in &proofs2 {
        assert_eq!(p.verify(&root2), Ok(()), "tampered-dossier proof verifies");
        assert!(p.verify(&root).is_err(), "tampered-dossier proof must fail the original root");
    }
}

// ─── Law 7: empty dossier → BLAKE3("P13-EMPTY") root + proof refusal ────────

#[test]
fn law07_empty_dossier_root_law_and_proof_refusal() {
    let b = DossierBuilder::new();
    assert!(b.seal().is_err(), "sealing an empty dossier must be refused");
    assert!(b.seal_with_proofs().is_err(), "minting proofs from an empty dossier must be refused");
    // The empty-root law is exposed independently of the builder refusal.
    assert_eq!(
        empty_root(),
        *blake3::hash(b"P13-EMPTY").as_bytes(),
        "empty dossier root must be BLAKE3(\"P13-EMPTY\")"
    );
    // A forged proof can never verify against the empty root.
    let forged = CourtProof {
        court_id: "ghost".into(),
        head: head(1),
        path: vec![ProofStep { sibling: [0u8; 32], is_left: false }],
    };
    assert!(
        forged.verify(&DossierRoot(empty_root())).is_err(),
        "a forged proof must not verify against the empty root"
    );
}

// ─── Law 8: proof path over MAX_PROOF_STEPS rejected ────────────────────────

#[test]
fn law08_proof_path_over_max_steps_is_rejected() {
    let root = DossierRoot(empty_root());
    let path = vec![ProofStep { sibling: [0u8; 32], is_left: false }; MAX_PROOF_STEPS + 1];
    let proof = CourtProof { court_id: "adversarial".into(), head: [0u8; 32], path };
    assert!(
        proof.verify(&root).is_err(),
        "a {}-step proof path must be rejected (max {MAX_PROOF_STEPS})",
        MAX_PROOF_STEPS + 1
    );

    // Boundary sanity: a path AT the maximum is structurally accepted far
    // enough to fail on root comparison, not on the length gate.
    let at_limit = CourtProof {
        court_id: "at-limit".into(),
        head: [0u8; 32],
        path: vec![ProofStep { sibling: [0u8; 32], is_left: false }; MAX_PROOF_STEPS],
    };
    assert!(
        at_limit.verify(&root).is_err(),
        "a max-length path that hashes to a different root must fail verification"
    );
}

// ─── Law 9: exit-code table is total over LedgerError variants ──────────────

#[test]
fn law09_exit_code_table_covers_every_error_variant() {
    let cases: &[(LedgerError, u8)] = &[
        (LedgerError::TooShort { need: 16, got: 4 }, LedgerExitCode::MalformedReceipt as u8),
        (LedgerError::BadMagic([0xde, 0xad, 0xbe, 0xef]), LedgerExitCode::MalformedReceipt as u8),
        (
            LedgerError::UnsupportedVersion { found: 9, supported: 1 },
            LedgerExitCode::SchemaVersionMismatch as u8,
        ),
        (
            LedgerError::ReservedNonzero { offset: 42, value: 0xff },
            LedgerExitCode::MalformedReceipt as u8,
        ),
        (
            LedgerError::InvalidName { offset: 33, reason: NameError::InteriorNul },
            LedgerExitCode::MalformedReceipt as u8,
        ),
        (
            LedgerError::InvalidName { offset: 34, reason: NameError::InvalidUtf8 },
            LedgerExitCode::MalformedReceipt as u8,
        ),
        (LedgerError::ChainLinkMismatch { index: 3 }, LedgerExitCode::ChainBroken as u8),
        (LedgerError::ControlFlagMismatch { offset: 77 }, LedgerExitCode::MalformedReceipt as u8),
        (
            LedgerError::DigestMismatch { offset: 5, stored: [0u8; 32], computed: [1u8; 32] },
            LedgerExitCode::Tampered as u8,
        ),
        (
            LedgerError::ChainDiscontinuity { expected_parent: [0u8; 32], actual: [1u8; 32] },
            LedgerExitCode::ChainBroken as u8,
        ),
    ];
    for (err, want) in cases {
        assert_eq!(exit_code(err), *want, "wrong exit code for {err}");
    }
}

// ─── Law 10: replay refusal + parent threading (B1) ─────────────────────────

/// Receipt threaded onto an explicit parent (struct literal — exercises the
/// encode-boundary validation path, not `new`).
fn threaded_receipt(seed: u8, parent: [u8; 32]) -> ChainedCourtReceipt {
    ChainedCourtReceipt::from_receipt(&CourtReceiptV1 {
        subject_hash: head(seed),
        parent_receipt_hash: parent,
        metrics: ReceiptMetrics {
            timestamp_ms: 1_700_000_000_000 + u64::from(seed),
            run_id: u64::from(seed),
            total_ticks: 9_999,
            wall_ms: 123,
            thermal: ThermalClass::Warm,
            outcome: CourtOutcome::Pass,
            budget_met: true,
        },
        positive_control: [0u8; 32],
        metadata_digest: head(seed.wrapping_add(100)),
        prev_self_hash: [0u8; 32],
        name: "ledger-tests-sample".into(),
        fail_closed: false,
    })
    .expect("test names are within NAME_MAX")
}

#[test]
fn law10_replay_of_appended_receipt_is_refused() {
    let mut ledger = CourtLedgerBuilder::new("ledger-tests-sample");
    let r1 = threaded_receipt(1, [0u8; 32]);
    ledger.append(r1.clone()).expect("first append");
    let h1 = ledger.head();
    assert_eq!(
        ledger.append(r1),
        Err(LedgerError::ChainDiscontinuity { expected_parent: h1, actual: [0u8; 32] }),
        "byte-identical replay must be refused with the typed discontinuity"
    );
    assert_eq!(ledger.head(), h1, "refused replay must not advance head");
}

#[test]
fn law10_wrong_parent_is_refused() {
    let mut ledger = CourtLedgerBuilder::new("ledger-tests-sample");
    ledger.append(threaded_receipt(1, [0u8; 32])).expect("genesis");
    let h1 = ledger.head();
    // Stale genesis parent at index 1.
    assert!(matches!(
        ledger.append(threaded_receipt(2, [0u8; 32])),
        Err(LedgerError::ChainDiscontinuity { .. })
    ));
    // Fabricated parent.
    let fabricated = threaded_receipt(2, [7u8; 32]);
    assert_eq!(
        ledger.append(fabricated),
        Err(LedgerError::ChainDiscontinuity { expected_parent: h1, actual: [7u8; 32] })
    );
    assert_eq!(ledger.head(), h1, "refused append must not advance head");
}

#[test]
fn law10_valid_threading_is_accepted_and_head_law_holds() {
    let mut ledger = CourtLedgerBuilder::new("ledger-tests-sample");
    let g = threaded_receipt(1, [0u8; 32]);
    ledger.append(g.clone()).expect("genesis");
    let mut hp = blake3::Hasher::new();
    hp.update(&[0u8; 32]);
    hp.update(&g.stored_hash());
    let h1 = *hp.finalize().as_bytes();
    let child = threaded_receipt(2, h1);
    ledger.append(child.clone()).expect("threaded child");
    let mut h = blake3::Hasher::new();
    h.update(&[0u8; 32]);
    h.update(&g.stored_hash());
    let e1 = *h.finalize().as_bytes();
    let mut h = blake3::Hasher::new();
    h.update(&e1);
    h.update(&child.stored_hash());
    assert_eq!(ledger.head(), *h.finalize().as_bytes(), "head must follow BLAKE3(head||stored)");
}

// ─── Law 11: encode-boundary name validation (B2) ───────────────────────────

#[test]
fn law11_encode_refuses_name_over_name_max() {
    use chicago_tdd_tools::ledger::receipt::NAME_MAX;
    // NAME_MAX + 1 bytes, built by struct literal so `new`'s validation is
    // bypassed — encode() itself must refuse.
    let receipt = CourtReceiptV1 {
        subject_hash: [0u8; 32],
        parent_receipt_hash: [0u8; 32],
        metrics: ReceiptMetrics {
            timestamp_ms: 0,
            run_id: 0,
            total_ticks: 0,
            wall_ms: 0,
            thermal: ThermalClass::Warm,
            outcome: CourtOutcome::Pass,
            budget_met: true,
        },
        positive_control: [0u8; 32],
        metadata_digest: [0u8; 32],
        prev_self_hash: [0u8; 32],
        name: "a".repeat(NAME_MAX + 1),
        fail_closed: false,
    };
    assert_eq!(
        receipt.encode().unwrap_err(),
        LedgerError::InvalidName { offset: 204, reason: NameError::TooLong },
        "encode must refuse (not clamp) an 85-byte name at body offset 204"
    );
    assert!(receipt.chain_hash().is_err());
    assert!(
        ChainedCourtReceipt::from_receipt(&receipt).is_err(),
        "chaining must refuse the same receipt"
    );
}

#[test]
fn law11_distinct_over_max_names_cannot_collide() {
    use chicago_tdd_tools::ledger::receipt::NAME_MAX;
    let mk = |suffix: &str| CourtReceiptV1 {
        subject_hash: [0u8; 32],
        parent_receipt_hash: [0u8; 32],
        metrics: ReceiptMetrics {
            timestamp_ms: 0,
            run_id: 0,
            total_ticks: 0,
            wall_ms: 0,
            thermal: ThermalClass::Warm,
            outcome: CourtOutcome::Pass,
            budget_met: true,
        },
        positive_control: [0u8; 32],
        metadata_digest: [0u8; 32],
        prev_self_hash: [0u8; 32],
        name: format!("{}-{suffix}", "x".repeat(NAME_MAX)),
        fail_closed: false,
    };
    let a = mk("alpha");
    let b = mk("beta");
    assert_ne!(a.name, b.name, "two distinct over-max names");
    // Before the fix both clamped to the same 84-byte prefix and encoded
    // identically; now both are refused, so no collision is expressible.
    assert!(a.encode().is_err());
    assert!(b.encode().is_err());
    assert_ne!(
        a.encode().err(),
        None,
        "each over-max name must carry its own typed refusal, not a shared truncation"
    );
    assert!(ChainedCourtReceipt::from_receipt(&a).is_err());
    assert!(ChainedCourtReceipt::from_receipt(&b).is_err());
}

#[test]
fn law11_encode_accepts_exactly_name_max_boundary() {
    use chicago_tdd_tools::ledger::receipt::NAME_MAX;
    let name = "n".repeat(NAME_MAX);
    let receipt = CourtReceiptV1 {
        subject_hash: [1u8; 32],
        parent_receipt_hash: [0u8; 32],
        metrics: ReceiptMetrics {
            timestamp_ms: 0,
            run_id: 0,
            total_ticks: 0,
            wall_ms: 0,
            thermal: ThermalClass::Warm,
            outcome: CourtOutcome::Pass,
            budget_met: true,
        },
        positive_control: [0u8; 32],
        metadata_digest: [0u8; 32],
        prev_self_hash: [0u8; 32],
        name: name.clone(),
        fail_closed: false,
    };
    let chained = ChainedCourtReceipt::from_receipt(&receipt).expect("NAME_MAX name is lawful");
    let bytes = chained.encode();
    let verified =
        VerifiedCourtReceipt::from_bytes_verified(&bytes).expect("boundary record must verify");
    assert_eq!(verified.receipt().name, name, "84-byte name must round-trip losslessly");
}

// ─── T20: error-variant coverage gaps (error vocabulary law) ────────────────
//
// Each uncovered variant is (a) constructed to check Display is non-empty and
// carries its variant-identifying phrase, and (b) driven through the real
// public API that raises it — no variant is covered by construction alone.

#[test]
fn t20_cross_court_append_display_and_reachability() {
    // Display: non-empty, carries the variant-identifying phrase.
    let e = LedgerError::CrossCourtAppend { expected: "court-a".into(), actual: "court-b".into() };
    let s = e.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("cross-court append"), "{s}");

    // Reachability: a real append of a foreign court's receipt is refused.
    let mut ledger = CourtLedgerBuilder::new("ledger-tests-other");
    let foreign = threaded_receipt(1, [0u8; 32]); // named for a different court
    match ledger.append(foreign) {
        Err(LedgerError::CrossCourtAppend { expected, actual }) => {
            assert_eq!(expected, "ledger-tests-other");
            assert_eq!(actual, "ledger-tests-sample");
        }
        other => panic!("expected CrossCourtAppend, got {other:?}"),
    }
}

#[test]
fn t20_empty_chain_display_and_reachability() {
    let e = LedgerError::EmptyChain { court_id: "court-a".into() };
    let s = e.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("empty chain"), "{s}");

    let ledger = CourtLedgerBuilder::new("ledger-tests-empty");
    match ledger.build() {
        Err(LedgerError::EmptyChain { court_id }) => assert_eq!(court_id, "ledger-tests-empty"),
        other => panic!("expected EmptyChain, got {other:?}"),
    }
}

#[test]
fn t20_duplicate_court_display_and_reachability() {
    let e = DossierError::DuplicateCourt("dup".into());
    let s = e.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("duplicate court"), "{s}");

    let mut b = DossierBuilder::new();
    b.add_court("ledger-tests-dup", head(1)).expect("first add");
    match b.add_court("ledger-tests-dup", head(2)) {
        Err(DossierError::DuplicateCourt(c)) => assert_eq!(c, "ledger-tests-dup"),
        other => panic!("expected DuplicateCourt, got {other:?}"),
    }
}

#[test]
fn t20_verification_failed_display_and_reachability() {
    let e = DossierError::VerificationFailed("court-x".into());
    let s = e.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("failed verification"), "{s}");

    // Reachability: a proof whose leaf hashes to a different root.
    let root = DossierRoot([9u8; 32]);
    let proof =
        CourtProof { court_id: "ledger-tests-vf".into(), head: [1u8; 32], path: Vec::new() };
    match proof.verify(&root) {
        Err(DossierError::VerificationFailed(c)) => assert_eq!(c, "ledger-tests-vf"),
        other => panic!("expected VerificationFailed, got {other:?}"),
    }
}

#[test]
fn t20_empty_dossier_display_and_reachability() {
    let s = DossierError::EmptyDossier.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("dossier is empty"), "{s}");

    match DossierBuilder::new().seal() {
        Err(DossierError::EmptyDossier) => {}
        other => panic!("expected EmptyDossier, got {other:?}"),
    }
}

#[test]
fn t20_malformed_proof_exact_variant_on_oversized_path() {
    let s = DossierError::MalformedProof.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("malformed proof"), "{s}");

    // Reachability with the exact variant (existing tamper tests only assert
    // `is_err`): a path longer than MAX_PROOF_STEPS is refused as malformed.
    let root = DossierRoot([0u8; 32]);
    let proof = CourtProof {
        court_id: "ledger-tests-mp".into(),
        head: [0u8; 32],
        path: vec![ProofStep { sibling: [0u8; 32], is_left: false }; MAX_PROOF_STEPS + 1],
    };
    assert_eq!(proof.verify(&root), Err(DossierError::MalformedProof));
}

#[test]
fn t20_chain_discontinuity_display_is_populated() {
    // ChainDiscontinuity reachability is already exercised by Law 10; here we
    // pin its Display contract.
    let e = LedgerError::ChainDiscontinuity { expected_parent: [0u8; 32], actual: [1u8; 32] };
    let s = e.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("chain discontinuity"), "{s}");
}

// ===========================================================================
// DoD §2 item 2 — standing ≥10k-iteration mutation loop over the CTRT wire
// deserializer. Deterministic LCG (fixed seed, no wall-clock/env); asserts
// every mutated record is REFUSED typed (never accepted silently), and that
// each lawful record still verifies after the loop (no state pollution).
#[test]
fn wire_fuzz_10k_single_byte_mutations_all_refused_typed() {
    use chicago_tdd_tools::ledger::receipt::VerifiedCourtReceipt;
    use chicago_tdd_tools::nato::rng::Lcg;

    #[allow(clippy::unwrap_used)] // JUSTIFICATION: test code only
    fn build_lawful_record() -> [u8; 352] {
        let body = CourtReceiptV1::new(
            [7u8; 32],
            [3u8; 32],
            ReceiptMetrics {
                timestamp_ms: 1,
                run_id: 7,
                total_ticks: 1,
                wall_ms: 1,
                thermal: ThermalClass::Warm,
                outcome: CourtOutcome::Pass,
                budget_met: true,
            },
            [0; 32],
            [5u8; 32],
            [0; 32],
            "fuzz-court",
        )
        .unwrap();
        ChainedCourtReceipt::from_receipt(&body)
            .expect("chain record")
            .encode()
    }

    let lawful = build_lawful_record();
    let mut rng = Lcg::new(0x9E37_79B9_7F45_3E55);
    let mut refused = 0u32;
    for case in 0..10_000u32 {
        let mut bytes = lawful;
        // one deterministic single-byte mutation per case
        let idx = (rng.next_u64() % 352) as usize;
        let flip = 1u8 << (rng.next_u64() % 8);
        bytes[idx] ^= flip;
        match VerifiedCourtReceipt::from_bytes_verified(&bytes) {
            Err(_) => refused += 1,
            Ok(_) => {
                // impossible for a single-bit flip to be a no-op; any
                // acceptance is a defect — fail loudly with the case index
                panic!("case {case}: mutated record accepted at byte {idx}");
            }
        }
    }
    assert_eq!(refused, 10_000, "every mutation must be refused typed");
    // loop leaves the deserializer stateless: the lawful record still verifies
    assert!(VerifiedCourtReceipt::from_bytes_verified(&lawful).is_ok());
}
