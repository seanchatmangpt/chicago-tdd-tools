#![allow(clippy::unwrap_used)] // JUSTIFICATION: test module unwraps are test-only; refusal rendering uses unwrap_or
//! ctdd_ledger — thin clap front-end over the ledger domain API.
//!
//! Pure-Rust constraint boundary: all parsing, hashing, and admission
//! decisions happen in-binary via `chicago_tdd_tools::ledger`. The shell
//! layer stays in Justfile orchestration only — this binary performs no
//! subprocess/shell-out work itself.
//!
//! RECONCILED (lane R8, plan E4): wired to the landed `src/ledger/mod.rs`
//! surface — `validate_receipt(&[u8])`, generic `chain_verify(&[E])`,
//! `seal_dossier(&DossierRoot, &Signer)`, `verify_sealed_file(&SealedDossier,
//! &[u8; 1952])`, and `exit_code(&LedgerError) -> u8`. The CLI owns all
//! filesystem traversal (the domain layer is in-memory only) and maps IO
//! failures to exit code 2 directly — there is deliberately no
//! `LedgerExitCode::Io` variant; the enum belongs to mod.rs.

use std::io::Read;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use chicago_tdd_tools::ledger::receipt::{ChainedCourtReceipt, VerifiedCourtReceipt};
use chicago_tdd_tools::ledger::signing::sealer;
use chicago_tdd_tools::ledger::signing::{key_id, KeyStore, Signer, SIGNATURE_LEN};
use chicago_tdd_tools::ledger::{
    chain_verify, exit_code, seal_dossier, validate_receipt, verify_sealed_file,
    CourtLedgerBuilder, DossierBuilder, DossierError, LedgerError, LedgerExitCode, SealError,
    SealVerdict,
};
use chicago_tdd_tools::nato::court::Verdict;
use chicago_tdd_tools::nato::dossier_court::{dossier_admits, dossier_root};
use chicago_tdd_tools::nato::receipt::{ControlResult, CourtReceipt};
use chicago_tdd_tools::observability::receipt::Blake3ReceiptEntry;

#[derive(Parser)]
#[command(
    name = "ctdd_ledger",
    about = "Receipt validation, chain verification, and dossier sealing"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    /// Emit a single JSON object to stdout instead of human-readable lines.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum Cmd {
    /// Validate one receipt record (file path, or `-`/omitted for stdin).
    Validate {
        /// Receipt path; omitted or `-` reads stdin.
        path: Option<String>,
    },
    /// Verify the receipt chain across all records in a directory.
    ChainVerify {
        /// Directory containing the receipt record files.
        dir: String,
    },
    /// Seal a dossier directory (one 352-byte court record per file) into an
    /// output file.
    Seal {
        /// Dossier directory to seal.
        dir: String,
        /// Output path for the sealed dossier.
        #[arg(long)]
        output: String,
    },
    /// Verify a previously sealed dossier file against the local key store.
    Verify {
        /// Sealed dossier file path.
        file: String,
    },
    /// Emit a signed NATO court dossier (CTT-DOSSIER-v1 JSON) from the 20
    /// court receipts in a directory (N01.json..N20.json).
    DossierEmit {
        /// Directory holding the 20 court receipt JSON files.
        #[arg(long)]
        receipts_dir: String,
        /// Output dossier path.
        #[arg(long, default_value = DEFAULT_DOSSIER_OUT)]
        out: String,
    },
    /// Inspect a CTT-DOSSIER-v1 JSON dossier: recompute the Merkle root from
    /// the embedded receipts, and optionally verify the ML-DSA-65 seal.
    DossierInspect {
        /// Dossier JSON path.
        path: String,
        /// Verify the ML-DSA-65 signature (only `ML-DSA-65` is accepted).
        #[arg(long)]
        verify_pqc_signature: Option<String>,
        /// Assert the recomputed Merkle root equals the stored root. (The
        /// recomputation-and-compare always runs — fail-closed by default —
        /// the flag records the assertion in the output.)
        #[arg(long)]
        assert_merkle_root: bool,
    },
}

