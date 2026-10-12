#![cfg(any(feature = "ledger", feature = "nato"))]
//! P15 ledger CLI-facing domain surface.
//!
//! This module is the single entry point the CLI consumes. It composes the
//! landed subsystems — it never reimplements a hash law:
//!
//! - receipt decoding/verification delegates to
//!   `receipt::VerifiedCourtReceipt::from_bytes_verified`;
//! - chain replay delegates to
//!   [`crate::observability::receipt::Blake3ChainValidator`] (the one BLAKE3
//!   chaining implementation in the crate);
//! - dossier sealing composes `dossier::DossierBuilder::seal` with
//!   `signing::sealer::seal` (ML-DSA-65).
//!
//! [`LedgerExitCode`] is the single error→process-exit mapping the CLI
//! consumes; [`exit_code`] is total over [`LedgerError`].

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

pub mod error;

#[cfg(feature = "ledger")]
pub mod dossier;
#[cfg(feature = "nato")]
pub mod dossier_wire;
#[cfg(feature = "ledger")]
pub mod proof;
#[cfg(feature = "ledger")]
pub mod receipt;
#[cfg(feature = "nato")]
pub mod signing;

use crate::observability::receipt::{Blake3ChainValidator, Blake3ReceiptEntry, ChainError};
pub use error::{DossierError, LedgerError, NameError, RefusalDigest, SealError};

#[cfg(feature = "ledger")]
pub use dossier::{CourtLedgerBuilder, DossierBuilder, DossierRoot};
#[cfg(feature = "ledger")]
pub use proof::{CourtProof, ProofStep, MAX_PROOF_STEPS};
#[cfg(feature = "nato")]
pub use signing::sealer::{
    seal as seal_envelope, verify_sealed, SealEnvelope, SealVerdict, SealedDossier, FORMAT_VERSION,
};
#[cfg(feature = "nato")]
pub use signing::signer::Signer;

// ─── Receipt validation ──────────────────────────────────────────────────────

/// Decode and fully verify a raw receipt buffer.
///
/// Delegates to
/// [`VerifiedCourtReceipt::from_bytes_verified`](crate::ledger::receipt::VerifiedCourtReceipt::from_bytes_verified);
/// the ledger adds no second decoding path.
///
/// # Errors
/// Every decode/schema failure is a typed [`LedgerError`].
#[cfg(feature = "ledger")]
pub fn validate_receipt(
    raw: &[u8],
) -> Result<crate::ledger::receipt::VerifiedCourtReceipt, LedgerError> {
    crate::ledger::receipt::VerifiedCourtReceipt::from_bytes_verified(raw)
}

// ─── Chain verification ──────────────────────────────────────────────────────

/// Result of a successful [`chain_verify`] run: the number of records replayed
/// and the final chain head (zeroed genesis for a chain that never advanced).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainReport {
    /// Number of records replayed.
    pub count: usize,
    /// Stored hash of the final record (`[0u8; 32]` when the chain is empty
    /// and the caller supplied no records — see [`chain_verify`] for the
    /// empty-chain refusal).
    pub head: [u8; 32],
}

/// Replay a BLAKE3 receipt chain through the crate's single chain validator.
///
/// The hash law lives in
/// [`Blake3ChainValidator`];
/// this function is a mapping, not a second implementation.
///
/// # Errors
/// - [`LedgerError::TooShort`] (need 1, got 0) for an empty chain — there is
///   nothing to replay, and reporting success on zero records would launder
///   absence into a verdict.
/// - [`LedgerError::ChainLinkMismatch`] for any forward-hash or prev-link
///   mismatch, carrying the failing record index.
#[cfg(feature = "ledger")]
pub fn chain_verify<E: Blake3ReceiptEntry>(records: &[E]) -> Result<ChainReport, LedgerError> {
    if records.is_empty() {
        return Err(LedgerError::TooShort { need: 1, got: 0 });
    }
    Blake3ChainValidator::validate_chain(records).map_err(chain_error_to_ledger)?;
    Ok(ChainReport { count: records.len(), head: records[records.len() - 1].stored_hash() })
}

/// Map the validator's chain failure into the ledger's vocabulary,
/// preserving the failing index.
#[must_use]
pub fn chain_error_to_ledger(err: ChainError) -> LedgerError {
    match err {
        ChainError::HashMismatch { index, .. } | ChainError::PrevHashMismatch { index, .. } => {
            LedgerError::ChainLinkMismatch { index }
        }
        // An empty chain cannot reach the validator from `chain_verify` (it is
        // refused above), so this arm is defensive only.
        ChainError::Empty => LedgerError::TooShort { need: 1, got: 0 },
    }
}

// ─── Dossier sealing (ledger + nato) ─────────────────────────────────────────

/// Seal a dossier root with an ML-DSA-65 signer.
///
/// Composition, not reimplementation: the Merkle root comes from the dossier
/// builder (`dossier::DossierBuilder::seal` → [`DossierRoot`]) and the
/// envelope + signature come from `signing::sealer::seal`.
///
/// # Errors
/// [`SealError`] from the sealing engine (custody, serialization, entropy
/// refusal).
#[cfg(all(feature = "ledger", feature = "nato"))]
pub fn seal_dossier(
    builder_output: &DossierRoot,
    signer: &Signer,
) -> Result<SealedDossier, SealError> {
    seal_envelope(&builder_output.0, signer)
}

