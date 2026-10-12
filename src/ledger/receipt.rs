#![cfg(feature = "ledger")]
//! Strict serde-free wire format for court receipts (`P12`).
//!
//! # Wire layout (little-endian)
//!
//! A [`CourtReceiptV1`] encodes to a fixed 320-byte record
//! ([`COURT_RECEIPT_V1_LEN`]). A [`ChainedCourtReceipt`] appends a 32-byte
//! BLAKE3 chain hash, giving the fixed 352-byte chained record
//! ([`COURT_RECORD_LEN`]).
//!
//! ```text
//! 320-byte receipt record:
//!   [0..4]    magic  b"CTRT"
//!   [4..6]    u16 version = 1
//!   [6]       u8 flags   (bit0 = positive_control_present, bit1 = fail_closed)
//!   [7]       u8 reserved (must be 0)
//!   [8..16]   u64 body_len = 304
//!   [16..320] 304-byte body (offsets below are body-relative):
//!     [0..8]      reserved (must be 0)
//!     [8..40]     subject_hash         [u8; 32]
//!     [40..72]    parent_receipt_hash  [u8; 32]
//!     [72..80]    timestamp_ms         u64
//!     [80..88]    run_id               u64
//!     [88..96]    total_ticks          u64
//!     [96..104]   wall_ms              u64
//!     [104]       thermal              u8 (0 = Hot, 1 = Warm, 2 = Cold)
//!     [105]       outcome              u8 (0 = Pass, 1 = Fail, 2 = Skip,
//!                                          3 = Error, 4 = Refused)
//!     [106]       budget_met           u8 (0 or 1)
//!     [107]       padding              u8 (must be 0)
//!     [108..140]  positive_control     [u8; 32]
//!     [140..172]  metadata_digest      [u8; 32]
//!     [172..204]  prev_self_hash       [u8; 32]
//!     [204..304]  name                 100-byte NUL-padded UTF-8; the final
//!                 16 bytes of the body ([288..304]) are the zero trailer and
//!                 must be 0, capping usable name bytes at NAME_MAX.
//! 352-byte chained record = 320-byte receipt + [0..32] chain_hash
//!   chain_hash = BLAKE3(parent_receipt_hash ‖ receipt_bytes[0..304])
//! ```

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

use crate::observability::receipt::Blake3ReceiptEntry;
use blake3::Hasher;

/// Magic bytes at the start of every receipt record.
pub const MAGIC: [u8; 4] = *b"CTRT";
/// Wire-format version.
pub const VERSION: u16 = 1;
/// Length of the fixed-size receipt record (header + body).
pub const COURT_RECEIPT_V1_LEN: usize = 320;
/// Length of the fixed-size chained record (receipt + 32-byte chain hash).
pub const COURT_RECORD_LEN: usize = 352;
/// Length of the BLAKE3 chain hash appended to the receipt.
pub const CHAIN_HASH_LEN: usize = 32;

/// Header length in bytes.
const HEADER_LEN: usize = 16;
/// Body length in bytes (must equal the stored `body_len` field).
const BODY_LEN: usize = 304;
/// Length of the trailing zero-only region at the end of the body.
const TRAILER_LEN: usize = 16;
/// Width of the `name` field in the body.
const NAME_LEN: usize = 100;
/// Maximum usable bytes for `name` (field width minus the zero trailer).
pub const NAME_MAX: usize = NAME_LEN - TRAILER_LEN;

// Body-relative offsets.
const B_SUBJECT: usize = 8;
const B_PARENT: usize = 40;
const B_TIMESTAMP: usize = 72;
const B_RUN_ID: usize = 80;
const B_TOTAL_TICKS: usize = 88;
const B_WALL_MS: usize = 96;
const B_THERMAL: usize = 104;
const B_OUTCOME: usize = 105;
const B_BUDGET: usize = 106;
const B_PAD: usize = 107;
const B_CONTROL: usize = 108;
const B_METADATA: usize = 140;
const B_PREV_SELF: usize = 172;
const B_NAME: usize = 204;

/// Length of the body prefix reserved for future use (must be zero).
const BODY_PREFIX_LEN: usize = 8;

// Flag bits.
const FLAG_POSITIVE_CONTROL: u8 = 1 << 0;
const FLAG_FAIL_CLOSED: u8 = 1 << 1;
/// All flag bits that are defined; any other bit set is a decode error.
const FLAG_KNOWN_MASK: u8 = FLAG_POSITIVE_CONTROL | FLAG_FAIL_CLOSED;

/// Thermal classification recorded alongside the tick metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThermalClass {
    /// Thermals constrained the run (lower confidence in tick counts).
    Hot,
    /// Nominal thermals.
    Warm,
    /// Ideal thermals.
    Cold,
}

