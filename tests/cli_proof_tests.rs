//! Direct tests for `cli_proof::{harness, sabotage, workspace}`.
//!
//! Chicago style: real subprocesses, real files on real disk, assertions on
//! final state. No mocks. No stubs.
//!
//! Run with:
//!   `cargo test --test cli_proof_tests --features cli-proof`

#![cfg(feature = "cli-proof")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only

use chicago_tdd_tools::cli_proof::harness::CliHarnessError;
use chicago_tdd_tools::cli_proof::{CliHarness, SabotageFixture, TempWorkspace};

// ─── TempWorkspace: creation and layout ──────────────────────────────────────

/// A fresh workspace exists on disk, is empty, and `resolve` anchors relative
/// paths under the same root.
#[test]
fn workspace_new_creates_empty_dir_on_disk() {
    let ws = TempWorkspace::new().unwrap();

    let path = ws.path();
    assert!(path.is_dir(), "workspace root must be a real directory: {}", path.display());
    assert_eq!(std::fs::read_dir(path).unwrap().count(), 0, "fresh workspace must be empty");

    let resolved = ws.resolve("nested/file.txt");
    assert!(resolved.starts_with(path), "resolve() must stay under the workspace root");
}

/// `Default` delegates to `new` and produces the same on-disk guarantee.
#[test]
fn workspace_default_is_equivalent_to_new() {
    let ws = TempWorkspace::default();
    assert!(ws.path().is_dir());
    assert_eq!(std::fs::read_dir(ws.path()).unwrap().count(), 0);
}

/// `write_file` creates parent directories, `read_file` reads back the exact
/// bytes, and `assert_file_exists` agrees.
#[test]
fn workspace_write_then_read_round_trips() {
    let ws = TempWorkspace::new().unwrap();

    ws.write_file("a/b/c.txt", "hello workspace").unwrap();

    let _ = ws.assert_file_exists("a/b/c.txt");
    assert_eq!(ws.read_file("a/b/c.txt").unwrap(), "hello workspace");
}

/// `append_file` creates a missing file and appends to an existing one,
/// byte-exactly, across multiple appends.
#[test]
fn workspace_append_file_accumulates_bytes() {
    let ws = TempWorkspace::new().unwrap();

    ws.append_file("logs/out.bin", b"AAA").unwrap();
    ws.append_file("logs/out.bin", b"BBB").unwrap();

    let _ = ws.assert_file_exists("logs/out.bin");
    assert_eq!(ws.read_file("logs/out.bin").unwrap(), "AAABBB");
}

/// `corrupt_file` reads the current bytes, applies the mutation, and writes
/// the result back — observable on disk.
#[test]
fn workspace_corrupt_file_mutates_real_bytes() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("data.bin", "ABCDE").unwrap();

    ws.corrupt_file("data.bin", |bytes| bytes.into_iter().map(|b| b ^ 0x20).collect())
        .unwrap();

    // 'A' ^ 0x20 == 'a' — ASCII-safe case flip so read_file() still parses UTF-8.
    assert_eq!(ws.read_file("data.bin").unwrap(), "abcde");
}

/// `read_json` parses well-formed content and returns a typed error for
/// malformed content — `Err`, never a panic.
#[test]
fn workspace_read_json_ok_and_typed_err() {
    let ws = TempWorkspace::new().unwrap();

    ws.write_file("good.json", r#"{"name": "proof", "count": 3}"#).unwrap();
    let value = ws.read_json("good.json").unwrap();
    assert_eq!(value["name"], "proof");
    assert_eq!(value["count"], 3);

    ws.write_file("bad.json", "{not json at all").unwrap();
    let err = ws.read_json("bad.json").unwrap_err();
    assert!(!err.to_string().is_empty(), "error must carry a diagnostic message");
}

/// Reading a missing file is a typed `Err`, not a panic.
#[test]
fn workspace_read_missing_file_is_err() {
    let ws = TempWorkspace::new().unwrap();
    assert!(ws.read_file("does/not/exist.txt").is_err());
    let _ = ws.assert_file_absent("does/not/exist.txt");
}

/// Writing through the workspace leaves state observable through plain
/// `std::fs` — the file is real, not a fixture abstraction.
#[test]
fn workspace_files_are_visible_to_std_fs() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("plain.txt", "visible").unwrap();

    let via_std = std::fs::read_to_string(ws.resolve("plain.txt")).unwrap();
    assert_eq!(via_std, "visible");
}

