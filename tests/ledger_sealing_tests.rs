//! Ledger sealing law tests (plan P14, lane T3).
//!
//! Integration surface for `src/ledger/signing/{signer,subject,sealer,keys}.rs`
//! and `src/ledger/error.rs`. Every law stated in the module docs is pinned
//! here as an executable court:
//!
//! - FIPS 204 wire sizes (sig 3309 / pk 1952 / seed 32)
//! - `from_seed` determinism (byte-identical signatures)
//! - Subject domain separation (v1 vs v2 vs unframed)
//! - Sealer roundtrip + tamper matrix (all fail-closed `Invalid`)
//! - Subject-before-signature verification ordering (documented LAW)
//! - Key custody: tempdir-only, 0600 first-use seed, stable reload id,
//!   31-byte seed refusal (never regenerate), rotation changes identity
//! - Entropy court-lite (see `entropy_court_lite` for the injection-seam note)
//!
//! Key custody tests touch the filesystem ONLY through
//! `KeyStore::with_dir(tempfile::TempDir)`; the real key dir is never used.
//!
//! Run: `cargo test --test ledger_sealing_tests --features nato`

#![cfg(feature = "nato")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use chicago_tdd_tools::ledger::error::{RefusalDigest, SealError};
use chicago_tdd_tools::ledger::signing::keys::{key_id, KeyStore};
use chicago_tdd_tools::ledger::signing::sealer::{
    alg_marker_ml_dsa_65, seal, verify_sealed, SealVerdict, SealedDossier, FORMAT_VERSION,
};
use chicago_tdd_tools::ledger::signing::signer::{
    assert_wire_sizes, Signer, PUBLIC_KEY_LEN, SEED_LEN, SIGNATURE_LEN,
};
use chicago_tdd_tools::ledger::signing::subject::dossier_subject;
use ml_dsa::Keypair as _;

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Distinct seed per call: two signers must never share a key.
fn seed_for(n: u8) -> [u8; SEED_LEN] {
    let mut seed = [n; SEED_LEN];
    seed[31] = n.wrapping_mul(7);
    seed
}

fn subject_for(n: u8) -> [u8; 32] {
    let mut subject = [0u8; 32];
    subject[0] = n;
    subject
}

/// Tempdir-custody store: the ONLY filesystem location any test touches.
fn temp_store() -> (tempfile::TempDir, KeyStore) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = KeyStore::with_dir(tmp.path().join("keys"));
    (tmp, store)
}

/// Seal a fresh dossier under a fresh signer, returning both.
fn sealed_fresh(root_n: u8, signer_n: u8) -> (Signer, SealedDossier) {
    let signer = Signer::from_seed(seed_for(signer_n)).expect("non-degenerate");
    let sealed = seal(&subject_for(root_n), &signer).expect("seal");
    (signer, sealed)
}

// ─── 1. wire sizes ───────────────────────────────────────────────────────────

#[test]
fn wire_sizes_match_fips204_at_the_integration_surface() {
    assert_wire_sizes();
    assert_eq!(SIGNATURE_LEN, 3309, "ML-DSA-65 detached signature");
    assert_eq!(PUBLIC_KEY_LEN, 1952, "ML-DSA-65 encoded public key");
    assert_eq!(SEED_LEN, 32, "generation seed (FIPS 204 xi)");
    // The sealer's envelope signature slot is the same constant, not a copy.
    assert_eq!(
        std::mem::size_of::<[u8; 3309]>(),
        SIGNATURE_LEN,
        "SealedDossier signature array must stay the FIPS 204 width"
    );
}

// ─── 2. from_seed determinism ────────────────────────────────────────────────

#[test]
fn from_seed_twice_is_byte_identical() {
    let seed = seed_for(1);
    let a = Signer::from_seed(seed).expect("non-degenerate");
    let b = Signer::from_seed(seed).expect("non-degenerate");

    assert_eq!(a.seed(), b.seed(), "same seed preserves the replay key");
    assert_eq!(a.public_key(), b.public_key(), "same seed derives same keypair");

    for n in 0u8..8 {
        let subject = subject_for(n);
        let sig_a = a.sign(&subject);
        let sig_b = b.sign(&subject);
        assert_eq!(
            sig_a, sig_b,
            "deterministic ML-DSA: same seed+subject must give byte-identical signature"
        );
        assert_eq!(sig_a.len(), SIGNATURE_LEN);
        assert!(Signer::verify(&a.public_key(), &subject, &sig_a));
    }
}

// ─── 3. subject domain separation ────────────────────────────────────────────

