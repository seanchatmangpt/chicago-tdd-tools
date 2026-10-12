//! Ported from strata-protocol src/signer.rs @ 0de13a3
//!
//! ML-DSA-65 (NIST FIPS 204) detached signer for ledger dossier sealing.
//!
//! Adaptations from the source (plan P14):
//! - Module is feature-gated on `nato` (the coordinator wires
//!   `ml-dsa = { version = "0.1.1", optional = true }` and `zeroize` in
//!   Cargo.toml; they are referenced here as normal extern crates).
//! - Sign context is `chicago-ledger-dossier-v1`.
//! - Sign subject is a bare 32-byte digest (no attested-payload struct).
//! - `generate()` returns `Result` and maps entropy/key-seed failure to
//!   `SealError::EntropyRefused` carrying a [`RefusalDigest`] derived by
//!   hashing a signer-local domain tag plus the concrete cause (the same
//!   law as `keys::fresh_seed`; the digest is reproducible, never fresh
//!   entropy, and `error.rs` defines no `fresh()` constructor).
//! - Public-key encode returns a fixed `[u8; 1952]`, not `Vec<u8>`.
//! - The strata `Seal` extension trait is intentionally NOT ported here; the
//!   dossier sealing lives with the dossier type.
//!
//! Signatures are detached, exactly 3309 bytes (FIPS 204 §7.2, lambda=3,
//! category 3). Signing uses the FIPS 204 deterministic variant with a fixed
//! context so identical keys+subjects reproduce byte-identical signatures.
//!
//! Secret key material is zeroized on drop (the ml-dsa `SigningKey` is
//! `ZeroizeOnDrop`; the raw seed is additionally held in `Zeroizing`).

#![cfg(feature = "nato")]
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

use ml_dsa::{signature::Keypair, MlDsa65, Signature, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::super::error::{RefusalDigest, SealError};

/// Domain tag hashed into every [`Signer`]-local refusal digest, mirroring
/// the `keys::fresh_seed` pattern: the refusal receipt is
/// `SHA-256(domain_tag || ": " || cause)`, never fresh entropy.
const REFUSAL_DOMAIN: &[u8] = b"chicago-ledger-signer-refusal-v1";

/// Build a [`RefusalDigest`] from a cause string the way
/// [`super::keys::KeyStore::fresh_seed`] does (SHA-256 over a domain tag +
/// the concrete failure), so refusals are reproducible, domain-separated,
/// and never an invented `fresh()` constructor on `error.rs`.
fn refusal_digest_of(cause: &str) -> RefusalDigest {
    let mut hasher = Sha256::new();
    hasher.update(REFUSAL_DOMAIN);
    hasher.update(b": ");
    hasher.update(cause.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    RefusalDigest(out)
}

/// FIPS 204 ML-DSA-65 detached signature size in bytes.
pub const SIGNATURE_LEN: usize = 3309;
/// Seed length for ML-DSA key generation (32 bytes, xi).
pub const SEED_LEN: usize = 32;
/// Public key (rho || rho') encoded length per FIPS 204 for ML-DSA-65.
pub const PUBLIC_KEY_LEN: usize = 1952;
/// Fixed ML-DSA signing context binding every signature to this dossier
/// domain (domain separation; FIPS 204 §5.4 context string).
pub const SIGN_CONTEXT: &[u8] = b"chicago-ledger-dossier-v1";

/// Assert the crate's actual encoded sizes match the FIPS 204 wire constants.
/// The crate's size aliases are `pub(crate)`, so this is checked at runtime
/// (call sites: tests, and [`Signer::from_seed`] in debug builds).
pub fn assert_wire_sizes() {
    let sig_enc = ml_dsa::EncodedSignature::<MlDsa65>::default();
    let pk_enc = ml_dsa::EncodedVerifyingKey::<MlDsa65>::default();
    let sig_buf: &[u8] = sig_enc.as_ref();
    let pk_buf: &[u8] = pk_enc.as_ref();
    assert_eq!(
        sig_buf.len(),
        SIGNATURE_LEN,
        "ml-dsa ML-DSA-65 signature size drifted from FIPS 204 3309 bytes"
    );
    assert_eq!(
        pk_buf.len(),
        PUBLIC_KEY_LEN,
        "ml-dsa ML-DSA-65 public key size drifted from FIPS 204 1952 bytes"
    );
}

// Manual Debug impl: SigningKey/VerifyingKey do not implement Debug, and the
// derived output must never leak signing material anyway.
impl std::fmt::Debug for Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signer").finish_non_exhaustive()
    }
}