/// `with_fixture` for a fixture that does not exist yields an empty workspace
/// (documented fallback), observable as a zero-entry directory.
#[test]
fn workspace_with_fixture_missing_yields_empty_workspace() {
    let ws = TempWorkspace::with_fixture("no-such-fixture-anywhere-xyz").unwrap();
    assert!(ws.path().is_dir());
    assert_eq!(std::fs::read_dir(ws.path()).unwrap().count(), 0);
}

/// `with_fixture` copies a real fixture directory tree into the workspace.
#[test]
fn workspace_with_fixture_copies_tree() {
    // Build a real fixture directory under tests/fixtures so with_fixture
    // finds it via CARGO_MANIFEST_DIR/../tests/fixtures/<name>/.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let fixture_root = std::path::PathBuf::from(&manifest_dir)
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("cli_proof_tests_synthetic");
    std::fs::create_dir_all(fixture_root.join("nested")).unwrap();
    std::fs::write(fixture_root.join("top.txt"), "top-level").unwrap();
    std::fs::write(fixture_root.join("nested/deep.txt"), "deep").unwrap();

    let ws = TempWorkspace::with_fixture("cli_proof_tests_synthetic").unwrap();

    let _ = ws.assert_file_exists("top.txt");
    let _ = ws.assert_file_exists("nested/deep.txt");
    assert_eq!(ws.read_file("nested/deep.txt").unwrap(), "deep");

    // Clean up the synthetic fixture source so the tree stays as before the run.
    let _ = std::fs::remove_dir_all(&fixture_root);
}

/// `assert_file_contains` passes on matching content and fails on mismatch
/// (the failure mode is a panic by contract — verified via catch_unwind so
/// the pass path is what we assert here, and the mismatch is asserted with a
/// should_panic test below).
#[test]
fn workspace_assert_helpers_pass_on_real_state() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("note.txt", "expected substring here").unwrap();

    let _ = ws.assert_file_exists("note.txt");
    let _ = ws.assert_file_contains("note.txt", "substring");
    let _ = ws.assert_dir_count("", 1);
}

#[test]
#[should_panic(expected = "expected file to be absent")]
fn workspace_assert_file_absent_panics_when_present() {
    let ws = TempWorkspace::default();
    ws.write_file("there.txt", "x").unwrap();
    let _ = ws.assert_file_absent("there.txt");
}

#[test]
#[should_panic(expected = "expected file")]
fn workspace_assert_file_contains_panics_on_mismatch() {
    let ws = TempWorkspace::default();
    ws.write_file("note.txt", "actual content").unwrap();
    let _ = ws.assert_file_contains("note.txt", "not-present-needle");
}

// ─── SabotageFixture: real corruption on real files ──────────────────────────

/// `write_garbage` replaces file content with bytes that no JSON parser
/// accepts — observable via a failed `read_json`.
#[test]
fn sabotage_write_garbage_makes_file_unparseable() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("receipt.json", r#"{"valid": true}"#).unwrap();

    SabotageFixture::write_garbage(ws.resolve("receipt.json")).unwrap();

    let _ = ws.assert_file_exists("receipt.json");
    assert!(ws.read_json("receipt.json").is_err(), "garbage bytes must not parse as JSON");
}

/// `bit_flip` inverts exactly the first byte and leaves the rest intact.
#[test]
fn sabotage_bit_flip_inverts_first_byte_only() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("blob.bin", "AB").unwrap();

    SabotageFixture::bit_flip(ws.resolve("blob.bin")).unwrap();

    let bytes = std::fs::read(ws.resolve("blob.bin")).unwrap(); // JUSTIFICATION: Test code only
    assert_eq!(bytes[0], b'A' ^ 0xFF, "first byte must be inverted");
    assert_eq!(bytes[1], b'B', "second byte must be untouched");
}

/// `bit_flip` on an empty file is a documented no-op — the file stays empty.
#[test]
fn sabotage_bit_flip_on_empty_file_is_noop() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("empty.bin", "").unwrap();

    SabotageFixture::bit_flip(ws.resolve("empty.bin")).unwrap();

    let bytes = std::fs::read(ws.resolve("empty.bin")).unwrap();
    assert!(bytes.is_empty(), "empty file must stay empty after bit_flip");
}