impl ThermalClass {
    /// Encodes this class as its one-byte wire tag.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Hot => 0,
            Self::Warm => 1,
            Self::Cold => 2,
        }
    }

    /// Decodes a one-byte wire tag, returning `None` when out of range.
    #[must_use]
    pub const fn from_u8(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Hot),
            1 => Some(Self::Warm),
            2 => Some(Self::Cold),
            _ => None,
        }
    }
}

/// Verdict recorded for a court run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CourtOutcome {
    /// The court passed.
    Pass,
    /// The court failed.
    Fail,
    /// The court skipped (precondition unmet).
    Skip,
    /// The court errored while running.
    Error,
    /// The court refused the subject outright.
    Refused,
}

impl CourtOutcome {
    /// Encodes this outcome as its one-byte wire tag.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Fail => 1,
            Self::Skip => 2,
            Self::Error => 3,
            Self::Refused => 4,
        }
    }

    /// Decodes a one-byte wire tag, returning `None` when out of range.
    #[must_use]
    pub const fn from_u8(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Pass),
            1 => Some(Self::Fail),
            2 => Some(Self::Skip),
            3 => Some(Self::Error),
            4 => Some(Self::Refused),
            _ => None,
        }
    }
}

/// Quantitative metrics captured during a court run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReceiptMetrics {
    /// Wall-clock timestamp of the run, milliseconds since the Unix epoch.
    pub timestamp_ms: u64,
    /// Opaque identifier of the run that produced this receipt.
    pub run_id: u64,
    /// Total measured ticks.
    pub total_ticks: u64,
    /// Wall-clock duration of the run in milliseconds.
    pub wall_ms: u64,
    /// Thermal class during the run.
    pub thermal: ThermalClass,
    /// Court verdict.
    pub outcome: CourtOutcome,
    /// Whether the budget gate was met (`0` or `1` on the wire).
    pub budget_met: bool,
}

/// A court receipt in its typed, in-memory form.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CourtReceiptV1 {
    /// BLAKE3 hash of the adjudicated subject.
    pub subject_hash: [u8; 32],
    /// Chain hash of the parent receipt (all zeros for a genesis receipt).
    pub parent_receipt_hash: [u8; 32],
    /// Metrics captured during the run.
    pub metrics: ReceiptMetrics,
    /// Optional positive-control hash; must be all zeros when the
    /// `positive_control_present` flag is clear.
    pub positive_control: [u8; 32],
    /// Digest of the court metadata blob.
    pub metadata_digest: [u8; 32],
    /// Chain hash of the previous receipt about the same subject.
    pub prev_self_hash: [u8; 32],
    /// Human-readable receipt name (at most [`NAME_MAX`] bytes of UTF-8,
    /// no interior NUL).
    pub name: String,
    /// Whether the court runs fail-closed on inconclusive evidence
    /// (flag bit 1 on the wire).
    pub fail_closed: bool,
}

impl CourtReceiptV1 {
    /// Validates and constructs a receipt.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ledger::error::LedgerError`] when the name exceeds
    /// [`NAME_MAX`] bytes, contains an interior NUL, or is not valid UTF-8;
    /// or when the flag/control combination is inconsistent.
    pub fn new(
        subject_hash: [u8; 32],
        parent_receipt_hash: [u8; 32],
        metrics: ReceiptMetrics,
        positive_control: [u8; 32],
        metadata_digest: [u8; 32],
        prev_self_hash: [u8; 32],
        name: &str,
    ) -> Result<Self, crate::ledger::error::LedgerError> {
        validate_name(name)?;
        let receipt = Self {
            subject_hash,
            parent_receipt_hash,
            metrics,
            positive_control,
            metadata_digest,
            prev_self_hash,
            name: name.to_owned(),
            fail_closed: false,
        };
        validate_flag_control(&receipt)?;
        Ok(receipt)
    }

