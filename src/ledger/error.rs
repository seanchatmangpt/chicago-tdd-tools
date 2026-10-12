#![cfg(feature = "ledger")]
//! Error types for the ledger subsystem.
//!
//! Every variant's [`Display`](std::fmt::Display) pinpoints the byte offset
//! (or index) at which decoding/sealing failed, so a court can replay the
//! failure against the exact subject bytes without re-deriving the position.
//!
//! No variant carries a foreign error via `#[from]`: the ledger owns its
//! failure vocabulary end to end.

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

use thiserror::Error;

/// Sub-error for name-field decoding failures.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NameError {
    /// The name field contained an interior NUL byte.
    #[error("interior NUL byte in name")]
    InteriorNul,
    /// The name field was not valid UTF-8.
    #[error("name field is not valid UTF-8")]
    InvalidUtf8,
    /// The name exceeded [`crate::ledger::receipt::NAME_MAX`] bytes.
    #[error("name exceeds maximum length")]
    TooLong,
}

/// Errors raised while decoding a ledger record or stream.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LedgerError {
    /// The buffer ended before the minimum record size was reached.
    #[error("record too short: need {need} bytes, got {got}")]
    TooShort {
        /// Minimum number of bytes required.
        need: usize,
        /// Number of bytes actually available.
        got: usize,
    },

    /// Extra bytes remained after the final record was consumed.
    #[error("{count} trailing byte(s) after final record")]
    TrailingBytes {
        /// Number of unconsumed trailing bytes.
        count: usize,
    },

    /// The stream magic did not match the expected 4-byte prefix.
    #[error("bad magic: expected ledger magic, found {}", bad_magic_hex(.0))]
    BadMagic([u8; 4]),

    /// The stream version is not supported by this reader.
    #[error("unsupported version {found} (supported: {supported})")]
    UnsupportedVersion {
        /// Version found in the stream.
        found: u16,
        /// Maximum/expected version supported.
        supported: u16,
    },

    /// A fixed-width field had the wrong length.
    #[error("invalid length: expected {expected} bytes, got {actual}")]
    InvalidLength {
        /// Expected byte count.
        expected: usize,
        /// Actual byte count.
        actual: usize,
    },

    /// The record header's stored `body_len` disagreed with the fixed body
    /// width.
    #[error("body length mismatch: expected {expected}, got {actual}")]
    BodyLenMismatch {
        /// The body length the format requires.
        expected: u64,
        /// The body length stored in the record header.
        actual: u64,
    },

    /// A range-checked enum byte held an unassigned value (distinct from
    /// [`LedgerError::InvalidTag`], which carries the byte offset).
    #[error("invalid enum tag {found:#04x} for field '{field}'")]
    InvalidEnum {
        /// Which field carried the bad value.
        field: &'static str,
        /// The unassigned value.
        found: u8,
    },

    /// The zero-only trailer at the end of the name field was non-zero.
    #[error("zero trailer violated: trailing 16 bytes of the name field must be zero")]
    ZeroTrailerViolation,

    /// A reserved field was non-zero (forward-compat violation).
    #[error("reserved field non-zero at offset {offset}: value {value:#x}")]
    ReservedNonzero {
        /// Byte offset of the reserved field.
        offset: usize,
        /// Non-zero value found.
        value: u64,
    },

    /// A discriminating tag byte held an unassigned value.
    #[error("invalid tag {tag:#04x} for field '{field}' at offset {offset}")]
    InvalidTag {
        /// Which field carried the bad tag.
        field: &'static str,
        /// The unassigned tag value.
        tag: u8,
        /// Byte offset of the tag.
        offset: usize,
    },

    /// A name field failed validation.
    #[error("invalid name at offset {offset}: {reason}")]
    InvalidName {
        /// Byte offset of the name field.
        offset: usize,
        /// Specific name failure.
        reason: NameError,
    },

    /// The stored digest did not match the recomputed digest.
    #[error("digest mismatch at offset {offset}: stored {stored:02x?}, computed {computed:02x?}")]
    DigestMismatch {
        /// Byte offset of the digest field.
        offset: usize,
        /// Digest as stored in the record.
        stored: [u8; 32],
        /// Digest recomputed over the payload.
        computed: [u8; 32],
    },

    /// A record's back-pointer did not chain to the previous record.
    #[error("chain link mismatch at record index {index}")]
    ChainLinkMismatch {
        /// Index of the record whose link failed.
        index: usize,
    },

    /// Control flags present in the record disagreed with the declared flags.
    #[error("control flag mismatch at offset {offset}")]
    ControlFlagMismatch {
        /// Byte offset of the control flags.
        offset: usize,
    },

    // ── Added by the L3 dossier/proof lane (disclosed cross-lane fix): the
    //    P13 hybrid contract requires these variants and they were absent
    //    from the landed vocabulary. ──
    /// A receipt belonging to a different court was offered to a court chain
    /// builder ([`crate::ledger::dossier::CourtLedgerBuilder::append`]).
    #[error("cross-court append: builder owns '{expected}', receipt belongs to '{actual}'")]
    CrossCourtAppend {
        /// Court the builder was created for.
        expected: String,
        /// Court the offered receipt actually belongs to.
        actual: String,
    },

    /// A court chain was sealed with no receipts appended.
    #[error("empty chain for court '{court_id}'")]
    EmptyChain {
        /// The court whose chain is empty.
        court_id: String,
    },
}