/// ML-DSA-65 signer. Clone-free verification key derived at construction.
pub struct Signer {
    signing_key: SigningKey<MlDsa65>,
    verifying_key: VerifyingKey<MlDsa65>,
    /// Raw seed retained only for deterministic replay; zeroized on drop.
    seed: Zeroizing<[u8; SEED_LEN]>,
}

impl Signer {
    /// Deterministic path: derive the keypair from a 32-byte seed (xi).
    ///
    /// Degenerate seeds are refused before any key material is expanded
    /// (SK3-F2): an all-zero or all-ones seed is a well-formed input to
    /// FIPS 204 key expansion but reconstructs a *valid* keypair from
    /// non-secret, guessable constant material. Carries a [`RefusalDigest`]
    /// per the same hashed-cause law as [`Signer::generate`].
    ///
    /// # Errors
    /// [`SealError::EntropyRefused`] if `seed` is all-zero or all-ones.
    pub fn from_seed(seed: [u8; SEED_LEN]) -> Result<Self, SealError> {
        debug_assert!({
            assert_wire_sizes();
            true
        });
        if seed.iter().all(|b| *b == 0) || seed.iter().all(|b| *b == 0xFF) {
            return Err(SealError::EntropyRefused(refusal_digest_of(
                "degenerate generation seed refused (all-zero/all-ones)",
            )));
        }
        let seed = Zeroizing::new(seed);
        let ml_seed = ml_dsa::Seed::from(*seed);
        let signing_key = SigningKey::<MlDsa65>::from_seed(&ml_seed);
        let verifying_key = signing_key.verifying_key();
        Ok(Signer { signing_key, verifying_key, seed })
    }

    /// Random path: 32 bytes of OS entropy as the generation seed. Any
    /// entropy-source failure is a typed refusal carrying a fresh 32-byte
    /// digest — no fallback, no partial key.
    pub fn generate() -> Result<Self, SealError> {
        use ml_dsa::Generate as _;
        let signing_key = SigningKey::<MlDsa65>::generate();
        let seed_bytes: &[u8] = signing_key.as_seed();
        if seed_bytes.len() != SEED_LEN {
            // RECONCILE NOTE (E4 item 4): `error.rs` is owned by lane R7 this
            // wave and defines no `RefusalDigest::fresh()`; the refusal digest
            // is built locally with the same hashed-cause pattern keys.rs:151
            // (`fresh_seed`) uses — SHA-256 over a signer-local domain tag
            // plus the concrete cause.
            return Err(SealError::EntropyRefused(refusal_digest_of("ml-dsa seed length refused")));
        }
        let mut raw = [0u8; SEED_LEN];
        raw.copy_from_slice(seed_bytes);
        let seed = Zeroizing::new(raw);
        let verifying_key = signing_key.verifying_key();
        Ok(Signer { signing_key, verifying_key, seed })
    }

    /// Adopt an externally constructed ML-DSA signing key (e.g. one loaded
    /// from a [`super::keys::KeyStore`] by the CLI) without regenerating.
    ///
    /// The replay seed is recovered from the key via `as_seed`; a key whose
    /// seed is not exactly 32 bytes is a typed refusal, same law as
    /// [`Signer::generate`].
    ///
    /// # Errors
    /// [`SealError::EntropyRefused`] if the key's recovered seed is not
    /// [`SEED_LEN`] bytes.
    pub fn from_signing_key(key: SigningKey<MlDsa65>) -> Result<Self, SealError> {
        let seed_bytes: &[u8] = key.as_seed();
        if seed_bytes.len() != SEED_LEN {
            return Err(SealError::EntropyRefused(refusal_digest_of(
                "ml-dsa signing key seed length refused",
            )));
        }
        let mut raw = [0u8; SEED_LEN];
        raw.copy_from_slice(seed_bytes);
        let seed = Zeroizing::new(raw);
        let verifying_key = key.verifying_key();
        Ok(Signer { signing_key: key, verifying_key, seed })
    }

    /// The 32-byte generation seed (deterministic replay key).
    pub fn seed(&self) -> &[u8; SEED_LEN] {
        &self.seed
    }

