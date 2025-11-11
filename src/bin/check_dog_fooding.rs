#![allow(missing_docs)] // Binary crate - documentation not required

//! Check for dog fooding violations in test files
//! Detects:
//! 1. Standard assertions (assert!, assert_eq!, assert_ne!) in test files
//! 2. #[test] and #[tokio::test] attributes in test files
//!
//! Exit code: 0 if no violations, 1 if violations found

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let violations = check_dog_fooding();
    if violations > 0 {
        std::process::exit(1);
    }
}

fn check_dog_fooding() -> u32 {
    let mut violations = 0;

    println!("🔍 Checking for dog fooding violations...");

    // Check for standard assertions in test files
    println!("Checking for standard assertions (assert!, assert_eq!, assert_ne!)...");
    let standard_assertions = find_standard_assertions();
    if standard_assertions > 0 {
        println!("❌ Found {standard_assertions} instances of standard assertions in test files");
        println!("   Use library assertion macros instead: assert_ok!, assert_err!, assert_eq_msg!, assert_that!");
        violations += 1;
    } else {
        println!("✅ No standard assertions found");
    }

    // Check for #[test] attributes in test files
    println!();
    println!("Checking for #[test] attributes...");
    let test_attributes = find_test_attributes();
    if test_attributes > 0 {
        println!("❌ Found {test_attributes} instances of #[test] in test files");
        println!("   Use test! macro instead: test!(test_name, {{ /* AAA */ }})");
        violations += 1;
    } else {
        println!("✅ No #[test] attributes found");
    }

    // Check for #[tokio::test] attributes in test files
    println!();
    println!("Checking for #[tokio::test] attributes...");
    let tokio_test_attributes = find_tokio_test_attributes();
    if tokio_test_attributes > 0 {
        println!("❌ Found {tokio_test_attributes} instances of #[tokio::test] in test files");
        println!("   Use async_test! or fixture_test! macro instead: async_test!(test_name, {{ /* AAA */ }})");
        violations += 1;
    } else {
        println!("✅ No #[tokio::test] attributes found");
    }

    println!();
    if violations == 0 {
        println!("✅ All dog fooding checks passed!");
    } else {
        println!("❌ Found {violations} violation(s)");
        println!("   See above for details");
    }

    violations
}

fn find_standard_assertions() -> u32 {
    let mut count = 0;
    let patterns = [
        (r"assert!\(", "assert!"),
        (r"assert_eq!\(", "assert_eq!"),
        (r"assert_ne!\(", "assert_ne!"),
    ];

    for file in find_rust_files(&["tests/", "src/"]) {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_num, line) in content.lines().enumerate() {
                // Skip comments and stringify! macros (matches bash grep -v behavior)
                let trimmed = line.trim();
                if trimmed.starts_with("//!")
                    || trimmed.starts_with("//")
                    || line.contains("stringify!")
                    || line.contains("Binary")
                    || is_in_string_literal(line)
                {
                    continue;
                }

                for (pattern, _name) in &patterns {
                    if line.contains(pattern) {
                        println!("   {}:{}: {}", file.display(), line_num + 1, line.trim());
                        count += 1;
                    }
                }
            }
        }
    }

    count
}

fn find_test_attributes() -> u32 {
    let mut count = 0;

    for file in find_rust_files(&["tests/", "src/"]) {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_num, line) in content.lines().enumerate() {
                // Skip comments (matches bash grep -v behavior)
                if line.trim().starts_with("//!") || line.contains("Binary") {
                    continue;
                }

                // Match patterns: ^#\[test\] or ^\s+#\[test\] (matches bash regex)
                let trimmed = line.trim();
                if trimmed == "#[test]" || trimmed.starts_with("#[test]") {
                    println!("   {}:{}: {}", file.display(), line_num + 1, line.trim());
                    count += 1;
                }
            }
        }
    }

    count
}

fn find_tokio_test_attributes() -> u32 {
    let mut count = 0;

    for file in find_rust_files(&["tests/", "src/"]) {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_num, line) in content.lines().enumerate() {
                // Skip comments (matches bash grep -v behavior)
                if line.trim().starts_with("//!") || line.contains("Binary") {
                    continue;
                }

                // Match patterns: ^#\[tokio::test\] or ^\s+#\[tokio::test\] (matches bash regex)
                let trimmed = line.trim();
                if trimmed == "#[tokio::test]" || trimmed.starts_with("#[tokio::test]") {
                    println!("   {}:{}: {}", file.display(), line_num + 1, line.trim());
                    count += 1;
                }
            }
        }
    }

    count
}

fn find_rust_files(dirs: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for dir in dirs {
        let path = Path::new(dir);
        if path.exists() {
            walk_dir(path, &mut files);
        }
    }

    // Exclude check_dog_fooding.rs itself (it contains patterns that would be false positives)
    files.retain(|f| !f.to_string_lossy().contains("check_dog_fooding.rs"));

    files
}

fn is_in_string_literal(line: &str) -> bool {
    // Simple heuristic: if line contains string patterns like r"..." or "..." with assert patterns, skip
    // This is a simple check - full Rust parsing would be more accurate but overkill
    let trimmed = line.trim();
    // Check if line contains string literal patterns with assert patterns
    if trimmed.contains(r#"r"assert"#)
        || trimmed.contains(r#""assert"#)
        || trimmed.contains(r#"r\"assert"#)
        || (trimmed.contains("assert") && (trimmed.contains(r#"r""#) || trimmed.contains(r#"""#)))
    {
        // Additional check: is it in a string literal context?
        if trimmed.starts_with("let") || trimmed.starts_with("const") || trimmed.contains("= ") {
            return true;
        }
    }
    false
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, files);
            } else if path.extension() == Some(std::ffi::OsStr::new("rs")) {
                files.push(path);
            }
        }
    }
}