/// Renders the 4-byte magic as contiguous lowercase hex (`Display` helper
/// for [`LedgerError::BadMagic`]).
fn bad_magic_hex(magic: &[u8; 4]) -> String {
    magic.iter().map(|b| format!("{b:02x}")).collect()
}

/// Errors raised while assembling or validating a dossier.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DossierError {
    /// The dossier contained no proofs.
    #[error("dossier is empty")]
    EmptyDossier,

    /// A proof's root did not match the dossier root.
    #[error("root mismatch between dossier and proof")]
    RootMismatch,

    /// A proof failed structural validation.
    #[error("malformed proof in dossier")]
    MalformedProof,

    /// A court identifier exceeded the maximum length.
    #[error("court id too long")]
    CourtIdTooLong,

    /// The referenced court is not known to this verifier.
    #[error("unknown court")]
    UnknownCourt,

    // ── Added by the L3 dossier/proof lane (disclosed cross-lane fix). ──
    /// The same `court_id` was added to a dossier twice.
    #[error("duplicate court '{0}'")]
    DuplicateCourt(String),

    /// A proof replayed to a hash other than the dossier root.
    #[error("proof for court '{0}' failed verification")]
    VerificationFailed(String),
}

/// Opaque 32-byte refusal receipt carried by [`SealError::EntropyRefused`].
///
/// The sealing engine refuses to mint a nonce from insufficient entropy; the
/// refusal is attested by this digest rather than a human-readable string so
/// the refusal itself is replayable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefusalDigest(pub [u8; 32]);

impl std::fmt::Display for RefusalDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Errors raised while sealing a record.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SealError {
    /// The subject bytes did not conform to the sealable format.
    #[error("malformed seal subject")]
    MalformedSubject,

    /// The signing key is not in the required custody state.
    #[error("key custody violation: {0}")]
    KeyCustody(String),

    /// Canonical serialization of the payload failed.
    #[error("serialization failure: {0}")]
    Serialization(String),

    /// The entropy source refused to supply sealing material; carries the
    /// opaque 32-byte refusal receipt.
    #[error("entropy source refused: refusal digest {0}")]
    EntropyRefused(RefusalDigest),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_error_too_short_display() {
        let e = LedgerError::TooShort { need: 16, got: 4 };
        assert_eq!(e.to_string(), "record too short: need 16 bytes, got 4");
    }

    #[test]
    fn test_ledger_error_trailing_bytes_display() {
        let e = LedgerError::TrailingBytes { count: 7 };
        assert_eq!(e.to_string(), "7 trailing byte(s) after final record");
    }

    #[test]
    fn test_ledger_error_bad_magic_display() {
        let e = LedgerError::BadMagic([0xde, 0xad, 0xbe, 0xef]);
        assert!(e.to_string().contains("bad magic"), "{}", e);
        assert!(e.to_string().contains("deadbeef"), "{}", e);
    }

    #[test]
    fn test_ledger_error_unsupported_version_display() {
        let e = LedgerError::UnsupportedVersion { found: 9, supported: 1 };
        assert_eq!(e.to_string(), "unsupported version 9 (supported: 1)");
    }

    #[test]
    fn test_ledger_error_reserved_nonzero_display() {
        let e = LedgerError::ReservedNonzero { offset: 42, value: 0xff };
        assert!(e.to_string().contains("offset 42"), "{}", e);
        assert!(e.to_string().contains("0xff"), "{}", e);
    }

    #[test]
    fn test_ledger_error_invalid_tag_display() {
        let e = LedgerError::InvalidTag { field: "kind", tag: 0x7f, offset: 12 };
        let s = e.to_string();
        assert!(s.contains("'kind'"), "{}", s);
        assert!(s.contains("0x7f"), "{}", s);
        assert!(s.contains("offset 12"), "{}", s);
    }