/// Verify a sealed dossier file against a claiming public key.
///
/// Semantics are exactly `signing::sealer::verify_sealed` (fail-closed,
/// subject-binding-before-signature); this wrapper exists so the CLI has one
/// import surface.
#[cfg(all(feature = "ledger", feature = "nato"))]
#[must_use = "an ignored verification verdict admits unverified dossiers"]
pub fn verify_sealed_file(sealed: &SealedDossier, public_key: &[u8; 1952]) -> SealVerdict {
    verify_sealed(sealed, public_key)
}

// ─── Exit codes ──────────────────────────────────────────────────────────────

/// Process exit codes for the ledger CLI. Zero is success; 10-19 are the
/// ledger failure classes, one class per code. Signature-verdict failures
/// surface as [`crate::ledger::signing::SealVerdict::Invalid`] from
/// `verify_sealed_file` (a verdict, not a `LedgerError`), so no exit-code
/// class is minted for them — adding one would be a dead variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LedgerExitCode {
    /// Chain (or other operation) verified.
    Ok = 0,
    /// The receipt bytes were malformed / undecodable.
    MalformedReceipt = 10,
    /// The receipt's schema version is not supported by this reader.
    SchemaVersionMismatch = 11,
    /// The BLAKE3 chain failed to replay.
    ChainBroken = 12,
    /// Tamper evidence fired: stored material disagrees with recomputed material.
    Tampered = 13,
    /// Admission was refused (custody / entropy / policy).
    AdmissionRefused = 14,
    /// A referenced object (proof, court, subject) is dangling / unknown.
    DanglingObject = 15,
    /// Sequence ordering regressed (records out of order).
    NonMonotonic = 16,
    /// The dossier contained no proofs / no courts.
    EmptyDossier = 17,
    /// A stored digest did not match the recomputed digest.
    DigestMismatch = 18,
    /// A sealed dossier is structurally invalid (wrong format tag, receipt
    /// count, or recomputed Merkle root disagrees with the stored root).
    DossierInvalid = 19,
}

impl From<LedgerExitCode> for u8 {
    fn from(code: LedgerExitCode) -> u8 {
        code as u8
    }
}

/// The single error→exit-code mapping the CLI consumes.
///
/// Every [`LedgerError`] variant lands in exactly one failure class:
///
/// | variant(s) | class | code |
/// |---|---|---|
/// | `TooShort`, `BadMagic`, `InvalidLength`, `BodyLenMismatch`, `ReservedNonzero`, `InvalidEnum`, `InvalidName`, `ZeroTrailerViolation`, `ControlFlagMismatch` | MalformedReceipt | 10 |
/// | `UnsupportedVersion` | SchemaVersionMismatch | 11 |
/// | `ChainLinkMismatch` | ChainBroken | 12 |
/// | `ChainDiscontinuity` | ChainBroken | 12 |
/// | `DigestMismatch` | Tampered | 13 |
/// | `CrossCourtAppend`, `EmptyChain` | NonMonotonic | 16 |
///
/// `SealError` (custody/entropy refusals) is a separate enum and never
/// reaches this mapping; the CLI maps it to `AdmissionRefused` (14) itself.
#[must_use]
pub fn exit_code(err: &LedgerError) -> u8 {
    match err {
        LedgerError::UnsupportedVersion { .. } => LedgerExitCode::SchemaVersionMismatch as u8,
        LedgerError::ChainLinkMismatch { .. } | LedgerError::ChainDiscontinuity { .. } => {
            LedgerExitCode::ChainBroken as u8
        }
        LedgerError::DigestMismatch { .. } => LedgerExitCode::Tampered as u8,
        LedgerError::CrossCourtAppend { .. } | LedgerError::EmptyChain { .. } => {
            LedgerExitCode::NonMonotonic as u8
        }
        // Everything else is a decode/structure failure of the receipt bytes.
        _ => LedgerExitCode::MalformedReceipt as u8,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "ledger"))]
mod tests {
    use super::*;
    use crate::observability::receipt::ReceiptChainBuilder;

