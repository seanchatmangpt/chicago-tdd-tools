//! Receipt persistence for NATO court receipts (lane P1).
//!
//! Courts mint [`CourtReceipt`]s in memory; this module is the seam that
//! makes them durable artifacts: one pretty-JSON file per receipt under a
//! directory (`N{NN}.json`, keyed by court id number), plus a loader that
//! reads the directory back.
//!
//! No overwrite protection: the directory is a regen artifact — re-running
//! `persist` rewrites the same deterministic bytes for the same receipts.

use std::fs;
use std::path::{Path, PathBuf};

use super::receipt::{CourtReceipt, ReceiptParseError};

/// Typed failure of a persist/load round trip.
#[derive(Debug)]
pub enum PersistError {
    /// Filesystem write/read/create failed.
    Io {
        /// Path the failed operation targeted.
        path: PathBuf,
        /// Human-readable IO failure description.
        message: String,
    },
    /// A receipt failed to serialize or deserialize (including the digest
    /// wire-invariant check from `CourtReceipt::from_json`).
    InvalidReceipt {
        /// Court id or file path of the receipt that failed (de)serialization.
        court_id: String,
        /// Underlying serde/parse error.
        source: ReceiptParseError,
    },
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistError::Io { path, message } => {
                write!(f, "io error at {}: {}", path.display(), message)
            }
            PersistError::InvalidReceipt { court_id, source } => {
                write!(f, "invalid receipt for {court_id}: {source}")
            }
        }
    }
}

impl std::error::Error for PersistError {}

/// Serialize each receipt to `dir/N{NN}.json` keyed by the court id's numeric
/// value, creating `dir` if needed.
///
/// The directory is a regen artifact: files are overwritten without
/// protection on re-runs. Returns the written paths in input order.
pub fn persist(
    dir: impl AsRef<Path>,
    receipts: &[CourtReceipt],
) -> Result<Vec<PathBuf>, PersistError> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir)
        .map_err(|e| PersistError::Io { path: dir.to_path_buf(), message: e.to_string() })?;

    let mut written = Vec::with_capacity(receipts.len());
    for receipt in receipts {
        let json = receipt.to_json().map_err(|e| PersistError::InvalidReceipt {
            court_id: format!("N{:02}", receipt.court_id.id_number()),
            source: e.into(),
        })?;
        let path = dir.join(format!("N{:02}.json", receipt.court_id.id_number()));
        fs::write(&path, json)
            .map_err(|e| PersistError::Io { path: path.clone(), message: e.to_string() })?;
        written.push(path);
    }
    Ok(written)
}

/// Load every `*.json` file in `dir` back into receipts, sorted by path.
/// Non-`.json` files are skipped; a `.json` file that fails to parse is a
/// typed error, not a silent skip.
pub fn load_dir(dir: impl AsRef<Path>) -> Result<Vec<CourtReceipt>, PersistError> {
    let dir = dir.as_ref();
    let entries = fs::read_dir(dir)
        .map_err(|e| PersistError::Io { path: dir.to_path_buf(), message: e.to_string() })?;

    let mut paths: Vec<PathBuf> = entries
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json"))
        .collect();
    paths.sort();

    let mut receipts = Vec::with_capacity(paths.len());
    for path in paths {
        let json = fs::read_to_string(&path)
            .map_err(|e| PersistError::Io { path: path.clone(), message: e.to_string() })?;
        let receipt = CourtReceipt::from_json(&json).map_err(|e| PersistError::InvalidReceipt {
            court_id: path.display().to_string(),
            source: e,
        })?;
        receipts.push(receipt);
    }
    Ok(receipts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control_receipts() -> Vec<CourtReceipt> {
        crate::nato::dossier_court::sample_control_set()
    }

    #[test]
    fn roundtrip_preserves_receipts() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let receipts = control_receipts();
        let paths = persist(tmp.path(), &receipts).expect("persist");
        assert_eq!(paths.len(), receipts.len());

        let loaded = load_dir(tmp.path()).expect("load_dir");
        assert_eq!(loaded, receipts);
    }

    #[test]
    fn filenames_are_nnn_json_sorted_by_court_number() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let receipts = control_receipts();
        let paths = persist(tmp.path(), &receipts).expect("persist");
        for (path, receipt) in paths.iter().zip(&receipts) {
            let expected = format!("N{:02}.json", receipt.court_id.id_number());
            assert_eq!(path.file_name().and_then(|n| n.to_str()), Some(expected.as_str()));
        }
    }

    #[test]
    fn load_dir_skips_non_json() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let receipts = control_receipts();
        persist(tmp.path(), &receipts).expect("persist");
        std::fs::write(tmp.path().join("README.md"), "not a receipt").expect("write decoy");
        std::fs::write(tmp.path().join("stray.txt"), "nope").expect("write decoy");

        let loaded = load_dir(tmp.path()).expect("load_dir");
        assert_eq!(loaded.len(), receipts.len());
    }

    #[test]
    fn load_dir_refuses_short_digest_and_names_the_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut receipts = control_receipts();
        receipts[3].digest = "abc123".to_string();
        persist(tmp.path(), &receipts).expect("persist");
        match load_dir(tmp.path()) {
            Err(PersistError::InvalidReceipt { court_id, source }) => {
                assert!(court_id.ends_with("N04.json"), "must name the file, got {court_id}");
                assert!(
                    source.to_string().contains("64 hex chars"),
                    "typed digest reason, got {source}"
                );
            }
            other => panic!("expected InvalidReceipt, got {other:?}"),
        }
    }

    #[test]
    fn load_dir_refuses_non_hex_digest() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut receipts = control_receipts();
        receipts[7].digest = "z".repeat(64);
        persist(tmp.path(), &receipts).expect("persist");
        match load_dir(tmp.path()) {
            Err(PersistError::InvalidReceipt { court_id, source }) => {
                assert!(court_id.ends_with("N08.json"), "must name the file, got {court_id}");
                assert!(
                    source.to_string().contains("64 hex chars"),
                    "typed digest reason, got {source}"
                );
            }
            other => panic!("expected InvalidReceipt, got {other:?}"),
        }
    }

    #[test]
    fn load_dir_accepts_64_hex_digest() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let receipts = control_receipts();
        assert!(receipts.iter().all(|r| r.digest.len() == 64));
        persist(tmp.path(), &receipts).expect("persist");
        let loaded = load_dir(tmp.path()).expect("64-hex digests must load");
        assert_eq!(loaded, receipts);
    }

    #[test]
    fn load_dir_missing_dir_is_io_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("no-such-dir");
        match load_dir(&missing) {
            Err(PersistError::Io { path, .. }) => assert_eq!(path, missing),
            other => panic!("expected Io error, got {other:?}"),
        }
    }
}
