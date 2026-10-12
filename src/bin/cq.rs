#![allow(clippy::unwrap_used)] // JUSTIFICATION: test module unwraps are test-only; refusal rendering uses unwrap_or
//! `cq` — operator-facing alias for dossier inspection.
//!
//! Per the release review spec, the operator review surface is
//! `cq inspect <dossier> --verify-pqc-signature ML-DSA-65 --assert-merkle-root`.
//! `cq` is a thin clap alias front-end only: `ctdd_ledger` remains the
//! canonical CLI surface, and this binary calls the exact same landed domain
//! primitives (`chicago_tdd_tools::ledger::{seal_dossier, verify_sealed_file}`,
//! `signing::{KeyStore, Signer, sealer}`) that `ctdd_ledger`'s dossier
//! seal/verify subcommands use. Cross-bin reuse of the bin-private helpers is
//! not possible, so the domain calls are mirrored here one-for-one; the shell
//! layer stays in Justfile orchestration only — this binary performs no
//! subprocess/shell-out work itself.
//!
//! Exit codes mirror `ctdd_ledger`: 0 ok, 2 IO/usage-classified, 13 sealed
//! dossier tampered (`SealVerdict::Invalid`), 14 admission refused (sealing
//! engine refusal or unsupported `--verify-pqc-signature` algorithm).
//! Clap itself exits 2 on usage errors.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use chicago_tdd_tools::ledger::receipt::{ChainedCourtReceipt, VerifiedCourtReceipt};
use chicago_tdd_tools::ledger::signing::{KeyStore, Signer};
use chicago_tdd_tools::ledger::{
    seal_dossier, verify_sealed_file, CourtLedgerBuilder, DossierBuilder, DossierError,
    LedgerError, LedgerExitCode, SealError, SealVerdict,
};

/// Process exit code for an IO-classified failure (mirrors `ctdd_ledger`).
const IO_EXIT: u8 = 2;
/// Exit code when a sealed dossier fails verification (tamper evidence).
const VERIFY_INVALID_EXIT: u8 = 13;

#[derive(Parser)]
#[command(name = "cq", about = "Dossier inspection alias (canonical surface: ctdd_ledger)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Inspect a dossier directory: replay its court chains, recompute the
    /// Merkle root, and optionally verify the PQC seal over it.
    Inspect {
        /// Dossier directory containing one 352-byte court record per file.
        path: String,
        /// Verify the PQC seal over the dossier; the only supported algorithm
        /// is the domain's ML-DSA-65 (FIPS 204). Any other value is refused.
        #[arg(long)]
        verify_pqc_signature: Option<String>,
        /// Assert the sealed dossier's attested Merkle root equals the root
        /// recomputed from the directory's records.
        #[arg(long)]
        assert_merkle_root: bool,
    },
}