/// Process exit code for an IO-classified failure (no `LedgerExitCode::Io`
/// variant exists — mod.rs owns the enum — so the bin maps IO to 2 itself).
const IO_EXIT: u8 = 2;
/// Exit code when a sealed dossier fails verification (`SealVerdict::Invalid`
/// is tamper evidence, so it lands in the Tampered class, 13).
const VERIFY_INVALID_EXIT: u8 = 13;
/// Exit code when a dossier JSON is structurally invalid (wrong format, wrong
/// receipt count, recomputed Merkle root disagrees with the stored root).
/// GAP (mod.rs owns the enum): the operator spec names this
/// `DossierInvalid = 19`; mod.rs has no such variant, so the bin reuses the
/// NonMonotonic (16) class until the coordinator adds 19/20.
const DOSSIER_INVALID_EXIT: u8 = u8::from(LedgerExitCode::NonMonotonic);
/// Dossier JSON format tag.
const DOSSIER_FORMAT: &str = "CTT-DOSSIER-v1";
/// Default output path for `dossier-emit`.
const DEFAULT_DOSSIER_OUT: &str = "docs/nato/NATO_STANAG_4107_DOSSIER.json";

/// One structured outcome, shared by the human and JSON renderers.
struct Outcome {
    ok: bool,
    /// Refusal kind (`validate`/`chain`/`seal`/`verify`/`io`); None on success.
    refusal: Option<&'static str>,
    /// Refusal detail, normalized to a single line.
    detail: Option<String>,
    /// Number of receipts processed (where meaningful).
    receipts: usize,
    /// Digest of the verified/sealed artifact (where meaningful).
    digest: Option<String>,
    /// Process exit code (already domain-mapped; 0 on success).
    code: u8,
}

impl Outcome {
    fn success(receipts: usize, digest: Option<String>) -> Self {
        Self { ok: true, refusal: None, detail: None, receipts, digest, code: 0 }
    }

    fn refusal(kind: &'static str, detail: String, code: u8) -> Self {
        Self {
            ok: false,
            refusal: Some(kind),
            detail: Some(detail),
            receipts: 0,
            digest: None,
            code,
        }
    }
}

/// `{"cmd","ok","refusal","detail","receipts","digest"}` — single object, stdout.
fn to_json(cmd: &str, out: &Outcome) -> String {
    fn esc(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' | '\r' | '\t' => out.push(' '),
                c if (c as u32) < 0x20 => {}
                c => out.push(c),
            }
        }
        out
    }
    let opt_str = |o: Option<&str>| match o {
        Some(s) => format!("\"{}\"", esc(s)),
        None => "null".to_string(),
    };
    format!(
        "{{\"cmd\":\"{}\",\"ok\":{},\"refusal\":{},\"detail\":{},\"receipts\":{},\"digest\":{}}}",
        esc(cmd),
        out.ok,
        opt_str(out.refusal),
        opt_str(out.detail.as_deref()),
        out.receipts,
        opt_str(out.digest.as_deref()),
    )
}

/// Human one-line summary for a success outcome (stdout).
fn success_line(cmd: &str, out: &Outcome) -> String {
    let mut line = format!("{cmd}: ok ({} receipts", out.receipts);
    if let Some(d) = &out.digest {
        line.push_str(&format!(", digest {d}"));
    }
    line.push(')');
    line
}

/// Collapse any error Display into a single-line detail string.
fn one_line(e: impl std::fmt::Display) -> String {
    let mut s = e.to_string();
    if let Some(pos) = s.find(['\n', '\r']) {
        s.truncate(pos);
    }
    s
}

