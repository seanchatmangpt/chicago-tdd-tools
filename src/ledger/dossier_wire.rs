//! CTT-DOSSIER-v1 wire surface for library consumers (e.g. the `affi`
//! dossier verbs, pending Phase D M2). Formerly shared with the `ctdd_ledger`
//! CLI, which was excised in the V2 cleanup.
//!
//! Ported (V1 surface-parity phase) from the bin-private logic in
//! `src/bin/ctdd_ledger.rs` (since deleted). The split law: this module holds the pure
//! types/functions; filesystem custody (record/receipt reading, directory
//! walks, stdin) stays caller-side. [`write_output`] is the one retained IO
//! seam — the atomic tmp+rename write the dossier subcommands have always
//! relied on, kept here so its typed no-partial-output contract and tests
//! travel with the wire surface.
//!
//! No clap. No reading. Hash law is never reimplemented: chain replay runs
//! through the domain
//! [`Blake3ChainValidator`](crate::observability::receipt::Blake3ChainValidator)
//! via [`Blake3ReceiptEntry`]; dossier sealing through
//! [`DossierBuilder`](crate::ledger::dossier::DossierBuilder).

use crate::ledger::receipt::VerifiedCourtReceipt;
use crate::nato::receipt::CourtReceipt;
use crate::observability::receipt::Blake3ReceiptEntry;
use thiserror::Error;

