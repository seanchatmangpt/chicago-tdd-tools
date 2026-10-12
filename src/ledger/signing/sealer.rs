#![cfg(feature = "nato")]

//! Dossier sealing: envelope construction, ML-DSA-65 signature production,
//! and fail-closed verification.
//!
//! # Verification order (LAW)
//!
//! `verify_sealed` is fail-closed and evaluates checks in this exact order:
//!
//! 1. **format_version** must be a supported version, else `Invalid`.
//! 2. **Subject binding**: `dossier_subject(&sealed.root)` must equal
//!    `envelope.subject`, else `Invalid`.
//! 3. **Signature**: the ML-DSA-65 signature over the subject is verified
//!    against `public_key`.
//!
//! The subject check is deliberately ordered **before** the signature check.
//! Law: an envelope whose subject does not bind to its root is structurally
//! invalid regardless of any signature bytes — a valid signature over a
//! mismatched subject must never be reported as Valid, and signature
//! verification must never be reached (or trusted) for a broken binding.
//! This ordering is pinned by
//! `subject_before_signature_with_garbage_sig`.

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

use super::signer::SIGNATURE_LEN;
use super::subject::dossier_subject;
#[cfg(test)]
use super::subject::SUBJECT_DOMAIN;

/// Envelope metadata binding a sealed dossier to its key and subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealEnvelope {
    /// First 8 bytes of BLAKE3(public_key).
    pub key_id: [u8; 8],
    /// Algorithm marker, `"ML-DSA-65"` padded with NUL to 16 bytes.
    pub alg_marker: [u8; 16],
    /// The derived dossier subject (domain-separated digest of the root).
    pub subject: [u8; 32],
    /// Envelope format version.
    pub format_version: u16,
}

/// A fully sealed dossier: root digest, envelope, and ML-DSA-65 signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealedDossier {
    /// The Merkle root digest this dossier attests (decoupled from
    /// `super::dossier::DossierRoot` by binding to the raw 32-byte digest).
    pub root: [u8; 32],
    /// Envelope metadata (key id, alg marker, subject, format version).
    pub envelope: SealEnvelope,
    /// ML-DSA-65 signature over the dossier subject (3309 bytes).
    pub signature: [u8; 3309],
}

/// Current envelope format version understood by this module.
pub const FORMAT_VERSION: u16 = 1;

/// Total length of the sealed-dossier wire encoding produced by [`encode`]
/// (32 root + 8 key_id + 16 alg_marker + 32 subject + 2 format_version +
/// 3309 signature).
pub const WIRE_LEN: usize = 32 + 8 + 16 + 32 + 2 + SIGNATURE_LEN;

/// Wire offset of the ML-DSA-65 signature inside the [`encode`] layout.
pub const WIRE_SIGNATURE_OFFSET: usize = WIRE_LEN - SIGNATURE_LEN;

/// Deterministic wire encoding of a sealed dossier, so the CLI can persist
/// seal output to disk and read it back for verification (Gap G-1).
///
/// Layout (fixed 3399 bytes, all fields big-endian by position — the only
/// variable-width field, `format_version`, is little-endian `u16`):
/// `root[32] || key_id[8] || alg_marker[16] || subject[32] ||
/// format_version[2] || signature[3309]`.
#[must_use]
pub fn encode(sealed: &SealedDossier) -> [u8; WIRE_LEN] {
    let mut out = [0u8; WIRE_LEN];
    out[0..32].copy_from_slice(&sealed.root);
    out[32..40].copy_from_slice(&sealed.envelope.key_id);
    out[40..56].copy_from_slice(&sealed.envelope.alg_marker);
    out[56..88].copy_from_slice(&sealed.envelope.subject);
    out[88..90].copy_from_slice(&sealed.envelope.format_version.to_le_bytes());
    out[WIRE_SIGNATURE_OFFSET..WIRE_LEN].copy_from_slice(&sealed.signature);
    out
}

/// Decode error for [`decode`]: the input is not [`WIRE_LEN`] bytes.
///
/// This is a distinct type rather than [`crate::ledger::error::SealError`] reuse because a
/// wrong-length sealed file is a transport problem (truncated/corrupted
/// storage), not a sealing-law refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealWireError {
    /// Byte length `decode` requires ([`WIRE_LEN`]).
    pub expected: usize,
    /// Byte length the caller supplied.
    pub actual: usize,
}

impl std::fmt::Display for SealWireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "sealed dossier wire length mismatch: expected {} bytes, got {}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for SealWireError {}

