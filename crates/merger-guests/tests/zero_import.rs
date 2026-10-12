//! MG5 — zero-import census court over the merger-guests cdylib artifact.
//!
//! # Honest two-tier scoping (read this first)
//!
//! Tier 1 (this file, now): the **artifact-level** zero-import law. The
//! landed `chicago_tdd_tools::nato::guest_audit::guest_import_census`
//! parses the import section of the built wasm cdylib directly from its
//! bytes (hand-rolled LEB128 section walk, no wasm tooling). The court
//! drives that census over the E-M13 single artifact — one cdylib per the
//! single-artifact decision — and asserts the doctrine bound: a `Pass`
//! receipt must census `imports == ∅`; an absent artifact is an honest
//! typed skip (`Skipped{ArtifactMissing}`), never a silent pass.
//!
//! Tier 2 (later, via the temprun bridge): instantiation-level proof that
//! the module can be instantiated by a fuel-limited runtime with no host
//! functions bound. The census proves the module *declares* no imports;
//! the temprun embed will prove the module *runs* import-free. Tier 2 will
//! extend, not change, this court.
//!
//! No `println!` anywhere: failures speak through the harness.

#![allow(clippy::unwrap_used, clippy::expect_used)] // JUSTIFICATION: test code only

use chicago_tdd_tools::nato::guest_audit::{
    guest_import_census, GuestAuditError, GuestAuditStatus, SkipKind,
};

/// The E-M13 single artifact: one cdylib for the whole merger-guests leaf
/// family (workspace target dir, wasm32 release profile).
fn artifact_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32-unknown-unknown/release/merger_guests.wasm")
        .components()
        .collect()
}

#[test]
fn missing_artifact_is_an_honest_skip() {
    // A name that cannot exist: the census must return the typed skip, not
    // a silent pass and not an error.
    let missing = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32-unknown-unknown/release/merger_guests_mg5_missing.wasm");
    let receipt = guest_import_census(&missing).expect("missing artifact is Ok(Skipped)");
    assert_eq!(
        receipt.status,
        GuestAuditStatus::Skipped {
            reason: SkipKind::ArtifactMissing { path: missing.display().to_string() }
        }
    );
    assert!(receipt.imports.is_empty());
    assert_eq!(receipt.sections_checked, 0);
}

#[test]
fn single_artifact_census_passes_or_honestly_skips() {
    let path = artifact_path();
    let receipt = guest_import_census(&path).expect("census of the single artifact");
    match receipt.status {
        GuestAuditStatus::Pass => {
            // Doctrine bound: imports == ∅.
            assert!(
                receipt.imports.is_empty(),
                "merger-guests cdylib must have an empty import section; got {:?}",
                receipt.imports
            );
            assert!(receipt.sections_checked >= 1, "Pass with zero sections walked");
        }
        GuestAuditStatus::Skipped { reason: SkipKind::ArtifactMissing { path: missing } } => {
            // Honest-skip law: the absent artifact names the probed path.
            assert_eq!(missing, path.display().to_string(), "skip must name the probed path");
        }
        other => panic!("unexpected audit status: {other:?}"),
    }
}

#[test]
fn census_refuses_non_wasm_bytes() {
    // The census must never silently pass non-wasm bytes: a file with bad
    // magic is a typed NotWasm error, not a Pass and not a skip.
    let dir = std::env::temp_dir().join("merger_guests_mg5_zero_import_control");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("not_wasm.bin");
    std::fs::write(&path, b"MGI1 header, not a wasm module").expect("write control file");
    let err = guest_import_census(&path).expect_err("non-wasm bytes must be refused");
    assert!(matches!(err, GuestAuditError::NotWasm));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}
