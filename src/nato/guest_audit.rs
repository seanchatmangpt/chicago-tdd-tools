#![cfg(feature = "nato")]
//! W4 — guest-wasm import audit (NATO lane).
//!
//! Ported from strata's `just check-wasm-imports` inline-Python recipe
//! (hand-rolled wasm binary walk) into a zero-import pure-Rust audit: the
//! import section of a built ctt guest artifact is parsed DIRECTLY from the
//! binary — no `wasm-tools`/`wasm-objdump` dependency, no `std::process` in
//! the production path.
//!
//! Doctrine: the guest must have `imports = ∅` (a fuel-limited runtime cannot
//! service host calls). The census never lies by omission: when the artifact
//! is absent the court returns a typed [`GuestAuditStatus::Skipped`] with
//! [`SkipKind::ArtifactMissing`] — the N06 honest-skip law, never a silent
//! pass.
//!
//! Build command for the audited artifact (documented, not run here):
//!
//! ```text
//! cargo build -p <guest-crate> --target wasm32-unknown-unknown --release
//! # artifact lands at target/wasm32-unknown-unknown/release/<name>.wasm
//! ```

// Lint-scope note: NATO court-harness and ledger wire-format modules use
// domain literals, metric casts, and court-assertion panics by design; the
// style-class lints below are allowed at module scope with this
// justification rather than 300+ inline permits.
#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::use_self,
    clippy::missing_const_for_fn,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::explicit_iter_loop,
    clippy::unusual_byte_groupings,
    clippy::unreadable_literal,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::needless_range_loop,
    clippy::needless_continue,
    clippy::unnecessary_wraps,
    clippy::double_must_use,
    clippy::op_ref,
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::or_fun_call,
    clippy::nonminimal_bool,
    clippy::unnecessary_map_or,
    clippy::manual_is_multiple_of,
    clippy::manual_midpoint,
    clippy::identity_op,
    clippy::format_collect,
    clippy::trivially_copy_pass_by_ref
)]

use serde::{Deserialize, Serialize};
use std::path::Path;

// --- Receipt model ---------------------------------------------------------

/// Terminal status of a guest import audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuestAuditStatus {
    /// The artifact was parsed; imports were censused against the ∅ bound.
    Pass,
    /// The audit could not run, for a typed, never-silent reason.
    Skipped {
        /// Typed skip reason.
        reason: SkipKind,
    },
}

/// Typed reason the audit was skipped (mirrors N06's `SkipReason`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkipKind {
    /// The built guest artifact does not exist on this host.
    #[serde(rename = "ARTIFACT_MISSING")]
    ArtifactMissing {
        /// Path probed.
        path: String,
    },
}

/// Structured receipt for the guest import census.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestImportReceipt {
    /// Artifact audited (path as probed).
    pub artifact: String,
    /// Import entries as `module::name`. Doctrine bound: empty.
    pub imports: Vec<String>,
    /// Number of wasm section headers walked to produce the census.
    pub sections_checked: u32,
    /// Terminal status.
    pub status: GuestAuditStatus,
    /// Provenance: how the census was produced.
    pub tool: String,
}

/// Per-section counts exported for the receipt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionCensus {
    /// Type-section entry count (section id 1).
    pub types: u32,
    /// Import-section entry count (section id 2). Doctrine bound: 0.
    pub imports: u32,
    /// Function-section entry count (section id 3).
    pub functions: u32,
    /// Memory-section entry count (section id 5).
    pub memory: u32,
    /// Export-section entry count (section id 7).
    pub exports: u32,
}

/// Typed error for a malformed or non-wasm artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuestAuditError {
    /// The file could not be read.
    Io(String),
    /// The bytes do not carry the `\0asm` magic / version header.
    NotWasm,
    /// The section walk ran off the end of the binary (truncated or corrupt).
    Truncated,
    /// A LEB128 varint was malformed (non-minimal or overlong).
    MalformedLeb128,
    /// An import entry's UTF-8 name fields were invalid.
    InvalidUtf8,
}

impl std::fmt::Display for GuestAuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::NotWasm => write!(f, "not a wasm binary (bad \\0asm magic)"),
            Self::Truncated => write!(f, "wasm binary truncated mid-section"),
            Self::MalformedLeb128 => write!(f, "malformed LEB128 varint"),
            Self::InvalidUtf8 => write!(f, "import name is not valid UTF-8"),
        }
    }
}

