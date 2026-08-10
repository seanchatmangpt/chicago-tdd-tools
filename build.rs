//! Build script to download weaver CLI binary and clone registry during compilation
//!
//! This script downloads the weaver binary from GitHub releases and clones the OpenTelemetry
//! semantic conventions registry if the weaver feature is enabled.
//! The binary is placed in target/debug/weaver or target/release/weaver for use at runtime.
//! The registry is cloned to registry/ directory in the project root.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Only run if weaver feature is enabled
    if !cfg!(feature = "weaver") {
        return;
    }

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Determine output path based on profile
    // Use CARGO_MANIFEST_DIR to get project root, then build target/<profile>/weaver
    #[allow(clippy::expect_used)] // Cargo guarantees CARGO_MANIFEST_DIR is set
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let mut output_path = manifest_dir;
    output_path.push("target");
    output_path.push(&profile);
    output_path.push("weaver");

    // Skip if binary already exists
    if output_path.exists() {
        println!(
            "cargo:warning=Weaver binary already exists at {}, skipping download",
            output_path.display()
        );
        return;
    }

    // Detect platform
    let (arch, os) = detect_platform();
    let weaver_version = "0.22.1"; // Match weaver workspace version

    // Construct download URL (weaver uses tar.xz format)
    let download_url = format!(
        "https://github.com/open-telemetry/weaver/releases/download/v{weaver_version}/weaver-{arch}-{os}.tar.xz"
    );

    println!("cargo:warning=Downloading weaver from: {download_url}");
    if let Some(parent) = output_path.parent() {
        println!("cargo:warning=Target directory: {}", parent.display());
    }

    // Download weaver binary
    if let Err(e) = download_weaver(&download_url, &output_path) {
        println!("cargo:warning=Failed to download weaver binary: {e}");
        println!("cargo:warning=Weaver will be downloaded at runtime if not found in PATH");
    } else {
        // Make binary executable (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(mut perms) = fs::metadata(&output_path).map(|m| m.permissions()) {
                perms.set_mode(0o755);
                if let Err(e) = fs::set_permissions(&output_path, perms) {
                    println!("cargo:warning=Failed to set executable permissions: {e}");
                }
            }
        }

        println!(
            "cargo:warning=Weaver binary downloaded successfully to {}",
            output_path.display()
        );
    }

    // Setup registry directory (OpenTelemetry semantic conventions)
    #[allow(clippy::expect_used)] // Cargo guarantees CARGO_MANIFEST_DIR is set
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let registry_path = manifest_dir.join("registry");

    // Skip if registry already exists
    if registry_path.exists() {
        println!(
            "cargo:warning=Registry already exists at {}, skipping clone",
            registry_path.display()
        );
        return;
    }

    // Clone registry
    if let Err(e) = clone_registry(&registry_path) {
        println!("cargo:warning=Failed to clone registry: {e}");
        println!("cargo:warning=Registry will be cloned at runtime if not found");
    } else {
        println!("cargo:warning=Registry cloned successfully to {}", registry_path.display());
    }
}

fn detect_platform() -> (&'static str, &'static str) {
    #[allow(clippy::expect_used)] // Cargo guarantees TARGET is set
    let target = env::var("TARGET").expect("TARGET environment variable must be set by Cargo");

    if target.contains("x86_64") {
        if target.contains("linux") {
            ("x86_64", "unknown-linux-gnu")
        } else if target.contains("darwin") || target.contains("macos") {
            ("x86_64", "apple-darwin")
        } else if target.contains("windows") {
            ("x86_64", "pc-windows-msvc")
        } else {
            ("x86_64", "unknown")
        }
    } else if target.contains("aarch64") || target.contains("arm64") {
        if target.contains("linux") {
            ("aarch64", "unknown-linux-gnu")
        } else if target.contains("darwin") || target.contains("macos") {
            ("aarch64", "apple-darwin")
        } else {
            ("aarch64", "unknown")
        }
    } else {
        ("unknown", "unknown")
    }
}