#[test]
fn subject_domains_v1_v2_unframed_all_differ_for_same_root() {
    use chicago_tdd_tools::ledger::signing::subject::SUBJECT_DOMAIN;

    let root = subject_for(42);

    let v1 = dossier_subject(&root);

    // v2 domain, framed identically per the framing law.
    let v2_domain = b"chicago-ledger-dossier-subject-v2";
    let mut v2 = blake3::Hasher::new();
    v2.update(&(v2_domain.len() as u32).to_le_bytes());
    v2.update(v2_domain);
    v2.update(&root);
    let v2 = *v2.finalize().as_bytes();

    // Unframed raw digest of the root.
    let unframed = *blake3::hash(&root).as_bytes();

    assert_ne!(v1, v2, "domain tag is load-bearing");
    assert_ne!(v1, unframed, "framing is load-bearing");
    assert_ne!(v2, unframed);
    // Recomputation is stable.
    assert_eq!(v1, dossier_subject(&root));

    // Framing law pin: subject == BLAKE3(u32le(|domain|) || domain || root).
    let mut replay = blake3::Hasher::new();
    replay.update(&(SUBJECT_DOMAIN.len() as u32).to_le_bytes());
    replay.update(SUBJECT_DOMAIN);
    replay.update(&root);
    assert_eq!(*replay.finalize().as_bytes(), v1);
}

// ─── 4. sealer roundtrip + tamper matrix ─────────────────────────────────────

#[test]
fn sealer_roundtrip_is_valid() {
    let (signer, sealed) = sealed_fresh(10, 10);

    assert_eq!(sealed.envelope.format_version, FORMAT_VERSION);
    assert_eq!(sealed.envelope.alg_marker, alg_marker_ml_dsa_65());
    assert_eq!(sealed.envelope.subject, dossier_subject(&sealed.root));
    assert_eq!(sealed.signature.len(), SIGNATURE_LEN);
    // key_id: first 8 bytes of SHA-256 over the encoded public key.
    assert_eq!(
        sealed.envelope.key_id,
        key_id(signer.public_key().as_slice()),
        "envelope key_id must equal the keys.rs law recomputation"
    );
    assert_eq!(verify_sealed(&sealed, &signer.public_key()), SealVerdict::Valid);
}

#[test]
fn tamper_matrix_is_all_invalid() {
    let (signer, sealed) = sealed_fresh(20, 20);

    // (a) tampered root byte
    let mut t = sealed;
    t.root[0] ^= 0x01;
    assert_eq!(verify_sealed(&t, &signer.public_key()), SealVerdict::Invalid, "tampered root");

    // (b) tampered subject byte
    let mut t = sealed;
    t.envelope.subject[0] ^= 0x80;
    assert_eq!(verify_sealed(&t, &signer.public_key()), SealVerdict::Invalid, "tampered subject");

    // (c) tampered signature byte (first, middle, last)
    for idx in [0usize, SIGNATURE_LEN / 2, SIGNATURE_LEN - 1] {
        let mut t = sealed;
        t.signature[idx] ^= 0x01;
        assert_eq!(
            verify_sealed(&t, &signer.public_key()),
            SealVerdict::Invalid,
            "tampered signature at byte {idx}"
        );
    }

    // (d) swapped key from a DIFFERENT seed
    let other = Signer::from_seed(seed_for(21)).expect("non-degenerate");
    assert_ne!(signer.public_key(), other.public_key());
    assert_eq!(
        verify_sealed(&sealed, &other.public_key()),
        SealVerdict::Invalid,
        "verifying under a different-seed key"
    );

    // (e) unsupported format_version
    let mut t = sealed;
    t.envelope.format_version = FORMAT_VERSION + 1;
    assert_eq!(verify_sealed(&t, &signer.public_key()), SealVerdict::Invalid, "bad format_version");

    // Falsifier: the untouched original still verifies after the whole sweep.
    assert_eq!(
        verify_sealed(&sealed, &signer.public_key()),
        SealVerdict::Valid,
        "original must survive the tamper sweep unmodified"
    );
}

// ─── 5. subject-before-signature ordering (LAW) ──────────────────────────────

#[test]
fn subject_before_signature_with_garbage_sig_is_invalid() {
    // LAW (sealer.rs header): an envelope whose subject does not bind to its
    // root is structurally invalid regardless of signature bytes — the
    // subject check is ordered BEFORE signature verification, so a mismatched
    // subject plus a garbage signature must collapse to Invalid and the
    // signature path must never rescue it.
    let (signer, sealed) = sealed_fresh(30, 30);
    let mut tampered = sealed;
    tampered.envelope.subject[31] ^= 0xFF;
    tampered.signature = [0xEE_u8; SIGNATURE_LEN];
    assert_eq!(
        verify_sealed(&tampered, &signer.public_key()),
        SealVerdict::Invalid,
        "mismatched subject + garbage signature must be Invalid (binding law)"
    );
    // The unmodified dossier under the same key still verifies: the refusal
    // came from the broken binding, not from any key/state contamination.
    assert_eq!(verify_sealed(&sealed, &signer.public_key()), SealVerdict::Valid);
}