fn main() -> ExitCode {
    let cli = Cli::parse(); // clap exits 2 on usage errors by itself.
    let cmd_name = match &cli.cmd {
        Cmd::Validate { .. } => "validate",
        Cmd::ChainVerify { .. } => "chain-verify",
        Cmd::Seal { .. } => "seal",
        Cmd::Verify { .. } => "verify",
        Cmd::DossierEmit { .. } => "dossier-emit",
        Cmd::DossierInspect { .. } => "dossier-inspect",
    };
    match run(&cli) {
        Ok(out) => {
            if cli.json {
                println!("{}", to_json(cmd_name, &out));
            } else if out.ok {
                println!("{}", success_line(cmd_name, &out));
            } else {
                eprintln!(
                    "refusal: {} {}",
                    out.refusal.unwrap_or("unknown"),
                    out.detail.as_deref().unwrap_or("unknown")
                );
            }
            ExitCode::from(out.code)
        }
        Err(io_detail) => {
            // Strictly IO-classified: every domain refusal is an Ok(Outcome).
            if cli.json {
                println!(
                    "{}",
                    to_json(cmd_name, &Outcome::refusal("io", io_detail.clone(), IO_EXIT))
                );
            } else {
                eprintln!("refusal: io {io_detail}");
            }
            ExitCode::from(IO_EXIT)
        }
    }
}