/// `truncate` leaves an existing but zero-byte file on disk.
#[test]
fn sabotage_truncate_leaves_empty_file() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("data.txt", "long content").unwrap();

    SabotageFixture::truncate(ws.resolve("data.txt")).unwrap();

    let _ = ws.assert_file_exists("data.txt");
    assert_eq!(std::fs::read(ws.resolve("data.txt")).unwrap().len(), 0);
}

/// `remove` deletes the file entirely — observable as absence on disk.
#[test]
fn sabotage_remove_deletes_file() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("gone.txt", "bye").unwrap();

    SabotageFixture::remove(ws.resolve("gone.txt")).unwrap();

    let _ = ws.assert_file_absent("gone.txt");
}

/// `corrupt_json` applies the closure to the parsed value and persists the
/// result as pretty-printed JSON on disk.
#[test]
fn sabotage_corrupt_json_persists_mutated_value() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("state.json", r#"{"status": "ok", "n": 1}"#).unwrap();

    SabotageFixture::corrupt_json(ws.resolve("state.json"), |mut v| {
        v["status"] = serde_json::Value::String("corrupted".into());
        v["n"] = serde_json::Value::from(999);
        v
    })
    .unwrap();

    let value = ws.read_json("state.json").unwrap();
    assert_eq!(value["status"], "corrupted");
    assert_eq!(value["n"], 999);
}

/// `corrupt_json` on an invalid-JSON file returns a typed error instead of
/// writing anything.
#[test]
fn sabotage_corrupt_json_on_invalid_json_is_err() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("bad.json", "not-json{{").unwrap();

    let result = SabotageFixture::corrupt_json(ws.resolve("bad.json"), |v| v);
    assert!(result.is_err(), "corrupt_json on invalid JSON must fail");
    // The file is untouched: still present and still unparseable.
    let _ = ws.assert_file_exists("bad.json");
    assert!(ws.read_json("bad.json").is_err());
}

/// Sabotage on a missing path propagates a typed I/O error, not a panic.
#[test]
fn sabotage_on_missing_file_is_typed_err() {
    let ws = TempWorkspace::new().unwrap();
    let missing = ws.resolve("no/such/file.json");

    assert!(SabotageFixture::write_garbage(&missing).is_err());
    assert!(SabotageFixture::bit_flip(&missing).is_err());
    assert!(SabotageFixture::truncate(&missing).is_err());
    assert!(SabotageFixture::remove(&missing).is_err());
    assert!(SabotageFixture::corrupt_json(&missing, |v| v).is_err());
}

/// Fail-closed end-to-end: garbage written by sabotage is detectable by any
/// reader that validates its input (here: JSON parsing).
#[test]
fn sabotage_then_validation_fails_closed() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("cfg.json", r#"{"trusted": true}"#).unwrap();

    SabotageFixture::truncate(ws.resolve("cfg.json")).unwrap();
    assert!(ws.read_json("cfg.json").is_err(), "empty file must fail closed");

    SabotageFixture::bit_flip(ws.resolve("cfg.json")).unwrap();
    assert!(ws.read_json("cfg.json").is_err(), "still not parseable");
}

// ─── CliHarness: real subprocess boundary crossings ─────────────────────────

/// Running a real binary that exits 0 succeeds and captures its stdout.
#[test]
fn harness_runs_real_binary_and_captures_stdout() {
    let output = CliHarness::from_path("/bin/echo").args(["hello", "harness"]).run().unwrap();

    let _ = output.assert_success();
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout, "hello harness\n");
    assert!(output.stderr.is_empty());
    let _ = output.assert_stdout_contains("hello").assert_stderr_empty();
}

/// A binary exiting non-zero is observed as failure with the real exit code.
#[test]
fn harness_observes_nonzero_exit_code() {
    let output = CliHarness::from_path("/usr/bin/false").run().unwrap();

    let _ = output.assert_failure();
    let _ = output.assert_exit_code(1);
}

/// Env vars are really delivered to the child process, observed via
/// `/usr/bin/env` printing them.
#[test]
fn harness_passes_environment_to_child() {
    let output = CliHarness::from_path("/usr/bin/env")
        .env("CLI_PROOF_TEST_MARKER", "proof-value-42")
        .run()
        .unwrap();

    let _ = output.assert_success();
    let _ = output.assert_stdout_contains("CLI_PROOF_TEST_MARKER=proof-value-42");
}