    #[test]
    fn exit_code_table_maps_every_ledger_error_variant() {
        let cases: &[(LedgerError, u8)] = &[
            (LedgerError::TooShort { need: 16, got: 4 }, LedgerExitCode::MalformedReceipt as u8),
            (LedgerError::BadMagic([0u8; 4]), LedgerExitCode::MalformedReceipt as u8),
            (
                LedgerError::UnsupportedVersion { found: 9, supported: 1 },
                LedgerExitCode::SchemaVersionMismatch as u8,
            ),
            (
                LedgerError::ReservedNonzero { offset: 0, value: 1 },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (
                LedgerError::InvalidName { offset: 0, reason: NameError::InteriorNul },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (
                LedgerError::InvalidName { offset: 0, reason: NameError::InvalidUtf8 },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (LedgerError::ChainLinkMismatch { index: 3 }, LedgerExitCode::ChainBroken as u8),
            (
                LedgerError::ChainDiscontinuity { expected_parent: [0u8; 32], actual: [1u8; 32] },
                LedgerExitCode::ChainBroken as u8,
            ),
            (
                LedgerError::ControlFlagMismatch { offset: 7 },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (
                LedgerError::DigestMismatch { offset: 5, stored: [0u8; 32], computed: [1u8; 32] },
                LedgerExitCode::Tampered as u8,
            ),
            (
                LedgerError::InvalidLength { expected: 352, actual: 351 },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (
                LedgerError::BodyLenMismatch { expected: 304, actual: 303 },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (
                LedgerError::InvalidEnum { field: "thermal", found: 0xFF },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (LedgerError::ZeroTrailerViolation, LedgerExitCode::MalformedReceipt as u8),
            (
                LedgerError::InvalidName { offset: 0, reason: NameError::TooLong },
                LedgerExitCode::MalformedReceipt as u8,
            ),
            (
                LedgerError::CrossCourtAppend { expected: "a".into(), actual: "b".into() },
                LedgerExitCode::NonMonotonic as u8,
            ),
            (LedgerError::EmptyChain { court_id: "a".into() }, LedgerExitCode::NonMonotonic as u8),
        ];
        for (err, want) in cases {
            assert_eq!(exit_code(err), *want, "wrong exit code for {err}");
        }
    }

    #[test]
    fn exit_code_values_are_distinct_and_stable() {
        let observed = [
            LedgerExitCode::Ok as u8,
            LedgerExitCode::MalformedReceipt as u8,
            LedgerExitCode::SchemaVersionMismatch as u8,
            LedgerExitCode::ChainBroken as u8,
            LedgerExitCode::Tampered as u8,
            LedgerExitCode::AdmissionRefused as u8,
            LedgerExitCode::DanglingObject as u8,
            LedgerExitCode::NonMonotonic as u8,
            LedgerExitCode::EmptyDossier as u8,
            LedgerExitCode::DigestMismatch as u8,
            LedgerExitCode::DossierInvalid as u8,
        ];
        let expected: [u8; 11] = [0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
        assert_eq!(observed, expected, "exit-code table drifted from the CLI contract");
    }

    /// Real chained records from the landed `ReceiptChainBuilder`.
    fn real_chain(n: usize) -> Vec<crate::observability::receipt::RawReceiptEntry> {
        let mut b = ReceiptChainBuilder::new();
        for i in 0..n {
            b = b.add_entry(i as u64, 0b111, 0);
        }
        b.build()
    }

    #[test]
    fn chain_verify_happy_path_reports_count_and_head() {
        let chain = real_chain(4);
        let report = chain_verify(&chain).expect("valid chain must verify");
        assert_eq!(report.count, 4);
        assert_eq!(report.head, chain[3].stored_hash());
    }

    #[test]
    fn chain_verify_single_record_satisfied_by_genesis_link() {
        let chain = real_chain(1);
        let report = chain_verify(&chain).expect("single-entry chain must verify");
        assert_eq!(report.count, 1);
        assert_eq!(report.head, chain[0].stored_hash());
    }

    #[test]
    fn chain_verify_empty_chain_is_refused_not_laundered() {
        let err = chain_verify::<crate::observability::receipt::RawReceiptEntry>(&[])
            .expect_err("empty chain must not verify");
        assert_eq!(exit_code(&err), LedgerExitCode::MalformedReceipt as u8);
    }

    #[test]
    fn chain_verify_detects_tampered_content_as_chain_broken() {
        let mut chain = real_chain(4);
        // Mutate entry 2's content without updating its stored hash: the
        // recomputed BLAKE3 can no longer match.
        chain[2].run_id_le = 0xffff_ffff_ffff_ffffu64.to_le_bytes();
        let err = chain_verify(&chain).expect_err("tampered chain must fail");
        assert_eq!(exit_code(&err), LedgerExitCode::ChainBroken as u8);
        assert_eq!(
            err,
            LedgerError::ChainLinkMismatch { index: 2 },
            "the failing index must be the tampered record"
        );
    }

    #[test]
    fn chain_verify_detects_broken_prev_link() {
        let mut chain = real_chain(3);
        // Sever the link: entry 1 claims a prev hash that is not entry 0's head.
        chain[1].prev = [0xaa; 32];
        let err = chain_verify(&chain).expect_err("broken link must fail");
        assert_eq!(exit_code(&err), LedgerExitCode::ChainBroken as u8);
        assert_eq!(err, LedgerError::ChainLinkMismatch { index: 1 });
    }
}

#[cfg(all(test, feature = "nato", not(feature = "ledger")))]
mod nato_only_tests {
    use super::*;

    #[test]
    fn exit_code_values_are_distinct_and_stable_nato_view() {
        assert_eq!(LedgerExitCode::Ok as u8, 0);
        assert_eq!(LedgerExitCode::DigestMismatch as u8, 18);
    }
}
