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
/// content = record[..304], stored hash = record[320..352], prev = the
/// receipt's `parent_receipt_hash`** — from a [`VerifiedCourtReceipt`].
/// Same bytes, same law, no second hash implementation (replay still runs
/// through the domain chain validator).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainRecord {
    /// Previous record's stored hash (genesis `[0u8; 32]` for the first).
    pub prev: [u8; 32],
    /// Content bytes hashed into the chain: `record[..304]`.
    pub content: [u8; 304],
    /// Stored hash bytes: `record[320..352]`.
    pub stored: [u8; 32],
}

impl ChainRecord {
    /// Reconstruct the chain projection from a verified 352-byte record.
    pub fn of(verified: &VerifiedCourtReceipt) -> Self {
        let record = verified.as_bytes();
        // `B_PARENT` is private to receipt.rs; the typed accessor is the
        // lawful surface for the parent link.
        let prev = verified.receipt().parent_receipt_hash;
        let mut content = [0u8; 304];
        content.copy_from_slice(&record[..304]);
        let mut stored = [0u8; 32];
        stored.copy_from_slice(&record[320..352]);
        ChainRecord { prev, content, stored }
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

/// Decode embedded receipt JSON values through the domain
/// [`CourtReceipt::from_json`](crate::nato::receipt::CourtReceipt::from_json).
///
/// # Errors
/// `Err(String)` naming the first malformed embedded receipt.
pub fn decode_receipts(values: &[serde_json::Value]) -> Result<Vec<CourtReceipt>, String> {
    values
        .iter()
        .map(|v| {
            CourtReceipt::from_json(&v.to_string())
                .map_err(|e| format!("embedded court receipt: {}", one_line(e)))
        })
        .collect()
}

/// Build the dossier JSON value (CTT-DOSSIER-v1).
///
/// # Errors
/// `Err(String)` if the system clock is before the Unix epoch or a receipt
/// fails JSON serialization.
pub fn dossier_json(
    receipts: &[CourtReceipt],
    root: &[u8; 32],
    kid: &[u8; 8],
    signature: &[u8; SIGNATURE_LEN],
) -> Result<serde_json::Value, String> {
    let sealed_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("system clock before epoch: {e}"))?
        .as_secs();
    let receipts_json: Vec<serde_json::Value> = receipts
        .iter()
        .map(|r| serde_json::to_value(r).map_err(|e| format!("receipt {}: {e}", r.name)))
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
/// `Err(String)` on bad hex or wrong length.
pub fn parse_root(s: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(s.trim()).map_err(|e| format!("merkle_root hex: {e}"))?;
    let out: [u8; 32] = bytes
        .try_into()
        .map_err(|b: Vec<u8>| format!("merkle_root must be 64 hex chars, got {}", b.len()))?;
    Ok(out)
}

/// Parse a 16-char hex key id.
///
/// # Errors
/// `Err(String)` on bad hex or wrong length.
pub fn parse_key_id(s: &str) -> Result<[u8; 8], String> {
    let bytes = hex::decode(s.trim()).map_err(|e| format!("key_id hex: {e}"))?;
    let out: [u8; 8] = bytes
        .try_into()
        .map_err(|b: Vec<u8>| format!("key_id must be 16 hex chars, got {}", b.len()))?;
    Ok(out)
}

/// Parse a 6618-char hex ML-DSA-65 signature.
///
/// # Errors
/// `Err(String)` on bad hex or wrong length.
pub fn parse_signature(s: &str) -> Result<[u8; SIGNATURE_LEN], String> {
    let bytes = hex::decode(s.trim()).map_err(|e| format!("signature hex: {e}"))?;
    let out: [u8; SIGNATURE_LEN] = bytes.try_into().map_err(|b: Vec<u8>| {
        format!("signature must be {} bytes hex, got {}", SIGNATURE_LEN, b.len())
    })?;
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
/// `Err(String)` carrying the output path on any IO failure.
pub fn write_output(path: &str, bytes: &[u8]) -> Result<(), String> {
    let tmp = format!("{path}.tmp");
    let cleanup = |e: String| -> String {
        match std::fs::remove_file(&tmp) {
            Ok(()) => e,
            Err(cleanup_err) => format!("{e} (tmp cleanup also failed: {cleanup_err})"),
        }
    };
    let result = (|| -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)?;
        Ok(())
    })()
    .map_err(|e| cleanup(format!("{path}: {e}")));
    // On success the tmp path no longer exists (renamed); on error cleanup
    // already removed it. Nothing to do here either way.
    result
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
        assert!(
            err.contains(out.to_string_lossy().as_ref()) || err.contains("tmp"),
            "typed error must name the path: {err}"
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
        let signer = Signer::from_seed([0x41; 32]);
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
}