/// Fallible driver. Domain calls that succeed produce `Ok(Outcome)` even on
/// refusal; only IO-classified failures (reading input paths, walking
/// directories) return `Err`, which exits 2.
fn run(cli: &Cli) -> Result<Outcome, String> {
    match &cli.cmd {
        Cmd::Validate { path } => {
            let input = read_input(path.as_deref())?;
            match validate_receipt(&input) {
                Ok(_verdict) => Ok(Outcome::success(1, None)),
                Err(e) => {
                    let code = exit_code(&e);
                    Ok(Outcome::refusal("validate", one_line(e), code))
                }
            }
        }
        Cmd::ChainVerify { dir } => {
            let records = read_records(dir)?;
            match chain_verify(&records) {
                Ok(report) => Ok(Outcome::success(report.count, Some(hex_head(&report.head)))),
                Err(e) => {
                    let code = exit_code(&e);
                    Ok(Outcome::refusal("chain", one_line(e), code))
                }
            }
        }
        Cmd::Seal { dir, output } => {
            let root = build_dossier_root(dir)?;
            let signer = load_signer()?;
            let sealed = match seal_dossier(&root, &signer) {
                Ok(s) => s,
                Err(e) => {
                    return Ok(Outcome::refusal("seal", one_line(&e), seal_exit(&e)));
                }
            };
            let bytes = sealer::encode(&sealed);
            write_output(output, &bytes)?;
            Ok(Outcome::success(0, Some(hex_head(&root.0))))
        }
        Cmd::Verify { file } => {
            let bytes = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
            let sealed = match sealer::decode(&bytes) {
                Ok(s) => s,
                Err(e) => return Ok(Outcome::refusal("verify", e.to_string(), IO_EXIT)),
            };
            let signer = load_signer()?;
            match verify_sealed_file(&sealed, &signer.public_key()) {
                SealVerdict::Valid => Ok(Outcome::success(0, Some(hex_head(&sealed.root)))),
                SealVerdict::Invalid => Ok(Outcome::refusal(
                    "verify",
                    format!("sealed dossier {} is invalid", hex_head(&sealed.root)),
                    VERIFY_INVALID_EXIT,
                )),
            }
        }
        Cmd::DossierEmit { receipts_dir, out } => {
            // G3 reconciliation: nothing in the court harness persists
            // receipt files today — an empty/missing dir is the expected
            // first-run state, so it gets a typed refusal naming the
            // prerequisite, not an IO-looking accident.
            if !std::path::Path::new(receipts_dir).is_dir() {
                return Ok(Outcome::refusal(
                    "dossier-emit",
                    format!(
                        "receipts dir {receipts_dir} is missing or not a directory; \
                         prerequisite: run the NATO courts with receipt persistence \
                         (expected {receipts_dir}/N01.json..N20.json)"
                    ),
                    u8::from(LedgerExitCode::EmptyDossier),
                ));
            }
            let receipts = load_court_receipts(receipts_dir)?;
            if receipts.len() != 20 {
                return Ok(Outcome::refusal(
                    "dossier-emit",
                    format!(
                        "expected 20 court receipts, found {}; prerequisite: run the NATO courts with receipt persistence (docs/receipts/N01..N20.json)",
                        receipts.len()
                    ),
                    u8::from(LedgerExitCode::EmptyDossier),
                ));
            }
            if !dossier_admits(&receipts) {
                let bad: Vec<String> = receipts
                    .iter()
                    .filter(|r| {
                        r.verdict != Verdict::Pass
                            || !matches!(r.positive_control, ControlResult::MutantCaught { .. })
                    })
                    .map(|r| r.name.clone())
                    .collect();
                return Ok(Outcome::refusal(
                    "dossier-emit",
                    format!("dossier admission refused: courts without Pass+MutantCaught: {bad}"),
                    u8::from(LedgerExitCode::NonMonotonic),
                ));
            }
            let root = dossier_root(&receipts);
            let signer = load_signer()?;
            let signature = signer.sign(&root);
            let kid = key_id(&signer.public_key());
            let value = dossier_json(&receipts, &root, &kid, &signature)?;
            let bytes = serde_json::to_vec_pretty(&value).map_err(|e| format!("{out}: {e}"))?;
            write_output(out, &bytes)?;
            Ok(Outcome::success(receipts.len(), Some(hex_head(&root))))
        }
        Cmd::DossierInspect { path, verify_pqc_signature, assert_merkle_root } => {
            let raw = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            let doc: DossierJson = serde_json::from_slice(&raw).map_err(|e| one_line(e))?;
            if doc.format != DOSSIER_FORMAT {
                return Ok(Outcome::refusal(
                    "dossier-inspect",
                    format!(
                        "unsupported dossier format {:?}, expected {:?}",
                        doc.format, DOSSIER_FORMAT
                    ),
                    u8::from(LedgerExitCode::SchemaVersionMismatch),
                ));
            }
            let receipts = decode_receipts(&doc.court_receipts)?;
            if receipts.len() != 20 {
                return Ok(Outcome::refusal(
                    "dossier-inspect",
                    format!("dossier holds {} court receipts, expected 20", receipts.len()),
                    DOSSIER_INVALID_EXIT,
                ));
            }
            let recomputed = dossier_root(&receipts);
            let stored = parse_root(&doc.merkle_root)?;
            if stored != recomputed {
                return Ok(Outcome::refusal(
                    "dossier-inspect",
                    format!(
                        "merkle root mismatch: stored {} recomputed {}",
                        hex_head(&stored),
                        hex_head(&recomputed)
                    ),
                    DOSSIER_INVALID_EXIT,
                ));
            }
            let mut sig_verified: Option<bool> = None;
            if let Some(alg) = verify_pqc_signature {
                if alg != "ML-DSA-65" {
                    return Ok(Outcome::refusal(
                        "dossier-inspect",
                        format!("unsupported PQC signature algorithm {alg:?}; only ML-DSA-65 is implemented"),
                        u8::from(LedgerExitCode::SchemaVersionMismatch),
                    ));
                }
                let sig = parse_signature(&doc.signature)?;
                let signer = load_signer()?;
                let pk = signer.public_key();
                let kid_ok = key_id(&pk) == parse_key_id(&doc.key_id)?;
                let ok = kid_ok && Signer::verify(&pk, &recomputed, &sig);
                sig_verified = Some(ok);
                if !ok {
                    return Ok(Outcome::refusal(
                        "dossier-inspect",
                        format!("ML-DSA-65 signature verification failed (key_id {})", doc.key_id),
                        VERIFY_INVALID_EXIT,
                    ));
                }
            }
            let mut out = Outcome::success(receipts.len(), Some(hex_head(&recomputed)));
            out.detail = Some(format!(
                "format {} ok; merkle_root {} (assert_merkle_root={}); signature verified: {}",
                doc.format,
                hex_head(&recomputed),
                assert_merkle_root,
                match sig_verified {
                    Some(true) => "true",
                    Some(false) => "false",
                    None => "not requested",
                }
            ));
            Ok(out)
        }
    }
}