/// Typed errors for the CTT-DOSSIER-v1 wire surface. Every public function in
/// this module returns `Result<_, DossierWireError>` — no `String` errors.
#[derive(Debug, Error)]
pub enum DossierWireError {
    /// A record's unhashed zero trailer (`record[304..320]`) was tampered
    /// with (T7 law, [`validate_dossier_chain`]).
    #[error(
        "record {index}: zero trailer (record[304..320]) tampered — non-zero byte in unhashed receipt tail"
    )]
    TrailerTampered {
        /// Zero-based index of the tampered record.
        index: usize,
    },

    /// Chain replay through the domain
    /// [`Blake3ChainValidator`](crate::observability::receipt::Blake3ChainValidator)
    /// failed. (`ChainError` predates the typed-error surface and carries
    /// `Display` but not `std::error::Error`, so no `#[from]`.)
    #[error("chain replay: {0}")]
    Chain(crate::observability::receipt::ChainError),

    /// An embedded receipt JSON object failed to decode through
    /// [`CourtReceipt::from_json`](crate::nato::receipt::CourtReceipt::from_json).
    #[error("embedded court receipt: {message}")]
    ReceiptDecode {
        /// Single-line decoder message for the first malformed receipt.
        message: String,
    },

    /// The system clock was before the Unix epoch when sealing a dossier.
    #[error("system clock before epoch: {0}")]
    ClockBeforeEpoch(#[from] std::time::SystemTimeError),

    /// A receipt failed JSON serialization while embedding.
    #[error("receipt {name}: {message}")]
    ReceiptSerialize {
        /// Name of the receipt that failed to serialize.
        name: String,
        /// Serialization error message.
        message: String,
    },

    /// The `merkle_root` field was not valid hex.
    #[error("merkle_root hex: {0}")]
    RootHex(#[from] hex::FromHexError),

    /// The `merkle_root` field was valid hex but the wrong byte length.
    #[error("merkle_root must be 64 hex chars, got {got}")]
    RootLength {
        /// Actual decoded byte count.
        got: usize,
    },

    /// The `key_id` field was not valid hex.
    #[error("key_id hex: {0}")]
    KeyIdHex(hex::FromHexError),

    /// The `key_id` field was valid hex but the wrong byte length.
    #[error("key_id must be 16 hex chars, got {got}")]
    KeyIdLength {
        /// Actual decoded byte count.
        got: usize,
    },

    /// The `signature` field was not valid hex.
    #[error("signature hex: {0}")]
    SignatureHex(hex::FromHexError),

    /// The `signature` field was valid hex but the wrong byte length.
    #[error("signature must be {} bytes hex, got {got}", SIGNATURE_LEN)]
    SignatureLength {
        /// Actual decoded byte count.
        got: usize,
    },

    /// An atomic output write failed. The real output path never holds a
    /// partial file; the tmp file is removed before this error is returned.
    #[error("{path}: {source}{}", tmp_cleanup_note(cleanup.as_deref()))]
    Write {
        /// Output path the write was targeting.
        path: String,
        /// The underlying IO failure.
        source: std::io::Error,
        /// Present when deleting the tmp file also failed.
        cleanup: Option<String>,
    },
}

/// Display helper rendering the tmp-cleanup suffix for
/// [`DossierWireError::Write`].
fn tmp_cleanup_note(cleanup: Option<&str>) -> String {
    cleanup.map(|c| format!(" (tmp cleanup also failed: {c})")).unwrap_or_default()
}

/// Wire-format discriminator of a CTT-DOSSIER-v1 document.
pub const DOSSIER_FORMAT: &str = "CTT-DOSSIER-v1";

/// Default output path the `affi dossier emit` verb (landed in ~/affidavit, Phase D M2;
/// formerly the `ctdd_ledger dossier seal` subcommand) writes to.
pub const DEFAULT_DOSSIER_OUT: &str = "docs/nato/NATO_STANAG_4107_DOSSIER.json";

/// ML-DSA-65 signature length in bytes (wire: 6618 lowercase hex chars).
pub const SIGNATURE_LEN: usize = crate::ledger::signing::signer::SIGNATURE_LEN;

/// Structural view of a CTT-DOSSIER-v1 JSON document. Receipt objects are
/// decoded one-by-one through the domain
/// [`CourtReceipt::from_json`](crate::nato::receipt::CourtReceipt::from_json)
/// so a single malformed receipt is refused with a typed receipt-class error
/// rather than failing the whole document decode.
#[derive(serde::Deserialize)]
pub struct DossierJson {
    /// Wire-format discriminator; must equal [`DOSSIER_FORMAT`].
    pub format: String,
    /// Embedded receipt JSON objects; decode with [`decode_receipts`].
    pub court_receipts: Vec<serde_json::Value>,
    /// 64-char lowercase-hex Merkle root; parse with [`parse_root`].
    pub merkle_root: String,
    /// 16-char lowercase-hex ML-DSA-65 key id; parse with [`parse_key_id`].
    pub key_id: String,
    /// 6618-char lowercase-hex ML-DSA-65 signature; parse with
    /// [`parse_signature`].
    pub signature: String,
    /// Unix-seconds seal timestamp. Decoded to validate dossier schema;
    /// not yet consumed by any inspection path.
    pub sealed_at: u64,
}

/// Chain-replayable view of a verified 352-byte record.
///
/// The domain's
/// [`ChainedCourtReceipt`](crate::ledger::receipt::ChainedCourtReceipt) has
/// no public decode constructor, so this reconstructs the exact
/// [`Blake3ReceiptEntry`] projection the domain uses — **RM1 offset law:
/// content = record[..304], stored hash = record[320..352], zero trailer =
/// record[304..320], prev = the receipt's `parent_receipt_hash`** — from a
/// [`VerifiedCourtReceipt`].
/// Same bytes, same law, no second hash implementation (replay still runs
/// through the domain chain validator).
///
/// T7 tamper-evidence law: the chain hash covers only `record[..304]`, so
/// the trailer bytes `record[304..320]` are otherwise invisible to chain
/// replay. [`ChainRecord`] carries them and
/// [`validate_dossier_chain`] refuses any record whose trailer is not the
/// lawful all-zero trailer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainRecord {
    /// Previous record's stored hash (genesis `[0u8; 32]` for the first).
    pub prev: [u8; 32],
    /// Content bytes hashed into the chain: `record[..304]`.
    pub content: [u8; 304],
    /// Zero trailer (unhashed receipt tail): `record[304..320]`.
    pub trailer: [u8; 16],
    /// Stored hash bytes: `record[320..352]`.
    pub stored: [u8; 32],
}

/// The lawful zero trailer length: record bytes `304..320`.
pub const TRAILER_LEN: usize = 16;

impl ChainRecord {
    /// Reconstruct the chain projection from a verified 352-byte record.
    pub fn of(verified: &VerifiedCourtReceipt) -> Self {
        let record = verified.as_bytes();
        // `B_PARENT` is private to receipt.rs; the typed accessor is the
        // lawful surface for the parent link.
        let prev = verified.receipt().parent_receipt_hash;
        let mut content = [0u8; 304];
        content.copy_from_slice(&record[..304]);
        let mut trailer = [0u8; TRAILER_LEN];
        trailer.copy_from_slice(&record[304..320]);
        let mut stored = [0u8; 32];
        stored.copy_from_slice(&record[320..352]);
        ChainRecord { prev, content, trailer, stored }
    }
}