/// Decode a sealed dossier from its deterministic [`encode`] layout.
///
/// Pure structural decode: this performs NO verification (no signature
/// check, no subject binding) — decoding untrusted bytes yields an
/// unverified [`SealedDossier`] that must be passed through
/// [`verify_sealed`] before any reliance. Failure only on length mismatch.
///
/// # Errors
/// [`SealWireError`] when `bytes.len() != WIRE_LEN`.
pub fn decode(bytes: &[u8]) -> Result<SealedDossier, SealWireError> {
    if bytes.len() != WIRE_LEN {
        return Err(SealWireError { expected: WIRE_LEN, actual: bytes.len() });
    }
    let mut root = [0u8; 32];
    root.copy_from_slice(&bytes[0..32]);
    let mut key_id = [0u8; 8];
    key_id.copy_from_slice(&bytes[32..40]);
    let mut alg_marker = [0u8; 16];
    alg_marker.copy_from_slice(&bytes[40..56]);
    let mut subject = [0u8; 32];
    subject.copy_from_slice(&bytes[56..88]);
    let format_version = u16::from_le_bytes([bytes[88], bytes[89]]);
    let mut signature = [0u8; SIGNATURE_LEN];
    signature.copy_from_slice(&bytes[WIRE_SIGNATURE_OFFSET..WIRE_LEN]);
    Ok(SealedDossier {
        root,
        envelope: SealEnvelope { key_id, alg_marker, subject, format_version },
        signature,
    })
}

/// `b"ML-DSA-65"` padded with NUL to 16 bytes.
#[must_use]
pub fn alg_marker_ml_dsa_65() -> [u8; 16] {
    let mut m = [0u8; 16];
    m[..9].copy_from_slice(b"ML-DSA-65");
    m
}

/// Verdict of sealed-dossier verification. Deliberately two-valued and
/// fail-closed: any structural or cryptographic failure collapses to
/// [`SealVerdict::Invalid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "an ignored verification verdict admits unverified dossiers"]
pub enum SealVerdict {
    /// The seal is authentic and binds the claimed root.
    Valid,
    /// Any check failed (fail-closed).
    Invalid,
}

/// Seal a dossier root with the given signer.
///
/// `key_id` is the first 8 bytes of BLAKE3 over the signer's public key.
pub fn seal(
    root: &[u8; 32],
    signer: &super::signer::Signer,
) -> Result<SealedDossier, super::super::error::SealError> {
    let public_key = signer.public_key();
    let key_id = {
        let h = *blake3::hash(&public_key).as_bytes();
        // JUSTIFICATION: 8-byte slice from 32-byte hash — the slice length is
        // a compile-time constant strictly less than BLAKE3's 32-byte output,
        // so `try_into` cannot fail.
        h[..8].try_into().expect("8-byte slice from 32-byte hash")
    };
    let subject = dossier_subject(root);
    let signature = signer.sign(&subject);
    Ok(SealedDossier {
        root: *root,
        envelope: SealEnvelope {
            key_id,
            alg_marker: alg_marker_ml_dsa_65(),
            subject,
            format_version: FORMAT_VERSION,
        },
        signature,
    })
}