/// One structured inspection outcome, rendered human-readable on stdout or as
/// a refusal line on stderr.
struct Outcome {
    ok: bool,
    /// Refusal kind (`inspect`/`verify`/`io`); None on success.
    refusal: Option<&'static str>,
    detail: Option<String>,
    /// Number of court records walked.
    receipts: usize,
    /// Lowercase hex of the recomputed Merkle root (where meaningful).
    digest: Option<String>,
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

/// Collapse any error Display into a single-line detail string.
fn one_line(e: impl std::fmt::Display) -> String {
    let mut s = e.to_string();
    if let Some(pos) = s.find(['\n', '\r']) {
        s.truncate(pos);
    }
    s
}

/// Lowercase hex of a 32-byte digest (stdout rendering only).
fn hex_head(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() -> ExitCode {
    let cli = Cli::parse(); // clap exits 2 on usage errors by itself.
    match run(&cli) {
        Ok(out) => {
            if out.ok {
                let mut line = format!("inspect: ok ({} receipts", out.receipts);
                if let Some(d) = &out.digest {
                    line.push_str(&format!(", merkle root {d}"));
                }
                line.push(')');
                println!("{line}");
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
            eprintln!("refusal: io {io_detail}");
            ExitCode::from(IO_EXIT)
        }
    }
}

/// Fallible driver. Domain refusals are `Ok(Outcome)`; only IO-classified
/// failures (reading paths, walking the dossier directory) are `Err` (exit 2).
fn run(cli: &Cli) -> Result<Outcome, String> {
    let Cmd::Inspect { path, verify_pqc_signature, assert_merkle_root } = &cli.cmd;
    let root = build_dossier_root(path)?;
    let record_count = count_records(path)?;

    // The no-flag invocation is a pure structural inspection: the directory
    // walk above already verified every 352-byte record and replayed every
    // court chain, so reaching here with a root is the pass.
    let Some(alg) = verify_pqc_signature else {
        return Ok(Outcome::success(record_count, Some(hex_head(&root.0))));
    };

    // The domain's only sealed-dossier algorithm is ML-DSA-65 (FIPS 204);
    // refusing any other requested algorithm is a typed admission refusal.
    if alg != "ML-DSA-65" {
        return Ok(Outcome::refusal(
            "inspect",
            format!(
                "unsupported --verify-pqc-signature algorithm {alg:?}: the ledger domain seals only ML-DSA-65 (FIPS 204)"
            ),
            u8::from(LedgerExitCode::AdmissionRefused),
        ));
    }

    let signer = load_signer()?;
    let sealed = match seal_dossier(&root, &signer) {
        Ok(s) => s,
        Err(e) => {
            return Ok(Outcome::refusal("inspect", one_line(&e), seal_exit(&e)));
        }
    };

    // --assert-merkle-root: the sealed dossier must attest exactly the root
    // recomputed from the directory's records, not merely some valid root.
    if *assert_merkle_root && sealed.root != root.0 {
        return Ok(Outcome::refusal(
            "inspect",
            format!(
                "merkle root mismatch: sealed dossier attests {}, directory records recompute {}",
                hex_head(&sealed.root),
                hex_head(&root.0)
            ),
            VERIFY_INVALID_EXIT,
        ));
    }

    match verify_sealed_file(&sealed, &signer.public_key()) {
        SealVerdict::Valid => Ok(Outcome::success(record_count, Some(hex_head(&sealed.root)))),
        SealVerdict::Invalid => Ok(Outcome::refusal(
            "verify",
            format!("sealed dossier {} is invalid", hex_head(&sealed.root)),
            VERIFY_INVALID_EXIT,
        )),
    }
}

/// The single mapping from a sealing-engine refusal to the process exit code
/// (mirrors `ctdd_ledger::seal_exit`: all SealError classes are admission
/// refusals, class 14 — the enum lives in the domain mod, not here).
fn seal_exit(_e: &SealError) -> u8 {
    u8::from(LedgerExitCode::AdmissionRefused)
}

/// Load (or create) the local ML-DSA-65 signing key and wrap it in a domain
/// [`Signer`]. IO/custody failures are IO-classified (exit 2).
fn load_signer() -> Result<Signer, String> {
    let key = KeyStore::new().load_or_create().map_err(|e: SealError| one_line(e))?;
    Signer::from_signing_key(key).map_err(|e| one_line(e))
}

/// Count the regular files walked in `dir` (same filter as the walker).
fn count_records(dir: &str) -> Result<usize, String> {
    let paths = sorted_files(dir)?;
    Ok(paths.len())
}

/// List `dir`'s regular files sorted by path.
fn sorted_files(dir: &str) -> Result<Vec<std::path::PathBuf>, String> {
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{dir}: {e}"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    paths.sort();
    Ok(paths)
}

/// Walk a dossier directory: verify each 352-byte record, group it under its
/// court (the record's validated name field), replay each court's chain
/// through the domain [`CourtLedgerBuilder`], and seal the Merkle root.
/// Mirrors `ctdd_ledger`'s bin-private `build_dossier_root` one-for-one.
fn build_dossier_root(dir: &str) -> Result<chicago_tdd_tools::ledger::DossierRoot, String> {
    let paths = sorted_files(dir)?;
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