/// Recursively search `dir` for a file named exactly `name`, returning the first match.
///
/// GitHub release archives for `weaver` extract into a versioned subdirectory
/// (e.g. `weaver-aarch64-apple-darwin/weaver`), not a flat file directly in the
/// extraction root, so callers cannot assume `dir.join(name)`.
fn find_file_recursive(dir: &Path, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if path.file_name().is_some_and(|f| f == name) {
            return Some(path);
        }
    }
    for subdir in subdirs {
        if let Some(found) = find_file_recursive(&subdir, name) {
            return Some(found);
        }
    }
    None
}

fn download_weaver(url: &str, output_path: &PathBuf) -> Result<(), String> {
    // Create parent directory if it doesn't exist
    let output_dir = output_path
        .parent()
        .ok_or_else(|| "Output path has no parent directory".to_string())?;
    fs::create_dir_all(output_dir).map_err(|e| format!("Failed to create directory: {e}"))?;

    // Download to a distinct archive path — `output_path` is reserved for the
    // final extracted binary, and downloading the raw archive directly onto it
    // would make the later "does the binary exist yet" check vacuously true
    // (it'd be matching the still-unextracted archive, not extracted content).
    #[allow(clippy::case_sensitive_file_extension_comparisons)] // URL extensions are case-sensitive
    let archive_ext = if url.ends_with(".tar.xz") {
        "tar.xz"
    } else if url.ends_with(".tar.gz") {
        "tar.gz"
    } else if url.ends_with(".zip") {
        "zip"
    } else {
        "archive"
    };
    let archive_path = output_path.with_extension(archive_ext);

    // Use curl if available (most Unix systems)
    if Command::new("curl").arg("--version").output().is_ok() {
        let archive_str = archive_path
            .to_str()
            .ok_or_else(|| "Archive path is not valid UTF-8".to_string())?;
        let status = Command::new("curl")
            .args(["-L", "-o", archive_str, url])
            .status()
            .map_err(|e| format!("Failed to execute curl: {e}"))?;

        if !status.success() {
            return Err("curl download failed".to_string());
        }
    } else if Command::new("wget").arg("--version").output().is_ok() {
        // Fallback to wget
        let archive_str = archive_path
            .to_str()
            .ok_or_else(|| "Archive path is not valid UTF-8".to_string())?;
        let status = Command::new("wget")
            .args(["-O", archive_str, url])
            .status()
            .map_err(|e| format!("Failed to execute wget: {e}"))?;

        if !status.success() {
            return Err("wget download failed".to_string());
        }
    } else {
        return Err(
            "Neither curl nor wget found. Please install one to download weaver.".to_string()
        );
    }

    // Extract if archive (tar.xz, tar.gz, or zip)
    #[allow(clippy::case_sensitive_file_extension_comparisons)] // URL extensions are case-sensitive
    if url.ends_with(".tar.xz") {
        extract_tar_xz(&archive_path, output_path)?;
    } else if url.ends_with(".tar.gz") {
        extract_tar_gz(&archive_path, output_path)?;
    } else if url.ends_with(".zip") {
        extract_zip(&archive_path, output_path)?;
    }

    Ok(())
}

/// Extract `archive_path` (tar.xz) into its parent directory, locate the `weaver`
/// binary anywhere within the extracted tree, and move it to `final_path`.
fn extract_tar_xz(archive_path: &PathBuf, final_path: &PathBuf) -> Result<(), String> {
    let output_dir = archive_path
        .parent()
        .ok_or_else(|| "Archive path has no parent directory".to_string())?;

    let archive_str = archive_path
        .to_str()
        .ok_or_else(|| "Archive path is not valid UTF-8".to_string())?;
    let output_dir_str = output_dir
        .to_str()
        .ok_or_else(|| "Output directory path is not valid UTF-8".to_string())?;

    // Use tar with xz decompression
    let status = Command::new("tar")
        .args(["-xJf", archive_str, "-C", output_dir_str])
        .status()
        .map_err(|e| format!("Failed to extract tar.xz: {e}"))?;

    if !status.success() {
        return Err("tar extraction failed".to_string());
    }

    finish_extraction(output_dir, archive_path, final_path, "weaver")
}