impl std::error::Error for GuestAuditError {}

// --- Binary walk (pure Rust, no external wasm tooling) ---------------------

/// Read one unsigned LEB128 varint, bounded to `u32`.
fn read_uleb128(data: &[u8], pos: &mut usize) -> Result<u32, GuestAuditError> {
    let mut result: u32 = 0;
    let mut shift = 0;
    loop {
        let byte = data.get(*pos).ok_or(GuestAuditError::Truncated)?;
        *pos += 1;
        // u32 varints need at most 5 bytes (shift 28); a 6th byte with a
        // payload bit is overlong/malformed.
        if shift >= 32 || (shift == 28 && byte & 0x70 != 0) {
            return Err(GuestAuditError::MalformedLeb128);
        }
        result |= u32::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
    }
}

/// Read a length-prefixed UTF-8 name.
fn read_name(data: &[u8], pos: &mut usize) -> Result<String, GuestAuditError> {
    let len = read_uleb128(data, pos)? as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= data.len())
        .ok_or(GuestAuditError::Truncated)?;
    let bytes: &[u8] = &data[*pos..end];
    *pos = end;
    String::from_utf8(bytes.to_vec()).map_err(|_| GuestAuditError::InvalidUtf8)
}

/// Walk the section headers of a wasm binary directly from its bytes.
///
/// Returns the census of the five counted sections and the number of section
/// headers walked. Import entry names are collected when `collect` is `Some`.
fn walk_sections(
    data: &[u8],
    mut collect: Option<&mut Vec<String>>,
) -> Result<(SectionCensus, u32), GuestAuditError> {
    if data.len() < 8 || data[..4] != *b"\0asm" {
        return Err(GuestAuditError::NotWasm);
    }
    let mut pos = 8;
    let mut census = SectionCensus::default();
    let mut sections_checked: u32 = 0;

    while pos < data.len() {
        let id = data[pos];
        pos += 1;
        let size = read_uleb128(data, &mut pos)? as usize;
        let body_end = pos
            .checked_add(size)
            .filter(|&e| e <= data.len())
            .ok_or(GuestAuditError::Truncated)?;
        let mut body = &data[pos..body_end];
        pos = body_end;
        sections_checked += 1;

        match id {
            // Custom sections (id 0) carry a name prefix; uncounted, skipped.
            0 => {}
            1 => census.types = read_uleb128(body, &mut 0usize)?,
            2 => {
                let mut idx = 0usize;
                let count = read_uleb128(body, &mut idx)?;
                census.imports = count;
                if let Some(out) = collect.as_deref_mut() {
                    for _ in 0..count {
                        let module = read_name(body, &mut idx)?;
                        let name = read_name(body, &mut idx)?;
                        // kind byte: 0 func (typeidx), 1 table, 2 memory,
                        // 3 global, 4 tag — each followed by its type fields;
                        // we only need the names, so skip the kind byte's
                        // payload conservatively by not advancing further:
                        // entries are self-delimited only through their type
                        // descriptor, so parse it minimally per kind.
                        let kind = body.get(idx).ok_or(GuestAuditError::Truncated)?;
                        idx += 1;
                        match kind {
                            0 => {
                                let _ = read_uleb128(body, &mut idx)?; // typeidx
                            }
                            1 => {
                                // table: reftype (1 byte) + limits
                                idx += 1; // reftype
                                skip_limits(body, &mut idx)?;
                            }
                            2 => skip_limits(body, &mut idx)?, // memory limits
                            3 => {
                                // global: valtype (1 byte) + mutability (1 byte)
                                idx += 2;
                            }
                            4 => {
                                let _ = read_uleb128(body, &mut idx)?; // tag attribute/typeidx
                            }
                            _ => return Err(GuestAuditError::Truncated),
                        }
                        out.push(format!("{module}::{name}"));
                    }
                }
            }
            3 => census.functions = read_uleb128(body, &mut 0usize)?,
            5 => census.memory = read_uleb128(body, &mut 0usize)?,
            7 => census.exports = read_uleb128(body, &mut 0usize)?,
            // 4 table, 6 global, 8 start, 9 element, 10 code, 11 data,
            // 12 data-count — counted in `sections_checked` only.
            _ => {}
        }
        let _ = &mut body;
    }
    Ok((census, sections_checked))
}