    /// Returns the derived flag byte for this receipt.
    ///
    /// The flag byte is never stored in the struct: bit 0
    /// (`positive_control_present`) is derived from the control bytes and
    /// bit 1 (`fail_closed`) from [`Self::fail_closed`], so the encoded
    /// record always satisfies the flag/control consistency check enforced
    /// by [`VerifiedCourtReceipt::from_bytes_verified`].
    #[must_use]
    fn flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.positive_control != [0u8; 32] {
            flags |= FLAG_POSITIVE_CONTROL;
        }
        if self.fail_closed {
            flags |= FLAG_FAIL_CLOSED;
        }
        flags
    }

    /// Serializes to the fixed 320-byte wire record.
    ///
    /// The flag byte is derived from [`Self::positive_control`] and
    /// [`Self::fail_closed`], so the output always satisfies the
    /// flag/control consistency check enforced by
    /// [`VerifiedCourtReceipt::from_bytes_verified`].
    #[must_use]
    pub fn encode(&self) -> [u8; COURT_RECEIPT_V1_LEN] {
        let mut out = [0u8; COURT_RECEIPT_V1_LEN];
        out[0..4].copy_from_slice(&MAGIC);
        out[4..6].copy_from_slice(&VERSION.to_le_bytes());
        out[6] = self.flags();
        out[7] = 0; // reserved
        out[8..16].copy_from_slice(&(BODY_LEN as u64).to_le_bytes());

        let body = &mut out[HEADER_LEN..];
        // body[0..8] reserved, already zero.
        body[B_SUBJECT..B_SUBJECT + 32].copy_from_slice(&self.subject_hash);
        body[B_PARENT..B_PARENT + 32].copy_from_slice(&self.parent_receipt_hash);
        body[B_TIMESTAMP..B_TIMESTAMP + 8]
            .copy_from_slice(&self.metrics.timestamp_ms.to_le_bytes());
        body[B_RUN_ID..B_RUN_ID + 8].copy_from_slice(&self.metrics.run_id.to_le_bytes());
        body[B_TOTAL_TICKS..B_TOTAL_TICKS + 8]
            .copy_from_slice(&self.metrics.total_ticks.to_le_bytes());
        body[B_WALL_MS..B_WALL_MS + 8].copy_from_slice(&self.metrics.wall_ms.to_le_bytes());
        body[B_THERMAL] = self.metrics.thermal.to_u8();
        body[B_OUTCOME] = self.metrics.outcome.to_u8();
        body[B_BUDGET] = u8::from(self.metrics.budget_met);
        body[B_PAD] = 0;
        body[B_CONTROL..B_CONTROL + 32].copy_from_slice(&self.positive_control);
        body[B_METADATA..B_METADATA + 32].copy_from_slice(&self.metadata_digest);
        body[B_PREV_SELF..B_PREV_SELF + 32].copy_from_slice(&self.prev_self_hash);

        let name_bytes = self.name.as_bytes();
        let usable = NAME_LEN - TRAILER_LEN;
        let n = if name_bytes.len() < usable { name_bytes.len() } else { usable };
        body[B_NAME..B_NAME + n].copy_from_slice(&name_bytes[..n]);
        // Remaining name bytes and the zero trailer stay zero.

        out
    }

    /// Computes the BLAKE3 chain hash for this receipt:
    /// `BLAKE3(parent_receipt_hash ‖ encoded_bytes[0..304])`.
    #[must_use]
    pub fn chain_hash(&self) -> [u8; 32] {
        chain_hash_for(&self.parent_receipt_hash, &self.encode())
    }
}

// ─── Chained record ──────────────────────────────────────────────────────────

/// A 352-byte chained receipt record: the 320-byte receipt plus its 32-byte
/// BLAKE3 chain hash. Implements the existing
/// [`crate::observability::receipt::Blake3ReceiptEntry`] trait so generic
/// chain validation and tamper-evidence checks apply unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainedCourtReceipt {
    /// The parent chain hash (prepended to the content when hashing).
    parent_receipt_hash: [u8; 32],
    /// The full 320-byte encoded receipt record.
    content: [u8; COURT_RECEIPT_V1_LEN],
    /// The stored BLAKE3 chain hash (record bytes 320..352).
    chain_hash: [u8; 32],
}

impl ChainedCourtReceipt {
    /// Chains a receipt: computes and stores its chain hash.
    #[must_use]
    pub fn from_receipt(receipt: &CourtReceiptV1) -> Self {
        let encoded = receipt.encode();
        Self {
            parent_receipt_hash: receipt.parent_receipt_hash,
            chain_hash: chain_hash_for(&receipt.parent_receipt_hash, &encoded),
            content: encoded,
        }
    }

    /// Serializes to the fixed 352-byte chained record.
    #[must_use]
    pub fn encode(&self) -> [u8; COURT_RECORD_LEN] {
        let mut out = [0u8; COURT_RECORD_LEN];
        out[..COURT_RECEIPT_V1_LEN].copy_from_slice(&self.content);
        out[COURT_RECEIPT_V1_LEN..].copy_from_slice(&self.chain_hash);
        out
    }
}

impl Blake3ReceiptEntry for ChainedCourtReceipt {
    fn prev_hash(&self) -> [u8; 32] {
        self.parent_receipt_hash
    }

    fn content_bytes(&self) -> Vec<u8> {
        // The chain hash covers receipt bytes [0..304] only (see
        // [`chain_hash_for`]); the zero-trailer tail is excluded.
        self.content[..304].to_vec()
    }