/// Extract `archive_path` (tar.gz) into its parent directory, locate the `weaver`
/// binary anywhere within the extracted tree, and move it to `final_path`.
fn extract_tar_gz(archive_path: &PathBuf, final_path: &PathBuf) -> Result<(), String> {
    let output_dir = archive_path
        .parent()
        .ok_or_else(|| "Archive path has no parent directory".to_string())?;

    let archive_str = archive_path
        .to_str()
        .ok_or_else(|| "Archive path is not valid UTF-8".to_string())?;
    let output_dir_str = output_dir
        .to_str()
        .ok_or_else(|| "Output directory path is not valid UTF-8".to_string())?;

    let status = Command::new("tar")
        .args(["-xzf", archive_str, "-C", output_dir_str])
        .status()
        .map_err(|e| format!("Failed to extract tar.gz: {e}"))?;

    if !status.success() {
        return Err("tar extraction failed".to_string());
    }

    finish_extraction(output_dir, archive_path, final_path, "weaver")
}

/// Extract `archive_path` (zip) into its parent directory, locate the `weaver.exe`
/// binary anywhere within the extracted tree, and move it to `final_path`.
fn extract_zip(archive_path: &PathBuf, final_path: &PathBuf) -> Result<(), String> {
    let output_dir = archive_path
        .parent()
        .ok_or_else(|| "Archive path has no parent directory".to_string())?;

    let archive_str = archive_path
        .to_str()
        .ok_or_else(|| "Archive path is not valid UTF-8".to_string())?;
    let output_dir_str = output_dir
        .to_str()
        .ok_or_else(|| "Output directory path is not valid UTF-8".to_string())?;

    let status = Command::new("unzip")
        .args(["-o", archive_str, "-d", output_dir_str])
        .status()
        .map_err(|e| format!("Failed to extract zip: {e}"))?;

    if !status.success() {
        return Err("unzip extraction failed".to_string());
    }

    finish_extraction(output_dir, archive_path, final_path, "weaver.exe")
}

/// Shared tail of the three extractors: find `binary_name` anywhere under
/// `output_dir`, move it to `final_path`, then clean up the archive and any
/// extracted scaffolding directory it came from.
fn finish_extraction(
    output_dir: &Path,
    archive_path: &PathBuf,
    final_path: &PathBuf,
    binary_name: &str,
) -> Result<(), String> {
    let found = find_file_recursive(output_dir, binary_name)
        .ok_or_else(|| format!("could not find `{binary_name}` in extracted archive"))?;

    // The versioned scaffolding directory the binary was found in (if any), so
    // it can be cleaned up after the binary is moved out of it.
    let scaffold_dir =
        found.parent().filter(|p| *p != output_dir).map(std::path::Path::to_path_buf);

    fs::rename(&found, final_path).map_err(|e| format!("Failed to move weaver binary: {e}"))?;

    // Clean up archive and any leftover extraction scaffolding (best-effort; a
    // leftover archive/directory is harmless).
    if let Err(e) = fs::remove_file(archive_path) {
        println!("cargo:warning=failed to remove archive {}: {e}", archive_path.display());
    }
    if let Some(dir) = scaffold_dir {
        if let Err(e) = fs::remove_dir_all(&dir) {
            println!("cargo:warning=failed to remove extraction directory {}: {e}", dir.display());
        }
    }

    Ok(())
}

/// Clone OpenTelemetry semantic conventions registry
///
/// Clones the registry repository to the specified path.
/// Uses git clone with shallow clone (--depth 1) for faster download.
fn clone_registry(registry_path: &Path) -> Result<(), String> {
    // Check if git is available
    if Command::new("git").arg("--version").output().is_err() {
        return Err("git not found. Please install git to clone registry.".to_string());
    }

    let registry_url = "https://github.com/open-telemetry/semantic-conventions.git";
    let registry_str = registry_path
        .to_str()
        .ok_or_else(|| "Registry path is not valid UTF-8".to_string())?;

    println!("cargo:warning=Cloning registry from: {registry_url}");
    println!("cargo:warning=Target directory: {}", registry_path.display());

    // Clone with shallow clone for faster download
    // Use --depth 1 to only clone the latest commit
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--single-branch", registry_url, registry_str])
        .status()
        .map_err(|e| format!("Failed to execute git clone: {e}"))?;

    if !status.success() {
        return Err("git clone failed".to_string());
    }

    Ok(())
}