/// `current_dir` really changes the child's working directory, observed via
/// `/bin/pwd`.
#[test]
fn harness_sets_child_working_directory() {
    let output = CliHarness::from_path("/bin/pwd").current_dir("/tmp").run().unwrap();

    let _ = output.assert_success();
    let observed = output.stdout.trim();
    assert_eq!(observed, "/private/tmp", "pwd must report the requested cwd (macOS /tmp symlink)");
}

/// `workspace()` anchors the child's cwd at the temp workspace root.
#[test]
fn harness_workspace_sets_cwd_to_temp_workspace() {
    let ws = TempWorkspace::new().unwrap();
    let expected = ws.path().to_path_buf();

    let output = CliHarness::from_path("/bin/pwd").workspace(&ws).run().unwrap();

    let _ = output.assert_success();
    assert_eq!(output.stdout.trim(), expected.to_str().unwrap());
}

/// stdout is JSON and the assertion helper extracts the field.
#[test]
fn harness_asserts_json_field_in_stdout() {
    let payload = r#"{"status": "ok", "count": 2}"#;
    let output = CliHarness::from_path("/bin/echo").args([payload]).run().unwrap();

    let _ = output.assert_success();
    let _ = output.assert_stdout_json_field("status", "ok");
    let _ = output.assert_stdout_json_field("count", "2");
    let _ = output.assert_stdout_not_contains("error");
}

/// stderr from the child is captured and assertable.
#[test]
fn harness_captures_stderr() {
    // /bin/echo writes to stdout; use sh to write to stderr instead.
    let output = CliHarness::from_path("/bin/sh").args(["-c", "echo oops >&2"]).run().unwrap();

    let _ = output.assert_success();
    let _ = output.assert_stderr_contains("oops");
}

/// A nonexistent absolute path yields the typed `BinaryNotFound` error, not a
/// panic.
#[test]
fn harness_missing_absolute_path_is_binary_not_found() {
    let result = CliHarness::from_path("/definitely/not/a/real/binary/xyz").run();

    match result {
        Err(CliHarnessError::BinaryNotFound(name)) => {
            assert!(name.contains("xyz"), "error must name the missing binary: {name}");
        }
        other => panic!("expected BinaryNotFound, got: {other:?}"),
    }
}

/// A cargo-bin name that exists nowhere (env, target dir, PATH) yields
/// `BinaryNotFound`.
#[test]
fn harness_missing_cargo_bin_is_binary_not_found() {
    let result = CliHarness::cargo_bin("no-such-cargo-bin-xyz-42").run();
    assert!(
        matches!(
            &result,
            Err(CliHarnessError::BinaryNotFound(name))
                if name == "no-such-cargo-bin-xyz-42"
        ),
        "expected BinaryNotFound naming the binary, got: {result:?}"
    );
}

/// A real system binary is locatable through the PATH fallback of
/// `cargo_bin` — the resolution chain works end to end.
#[test]
fn harness_cargo_bin_path_fallback_finds_system_binary() {
    let output = CliHarness::cargo_bin("sh").args(["-c", "echo via-path"]).run().unwrap();
    let _ = output.assert_success();
    let _ = output.assert_stdout_contains("via-path");
}

/// `CliHarnessError` renders a diagnostic Display message for both variants.
#[test]
fn harness_error_display_is_diagnostic() {
    let not_found = CliHarnessError::BinaryNotFound("some-tool".into());
    assert!(not_found.to_string().contains("some-tool"));
    assert!(not_found.to_string().contains("binary not found"));

    let io = CliHarnessError::Io(std::io::Error::other("boom"));
    assert!(io.to_string().contains("boom"));
}

/// The full composite: real workspace + real binary operating on it.
#[test]
fn harness_composite_workspace_and_binary() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("input.txt", "composite proof").unwrap();

    let output = CliHarness::from_path("/bin/cat")
        .args(["input.txt"])
        .workspace(&ws)
        .run()
        .unwrap();

    let _ = output.assert_success();
    assert_eq!(output.stdout, "composite proof"); // cat echoes the file verbatim (no trailing newline)
    let _ = ws.assert_file_contains("input.txt", "proof");
}