    fn stored_hash(&self) -> [u8; 32] {
        self.chain_hash
    }
}

/// Computes `BLAKE3(parent ‖ record_bytes[0..304])`.
fn chain_hash_for(parent: &[u8; 32], encoded: &[u8; COURT_RECEIPT_V1_LEN]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(parent);
    hasher.update(&encoded[..304]);
    let hash: [u8; 32] = hasher.finalize().into();
    hash
}

// ─── Verified decode ─────────────────────────────────────────────────────────

/// Constant-time equality for fixed-size byte slices.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// A receipt that has passed full deserialize-time verification. It can only
/// be created through [`VerifiedCourtReceipt::from_bytes_verified`], which
/// recomputes the BLAKE3 chain hash and compares it in constant time against
/// the stored value, and enforces every structural, range, and name
/// constraint. There is no other constructor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerifiedCourtReceipt(ChainedCourtReceipt);

impl VerifiedCourtReceipt {
    /// Verifies and decodes a 352-byte chained record.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ledger::error::LedgerError`] on any length, magic, version,
    /// reserved-byte, body-length, enum-range, flag/control, name, trailer,
    /// or chain-hash mismatch.
    pub fn from_bytes_verified(bytes: &[u8]) -> Result<Self, crate::ledger::error::LedgerError> {
        if bytes.len() != COURT_RECORD_LEN {
            return Err(crate::ledger::error::LedgerError::InvalidLength {
                expected: COURT_RECORD_LEN,
                actual: bytes.len(),
            });
        }

        // ── Header ──
        let magic = bytes.get(0..4).ok_or(crate::ledger::error::LedgerError::InvalidLength {
            expected: COURT_RECORD_LEN,
            actual: bytes.len(),
        })?;
        if !ct_eq(magic, &MAGIC) {
            return Err(crate::ledger::error::LedgerError::BadMagic(
                magic.try_into().unwrap_or_default(),
            ));
        }
        let version = read_u16(bytes, 4)?;
        if version != VERSION {
            return Err(crate::ledger::error::LedgerError::UnsupportedVersion {
                found: version,
                supported: VERSION,
            });
        }
        let flags = *bytes.get(6).ok_or(crate::ledger::error::LedgerError::InvalidLength {
            expected: COURT_RECORD_LEN,
            actual: bytes.len(),
        })?;
        if flags & !FLAG_KNOWN_MASK != 0 {
            return Err(crate::ledger::error::LedgerError::ReservedNonzero {
                offset: 6,
                value: u64::from(flags),
            });
        }
        let reserved = bytes.get(7).copied().unwrap_or(1);
        if reserved != 0 {
            return Err(crate::ledger::error::LedgerError::ReservedNonzero {
                offset: 7,
                value: u64::from(reserved),
            });
        }
        let body_len = read_u64(bytes, 8)?;
        if body_len != BODY_LEN as u64 {
            return Err(crate::ledger::error::LedgerError::BodyLenMismatch {
                expected: BODY_LEN as u64,
                actual: body_len,
            });
        }

        // ── Body ──
        let body = bytes.get(HEADER_LEN..COURT_RECEIPT_V1_LEN).ok_or(
            crate::ledger::error::LedgerError::InvalidLength {
                expected: COURT_RECORD_LEN,
                actual: bytes.len(),
            },
        )?;
        if body[..BODY_PREFIX_LEN].iter().any(|&b| b != 0) {
            // First non-zero byte of the reserved body prefix, reported at its
            // record offset.
            let bad =
                HEADER_LEN + body[..BODY_PREFIX_LEN].iter().position(|&b| b != 0).unwrap_or(0);
            return Err(crate::ledger::error::LedgerError::ReservedNonzero {
                offset: bad,
                value: u64::from(body[bad - HEADER_LEN]),
            });
        }
        if body[B_PAD] != 0 {
            return Err(crate::ledger::error::LedgerError::ReservedNonzero {
                offset: HEADER_LEN + B_PAD,
                value: u64::from(body[B_PAD]),
            });
        }
        if body[NAME_LEN_USABLE..NAME_LEN_USABLE + TRAILER_LEN].iter().any(|&b| b != 0) {
            return Err(crate::ledger::error::LedgerError::ZeroTrailerViolation);
        }

        let _subject_hash = read_arr32(body, B_SUBJECT)?;
        let parent_receipt_hash = read_arr32(body, B_PARENT)?;
        let _timestamp_ms = read_u64(body, B_TIMESTAMP)?;
        let _run_id = read_u64(body, B_RUN_ID)?;
        let _total_ticks = read_u64(body, B_TOTAL_TICKS)?;
        let _wall_ms = read_u64(body, B_WALL_MS)?;
        let _thermal = ThermalClass::from_u8(body[B_THERMAL]).ok_or(
            crate::ledger::error::LedgerError::InvalidEnum {
                field: "thermal",
                found: body[B_THERMAL],
            },
        )?;
        let _outcome = CourtOutcome::from_u8(body[B_OUTCOME]).ok_or(
            crate::ledger::error::LedgerError::InvalidEnum {
                field: "outcome",
                found: body[B_OUTCOME],
            },
        )?;
        let budget_raw = body[B_BUDGET];
        if budget_raw > 1 {
            return Err(crate::ledger::error::LedgerError::InvalidEnum {
                field: "budget_met",
                found: budget_raw,
            });
        }
        let positive_control = read_arr32(body, B_CONTROL)?;
        let _metadata_digest = read_arr32(body, B_METADATA)?;
        let _prev_self_hash = read_arr32(body, B_PREV_SELF)?;
        let _name = decode_name(&body[B_NAME..])?;

        // ── Flag / control consistency ──
        let control_present = positive_control != [0u8; 32];
        let flag_present = flags & FLAG_POSITIVE_CONTROL != 0;
        if control_present != flag_present {
            return Err(crate::ledger::error::LedgerError::ControlFlagMismatch { offset: 6 });
        }

        // ── Chain hash (constant-time compare) ──
        let mut receipt_bytes = [0u8; COURT_RECEIPT_V1_LEN];
        receipt_bytes[..HEADER_LEN].copy_from_slice(&bytes[..HEADER_LEN]);
        receipt_bytes[HEADER_LEN..].copy_from_slice(body);
        let computed = chain_hash_for(&parent_receipt_hash, &receipt_bytes);
        let stored = read_arr32(bytes, COURT_RECEIPT_V1_LEN)?;
        if !ct_eq(&computed, &stored) {
            return Err(crate::ledger::error::LedgerError::DigestMismatch {
                offset: COURT_RECEIPT_V1_LEN,
                stored,
                computed,
            });
        }

        Ok(Self(ChainedCourtReceipt {
            parent_receipt_hash,
            content: receipt_bytes,
            chain_hash: stored,
        }))
    }

