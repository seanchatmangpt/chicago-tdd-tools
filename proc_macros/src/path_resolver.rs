// proc-macro code runs at compile time; unwrap_or/unwrap_or_else are correct fallback patterns here
#[allow(clippy::unwrap_used)]
use std::path::{Path, PathBuf};

/// Walk ancestor dirs from manifest_dir to find the workspace root.
/// The workspace root is the first ancestor whose Cargo.toml contains "[workspace]".
pub fn workspace_root(manifest_dir: &Path) -> Option<PathBuf> {
    manifest_dir.ancestors().find(|p| {
        let cargo = p.join("Cargo.toml");
        std::fs::read_to_string(&cargo)
            .map(|s| s.contains("[workspace]"))
            .unwrap_or(false)
    }).map(|p| p.to_owned())
}

/// Extract ticket ID (e.g. "CC-001") from a filename stem.
/// Matches CC-\d+ pattern.
pub fn extract_ticket_id(filename: &str) -> Option<String> {
    let stem = Path::new(filename).file_stem()?.to_str()?;
    // Find "CC-" followed by digits
    let start = stem.find("CC-")?;
    let rest = &stem[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    // Ensure there are digits after "CC-"
    let candidate = &rest[..end];
    if candidate.len() > 3 { Some(candidate.to_string()) } else { None }
}