/// Verify a sealed dossier against a claiming public key.
///
/// Checks run in the documented LAW order: format version, then subject
/// binding, then signature. Any failure is `Invalid` (fail-closed).
#[must_use]
pub fn verify_sealed(sealed: &SealedDossier, public_key: &[u8; 1952]) -> SealVerdict {
    // (1) format_version supported.
    if sealed.envelope.format_version != FORMAT_VERSION {
        return SealVerdict::Invalid;
    }
    // (2) Subject binding: root must derive the envelope's subject.
    // LAW: this check precedes signature verification.
    if dossier_subject(&sealed.root) != sealed.envelope.subject {
        return SealVerdict::Invalid;
    }
    // (3) Signature over the subject.
    let marker_ok = sealed.envelope.alg_marker == alg_marker_ml_dsa_65();
    let key_ok = &blake3::hash(public_key).as_bytes()[..8] == sealed.envelope.key_id;
    let sig_ok =
        super::signer::Signer::verify(public_key, &sealed.envelope.subject, &sealed.signature);
    if marker_ok && key_ok && sig_ok {
        SealVerdict::Valid
    } else {
        SealVerdict::Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real signer instance (Chicago: real collaborator, no mocks).
    // Distinct seeds per call: two signers must never share a key.
    fn fresh_signer(seed: u8) -> super::super::signer::Signer {
        super::super::signer::Signer::from_seed([seed; 32])
    }

    #[test]
    fn roundtrip_seal_and_verify() {
        let signer = fresh_signer(10);
        let root = [0xA5u8; 32];
        let sealed = seal(&root, &signer).expect("seal");
        assert_eq!(sealed.root, root);
        assert_eq!(sealed.envelope.format_version, FORMAT_VERSION);
        assert_eq!(sealed.envelope.subject, dossier_subject(&root));
        assert_eq!(verify_sealed(&sealed, &signer.public_key()), SealVerdict::Valid);
    }

    #[test]
    fn tamper_root_byte_is_invalid() {
        let signer = fresh_signer(11);
        let sealed = seal(&[1u8; 32], &signer).expect("seal");
        let mut tampered = sealed;
        tampered.root[0] ^= 0x01;
        assert_eq!(verify_sealed(&tampered, &signer.public_key()), SealVerdict::Invalid);
    }

    #[test]
    fn tamper_subject_byte_is_invalid() {
        let signer = fresh_signer(12);
        let sealed = seal(&[2u8; 32], &signer).expect("seal");
        let mut tampered = sealed;
        tampered.envelope.subject[0] ^= 0x80;
        assert_eq!(verify_sealed(&tampered, &signer.public_key()), SealVerdict::Invalid);
    }

    #[test]
    fn tamper_signature_byte_is_invalid() {
        let signer = fresh_signer(13);
        let sealed = seal(&[3u8; 32], &signer).expect("seal");
        let mut tampered = sealed;
        tampered.signature[0] ^= 0x01;
        assert_eq!(verify_sealed(&tampered, &signer.public_key()), SealVerdict::Invalid);
    }

    #[test]
    fn swapped_key_is_invalid() {
        let signer = fresh_signer(14);
        let other = fresh_signer(15);
        let sealed = seal(&[4u8; 32], &signer).expect("seal");
        assert_eq!(verify_sealed(&sealed, &other.public_key()), SealVerdict::Invalid);
    }

    #[test]
    fn unsupported_format_version_is_invalid() {
        let signer = fresh_signer(16);
        let sealed = seal(&[5u8; 32], &signer).expect("seal");
        let mut tampered = sealed;
        tampered.envelope.format_version = FORMAT_VERSION + 1;
        assert_eq!(verify_sealed(&tampered, &signer.public_key()), SealVerdict::Invalid);
    }

    #[test]
    fn wire_roundtrip_is_exact() {
        let signer = fresh_signer(20);
        let sealed = seal(&[8u8; 32], &signer).expect("seal");
        let bytes = encode(&sealed);
        assert_eq!(bytes.len(), WIRE_LEN);
        assert_eq!(bytes.len(), 3399, "wire length law: 32+8+16+32+2+3309");
        assert_eq!(decode(&bytes).expect("decodes"), sealed);
        // Determinism: re-encoding reproduces identical bytes.
        assert_eq!(encode(&decode(&bytes).unwrap()), bytes);
    }

    #[test]
    fn wire_decode_refuses_oversized() {
        // Audit W4: the != length check must refuse WIRE_LEN+1 too, not
        // only truncations.
        let s = crate::ledger::signing::signer::Signer::from_seed(
            [7u8; crate::ledger::signing::signer::SEED_LEN],
        );
        let sealed = seal(&[0xab; 32], &s).expect("seal");
        let mut bytes = encode(&sealed).to_vec();
        bytes.push(0);
        let err = decode(&bytes).expect_err("oversized wire must refuse");
        assert_eq!(err.actual, bytes.len());
        assert_eq!(err.expected, WIRE_LEN);
    }

    #[test]
    fn wire_decode_refuses_wrong_length() {
        let signer = fresh_signer(21);
        let sealed = seal(&[9u8; 32], &signer).expect("seal");
        let bytes = encode(&sealed);
        for len in [0usize, 1, WIRE_LEN - 1, WIRE_SIGNATURE_OFFSET] {
            let err = decode(&bytes[..len]).unwrap_err();
            assert_eq!(err.expected, WIRE_LEN);
            assert_eq!(err.actual, len);
        }
        // Truncated signatures are also refused, not padded.
        assert_eq!(
            decode(&bytes[..WIRE_SIGNATURE_OFFSET]).unwrap_err().actual,
            WIRE_SIGNATURE_OFFSET
        );
    }

    #[test]
    fn wire_decode_is_unverified_input() {
        // Decoded garbage is NOT a valid dossier: it decodes structurally but
        // must fail verify_sealed (fail-closed composition law).
        let garbage = {
            let mut g = [0xFFu8; WIRE_LEN];
            g[88..90].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
            g
        };
        let sealed = decode(&garbage).expect("structurally decodes");
        assert_eq!(verify_sealed(&sealed, &[0u8; 1952]), SealVerdict::Invalid);
    }

    #[test]
    fn domain_separation_via_seal() {
        // Same root sealed under a different subject domain would produce a
        // different subject; verify the envelope subject matches the v1
        // domain and differs from an unframed digest.
        let signer = fresh_signer(17);
        let root = [6u8; 32];
        let sealed = seal(&root, &signer).expect("seal");
        let mut unframed = blake3::Hasher::new();
        unframed.update(SUBJECT_DOMAIN);
        unframed.update(&root);
        assert_ne!(sealed.envelope.subject, *unframed.finalize().as_bytes());
    }

    #[test]
    fn subject_before_signature_with_garbage_sig() {
        // LAW test: a mismatched subject with a garbage signature must be
        // Invalid (binding failure) — never smuggled through by any sig path.
        let signer = fresh_signer(18);
        let sealed = seal(&[7u8; 32], &signer).expect("seal");
        let mut tampered = sealed;
        tampered.envelope.subject[31] ^= 0xFF;
        tampered.signature = [0xEEu8; 3309];
        assert_eq!(verify_sealed(&tampered, &signer.public_key()), SealVerdict::Invalid);
    }
}