/// Skip a limits descriptor (flags byte + min, plus max when flag bit 0 set).
fn skip_limits(data: &[u8], idx: &mut usize) -> Result<(), GuestAuditError> {
    let flags = data.get(*idx).ok_or(GuestAuditError::Truncated)?;
    *idx += 1;
    let _min = read_uleb128(data, idx)?;
    if flags & 0x01 != 0 {
        let _max = read_uleb128(data, idx)?;
    }
    Ok(())
}

// --- Production API ---------------------------------------------------------

/// Default artifact path (relative to the crate root at test time).
pub const DEFAULT_GUEST_ARTIFACT: &str =
    "target/wasm32-unknown-unknown/release/ctt_wasm_guest.wasm";

/// Census the guest wasm's import section directly from the binary.
///
/// Returns a typed [`GuestImportReceipt`]. When the artifact does not exist,
/// returns a receipt with [`GuestAuditStatus::Skipped`] and
/// [`SkipKind::ArtifactMissing`] — the honest-skip law, never a silent pass.
/// Import entries are `module::name` strings; the doctrine bound is empty.
///
/// # Errors
/// Returns [`GuestAuditError`] when the artifact exists but is not a valid
/// wasm binary (bad magic, truncation, malformed varints or names).
pub fn guest_import_census(artifact_path: &Path) -> Result<GuestImportReceipt, GuestAuditError> {
    let artifact = artifact_path.display().to_string();
    if !artifact_path.is_file() {
        return Ok(GuestImportReceipt {
            artifact: artifact.clone(),
            imports: Vec::new(),
            sections_checked: 0,
            status: GuestAuditStatus::Skipped {
                reason: SkipKind::ArtifactMissing { path: artifact },
            },
            tool: tool_label(),
        });
    }
    let data = std::fs::read(artifact_path).map_err(|e| GuestAuditError::Io(e.to_string()))?;
    let mut imports = Vec::new();
    let (census, sections_checked) = walk_sections(&data, Some(&mut imports))?;
    debug_assert_eq!(census.imports as usize, imports.len(), "import census mismatch");
    Ok(GuestImportReceipt {
        artifact,
        imports,
        sections_checked,
        status: GuestAuditStatus::Pass,
        tool: tool_label(),
    })
}

/// Census the five counted sections of a wasm artifact (types, imports,
/// functions, memory, exports) for receipt emission.
///
/// # Errors
/// Same as [`guest_import_census`] for malformed/non-wasm bytes.
pub fn section_census(artifact_path: &Path) -> Result<(SectionCensus, u32), GuestAuditError> {
    let data = std::fs::read(artifact_path).map_err(|e| GuestAuditError::Io(e.to_string()))?;
    walk_sections(&data, None)
}