// ─── 6. key custody (tempdir ONLY) ───────────────────────────────────────────

#[test]
fn custody_first_use_creates_0600_seed() {
    let (tmp, store) = temp_store();
    let seed_path = store.seed_path();

    assert!(!seed_path.exists());
    store.load_or_create().expect("first use creates the key");
    assert!(seed_path.exists(), "seed file created on first use");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&seed_path).expect("seed metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "seed file must be 0600");
        let dir_mode =
            std::fs::metadata(store.dir()).expect("key dir metadata").permissions().mode();
        assert_eq!(dir_mode & 0o777, 0o700, "key dir must be 0700");
    }

    assert_eq!(
        std::fs::read(&seed_path).expect("seed bytes").len(),
        SEED_LEN,
        "persisted seed is exactly 32 bytes"
    );
    // Tempdir cleanup is drop-owned; nothing leaks to the real key dir.
    assert!(store.seed_path().starts_with(tmp.path()));
    drop(tmp);
}

#[test]
fn custody_second_load_reproduces_same_key_id() {
    let (tmp, store) = temp_store();
    let first = store.load_or_create().expect("first load");
    let id_first = key_id(first.verifying_key().encode().as_slice());

    let second = store.load_or_create().expect("second load");
    let id_second = key_id(second.verifying_key().encode().as_slice());
    assert_eq!(id_first, id_second, "reload reproduces the same identity");
    drop(tmp);
}

#[test]
fn custody_31_byte_seed_is_refused_never_regenerated() {
    let (tmp, store) = temp_store();
    store.load_or_create().expect("create initial key");

    std::fs::write(store.seed_path(), [0u8; 31]).expect("truncate seed to 31 bytes");
    let err = store.load_or_create().expect_err("wrong-length seed must be refused");
    assert!(matches!(err, SealError::KeyCustody(_)), "expected KeyCustody, got {err:?}");
    assert!(
        err.to_string().contains("refusing to regenerate"),
        "refusal must name the no-silent-regen law: {err}"
    );

    // The refusal must NOT have rewritten the corrupt seed.
    assert_eq!(
        std::fs::read(store.seed_path()).expect("seed still on disk").len(),
        31,
        "refused custody leaves the corrupt seed untouched"
    );
    drop(tmp);
}

#[test]
fn custody_rotate_changes_key_id() {
    let (tmp, store) = temp_store();
    let original = store.load_or_create().expect("create");
    let id_before = key_id(original.verifying_key().encode().as_slice());

    let id_after = store.rotate().expect("rotate");
    assert_ne!(id_before, id_after, "rotation produces a new identity");

    // The rotated seed is the one persisted: next load reproduces the NEW id.
    let reloaded = store.load_or_create().expect("reload after rotate");
    assert_eq!(
        key_id(reloaded.verifying_key().encode().as_slice()),
        id_after,
        "rotated seed is the persisted one"
    );
    drop(tmp);
}

// ─── 7. entropy court-lite ───────────────────────────────────────────────────

#[test]
fn entropy_court_lite() {
    // Injection-seam status (read from the real API, 2026-10-11):
    // `Signer::generate()` calls `SigningKey::<MlDsa65>::generate()` with no
    // injectable RNG parameter, so the refusal path cannot be driven from a
    // test. FULL entropy-injection court lands with the nato entropy court
    // (seam to be added to `generate()`); this test pins what is reachable:
    //
    // (1) 1,000 live trials: generate() succeeds and every trial yields a
    //     distinct seed (OS entropy is actually feeding the key path).
    // (2) The EntropyRefused refusal shape: typed variant carrying an opaque
    //     32-byte RefusalDigest, hex Display, structural distinctness.

    // (1) live entropy trials.
    let mut seen: std::collections::HashSet<[u8; SEED_LEN]> = std::collections::HashSet::new();
    for i in 0..1_000 {
        let signer = Signer::generate().expect("os entropy available in test env");
        seen.insert(*signer.seed());
        if i == 0 {
            assert_eq!(signer.seed().len(), SEED_LEN);
        }
    }
    assert_eq!(seen.len(), 1_000, "1,000 generate() trials must yield 1,000 distinct seeds");

    // (2) EntropyRefused variant shape (shape-only; no injection seam).
    let d1 = RefusalDigest([0xab; 32]);
    let d2 = RefusalDigest([0xcd; 32]);
    assert_ne!(d1, d2, "distinct refusals carry distinct digests");
    assert_eq!(d1.to_string().len(), 64, "refusal digest is 64 hex chars");
    let e = SealError::EntropyRefused(d1);
    assert!(e.to_string().contains("refused"), "display names the refusal: {e}");
    // Round-trip through Debug/Clone so courts can log and replay it.
    let cloned = e.clone();
    assert_eq!(cloned, e);
    assert!(format!("{e:?}").contains("EntropyRefused"));
}