    /// The verified chained record bytes.
    #[must_use]
    pub fn as_bytes(&self) -> [u8; COURT_RECORD_LEN] {
        self.0.encode()
    }

    /// The verified receipt in typed form.
    #[must_use]
    pub fn receipt(&self) -> CourtReceiptV1 {
        // `content` is record[0..304] (header included); body fields start
        // after the 16-byte header.
        let body = &self.0.content[HEADER_LEN..];
        let name_raw = &body[B_NAME..];
        let name_end = name_raw.iter().position(|&b| b == 0).unwrap_or(NAME_LEN_USABLE);
        // Post-verification this is valid UTF-8 with no interior NUL, so the
        // conversion cannot fail; fall back to lossy rather than panic.
        CourtReceiptV1 {
            subject_hash: read_arr32(body, B_SUBJECT).unwrap_or_default(),
            parent_receipt_hash: read_arr32(body, B_PARENT).unwrap_or_default(),
            metrics: ReceiptMetrics {
                timestamp_ms: read_u64(body, B_TIMESTAMP).unwrap_or_default(),
                run_id: read_u64(body, B_RUN_ID).unwrap_or_default(),
                total_ticks: read_u64(body, B_TOTAL_TICKS).unwrap_or_default(),
                wall_ms: read_u64(body, B_WALL_MS).unwrap_or_default(),
                thermal: ThermalClass::from_u8(body[B_THERMAL]).unwrap_or(ThermalClass::Warm),
                outcome: CourtOutcome::from_u8(body[B_OUTCOME]).unwrap_or(CourtOutcome::Error),
                budget_met: body[B_BUDGET] == 1,
            },
            positive_control: read_arr32(body, B_CONTROL).unwrap_or_default(),
            metadata_digest: read_arr32(body, B_METADATA).unwrap_or_default(),
            prev_self_hash: read_arr32(body, B_PREV_SELF).unwrap_or_default(),
            name: String::from_utf8_lossy(&name_raw[..name_end]).into_owned(),
            // `content` begins with the record header, so byte 6 is the
            // flag byte; post-verification it is in known-range.
            fail_closed: self.0.content[6] & FLAG_FAIL_CLOSED != 0,
        }
    }
}

/// Offset of the zero trailer within the body (name-usable bytes end here).
const NAME_LEN_USABLE: usize = 304 - TRAILER_LEN;

// ─── Field helpers ───────────────────────────────────────────────────────────