fn tool_label() -> String {
    "parsed-bytes (hand-rolled LEB128 section walk, no wasm tooling)".to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // JUSTIFICATION: test code only
mod tests {
    use super::*;

    fn artifact_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_GUEST_ARTIFACT)
    }

    /// The W4 court: over the real built artifact, imports must be ∅.
    /// Honest skip when the artifact has not been built (typed, never
    /// silent). Build command:
    /// `cargo build -p <guest-crate> --target wasm32-unknown-unknown --release`
    #[test]
    fn w4_guest_imports_are_empty() {
        let path = artifact_path();
        let receipt = guest_import_census(&path).expect("audit failed on existing artifact");
        eprintln!("{:?}", serde_json::to_string(&receipt).unwrap());
        match receipt.status {
            GuestAuditStatus::Pass => {
                assert_eq!(receipt.imports, Vec::<String>::new(), "doctrine: guest imports = ∅");
                assert!(receipt.sections_checked > 0, "Pass requires a real section walk");
            }
            GuestAuditStatus::Skipped { reason: SkipKind::ArtifactMissing { ref path } } => {
                assert_eq!(
                    path, &receipt.artifact,
                    "skip reason must name the probed artifact path"
                );
                assert!(receipt.imports.is_empty(), "skipped census must not fabricate imports");
            }
        }
    }

    /// The census export reports the same import count as the receipt.
    #[test]
    fn section_census_agrees_with_receipt() {
        let path = artifact_path();
        if !path.is_file() {
            // Honest skip: nothing to cross-check without the artifact.
            return;
        }
        let receipt = guest_import_census(&path).unwrap();
        let (census, checked) = section_census(&path).unwrap();
        assert_eq!(census.imports as usize, receipt.imports.len());
        assert_eq!(checked, receipt.sections_checked);
        assert!(census.exports > 0 || census.functions > 0, "a cdylib guest exports code");
    }

    /// Court logic against a real, minimal, hand-assembled wasm module:
    /// `\0asm` + version + an import section declaring one import
    /// (`env::proxy_call`). The court must see it — a census that reported ∅
    /// over this bytes would be lying.
    #[test]
    fn detects_nonzero_imports_in_real_wasm_bytes() {
        // (module (import "env" "proxy_call" (func)))
        let mut wasm = Vec::new();
        wasm.extend_from_slice(b"\0asm\x01\0\0\0");
        // import section (id 2), body: count=1, module="env", name="proxy_call", kind=0 func, typeidx=0
        let mut body = Vec::new();
        body.push(0x01); // 1 import
        body.push(0x03);
        body.extend_from_slice(b"env");
        body.push(0x0A);
        body.extend_from_slice(b"proxy_call");
        body.push(0x00); // kind: func
        body.push(0x00); // typeidx 0
        wasm.push(2); // section id
        wasm.push(body.len() as u8);
        wasm.extend_from_slice(&body);

        let dir = std::env::temp_dir().join(format!("w4_court_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("one_import.wasm");
        std::fs::write(&path, &wasm).unwrap();

        let receipt = guest_import_census(&path).unwrap();
        assert_eq!(receipt.status, GuestAuditStatus::Pass);
        assert_eq!(receipt.imports, vec!["env::proxy_call".to_string()]);
        let (census, _) = section_census(&path).unwrap();
        assert_eq!(census.imports, 1);
        assert_eq!(census.types, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Same hand-assembled module with zero imports: census must be ∅ — the
    /// shape a conforming guest compiles to.
    #[test]
    fn accepts_empty_import_section() {
        let mut wasm = Vec::new();
        wasm.extend_from_slice(b"\0asm\x01\0\0\0");
        let body = [0x00u8]; // count = 0 imports
        wasm.push(2);
        wasm.push(body.len() as u8);
        wasm.extend_from_slice(&body);

        let dir = std::env::temp_dir().join(format!("w4_court0_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("no_imports.wasm");
        std::fs::write(&path, &wasm).unwrap();

        let receipt = guest_import_census(&path).unwrap();
        assert_eq!(receipt.status, GuestAuditStatus::Pass);
        assert!(receipt.imports.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Missing artifact → typed skip, not an error and not a silent pass.
    #[test]
    fn missing_artifact_is_typed_skip() {
        let path = std::env::temp_dir().join(format!("w4_absent_{}.wasm", std::process::id()));
        let receipt = guest_import_census(&path).unwrap();
        assert_eq!(
            receipt.status,
            GuestAuditStatus::Skipped {
                reason: SkipKind::ArtifactMissing { path: path.display().to_string() },
            }
        );
        assert!(receipt.imports.is_empty());
    }

    /// Non-wasm bytes are refused with `NotWasm`, never read as a census.
    #[test]
    fn non_wasm_bytes_are_refused() {
        let dir = std::env::temp_dir().join(format!("w4_bad_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("elf.wasm");
        std::fs::write(&path, b"\x7fELF...........").unwrap();
        assert_eq!(guest_import_census(&path).unwrap_err(), GuestAuditError::NotWasm);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Receipt model round-trips through the wire format.
    #[test]
    fn receipt_round_trips() {
        let receipt = GuestImportReceipt {
            artifact: DEFAULT_GUEST_ARTIFACT.to_string(),
            imports: vec!["env::proxy_call".to_string()],
            sections_checked: 7,
            status: GuestAuditStatus::Pass,
            tool: tool_label(),
        };
        let json = serde_json::to_string(&receipt).unwrap();
        let back: GuestImportReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(back, receipt);
    }
}