impl Blake3ReceiptEntry for ChainRecord {
    fn prev_hash(&self) -> [u8; 32] {
        self.prev
    }
    fn content_bytes(&self) -> Vec<u8> {
        self.content.to_vec()
    }
    fn stored_hash(&self) -> [u8; 32] {
        self.stored
    }
}

/// Per-record pre-check that the unhashed zero trailer `record[304..320]`
/// matches the record law, then full chain replay through the domain
/// [`Blake3ChainValidator`](crate::observability::receipt::Blake3ChainValidator).
///
/// This closes the T7 gap: `Blake3ReceiptEntry` chain validation sees only
/// `prev ‖ content[..304] ‖ stored[320..352]`, so a tampered trailer byte was
/// invisible at chain level. Folding the trailer into `content_bytes` would
/// change the hash law (replayed digests would no longer match the stored
/// hashes minted over `[..304]`), so the trailer is checked separately,
/// before the chain replay, for every record.
///
/// # Errors
/// [`DossierWireError::TrailerTampered`] naming the first record whose
/// trailer is non-zero, or [`DossierWireError::Chain`] for any chain-replay
/// failure.
pub fn validate_dossier_chain(records: &[ChainRecord]) -> Result<(), DossierWireError> {
    for (i, r) in records.iter().enumerate() {
        if r.trailer != [0u8; TRAILER_LEN] {
            return Err(DossierWireError::TrailerTampered { index: i });
        }
    }
    crate::observability::receipt::Blake3ChainValidator::validate_chain(records)
        .map_err(DossierWireError::Chain)
}

/// Decode embedded receipt JSON values through the domain
/// [`CourtReceipt::from_json`](crate::nato::receipt::CourtReceipt::from_json).
///
/// # Errors
/// [`DossierWireError::ReceiptDecode`] naming the first malformed embedded
/// receipt.
pub fn decode_receipts(
    values: &[serde_json::Value],
) -> Result<Vec<CourtReceipt>, DossierWireError> {
    values
        .iter()
        .map(|v| {
            CourtReceipt::from_json(&v.to_string())
                .map_err(|e| DossierWireError::ReceiptDecode { message: one_line(e) })
        })
        .collect()
}