/// The single mapping from a sealing-engine refusal to the process exit code.
/// `SealError` is outside `exit_code(&LedgerError)`'s domain (mod.rs owns
/// that enum), so the bin maps it: custody/serialization/entropy/subject are
/// all admission refusals, class 14.
fn seal_exit(_e: &SealError) -> u8 {
    u8::from(LedgerExitCode::AdmissionRefused)
}

/// Load (or create) the local ML-DSA-65 signing key and wrap it in a
/// domain [`Signer`]. IO/custody failures are IO-classified (exit 2).
fn load_signer() -> Result<Signer, String> {
    let key = KeyStore::new().load_or_create().map_err(|e: SealError| one_line(e))?;
    Signer::from_signing_key(key).map_err(|e| one_line(e))
}

/// Read every receipt record file in `dir` (sorted by file name), verify each
/// 352-byte record, and collect them as chain-replayable entries. An empty
/// directory yields an empty vector, which `chain_verify` refuses.
fn read_records(dir: &str) -> Result<Vec<impl Blake3ReceiptEntry>, String> {
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{dir}: {e}"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    paths.sort();
    let mut records = Vec::with_capacity(paths.len());
    for p in paths {
        let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let verified = VerifiedCourtReceipt::from_bytes_verified(&bytes)
            .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?;
        records.push(ChainRecord::of(&verified));
    }
    Ok(records)
}

/// Chain-replayable view of a verified 352-byte record.
///
/// The domain's [`chicago_tdd_tools::ledger::receipt::ChainedCourtReceipt`]
/// has no public decode constructor, so this reconstructs the exact
/// `Blake3ReceiptEntry` projection the domain uses (content = record[..304],
/// stored hash = record[320..352], prev = the receipt's parent hash) from a
/// [`VerifiedCourtReceipt`] — same bytes, same law, no second hash
/// implementation (replay still runs through the domain `chain_verify`).
struct ChainRecord {
    prev: [u8; 32],
    content: [u8; 304],
    stored: [u8; 32],
}