    #[test]
    fn test_ledger_error_invalid_name_display() {
        let e = LedgerError::InvalidName { offset: 33, reason: NameError::InteriorNul };
        assert!(e.to_string().contains("offset 33"), "{}", e);
        assert!(e.to_string().contains("interior NUL"), "{}", e);

        let e = LedgerError::InvalidName { offset: 34, reason: NameError::InvalidUtf8 };
        assert!(e.to_string().contains("not valid UTF-8"), "{}", e);
    }

    #[test]
    fn test_ledger_error_digest_mismatch_display() {
        let e = LedgerError::DigestMismatch { offset: 5, stored: [0u8; 32], computed: [1u8; 32] };
        let s = e.to_string();
        assert!(s.contains("offset 5"), "{}", s);
        assert!(s.contains("stored"), "{}", s);
        assert!(s.contains("computed"), "{}", s);
    }

    #[test]
    fn test_ledger_error_chain_link_mismatch_display() {
        let e = LedgerError::ChainLinkMismatch { index: 3 };
        assert_eq!(e.to_string(), "chain link mismatch at record index 3");
    }

    #[test]
    fn test_ledger_error_control_flag_mismatch_display() {
        let e = LedgerError::ControlFlagMismatch { offset: 77 };
        assert_eq!(e.to_string(), "control flag mismatch at offset 77");
    }

    #[test]
    fn test_ledger_error_decode_vocabulary_displays() {
        let e = LedgerError::InvalidLength { expected: 352, actual: 351 };
        assert_eq!(e.to_string(), "invalid length: expected 352 bytes, got 351");

        let e = LedgerError::BodyLenMismatch { expected: 304, actual: 303 };
        assert_eq!(e.to_string(), "body length mismatch: expected 304, got 303");

        let e = LedgerError::InvalidEnum { field: "thermal", found: 0xFF };
        let s = e.to_string();
        assert!(s.contains("'thermal'"), "{}", s);
        assert!(s.contains("0xff"), "{}", s);

        let e = LedgerError::ZeroTrailerViolation;
        assert!(e.to_string().contains("zero trailer"), "{}", e);

        let e = LedgerError::InvalidName { offset: 220, reason: NameError::TooLong };
        assert!(e.to_string().contains("maximum length"), "{}", e);
    }

    #[test]
    fn test_name_error_too_long_display() {
        assert_eq!(NameError::TooLong.to_string(), "name exceeds maximum length");
    }

    #[test]
    fn test_dossier_error_displays() {
        assert_eq!(DossierError::EmptyDossier.to_string(), "dossier is empty");
        assert_eq!(
            DossierError::RootMismatch.to_string(),
            "root mismatch between dossier and proof"
        );
        assert_eq!(DossierError::MalformedProof.to_string(), "malformed proof in dossier");
        assert_eq!(DossierError::CourtIdTooLong.to_string(), "court id too long");
        assert_eq!(DossierError::UnknownCourt.to_string(), "unknown court");
    }

    #[test]
    fn test_refusal_digest_display_is_hex() {
        let mut bytes = [0u8; 32];
        bytes[..4].copy_from_slice(&[0x00, 0x01, 0xfe, 0xff]);
        let d = RefusalDigest(bytes);
        assert!(d.to_string().starts_with("0001feff"), "{}", d);
        assert_eq!(d.to_string().len(), 64);
    }

    #[test]
    fn test_seal_error_displays() {
        assert_eq!(SealError::MalformedSubject.to_string(), "malformed seal subject");

        let e = SealError::KeyCustody("key not held".into());
        assert!(e.to_string().contains("key not held"), "{}", e);

        let e = SealError::Serialization("bad canonical form".into());
        assert!(e.to_string().contains("bad canonical form"), "{}", e);

        let e = SealError::EntropyRefused(RefusalDigest([0xab; 32]));
        let s = e.to_string();
        assert!(s.contains("refused"), "{}", s);
        assert!(s.contains("abababab"), "{}", s);
    }

    #[test]
    fn test_name_error_displays() {
        assert_eq!(NameError::InteriorNul.to_string(), "interior NUL byte in name");
        assert_eq!(NameError::InvalidUtf8.to_string(), "name field is not valid UTF-8");
    }

    #[test]
    fn test_equality_and_clone() {
        let a = LedgerError::TooShort { need: 1, got: 0 };
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(
            LedgerError::ChainLinkMismatch { index: 0 },
            LedgerError::ChainLinkMismatch { index: 1 }
        );
    }
}
