//! Process-level courts for the `run_court` CLI (JSON-in/JSON-out court
//! runner over the NATO registry).
//!
//! Real binary runs: no mocks, real subprocess per case, asserts on real
//! state — exit codes, valid JSON receipts, typed refusal on unknown court.
//!
//! Run: `cargo test --test run_court_cli_tests --features nato`

#![cfg(feature = "nato")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

/// Path to the real binary (cargo sets this for same-package bins).
const BIN: &str = env!("CARGO_BIN_EXE_run_court");

/// Run the real binary with args, capture exit code + stdout + stderr.
fn run_bin(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("spawn run_court");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Every receipt must parse as JSON and carry a `verdict` field.
fn assert_receipts_json(stdout: &str) {
    let receipts: Vec<serde_json::Value> =
        serde_json::from_str(stdout.trim()).expect("stdout is a JSON array of receipts");
    assert!(!receipts.is_empty(), "at least one receipt");
    for r in &receipts {
        assert!(
            r.get("verdict").is_some(),
            "receipt missing verdict: {r}"
        );
        assert!(
            r.get("digest").is_some(),
            "receipt missing digest: {r}"
        );
        assert!(
            r.get("name").is_some(),
            "receipt missing court name: {r}"
        );
        assert!(
            r.get("subject_sha").is_some(),
            "receipt missing subject_sha: {r}"
        );
        assert!(
            r.get("receipt_id").is_some(),
            "receipt missing receipt_id: {r}"
        );
    }
}

#[test]
fn n01_run_emits_valid_receipt_json_exit_zero_on_pass() {
    let (code, stdout, stderr) = run_bin(&["N01", "--candidate", "cand-a", "--subject", "deadbeef"]);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(code, 0, "N01 on a plain candidate should pass");
    assert_receipts_json(&stdout);
    let receipts: Vec<serde_json::Value> = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0]["name"], "N01");
}

#[test]
fn unknown_court_name_refused_typed_error_exit_two() {
    let (code, stdout, stderr) = run_bin(&["N99"]);
    assert_eq!(code, 2, "typed refusal exit code");
    assert!(stdout.is_empty(), "no stdout on refusal");
    assert!(
        stderr.contains("REFUSED:unknown-court"),
        "typed refusal on stderr: {stderr}"
    );
}

#[test]
fn json_out_writes_receipt_file() {
    let path = std::env::temp_dir().join(format!("run-court-n01-{}.json", std::process::id()));
    let (code, stdout, stderr) = run_bin(&["N01", "--json-out", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert!(stdout.is_empty(), "receipt goes to the file, not stdout");
    let body = std::fs::read_to_string(&path).expect("json-out file exists");
    let receipts: Vec<serde_json::Value> = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0]["name"], "N01");
    std::fs::remove_file(&path).ok();
}

#[test]
fn all_run_emits_20_receipts_json() {
    let (code, stdout, stderr) = run_bin(&["all"]);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    let receipts: Vec<serde_json::Value> = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(receipts.len(), 20, "all courts N01..N20");
    assert_receipts_json(&stdout);
    // Exit code reflects the real verdicts; do not pin a specific outcome
    // here (courts own their law) — only that the code is a verdict code.
    assert!(
        code == 0 || code == 1,
        "expected verdict exit code, got {code}"
    );
}