/// Reads a little-endian `u16` at `off`, infallible for in-range offsets on
/// fixed-size buffers but kept fallible to avoid indexing panics.
fn read_u16(buf: &[u8], off: usize) -> Result<u16, crate::ledger::error::LedgerError> {
    let chunk = buf.get(off..off + 2).ok_or(crate::ledger::error::LedgerError::InvalidLength {
        expected: COURT_RECORD_LEN,
        actual: buf.len(),
    })?;
    let arr: [u8; 2] = chunk.try_into().unwrap_or_default();
    Ok(u16::from_le_bytes(arr))
}

/// Reads a little-endian `u64` at `off`.
fn read_u64(buf: &[u8], off: usize) -> Result<u64, crate::ledger::error::LedgerError> {
    let chunk = buf.get(off..off + 8).ok_or(crate::ledger::error::LedgerError::InvalidLength {
        expected: COURT_RECORD_LEN,
        actual: buf.len(),
    })?;
    let arr: [u8; 8] = chunk.try_into().unwrap_or_default();
    Ok(u64::from_le_bytes(arr))
}

/// Reads a 32-byte array at `off`.
fn read_arr32(buf: &[u8], off: usize) -> Result<[u8; 32], crate::ledger::error::LedgerError> {
    let chunk = buf.get(off..off + 32).ok_or(crate::ledger::error::LedgerError::InvalidLength {
        expected: COURT_RECORD_LEN,
        actual: buf.len(),
    })?;
    Ok(chunk.try_into().unwrap_or_default())
}

/// Validates a receipt name: at most [`NAME_MAX`] bytes, valid UTF-8 (by
/// construction of `&str`), no interior NUL.
fn validate_name(name: &str) -> Result<(), crate::ledger::error::LedgerError> {
    if name.len() > NAME_MAX {
        return Err(crate::ledger::error::LedgerError::InvalidName {
            offset: B_NAME,
            reason: crate::ledger::error::NameError::TooLong,
        });
    }
    if name.as_bytes().contains(&0u8) {
        return Err(crate::ledger::error::LedgerError::InvalidName {
            offset: B_NAME,
            reason: crate::ledger::error::NameError::InteriorNul,
        });
    }
    Ok(())
}

/// Decodes the 100-byte name field: NUL-padded UTF-8, no interior NUL.
fn decode_name(field: &[u8]) -> Result<String, crate::ledger::error::LedgerError> {
    let usable = &field[..NAME_LEN_USABLE.min(field.len())];
    let end = usable.iter().position(|&b| b == 0).unwrap_or(usable.len());
    let name_bytes = &usable[..end];
    if name_bytes.len() > NAME_MAX {
        return Err(crate::ledger::error::LedgerError::InvalidName {
            offset: B_NAME,
            reason: crate::ledger::error::NameError::TooLong,
        });
    }
    if name_bytes.contains(&0u8) {
        return Err(crate::ledger::error::LedgerError::InvalidName {
            offset: B_NAME,
            reason: crate::ledger::error::NameError::InteriorNul,
        });
    }
    std::str::from_utf8(name_bytes).map(str::to_owned).map_err(|_| {
        crate::ledger::error::LedgerError::InvalidName {
            offset: B_NAME,
            reason: crate::ledger::error::NameError::InvalidUtf8,
        }
    })
}