/// Build the dossier JSON value (CTT-DOSSIER-v1).
///
/// # Errors
/// [`DossierWireError::ClockBeforeEpoch`] if the system clock is before the
/// Unix epoch, or [`DossierWireError::ReceiptSerialize`] if a receipt fails
/// JSON serialization.
pub fn dossier_json(
    receipts: &[CourtReceipt],
    root: &[u8; 32],
    kid: &[u8; 8],
    signature: &[u8; SIGNATURE_LEN],
) -> Result<serde_json::Value, DossierWireError> {
    let sealed_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let receipts_json: Vec<serde_json::Value> = receipts
        .iter()
        .map(|r| {
            serde_json::to_value(r).map_err(|e| DossierWireError::ReceiptSerialize {
                name: r.name.clone(),
                message: e.to_string(),
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(serde_json::json!({
        "format": DOSSIER_FORMAT,
        "court_receipts": receipts_json,
        "merkle_root": hex_head(root),
        "key_id": hex::encode(kid),
        "signature": hex::encode(signature),
        "sealed_at": sealed_at,
    }))
}

/// Parse a 64-char hex Merkle root.
///
/// # Errors
/// [`DossierWireError::RootHex`] on bad hex or [`DossierWireError::RootLength`]
/// on the wrong length.
pub fn parse_root(s: &str) -> Result<[u8; 32], DossierWireError> {
    let bytes = hex::decode(s.trim())?;
    let out: [u8; 32] = bytes
        .try_into()
        .map_err(|b: Vec<u8>| DossierWireError::RootLength { got: b.len() })?;
    Ok(out)
}

/// Parse a 16-char hex key id.
///
/// # Errors
/// [`DossierWireError::KeyIdHex`] on bad hex or [`DossierWireError::KeyIdLength`]
/// on the wrong length.
pub fn parse_key_id(s: &str) -> Result<[u8; 8], DossierWireError> {
    let bytes = hex::decode(s.trim()).map_err(DossierWireError::KeyIdHex)?;
    let out: [u8; 8] = bytes
        .try_into()
        .map_err(|b: Vec<u8>| DossierWireError::KeyIdLength { got: b.len() })?;
    Ok(out)
}

/// Parse a 6618-char hex ML-DSA-65 signature.
///
/// # Errors
/// [`DossierWireError::SignatureHex`] on bad hex or
/// [`DossierWireError::SignatureLength`] on the wrong length.
pub fn parse_signature(s: &str) -> Result<[u8; SIGNATURE_LEN], DossierWireError> {
    let bytes = hex::decode(s.trim()).map_err(DossierWireError::SignatureHex)?;
    let out: [u8; SIGNATURE_LEN] = bytes
        .try_into()
        .map_err(|b: Vec<u8>| DossierWireError::SignatureLength { got: b.len() })?;
    Ok(out)
}

/// Write the sealed-dossier wire bytes to `output` atomically (tmp + rename).
///
/// Writes to `<out>.tmp` in the same directory (same filesystem, so the final
/// rename is atomic), fsyncs, then renames onto `out`. On ANY error the tmp
/// file is deleted and a typed `Err(String)` is returned — the real output
/// path is never left holding a partial file. Note SIGXFSZ (e.g. under
/// `ulimit -f`) can still kill the process at the OS level mid-write, but the
/// truncated artifact is then `.tmp`-suffixed, never at the real output path.
///
/// # Errors
/// [`DossierWireError::Write`] carrying the output path and the underlying
/// [`std::io::Error`] on any IO failure (plus the tmp-cleanup failure, if the
/// tmp delete also failed).
pub fn write_output(path: &str, bytes: &[u8]) -> Result<(), DossierWireError> {
    let tmp = format!("{path}.tmp");
    let result = (|| -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)?;
        Ok(())
    })();
    match result {
        Ok(()) => Ok(()),
        Err(source) => {
            let cleanup = match std::fs::remove_file(&tmp) {
                Ok(()) => None,
                Err(cleanup_err) => Some(cleanup_err.to_string()),
            };
            // On success the tmp path no longer exists (renamed); on error
            // cleanup already removed it. Nothing to do here either way.
            Err(DossierWireError::Write { path: path.to_string(), source, cleanup })
        }
    }
}

/// Lowercase hex of a 32-byte digest (stdout/wire rendering only).
#[must_use]
pub fn hex_head(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Single-line an error for typed-error surfaces.
fn one_line(e: impl std::fmt::Display) -> String {
    let s = e.to_string();
    s.lines().next().unwrap_or_default().to_string()
}

#[cfg(all(test, feature = "ledger", feature = "nato"))]
// JUSTIFICATION: Test code only — unwraps in test assertions/fixtures per repo convention.
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ledger::dossier::{CourtLedgerBuilder, DossierBuilder};
    use crate::ledger::receipt::{
        ChainedCourtReceipt, CourtReceiptV1, ReceiptMetrics, ThermalClass,
    };
    use crate::ledger::signing::signer::Signer;
    use crate::nato::court::{CourtId, CourtOutcome as NatoOutcome, MetricValue, Pillar, Verdict};
    use crate::nato::dossier_court::dossier_root;
    use crate::nato::receipt::ControlResult;
    use crate::nato::ScopingRecord;

    // ── write_output contract (ported from bin write_output_tests) ─────────

    // ── write_output contract (ported from bin write_output_tests) ─────────

    #[test]
    fn success_writes_full_bytes_and_leaves_no_tmp_residue() {
        let dir = std::env::temp_dir()
            .join(format!("ctdd_dossier_wire_wo_success_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("dossier.bin");
        let bytes: Vec<u8> = (0..=255u8).cycle().take(4096).collect();

        write_output(out.to_str().unwrap(), &bytes).expect("write_output must succeed");

        let on_disk = std::fs::read(&out).expect("output file must exist");
        assert_eq!(on_disk, bytes, "output must equal written bytes exactly");
        assert!(!dir.join("dossier.bin.tmp").exists(), "no .tmp residue allowed on success");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_only_dir_yields_typed_error_no_output_no_tmp_residue() {
        let dir =
            std::env::temp_dir().join(format!("ctdd_dossier_wire_wo_ro_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("dossier.bin");
        let bytes = b"payload".to_vec();

        // Read-only directory: File::create of the tmp file fails outright.
        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&dir, perms).unwrap();

        let result = write_output(out.to_str().unwrap(), &bytes);

        // Restore before asserting so cleanup is not skipped on failure.
        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(&dir, perms).unwrap();

        let err = result.expect_err("write into read-only dir must fail");
        let msg = err.to_string();
        assert!(
            msg.contains(out.to_string_lossy().as_ref()) || msg.contains("tmp"),
            "typed error must name the path: {msg}"
        );
        assert!(!out.exists(), "no output file may appear on failure");
        assert!(!dir.join("dossier.bin.tmp").exists(), "no .tmp residue on failure");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failure_over_existing_file_leaves_previous_output_intact() {
        let dir = std::env::temp_dir()
            .join(format!("ctdd_dossier_wire_wo_overwrite_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("dossier.bin");
        std::fs::write(&out, b"previous-good").unwrap();

        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&dir, perms).unwrap();
        let result = write_output(out.to_str().unwrap(), b"new");
        let mut perms = std::fs::metadata(&dir).unwrap().permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(&dir, perms).unwrap();

        assert!(result.is_err(), "must fail in read-only dir");
        assert_eq!(
            std::fs::read(&out).unwrap(),
            b"previous-good",
            "previous good output must remain intact after failed write"
        );
        assert!(!dir.join("dossier.bin.tmp").exists(), "no .tmp residue");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Roundtrip: decode → seal → serialize → parse laws ──────────────────

    /// One NATO `CourtReceipt` minted through the real `record` constructor.
    fn nato_receipt(court: CourtId) -> CourtReceipt {
        CourtReceipt::record(
            court,
            Pillar::Proof,
            court.canonical(),
            "dossier-wire-subject",
            NatoOutcome {
                verdict: Verdict::Pass,
                metrics: vec![("ticks", MetricValue::U64(u64::from(court.id_number())))],
                scoping: ScopingRecord::full("dossier wire roundtrip", "2 Pass receipts"),
            },
            ControlResult::MutantCaught {
                mutant_id: format!("m-{}", court.id_number()),
                detection_channel: "dossier-wire".to_string(),
            },
        )
    }

    /// One valid 352-byte chained record (Pass, budget met).
    fn ledger_receipt(seed: u8) -> [u8; 352] {
        let receipt = CourtReceiptV1::new(
            [seed; 32],
            [0u8; 32],
            ReceiptMetrics {
                timestamp_ms: 0,
                run_id: u64::from(seed),
                total_ticks: 1_000,
                wall_ms: 10,
                thermal: ThermalClass::Warm,
                outcome: crate::ledger::receipt::CourtOutcome::Pass,
                budget_met: true,
            },
            [9u8; 32],
            [7u8; 32],
            [0u8; 32],
            "court-wire",
        )
        .expect("valid receipt");
        ChainedCourtReceipt::from_receipt(&receipt).expect("chain").encode()
    }

    #[test]
    fn dossier_roundtrip_decode_seal_serialize_parse() {
        // Real receipts minted through the real constructors (no mocks).
        let receipts = [nato_receipt(CourtId::N01), nato_receipt(CourtId::N02)];
        let root = dossier_root(&receipts);
        let signer = Signer::from_seed([0x41; 32]).expect("non-degenerate");
        let kid = crate::ledger::signing::key_id(&signer.public_key());
        let signature = signer.sign(&root);

        // dossier_json → canonical wire → DossierJson re-decode.
        let doc_value =
            dossier_json(&receipts, &root, &kid, &signature).expect("dossier_json succeeds");
        let wire = serde_json::to_vec(&doc_value).expect("serialize");
        let doc: DossierJson = serde_json::from_slice(&wire).expect("DossierJson decodes");
        assert_eq!(doc.format, DOSSIER_FORMAT);

        // Decode laws roundtrip to the minted receipts.
        let decoded = decode_receipts(&doc.court_receipts).expect("embedded receipts decode");
        assert_eq!(decoded.len(), receipts.len());
        for (a, b) in decoded.iter().zip(&receipts) {
            assert_eq!(a, b, "receipt must survive the embed/decode roundtrip");
        }

        // Parse laws recover the exact sealed fields.
        assert_eq!(parse_root(&doc.merkle_root).expect("root"), root);
        assert_eq!(parse_key_id(&doc.key_id).expect("kid"), kid);
        assert_eq!(parse_signature(&doc.signature).expect("sig"), signature);

        // The embedded signature verifies against the root under the key.
        let pk = signer.public_key();
        assert!(
            Signer::verify(&pk, &root, &parse_signature(&doc.signature).unwrap()),
            "embedded signature must verify"
        );

        // Wire-format invariants: default out path constant, hex rendering.
        assert_eq!(hex_head(&root), doc.merkle_root);
        assert_eq!(DEFAULT_DOSSIER_OUT, "docs/nato/NATO_STANAG_4107_DOSSIER.json");
    }

    #[test]
    fn chain_record_preserves_rm1_offset_law_and_replays() {
        let bytes = ledger_receipt(7u8);
        let verified =
            VerifiedCourtReceipt::from_bytes_verified(&bytes).expect("minted record must verify");
        let record = ChainRecord::of(&verified);

        // RM1 offset law: content = record[..304], stored = record[320..352],
        // prev = the receipt's typed parent link.
        assert_eq!(record.content.as_slice(), &bytes[..304]);
        assert_eq!(record.stored, bytes[320..352]);
        assert_eq!(record.prev, verified.receipt().parent_receipt_hash);

        // The projection replays through the domain chain validator.
        {
            use blake3::Hasher;
            let mut h = Hasher::new();
            h.update(&record.prev);
            h.update(&record.content);
            assert_eq!(
                *h.finalize().as_bytes(),
                record.stored,
                "blake3(prev||content) must equal stored (prev={:?}, stored={:?})",
                record.prev,
                record.stored
            );
        }
        let report = crate::ledger::chain_verify(&[record]).expect("chain replays");
        assert_eq!(report.count, 1);
        assert_eq!(report.head, bytes[320..352]);

        // A mutated stored hash is refused by the domain validator — the
        // offset law is load-bearing, not cosmetic.
        let tampered = ChainRecord { stored: [0u8; 32], ..record };
        assert!(
            crate::ledger::chain_verify(&[tampered]).is_err(),
            "tampered stored hash must fail chain replay"
        );
    }

    /// Build a valid 3-record chain (each record's prev = prior stored), minted
    /// through the real constructors with correct parent threading.
    fn wire_chain() -> Vec<ChainRecord> {
        let mut prev = [0u8; 32];
        let mut chain = Vec::new();
        for seed in 1u8..=3 {
            let receipt = CourtReceiptV1::new(
                [seed; 32],
                prev,
                ReceiptMetrics {
                    timestamp_ms: 0,
                    run_id: u64::from(seed),
                    total_ticks: 1_000,
                    wall_ms: 10,
                    thermal: ThermalClass::Warm,
                    outcome: crate::ledger::receipt::CourtOutcome::Pass,
                    budget_met: true,
                },
                [9u8; 32],
                [7u8; 32],
                [0u8; 32],
                "court-wire",
            )
            .expect("valid receipt");
            let bytes = ChainedCourtReceipt::from_receipt(&receipt).expect("chain").encode();
            let r = ChainRecord::of(
                &VerifiedCourtReceipt::from_bytes_verified(&bytes).expect("verifies"),
            );
            prev = r.stored;
            chain.push(r);
        }
        chain
    }

    #[test]
    fn validate_dossier_chain_accepts_untampered_chain() {
        let chain = wire_chain();
        validate_dossier_chain(&chain).expect("untampered chain must validate");
    }

    #[test]
    fn validate_dossier_chain_refuses_last_record_trailer_tamper() {
        // T7: trailer bytes (record[304..320]) are outside the hashed content,
        // so the domain validator alone cannot see this tamper.
        let mut chain = wire_chain();
        let last = chain.len() - 1;
        chain[last].trailer[0] ^= 0x01;
        assert!(
            crate::ledger::chain_verify(&chain).is_ok(),
            "precondition: the domain validator alone misses a trailer tamper on the last record"
        );
        let err = validate_dossier_chain(&chain).expect_err("trailer tamper must be refused");
        let msg = err.to_string();
        assert!(msg.contains("zero trailer"), "typed trailer error: {msg}");
    }

    #[test]
    fn validate_dossier_chain_refuses_mid_chain_trailer_tamper() {
        let mut chain = wire_chain();
        chain[0].trailer[15] ^= 0x80;
        let err = validate_dossier_chain(&chain).expect_err("mid-chain trailer tamper refused");
        let msg = err.to_string();
        assert!(msg.contains("record 0"), "error names the tampered record: {msg}");
    }

    #[test]
    fn chain_record_of_carries_trailer_bytes() {
        let bytes = ledger_receipt(9u8);
        let verified = VerifiedCourtReceipt::from_bytes_verified(&bytes).expect("verifies");
        let record = ChainRecord::of(&verified);
        assert_eq!(record.trailer.as_slice(), &bytes[304..320]);
        assert_eq!(record.trailer, [0u8; TRAILER_LEN], "fresh record trailer is all-zero");
    }
}
