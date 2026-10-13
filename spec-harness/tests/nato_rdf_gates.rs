//! Differential: real NATO court receipts -> RDF lift -> SPARQL gates.
//!
//! Runs the receipt->RDF lift over the real emitter output
//! (`docs/receipts/`, produced by the real runner under
//! `CTT_EMIT_RECEIPTS=1` — never synthesized), runs the SPARQL gate set via
//! `spec-harness/run_gates.py` (rdflib), and asserts gate verdicts equal the
//! receipt verdicts. Sabotage twin: flipping one receipt's verdict in a tmp
//! copy must flip the gate report.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chatman_spec_harness::nato_rdf::lift_receipts_dir;

/// Workspace root (spec-harness lives at `<root>/spec-harness`).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("spec-harness has a parent")
        .to_path_buf()
}

fn receipts_dir() -> PathBuf {
    workspace_root().join("docs/receipts")
}

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "ctt-nato-gates-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .subsec_nanos()
    ));
    fs::create_dir_all(&d).expect("mkdir");
    d
}

fn run_gates_py(turtle: &Path) -> (i32, String) {
    let script = workspace_root().join("spec-harness/run_gates.py");
    let out = Command::new("python3")
        .arg(&script)
        .arg(turtle)
        .output()
        .expect("spawn python3 run_gates.py");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// One lifted receipt per court file, exactly the real runner's 20-court set.
#[test]
fn lift_covers_all_20_real_courts() {
    let dir = receipts_dir();
    let turtle =
        lift_receipts_dir(&dir).expect("real receipts must exist (CTT_EMIT_RECEIPTS=1 run)");
    let count = turtle.matches("a ct:CourtReceipt").count();
    assert_eq!(count, 20, "must lift exactly the 20 NATO courts");
    for n in 1..=20 {
        let court = format!("N{n:02}");
        assert!(
            turtle.contains(&format!("ct:courtId \"{court}\"")),
            "court {court} missing from lift"
        );
    }
}

/// Differential: gates on the real receipts must all pass (verdicts Pass,
/// seals present) and exit 0.
#[test]
fn gates_admit_real_receipts() {
    let dir = receipts_dir();
    let turtle = lift_receipts_dir(&dir).expect("lift real receipts");
    let tmp = tmp_dir("real");
    let ttl = tmp.join("receipts.ttl");
    fs::write(&ttl, &turtle).expect("write turtle");

    let (code, stdout) = run_gates_py(&ttl);
    assert_eq!(
        code, 0,
        "gates must admit the real receipt set; report:\n{stdout}"
    );
    assert!(stdout.contains("\"total_receipts\": 20"), "report:\n{stdout}");
    assert!(stdout.contains("\"ok\": true"), "report:\n{stdout}");
    fs::remove_dir_all(&tmp).ok();
}

/// Sabotage twin: flipping one receipt verdict (N01 Pass -> Fail) in a tmp
/// copy must flip the gate report (exit 1, N01 named as a violation).
#[test]
fn sabotage_flips_gate_report() {
    let src = receipts_dir();
    let tmp = tmp_dir("sabotage");
    for entry in fs::read_dir(&src).expect("read receipts") {
        let entry = entry.expect("dir entry");
        let dest = tmp.join(entry.file_name());
        fs::copy(entry.path(), &dest).expect("copy receipt");
    }
    let n01 = tmp.join("N01.json");
    let json = fs::read_to_string(&n01).expect("read N01");
    let sabotaged = json.replace("\"verdict\": \"Pass\"", "\"verdict\": \"Fail\"");
    assert_ne!(json, sabotaged, "sabotage must change the receipt bytes");
    fs::write(&n01, sabotaged).expect("write sabotaged N01");

    let turtle = lift_receipts_dir(&tmp).expect("lift sabotaged set");
    let ttl = tmp.join("receipts.ttl");
    fs::write(&ttl, &turtle).expect("write turtle");

    let (code, stdout) = run_gates_py(&ttl);
    assert_eq!(code, 1, "sabotaged set must fail the gates; report:\n{stdout}");
    assert!(
        stdout.contains("N01") && stdout.contains("Fail"),
        "gate report must name N01/Fail; report:\n{stdout}"
    );
    fs::remove_dir_all(&tmp).ok();
}

/// The lift refuses an unsealed receipt: a corrupted 32-hex digest must be a
/// typed refusal, never a lifted node.
#[test]
fn lift_refuses_short_digest() {
    let src = receipts_dir();
    let tmp = tmp_dir("digest");
    for entry in fs::read_dir(&src).expect("read receipts") {
        let entry = entry.expect("dir entry");
        fs::copy(entry.path(), tmp.join(entry.file_name())).expect("copy receipt");
    }
    let n01 = tmp.join("N01.json");
    let json = fs::read_to_string(&n01).expect("read N01");
    // Replace the first 64-hex digest occurrence with 32 hex chars.
    let digests: Vec<String> = json
        .split('"')
        .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_string)
        .collect();
    assert!(!digests.is_empty(), "N01 must contain a 64-hex digest");
    let sabotaged = json.replacen(&digests[0], &digests[0][..32], 1);
    assert_ne!(json, sabotaged);
    fs::write(&n01, sabotaged).expect("write truncated-digest N01");

    let err = lift_receipts_dir(&tmp).expect_err("must refuse unsealed receipt");
    assert!(
        err.contains("64 hex chars"),
        "refusal must name the seal law, got {err}"
    );
    fs::remove_dir_all(&tmp).ok();
}