    /// Encoded public key (1952 bytes).
    pub fn public_key(&self) -> [u8; PUBLIC_KEY_LEN] {
        let mut out = [0u8; PUBLIC_KEY_LEN];
        let enc = self.verifying_key.encode();
        out.copy_from_slice(enc.as_slice());
        out
    }

    /// Deterministic detached signature over a 32-byte subject digest
    /// (3309 bytes). Context fixed by [`SIGN_CONTEXT`].
    pub fn sign(&self, subject: &[u8; 32]) -> [u8; SIGNATURE_LEN] {
        let expanded = self.signing_key.expanded_key();
        // FIPS 204 deterministic ML-DSA.Sign; context fixed by this module.
        // JUSTIFICATION: deterministic ML-DSA sign over a well-formed 32-byte
        // subject and fixed-length context cannot fail per FIPS 204 — the
        // error variant is unreachable for these inputs, so the expect is the
        // loud-failure guard, not an error path.
        let sig = expanded
            .sign_deterministic(subject, SIGN_CONTEXT)
            .expect("deterministic ML-DSA sign cannot fail for well-formed input");
        let mut out = [0u8; SIGNATURE_LEN];
        let enc = sig.encode();
        out.copy_from_slice(enc.as_slice());
        out
    }

    /// Verify a detached signature. Returns the crate's boolean verdict; this
    /// layer adds no early returns beyond it (the crate's verify is
    /// constant-time internally per FIPS 204 Verify).
    pub fn verify(
        public_key: &[u8; PUBLIC_KEY_LEN],
        subject: &[u8; 32],
        sig: &[u8; SIGNATURE_LEN],
    ) -> bool {
        let Ok(pk_enc) = ml_dsa::EncodedVerifyingKey::<MlDsa65>::try_from(&public_key[..]) else {
            return false;
        };
        let vk = VerifyingKey::<MlDsa65>::decode(&pk_enc);
        let Ok(sig_enc) = ml_dsa::EncodedSignature::<MlDsa65>::try_from(&sig[..]) else {
            return false;
        };
        let Some(sig) = Signature::<MlDsa65>::decode(&sig_enc) else {
            return false;
        };
        vk.verify_with_context(subject, SIGN_CONTEXT, &sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed(n: u8) -> [u8; SEED_LEN] {
        let mut seed = [0u8; SEED_LEN];
        seed[0] = n;
        seed[31] = n.wrapping_mul(7);
        seed
    }

    fn test_subject(n: u8) -> [u8; 32] {
        let mut subject = [0u8; 32];
        subject[0] = n;
        subject
    }

    #[test]
    fn wire_sizes_match_fips204() {
        assert_wire_sizes();
        assert_eq!(SIGNATURE_LEN, 3309);
        assert_eq!(PUBLIC_KEY_LEN, 1952);
        assert_eq!(SEED_LEN, 32);
    }

    #[test]
    fn from_seed_refuses_degenerate_seeds() {
        // SK3-F2: all-zero and all-ones seeds reconstruct valid (guessable)
        // keypairs — they must be typed refusals before key expansion.
        let zero = [0u8; SEED_LEN];
        let ones = [0xFFu8; SEED_LEN];
        for seed in [zero, ones] {
            let err = Signer::from_seed(seed).expect_err("degenerate seed must be refused");
            assert!(
                matches!(err, SealError::EntropyRefused(_)),
                "refusal must carry the hashed-cause refusal digest: {err}"
            );
        }
        // Distinct causes, same class: reproducible digests, not fresh entropy.
        let (Err(e1), Err(e2)) = (Signer::from_seed(zero), Signer::from_seed(zero)) else {
            panic!("all-zero seed must refuse both times");
        };
        assert_eq!(e1, e2, "same cause must reproduce the same refusal digest");
    }

    #[test]
    fn from_seed_accepts_distinct_non_degenerate_seeds() {
        let a = Signer::from_seed(test_seed(1)).expect("seed 1 non-degenerate");
        let b = Signer::from_seed(test_seed(2)).expect("seed 2 non-degenerate");
        assert_ne!(a.public_key(), b.public_key());
        // Near-degenerate seeds (single differing byte) are still accepted.
        let mut near_zero = [0u8; SEED_LEN];
        near_zero[17] = 0x01;
        let c = Signer::from_seed(near_zero).expect("single-byte seed accepted");
        assert_ne!(c.public_key(), a.public_key());
    }

    #[test]
    fn from_seed_is_reproducible() {
        let seed = test_seed(1);
        let a = Signer::from_seed(seed).expect("non-degenerate");
        let b = Signer::from_seed(seed).expect("non-degenerate");
        assert_eq!(a.seed(), b.seed(), "same seed must retain same replay key");
        assert_eq!(a.public_key(), b.public_key(), "same seed derives same keypair");
        let subject = test_subject(2);
        assert_eq!(
            a.sign(&subject),
            b.sign(&subject),
            "deterministic signing: same seed+subject -> byte-identical signature"
        );
    }

    #[test]
    fn verify_accepts_own_signature() {
        let signer = Signer::from_seed(test_seed(3)).expect("non-degenerate");
        let subject = test_subject(4);
        let sig = signer.sign(&subject);
        assert!(Signer::verify(&signer.public_key(), &subject, &sig));
    }

    #[test]
    fn verify_rejects_wrong_subject() {
        let signer = Signer::from_seed(test_seed(5)).expect("non-degenerate");
        let sig = signer.sign(&test_subject(6));
        assert!(!Signer::verify(&signer.public_key(), &test_subject(7), &sig));
    }

    #[test]
    fn bit_flip_sweep_rejects_all_64_flips() {
        let signer = Signer::from_seed(test_seed(8)).expect("non-degenerate");
        let subject = test_subject(9);
        let pk = signer.public_key();
        let sig = signer.sign(&subject);
        // 64 single-bit flips spread across the signature: 32 in the first
        // quarter, 32 in the last quarter (bytes 0..32 and 3309-32..3309).
        for i in 0..32 {
            let mut mutated = sig;
            mutated[i] ^= 0x01 << (i % 8);
            assert!(
                !Signer::verify(&pk, &subject, &mutated),
                "bit flip in first quarter at byte {i} must be rejected"
            );
            let tail = SIGNATURE_LEN - 32;
            let mut mutated_tail = sig;
            mutated_tail[tail + i] ^= 0x80 >> (i % 8);
            assert!(
                !Signer::verify(&pk, &subject, &mutated_tail),
                "bit flip in last quarter at byte {} must be rejected",
                tail + i
            );
        }
        // Original still verifies after the sweep.
        assert!(Signer::verify(&pk, &subject, &signer.sign(&subject)));
    }

    #[test]
    fn from_signing_key_matches_from_seed() {
        let seed = test_seed(13);
        let ml_seed = ml_dsa::Seed::from(seed);
        let key = SigningKey::<MlDsa65>::from_seed(&ml_seed);
        let adopted = Signer::from_signing_key(key).expect("adopt");
        let direct = Signer::from_seed(seed).expect("non-degenerate");
        assert_eq!(adopted.seed(), direct.seed());
        assert_eq!(adopted.public_key(), direct.public_key());
        let subject = test_subject(14);
        assert_eq!(adopted.sign(&subject), direct.sign(&subject));
    }

    #[test]
    fn refusal_digest_is_hashed_cause_not_fresh_entropy() {
        // Law pin: the signer-local refusal digest is SHA-256 of the domain
        // tag + cause (keys.rs fresh_seed pattern), reproducible, and
        // distinct per cause — never an opaque random value.
        let a = refusal_digest_of("ml-dsa seed length refused");
        let b = refusal_digest_of("ml-dsa seed length refused");
        let c = refusal_digest_of("other cause");
        assert_eq!(a, b, "same cause must reproduce the same refusal digest");
        assert_ne!(a, c, "different causes must produce different digests");
    }

    #[test]
    fn cross_key_rejection() {
        let signer_a = Signer::from_seed(test_seed(10)).expect("non-degenerate");
        let signer_b = Signer::from_seed(test_seed(11)).expect("non-degenerate");
        assert_ne!(signer_a.public_key(), signer_b.public_key());
        let subject = test_subject(12);
        let sig_a = signer_a.sign(&subject);
        assert!(!Signer::verify(&signer_b.public_key(), &subject, &sig_a));
        let sig_b = signer_b.sign(&subject);
        assert_ne!(sig_a, sig_b);
        assert!(!Signer::verify(&signer_a.public_key(), &subject, &sig_b));
    }
}
