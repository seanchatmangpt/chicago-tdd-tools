//! Direct coverage for `cli_proof::TempWorkspace` (Chicago style: real file I/O,
//! assert final state). Pins the documented W27 caveat: absolute `rel` paths
//! ESCAPE the workspace via `PathBuf::join` — anchoring/refusal is a registered
//! work order, not fixed here.

#![cfg(feature = "cli-proof")]

use chicago_tdd_tools::cli_proof::TempWorkspace;

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn creates_real_directory_on_disk() {
    let ws = TempWorkspace::new().unwrap();
    let path = ws.path();
    assert!(path.is_dir(), "workspace root is not a directory: {}", path.display());
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn write_then_read_roundtrip() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("nested/dir/file.txt", "hello world").unwrap();
    let content = ws.read_file("nested/dir/file.txt").unwrap();
    assert_eq!(content, "hello world");
    assert!(ws.resolve("nested/dir/file.txt").exists());
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn append_file_accumulates_bytes() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("log/app.log", "line1\n").unwrap();
    ws.append_file("log/app.log", b"line2\n").unwrap();
    ws.append_file("log/app.log", b"line3\n").unwrap();
    assert_eq!(ws.read_file("log/app.log").unwrap(), "line1\nline2\nline3\n");
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn append_file_creates_missing_file_and_parents() {
    let ws = TempWorkspace::new().unwrap();
    ws.append_file("fresh/sub/out.bin", &[0u8, 1, 2, 3]).unwrap();
    let path = ws.resolve("fresh/sub/out.bin");
    assert_eq!(std::fs::read(&path).unwrap(), vec![0u8, 1, 2, 3]);
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn corrupt_file_rewrites_bytes_via_closure() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("data/blob.txt", "original").unwrap();
    ws.corrupt_file("data/blob.txt", |bytes| bytes.iter().map(|b| b.wrapping_add(1)).collect())
        .unwrap();
    assert_eq!(ws.read_file("data/blob.txt").unwrap(), "psjhjobm");
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn remove_deletes_file_from_workspace() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("gone.txt", "bye").unwrap();
    let _ = ws.assert_file_exists("gone.txt");
    std::fs::remove_file(ws.resolve("gone.txt")).unwrap();
    let _ = ws.assert_file_absent("gone.txt");
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn truncate_empties_file_content() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("trunc.txt", "long content here").unwrap();
    ws.corrupt_file("trunc.txt", |_bytes| Vec::new()).unwrap();
    let path = ws.resolve("trunc.txt");
    assert!(path.exists(), "file must still exist after truncate");
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
}

// ── W27 documented-behavior regression pins ────────────────────────────────

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn w27_absolute_rel_path_escapes_workspace_documented_behavior() {
    let ws = TempWorkspace::new().unwrap();
    let escaped = std::env::temp_dir().join(format!(
        "temp_ws_escape_probe_{}_{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Documented caveat: an absolute `rel` is joined as-is, escaping the sandbox.
    let abs_str = escaped.to_str().unwrap();
    ws.write_file(abs_str, "escaped").unwrap();

    assert!(
        escaped.exists(),
        "W27: absolute rel path should escape the workspace; expected file at {}",
        escaped.display()
    );
    assert_eq!(std::fs::read_to_string(&escaped).unwrap(), "escaped");
    assert!(
        !ws.resolve(abs_str).starts_with(ws.path()),
        "resolve should reflect the unanchored join"
    );

    // Clean up the escaped artifact.
    std::fs::remove_file(&escaped).unwrap();
    assert!(!escaped.exists(), "escaped file must be cleaned up");
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[test]
fn w27_relative_rel_path_stays_inside_workspace_documented_behavior() {
    let ws = TempWorkspace::new().unwrap();
    ws.write_file("inside/dir/stays.txt", "contained").unwrap();

    let path = ws.resolve("inside/dir/stays.txt");
    assert!(
        path.starts_with(ws.path()),
        "relative rel must resolve under workspace root: {}",
        path.display()
    );
    assert!(path.exists());
    assert_eq!(ws.read_file("inside/dir/stays.txt").unwrap(), "contained");
}