impl ChainRecord {
    fn of(verified: &VerifiedCourtReceipt) -> Self {
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

/// Walk a dossier directory: verify each record, group it under its court
/// (the record's validated name field), replay each court's chain through the
/// domain [`CourtLedgerBuilder`], and seal the Merkle root.
fn build_dossier_root(dir: &str) -> Result<chicago_tdd_tools::ledger::DossierRoot, String> {
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{dir}: {e}"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    paths.sort();
    let mut courts: Vec<CourtLedgerBuilder> = Vec::new();
    for p in paths {
        let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let verified = VerifiedCourtReceipt::from_bytes_verified(&bytes)
            .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?;
        let receipt = verified.receipt();
        let court = receipt.name.clone();
        let chained = ChainedCourtReceipt::from_receipt(&receipt);
        match courts.iter_mut().find(|b| b.court_id() == court) {
            Some(b) => b
                .append(chained)
                .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?,
            None => {
                let mut b = CourtLedgerBuilder::new(court);
                b.append(chained)
                    .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?;
                courts.push(b);
            }
        }
    }
    let mut dossier = DossierBuilder::new();
    for b in &courts {
        dossier
            .add_court(b.court_id().to_string(), b.head())
            .map_err(|e: DossierError| one_line(e))?;
    }
    dossier.seal().map_err(|e: DossierError| one_line(e))
}

/// Write the sealed-dossier wire bytes to `output`.
fn write_output(path: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
}

/// Read from `-`/None (stdin) or a file path as raw bytes. All failures are
/// `Err(String)` (IO-classified, exit 2).
fn read_input(path: Option<&str>) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    match path {
        None | Some("-") => {
            let _ = std::io::stdin()
                .read_to_end(&mut buf)
                .map_err(|e| format!("stdin read failed: {e}"))?;
        }
        Some(p) => buf = std::fs::read(p).map_err(|e| format!("{p}: {e}"))?,
    }
    Ok(buf)
}

/// Lowercase hex of a 32-byte digest (stdout rendering only).
fn hex_head(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// NATO dossier subcommands (lane G1)
// ---------------------------------------------------------------------------

/// Structural view of a CTT-DOSSIER-v1 JSON document. Receipt objects are
/// decoded one-by-one through the domain [`CourtReceipt::from_json`] so a
/// single malformed receipt is refused with a typed receipt-class exit
/// rather than failing the whole document decode.
#[derive(serde::Deserialize)]
struct DossierJson {
    format: String,
    court_receipts: Vec<serde_json::Value>,
    merkle_root: String,
    key_id: String,
    signature: String,
    sealed_at: u64,
}

/// Load the 20 court receipts (`N01.json`..`N20.json`) from `dir`, sorted by
/// file name. Missing dir / missing files / malformed JSON are IO- or
/// receipt-classified; the empty-dir case names the persistence prerequisite
/// (nothing in the court harness writes these files yet).
fn load_court_receipts(dir: &str) -> Result<Vec<CourtReceipt>, String> {
    let mut receipts = Vec::with_capacity(20);
    for i in 1..=20u8 {
        let path = std::path::Path::new(dir).join(format!("N{i:02}.json"));
        let bytes = std::fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!(
                    "{dir}: missing court receipt {} (prerequisite: run the NATO courts with receipt persistence, writing {dir}/N01..N20.json)",
                    path.display()
                )
            } else {
                format!("{}: {e}", path.display())
            }
        })?;
        let text = String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let receipt = CourtReceipt::from_json(&text)
            .map_err(|e| format!("{}: {}", path.display(), one_line(e)))?;
        receipts.push(receipt);
    }
    Ok(receipts)
}

/// Decode embedded receipt JSON values through the domain `from_json`.
fn decode_receipts(values: &[serde_json::Value]) -> Result<Vec<CourtReceipt>, String> {
    values
        .iter()
        .map(|v| {
            CourtReceipt::from_json(&v.to_string())
                .map_err(|e| format!("embedded court receipt: {}", one_line(e)))
        })
        .collect()
}

/// Build the dossier JSON value (CTT-DOSSIER-v1).
fn dossier_json(
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
fn parse_root(s: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(s.trim()).map_err(|e| format!("merkle_root hex: {e}"))?;
    let out: [u8; 32] = bytes
        .try_into()
        .map_err(|b: Vec<u8>| format!("merkle_root must be 64 hex chars, got {}", b.len()))?;
    Ok(out)
}

/// Parse a 16-char hex key id.
fn parse_key_id(s: &str) -> Result<[u8; 8], String> {
    let bytes = hex::decode(s.trim()).map_err(|e| format!("key_id hex: {e}"))?;
    let out: [u8; 8] = bytes
        .try_into()
        .map_err(|b: Vec<u8>| format!("key_id must be 16 hex chars, got {}", b.len()))?;
    Ok(out)
}

/// Parse a 6618-char hex ML-DSA-65 signature.
fn parse_signature(s: &str) -> Result<[u8; SIGNATURE_LEN], String> {
    let bytes = hex::decode(s.trim()).map_err(|e| format!("signature hex: {e}"))?;
    let out: [u8; SIGNATURE_LEN] = bytes.try_into().map_err(|b: Vec<u8>| {
        format!("signature must be {} bytes hex, got {}", SIGNATURE_LEN, b.len())
    })?;
    Ok(out)
}