/// Rejects a receipt whose flag/control combination is inconsistent.
fn validate_flag_control(
    receipt: &CourtReceiptV1,
) -> Result<(), crate::ledger::error::LedgerError> {
    let control_present = receipt.positive_control != [0u8; 32];
    let flag_present = receipt.flags() & FLAG_POSITIVE_CONTROL != 0;
    if control_present != flag_present {
        return Err(crate::ledger::error::LedgerError::ControlFlagMismatch { offset: 6 });
    }
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a valid receipt with deterministic fields.
    fn sample() -> CourtReceiptV1 {
        CourtReceiptV1::new(
            [0xAA; 32],
            [0xBB; 32],
            ReceiptMetrics {
                timestamp_ms: 1_700_000_000_000,
                run_id: 42,
                total_ticks: 9_999,
                wall_ms: 123,
                thermal: ThermalClass::Warm,
                outcome: CourtOutcome::Pass,
                budget_met: true,
            },
            [0; 32],
            [0xCC; 32],
            [0xDD; 32],
            "subject-court-pass",
        )
        .expect("sample is valid")
    }

    /// Builds a valid 352-byte chained record.
    fn sample_chained() -> [u8; COURT_RECORD_LEN] {
        ChainedCourtReceipt::from_receipt(&sample()).encode()
    }

    #[test]
    fn constants_are_consistent() {
        assert_eq!(HEADER_LEN + BODY_LEN, COURT_RECEIPT_V1_LEN);
        assert_eq!(COURT_RECEIPT_V1_LEN + CHAIN_HASH_LEN, COURT_RECORD_LEN);
        assert_eq!(BODY_LEN - TRAILER_LEN, 288);
        assert_eq!(B_NAME + NAME_LEN, BODY_LEN);
    }

    #[test]
    fn roundtrip_preserves_every_field() {
        let receipt = sample();
        let chained = ChainedCourtReceipt::from_receipt(&receipt);
        let bytes = chained.encode();
        let verified = VerifiedCourtReceipt::from_bytes_verified(&bytes).expect("verifies");
        assert_eq!(verified.as_bytes(), bytes);
        assert_eq!(verified.receipt(), receipt);
    }

    #[test]
    fn roundtrip_without_positive_control_sets_no_flag() {
        let receipt = sample();
        let encoded = receipt.encode();
        assert_eq!(encoded[6] & FLAG_POSITIVE_CONTROL, 0);
        let verified =
            VerifiedCourtReceipt::from_bytes_verified(&sample_chained()).expect("verifies");
        assert_eq!(verified.receipt(), receipt);
    }

    #[test]
    fn chain_hash_binds_parent_and_content() {
        let receipt = sample();
        let mut other = receipt.clone();
        other.parent_receipt_hash = [0x11; 32];
        assert_ne!(receipt.chain_hash(), other.chain_hash());
    }

    #[test]
    fn wrong_length_is_rejected() {
        for len in [0usize, 1, 351, 353, 704] {
            let bytes = vec![0u8; len];
            let err = VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err();
            assert_eq!(
                err,
                crate::ledger::error::LedgerError::InvalidLength {
                    expected: COURT_RECORD_LEN,
                    actual: len,
                }
            );
        }
    }

    #[test]
    fn bad_magic_is_rejected() {
        let mut bytes = sample_chained();
        bytes[0] = b'X';
        assert_eq!(
            VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
            crate::ledger::error::LedgerError::BadMagic(*b"XTRT")
        );
    }

    #[test]
    fn unsupported_version_is_rejected() {
        for v in [0u16, 2, 0xFFFF] {
            let mut bytes = sample_chained();
            bytes[4..6].copy_from_slice(&v.to_le_bytes());
            assert_eq!(
                VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
                crate::ledger::error::LedgerError::UnsupportedVersion {
                    found: v,
                    supported: VERSION
                }
            );
        }
    }

    #[test]
    fn reserved_header_byte_is_rejected() {
        let mut bytes = sample_chained();
        bytes[7] = 1;
        assert_eq!(
            VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
            crate::ledger::error::LedgerError::ReservedNonzero { offset: 7, value: 1 }
        );
    }

    #[test]
    fn body_len_mismatch_is_rejected() {
        let mut bytes = sample_chained();
        bytes[8..16].copy_from_slice(&303u64.to_le_bytes());
        assert_eq!(
            VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
            crate::ledger::error::LedgerError::BodyLenMismatch { expected: 304, actual: 303 }
        );
    }

    #[test]
    fn out_of_range_enums_are_rejected() {
        for (off, field) in [
            (HEADER_LEN + B_THERMAL, "thermal"),
            (HEADER_LEN + B_OUTCOME, "outcome"),
            (HEADER_LEN + B_BUDGET, "budget_met"),
        ] {
            let mut bytes = sample_chained();
            bytes[off] = 0xFF;
            assert_eq!(
                VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
                crate::ledger::error::LedgerError::InvalidEnum { field, found: 0xFF }
            );
        }
    }

    #[test]
    fn zero_trailer_violation_is_rejected() {
        let mut bytes = sample_chained();
        let trailer_start = HEADER_LEN + 304 - TRAILER_LEN;
        bytes[trailer_start] = 1;
        assert_eq!(
            VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
            crate::ledger::error::LedgerError::ZeroTrailerViolation
        );
    }

    #[test]
    fn invalid_names_are_rejected() {
        // Interior NUL.
        let receipt = CourtReceiptV1::new(
            [0; 32],
            [0; 32],
            ReceiptMetrics {
                timestamp_ms: 0,
                run_id: 0,
                total_ticks: 0,
                wall_ms: 0,
                thermal: ThermalClass::Cold,
                outcome: CourtOutcome::Skip,
                budget_met: false,
            },
            [0; 32],
            [0; 32],
            [0; 32],
            "bad\0name",
        );
        assert_eq!(
            receipt.unwrap_err(),
            crate::ledger::error::LedgerError::InvalidName {
                offset: B_NAME,
                reason: crate::ledger::error::NameError::InteriorNul,
            }
        );

        // Too long (NAME_MAX + 1).
        let too_long = "a".repeat(NAME_MAX + 1);
        let receipt = CourtReceiptV1::new(
            [0; 32],
            [0; 32],
            ReceiptMetrics {
                timestamp_ms: 0,
                run_id: 0,
                total_ticks: 0,
                wall_ms: 0,
                thermal: ThermalClass::Cold,
                outcome: CourtOutcome::Skip,
                budget_met: false,
            },
            [0; 32],
            [0; 32],
            [0; 32],
            &too_long,
        );
        assert_eq!(
            receipt.unwrap_err(),
            crate::ledger::error::LedgerError::InvalidName {
                offset: B_NAME,
                reason: crate::ledger::error::NameError::TooLong,
            }
        );
    }

    #[test]
    fn flag_control_mismatch_is_refused_on_decode() {
        // Flag cleared but control bytes present.
        let mut receipt = sample();
        receipt.positive_control = [0x77; 32];
        let mut bytes = ChainedCourtReceipt::from_receipt(&receipt).encode();
        bytes[6] &= !FLAG_POSITIVE_CONTROL; // break consistency
        assert_eq!(
            VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
            crate::ledger::error::LedgerError::ControlFlagMismatch { offset: 6 }
        );

        // Flag set but control bytes all zero.
        let mut bytes = sample_chained();
        bytes[6] |= FLAG_POSITIVE_CONTROL;
        assert_eq!(
            VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err(),
            crate::ledger::error::LedgerError::ControlFlagMismatch { offset: 6 }
        );
    }

    #[test]
    fn chain_hash_tamper_is_rejected() {
        let mut bytes = sample_chained();
        bytes[351] ^= 0x01;
        let err = VerifiedCourtReceipt::from_bytes_verified(&bytes).unwrap_err();
        assert!(
            matches!(err, crate::ledger::error::LedgerError::DigestMismatch { offset: 320, .. }),
            "expected DigestMismatch at the chain-hash offset, got {err}"
        );
    }

    /// Every single-byte mutation of the 352-byte chained record must be
    /// rejected. 352 flips x 1 case each = 352-case tamper matrix.
    #[test]
    fn tamper_matrix_every_single_byte_flip_is_rejected() {
        let good = sample_chained();
        assert_eq!(good.len(), 352);
        for i in 0..COURT_RECORD_LEN {
            let mut bytes = good;
            bytes[i] ^= 0x01;
            assert!(
                VerifiedCourtReceipt::from_bytes_verified(&bytes).is_err(),
                "byte {i} flip was accepted"
            );
        }
    }

    /// Flipping any bit of any byte (not just bit 0) must also be rejected.
    #[test]
    fn tamper_matrix_every_bit_of_first_and_last_32_bytes() {
        let good = sample_chained();
        for i in 0..32 {
            for bit in 0..8 {
                let mut bytes = good;
                bytes[i] ^= 1 << bit;
                assert!(
                    VerifiedCourtReceipt::from_bytes_verified(&bytes).is_err(),
                    "byte {i} bit {bit} flip was accepted"
                );
                let mut tail = good;
                tail[COURT_RECORD_LEN - 1 - i] ^= 1 << bit;
                assert!(
                    VerifiedCourtReceipt::from_bytes_verified(&tail).is_err(),
                    "tail byte {i} bit {bit} flip was accepted"
                );
            }
        }
    }

    #[test]
    fn chained_receipt_satisfies_blake3_receipt_entry_trait() {
        use crate::observability::receipt::Blake3ChainValidator;
        let genesis = {
            let mut g = sample();
            g.parent_receipt_hash = [0u8; 32];
            g
        };
        let entry = ChainedCourtReceipt::from_receipt(&genesis);
        // Two-entry chain: genesis parent is all-zero (the validator's
        // zeroed genesis), then the child threads the parent's stored hash.
        let mut child_receipt = genesis.clone();
        child_receipt.parent_receipt_hash = entry.stored_hash();
        child_receipt.prev_self_hash = entry.stored_hash();
        let child = ChainedCourtReceipt::from_receipt(&child_receipt);
        Blake3ChainValidator::assert_chain_valid(&[entry.clone(), child.clone()]);
        Blake3ChainValidator::assert_tamper_evident(&[entry, child]);
    }

    #[test]
    fn two_receipts_form_a_valid_chain() {
        use crate::observability::receipt::Blake3ChainValidator;
        let mut genesis = sample();
        genesis.parent_receipt_hash = [0u8; 32];
        let parent = ChainedCourtReceipt::from_receipt(&genesis);
        let mut child_receipt = sample();
        child_receipt.parent_receipt_hash = parent.stored_hash();
        child_receipt.prev_self_hash = parent.stored_hash();
        let child = ChainedCourtReceipt::from_receipt(&child_receipt);
        Blake3ChainValidator::assert_chain_valid(&[parent, child]);
    }
}
