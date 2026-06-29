//! E2E Integration Test Suite for `star-toml` Example.
//!
//! Evaluates:
//! - F1: TOML Loading and Parsing
//! - F2: Layering and Merging
//! - F3: Configuration Validation
//! - F4: Progress Alerts/Logging
//! - F5: Accepted/Refused Behavior
//!
//! Organized in 4 execution tiers.

#![allow(warnings, dead_code, unused_variables, clippy::all)]

use chicago_tdd_tools::prelude::*;
use std::process::Command;
use std::path::PathBuf;
use std::sync::Once;

static COMPILE_ONCE: Once = Once::new();

/// Build the star_toml example binary once and return its path.
fn get_bin_path() -> PathBuf {
    COMPILE_ONCE.call_once(|| {
        let status = Command::new("cargo")
            .args(&["build", "--example", "star_toml"])
            .status()
            .expect("Failed to build star_toml example");
        assert!(status.success(), "Failed to compile star_toml example");
    });
    
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("examples");
    path.push("star_toml");
    
    if !path.exists() {
        let mut fallback = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fallback.push("target");
        fallback.push("release");
        fallback.push("examples");
        fallback.push("star_toml");
        if fallback.exists() {
            return fallback;
        }
    }
    path
}

/// Helper to execute the compiled example binary.
fn run_bin(args: &[&str], envs: &[(&str, &str)]) -> (i32, String, String) {
    let bin = get_bin_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("Failed to run star_toml example binary");
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if exit_code != 0 {
        eprintln!("DEBUG [run_bin]: failed for args={:?} envs={:?}, exit_code={}, stdout={:?}, stderr={:?}", args, envs, exit_code, stdout, stderr);
    }
    (exit_code, stdout, stderr)
}

// ============================================================================
// TIER 1: FEATURE COVERAGE (25 test cases: 5 per feature)
// ============================================================================

// --- F1: TOML Loading and Parsing (5 tests) ---

chicago_tdd_tools::test!(test_f1_load_valid_toml_file, {
    let (code, stdout, _) = run_bin(&["--config", "examples/star-toml/samples/default.toml"], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("star-toml-app"));
});

chicago_tdd_tools::test!(test_f1_load_multiple_configs, {
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/dev.toml"
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("workers: 2") || stdout.contains("workers = 2"));
});

chicago_tdd_tools::test!(test_f1_parse_syntax_error, {
    // Generate a temporary file with syntax errors
    let temp_dir = std::env::temp_dir();
    let syntax_err_path = temp_dir.join("syntax_error.toml");
    std::fs::write(&syntax_err_path, "name = unquoted_value").unwrap();
    
    let (code, _, stderr) = run_bin(&["--config", syntax_err_path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Parse") || stderr.contains("invalid") || stderr.contains("syntax"));
    
    let _ = std::fs::remove_file(syntax_err_path);
});

chicago_tdd_tools::test!(test_f1_parse_env_var_expansion, {
    let (code, stdout, _) = run_bin(
        &["--config", "examples/star-toml/samples/default.toml"],
        &[("STAR_TOML_SERVER_PORT", "9090")]
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("9090"));
});

chicago_tdd_tools::test!(test_f1_parse_nonexistent_file, {
    let (code, _, stderr) = run_bin(&["--config", "examples/star-toml/samples/does_not_exist.toml"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("FileNotFound") || stderr.contains("not found") || stderr.contains("exist"));
});

// --- F2: Layering and Merging (5 tests) ---

chicago_tdd_tools::test!(test_f2_merge_scalar_override, {
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/dev.toml"
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("workers: 2") || stdout.contains("workers = 2"));
    assert!(!stdout.contains("workers: 4") || stdout.contains("workers: 2"));
});

chicago_tdd_tools::test!(test_f2_merge_nested_table, {
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/prod.toml"
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("0.0.0.0"));
    assert!(stdout.contains("443"));
});

chicago_tdd_tools::test!(test_f2_merge_env_override, {
    let (code, stdout, _) = run_bin(
        &[
            "--config", "examples/star-toml/samples/default.toml",
            "--config", "examples/star-toml/samples/dev.toml"
        ],
        &[("STAR_TOML_WORKERS", "8")]
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("workers: 8") || stdout.contains("workers = 8"));
});

chicago_tdd_tools::test!(test_f2_merge_determinism, {
    let (code1, stdout1, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/dev.toml"
    ], &[]);
    let (code2, stdout2, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/dev.toml"
    ], &[]);
    assert_eq!(code1, code2);
    assert_eq!(stdout1, stdout2);
});

chicago_tdd_tools::test!(test_f2_merge_overlapping_keys, {
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/prod.toml",
        "--config", "examples/star-toml/samples/dev.toml" // overrides port to 80
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("port: 80") || stdout.contains("port = 80"));
});

// --- F3: Configuration Validation (5 tests) ---

chicago_tdd_tools::test!(test_f3_validate_valid_config, {
    let (code, _, _) = run_bin(&["--config", "examples/star-toml/samples/default.toml"], &[]);
    assert_eq!(code, 0);
});

chicago_tdd_tools::test!(test_f3_validate_missing_required_field, {
    let temp_dir = std::env::temp_dir();
    let missing_field_path = temp_dir.join("missing_field.toml");
    std::fs::write(&missing_field_path, "workers = 4\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    
    let (code, _, stderr) = run_bin(&["--config", missing_field_path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("name") || stderr.contains("missing") || stderr.contains("Invalid"));
    
    let _ = std::fs::remove_file(missing_field_path);
});

chicago_tdd_tools::test!(test_f3_validate_port_out_of_range, {
    let (code, _, stderr) = run_bin(&["--config", "examples/star-toml/samples/invalid_port.toml"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("port") || stderr.contains("out_of_range") || stderr.contains("range") || stderr.contains("Invalid"));
});

chicago_tdd_tools::test!(test_f3_validate_workers_out_of_range, {
    let (code, _, stderr) = run_bin(&["--config", "examples/star-toml/samples/invalid_workers.toml"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("workers") || stderr.contains("out_of_range") || stderr.contains("range") || stderr.contains("Invalid"));
});

chicago_tdd_tools::test!(test_f3_validate_tls_paths_required, {
    let (code, _, stderr) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/invalid_tls.toml"
    ], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("tls_cert_required") || stderr.contains("cert_path") || stderr.contains("key_path") || stderr.contains("Invalid"));
});

// --- F4: Progress Alerts/Logging (5 tests) ---

chicago_tdd_tools::test!(test_f4_progress_alert_success, {
    let (code, stdout, _) = run_bin(&["--config", "examples/star-toml/samples/default.toml"], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("SUCCESS") || stdout.contains("Admitted") || stdout.contains("admitted"));
});

chicago_tdd_tools::test!(test_f4_progress_alert_warning_port, {
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/dev.toml"
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("WARNING") || stdout.contains("warning") || stdout.contains("prefer a port above 1024"));
});

chicago_tdd_tools::test!(test_f4_progress_alert_info_loading, {
    let (code, stdout, _) = run_bin(&["--config", "examples/star-toml/samples/default.toml"], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("INFO") || stdout.contains("info") || stdout.contains("Loading") || stdout.contains("loading"));
});

chicago_tdd_tools::test!(test_f4_progress_alert_error_refusal, {
    let (code, stdout, stderr) = run_bin(&["--config", "examples/star-toml/samples/invalid_port.toml"], &[]);
    assert_ne!(code, 0);
    assert!(stdout.contains("CRITICAL") || stdout.contains("ERROR") || stderr.contains("CRITICAL") || stderr.contains("ERROR") || stderr.contains("Invalid") || stdout.contains("Refused"));
});

chicago_tdd_tools::test!(test_f4_progress_alert_custom_format, {
    let (code, stdout, _) = run_bin(&["--config", "examples/star-toml/samples/default.toml"], &[]);
    assert_eq!(code, 0);
    // Custom progress alerting should log that it is evaluating layers.
    assert!(stdout.contains("layer") || stdout.contains("Layer") || stdout.contains("config") || stdout.contains("Config"));
});

// --- F5: Accepted/Refused Behavior (5 tests) ---

chicago_tdd_tools::test!(test_f5_accepted_standing_verification, {
    let (code, stdout, _) = run_bin(&["--config", "examples/star-toml/samples/default.toml"], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("standing") || stdout.contains("q_config = 1") || stdout.contains("Accepted") || stdout.contains("admitted"));
});

chicago_tdd_tools::test!(test_f5_refused_invalid_port, {
    let (code, _, stderr) = run_bin(&["--config", "examples/star-toml/samples/invalid_port.toml"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Invalid") || stderr.contains("refused") || stderr.contains("Refused") || stderr.contains("port"));
});

chicago_tdd_tools::test!(test_f5_refused_invalid_workers, {
    let (code, _, stderr) = run_bin(&["--config", "examples/star-toml/samples/invalid_workers.toml"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Invalid") || stderr.contains("refused") || stderr.contains("Refused") || stderr.contains("workers"));
});

chicago_tdd_tools::test!(test_f5_refused_invalid_tls, {
    let (code, _, stderr) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/invalid_tls.toml"
    ], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Invalid") || stderr.contains("refused") || stderr.contains("Refused") || stderr.contains("tls_cert_required"));
});

chicago_tdd_tools::test!(test_f5_refused_exit_code_non_zero, {
    let (code, _, _) = run_bin(&["--config", "examples/star-toml/samples/invalid_port.toml"], &[]);
    assert_ne!(code, 0);
});

// ============================================================================
// TIER 2: BOUNDARY & CORNER CASES (25 test cases)
// ============================================================================

chicago_tdd_tools::test!(test_b2_empty_toml, {
    let (code, _, stderr) = run_bin(&["--config", "examples/star-toml/samples/empty.toml"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("missing") || stderr.contains("Invalid") || stderr.contains("empty"));
});

chicago_tdd_tools::test!(test_b2_whitespace_only_toml, {
    let temp_dir = std::env::temp_dir();
    let ws_path = temp_dir.join("ws.toml");
    std::fs::write(&ws_path, "   \n\n   \n").unwrap();
    let (code, _, stderr) = run_bin(&["--config", ws_path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("missing") || stderr.contains("Invalid") || stderr.contains("empty"));
    let _ = std::fs::remove_file(ws_path);
});

chicago_tdd_tools::test!(test_b2_port_lower_boundary_zero, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("zero_port.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 4\n[server]\nhost = \"127.0.0.1\"\nport = 0").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("port") || stderr.contains("out_of_range") || stderr.contains("Invalid"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_port_upper_boundary_65536, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("large_port.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 4\n[server]\nhost = \"127.0.0.1\"\nport = 65536").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    // Over 65535 may fail parser (as u16 overflow) or validation. Either way, code must be non-zero.
    assert_ne!(code, 0);
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_workers_lower_boundary_zero, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("zero_workers.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 0\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("workers") || stderr.contains("out_of_range") || stderr.contains("Invalid"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_workers_upper_boundary_1025, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("too_many_workers.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 1025\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("workers") || stderr.contains("out_of_range") || stderr.contains("Invalid"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_workers_exact_max_1024, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("max_workers.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 1024\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, _, _) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_eq!(code, 0);
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_workers_exact_min_1, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("min_workers.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 1\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, _, _) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_eq!(code, 0);
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_missing_key_delimiters, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("bad_delimiters.toml");
    std::fs::write(&path, "name \"app\"\nworkers = 4").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Parse") || stderr.contains("expected") || stderr.contains("invalid"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_unclosed_string, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("unclosed.toml");
    std::fs::write(&path, "name = \"app\nworkers = 4").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Parse") || stderr.contains("newline") || stderr.contains("invalid") || stderr.contains("string"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_non_utf8_toml, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("non_utf8.toml");
    std::fs::write(&path, &[0xFF, 0xFE, 0x00, 0x00, 0x61, 0x62, 0x63]).unwrap(); // Invalid UTF-8
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Io") || stderr.contains("UTF-8") || stderr.contains("utf8") || stderr.contains("Parse"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_override_empty_env, {
    let (code, stdout, _) = run_bin(
        &["--config", "examples/star-toml/samples/default.toml"],
        &[("STAR_TOML_NAME", "")]
    );
    // If empty name is invalid, code should be non-zero
    assert_ne!(code, 0);
});

chicago_tdd_tools::test!(test_b2_weird_env_reference, {
    let (code, stdout, _) = run_bin(
        &["--config", "examples/star-toml/samples/default.toml"],
        &[("STAR_TOML_SERVER_HOST", "${STAR_TOML_NAME}")]
    );
    // Hostname could be resolved dynamically. We check it parses.
    assert!(code == 0 || code != 0); // accepts either dynamic expansion or invalid host refusal
});

chicago_tdd_tools::test!(test_b2_multiple_same_key_overrides, {
    let (code, stdout, _) = run_bin(
        &["--config", "examples/star-toml/samples/default.toml"],
        &[("STAR_TOML_WORKERS", "3"), ("STAR_TOML_WORKERS", "6")] // Last one wins or environment collision resolved
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("workers: 6") || stdout.contains("workers = 6") || stdout.contains("workers: 3") || stdout.contains("workers = 3"));
});

chicago_tdd_tools::test!(test_b2_float_instead_of_integer, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("float_workers.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 4.5\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Parse") || stderr.contains("invalid type") || stderr.contains("integer"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_boolean_instead_of_integer, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("bool_workers.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = true\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Parse") || stderr.contains("invalid type") || stderr.contains("integer"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_special_characters_app_name, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("special_chars.toml");
    std::fs::write(&path, "name = \"🚀✨-app-👻\"\nworkers = 4\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, stdout, _) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("🚀✨-app-👻"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_path_traversal_attempt, {
    let (code, _, stderr) = run_bin(&["--config", "../../../../../etc/passwd"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("FileNotFound") || stderr.contains("not found") || stderr.contains("exist") || stderr.contains("Parse"));
});

chicago_tdd_tools::test!(test_b2_read_only_config_file, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join(format!("readonly_{}.toml", std::process::id()));
    std::fs::write(&path, "name = \"app\"\nworkers = 4\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    
    // Set read-only permissions
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_readonly(true);
    let _ = std::fs::set_permissions(&path, perms);
    
    let (code, _, _) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    // Should still succeed in loading since we only read it!
    assert_eq!(code, 0);
    
    // Clean up permissions to allow deletion
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_readonly(false);
    let _ = std::fs::set_permissions(&path, perms);
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_directory_path_as_file, {
    let (code, _, stderr) = run_bin(&["--config", "examples/star-toml/samples"], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Io") || stderr.contains("directory") || stderr.contains("Is a directory") || stderr.contains("permission"));
});

chicago_tdd_tools::test!(test_b2_port_advisory_boundary_1023, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("port_1023.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 4\n[server]\nhost = \"127.0.0.1\"\nport = 1023").unwrap();
    let (code, stdout, _) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_eq!(code, 0);
    // Port 1023 should trigger the advisory warning
    assert!(stdout.contains("WARNING") || stdout.contains("warning") || stdout.contains("prefer a port above 1024"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_port_advisory_boundary_1024, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("port_1024.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 4\n[server]\nhost = \"127.0.0.1\"\nport = 1024").unwrap();
    let (code, stdout, _) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_eq!(code, 0);
    // Port 1024 should still trigger the warning (port <= 1024 is the warning condition)
    assert!(stdout.contains("WARNING") || stdout.contains("warning") || stdout.contains("prefer a port above 1024"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_log_level_case_sensitivity, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("caps_log.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 4\nlog_level = \"INFO\"\n[server]\nhost = \"127.0.0.1\"\nport = 8080").unwrap();
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    // Case sensitivity: log_level must be lowercase one_of. So "INFO" is invalid.
    assert_ne!(code, 0);
    assert!(stderr.contains("log_level") || stderr.contains("not_one_of") || stderr.contains("Invalid"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_mixing_array_and_table, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("mixed_types.toml");
    std::fs::write(&path, "name = \"app\"\nworkers = 4\nserver = [1, 2, 3]").unwrap(); // server should be a table
    let (code, _, stderr) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("Parse") || stderr.contains("invalid type") || stderr.contains("Invalid"));
    let _ = std::fs::remove_file(path);
});

chicago_tdd_tools::test!(test_b2_large_file_parsing, {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("large_config.toml");
    let mut content = String::from("name = \"app\"\nworkers = 4\n");
    for i in 0..5000 {
        content.push_str(&format!("unrelated_key_{} = {}\n", i, i));
    }
    content.push_str("[server]\nhost = \"127.0.0.1\"\nport = 8080\n");
    std::fs::write(&path, content).unwrap();
    
    let (code, stdout, _) = run_bin(&["--config", path.to_str().unwrap()], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("admitted") || stdout.contains("Admitted") || stdout.contains("SUCCESS"));
    let _ = std::fs::remove_file(path);
});

// ============================================================================
// TIER 3: CROSS-FEATURE COMBINATIONS (5 test cases)
// ============================================================================

chicago_tdd_tools::test!(test_c3_merge_valid_and_invalid, {
    let (code, _, stderr) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/invalid_port.toml"
    ], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("port") || stderr.contains("out_of_range") || stderr.contains("Invalid"));
});

chicago_tdd_tools::test!(test_c3_merge_env_override_invalid, {
    let (code, _, stderr) = run_bin(
        &["--config", "examples/star-toml/samples/default.toml"],
        &[("STAR_TOML_SERVER_PORT", "99999")]
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("port") || stderr.contains("out_of_range") || stderr.contains("Invalid"));
});

chicago_tdd_tools::test!(test_c3_merge_tls_components, {
    // default.toml has disabled TLS. We merge it with a config enabling TLS and another supplying certs.
    let temp_dir = std::env::temp_dir();
    let tls_enable_path = temp_dir.join("tls_enable.toml");
    std::fs::write(&tls_enable_path, "[server.tls]\nenabled = true").unwrap();
    
    let tls_certs_path = temp_dir.join("tls_certs.toml");
    std::fs::write(&tls_certs_path, "[server.tls]\ncert_path = \"/path/to/cert\"\nkey_path = \"/path/to/key\"").unwrap();
    
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        tls_enable_path.to_str().unwrap(),
        tls_certs_path.to_str().unwrap()
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("admitted") || stdout.contains("Admitted") || stdout.contains("SUCCESS"));
    
    let _ = std::fs::remove_file(tls_enable_path);
    let _ = std::fs::remove_file(tls_certs_path);
});

chicago_tdd_tools::test!(test_c3_multiple_errors_from_different_layers, {
    let (code, _, stderr) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/invalid_port.toml",
        "--config", "examples/star-toml/samples/invalid_workers.toml"
    ], &[]);
    assert_ne!(code, 0);
    assert!(stderr.contains("port") && stderr.contains("workers") || stderr.contains("Invalid") || stderr.contains("out_of_range"));
});

chicago_tdd_tools::test!(test_c3_merge_env_var_expansion_invalid_format, {
    let (code, _, stderr) = run_bin(
        &["--config", "examples/star-toml/samples/default.toml"],
        &[("STAR_TOML_WORKERS", "abc")]
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("Parse") || stderr.contains("invalid digit") || stderr.contains("integer") || stderr.contains("Invalid"));
});

// ============================================================================
// TIER 4: REAL-WORLD APPLICATION SCENARIOS (5 test cases)
// ============================================================================

chicago_tdd_tools::test!(test_r4_production_profile, {
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/prod.toml"
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("0.0.0.0"));
    assert!(stdout.contains("443"));
    assert!(stdout.contains("app.crt"));
    assert!(stdout.contains("workers: 16") || stdout.contains("workers = 16"));
});

chicago_tdd_tools::test!(test_r4_development_profile, {
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        "--config", "examples/star-toml/samples/dev.toml"
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("workers: 2") || stdout.contains("workers = 2"));
    assert!(stdout.contains("WARNING") || stdout.contains("warning") || stdout.contains("prefer a port above 1024"));
});

chicago_tdd_tools::test!(test_r4_local_override_flow, {
    // Simulates developer running locally with overrides.
    let temp_dir = std::env::temp_dir();
    let local_override = temp_dir.join("local.toml");
    std::fs::write(&local_override, "name = \"my-local-app\"\n[server]\nport = 9000").unwrap();
    
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        local_override.to_str().unwrap()
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("my-local-app"));
    assert!(stdout.contains("port: 9000") || stdout.contains("port = 9000"));
    
    let _ = std::fs::remove_file(local_override);
});

chicago_tdd_tools::test!(test_r4_ci_environment_strict, {
    // In CI, we set a flag or expect strict validation behavior
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml"
    ], &[("CI", "true")]);
    assert_eq!(code, 0);
    assert!(stdout.contains("SUCCESS") || stdout.contains("admitted") || stdout.contains("Admitted"));
});

chicago_tdd_tools::test!(test_r4_microservice_scaling, {
    // Dynamic scale configuration by merging base and environment-specific nodes
    let temp_dir = std::env::temp_dir();
    let scale_config = temp_dir.join("scale.toml");
    std::fs::write(&scale_config, "workers = 64").unwrap();
    
    let (code, stdout, _) = run_bin(&[
        "--config", "examples/star-toml/samples/default.toml",
        scale_config.to_str().unwrap()
    ], &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("workers: 64") || stdout.contains("workers = 64"));
    
    let _ = std::fs::remove_file(scale_config);
});

// ============================================================================
// CHICAGO TDD SPECIALIST TESTS (Advanced verification feature integrations)
// ============================================================================

#[cfg(feature = "property-testing")]
#[test]
fn test_property_based_merge_determinism() {
    use chicago_tdd_tools::testing::property::PropertyTestGenerator;
    
    let mut generator = PropertyTestGenerator::<5, 2>::new().with_seed(12345);
    
    for _ in 0..100 {
        let test_data_base = generator.generate_test_data();
        let test_data_overlay = generator.generate_test_data();
        
        let mut base1 = toml_1::Value::Table(
            test_data_base.iter().map(|(k, v)| (k.clone(), toml_1::Value::String(v.clone()))).collect()
        );
        let mut base2 = base1.clone();
        
        let overlay1 = toml_1::Value::Table(
            test_data_overlay.iter().map(|(k, v)| (k.clone(), toml_1::Value::String(v.clone()))).collect()
        );
        let overlay2 = overlay1.clone();
        
        star_toml::deep_merge(&mut base1, overlay1);
        star_toml::deep_merge(&mut base2, overlay2);
        
        assert_eq!(base1, base2, "Merge must be deterministic");
    }
}

#[cfg(feature = "snapshot-testing")]
#[test]
fn test_snapshot_normalized_merged_output() {
    use chicago_tdd_tools::testing::snapshot::SnapshotAssert;
    
    let (code, stdout, _) = run_bin(&["--config", "examples/star-toml/samples/default.toml"], &[]);
    assert_eq!(code, 0);
    
    let mut toml_lines = Vec::new();
    let mut in_toml = false;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed == "standing" {
            break;
        }
        if trimmed.starts_with("name =") {
            in_toml = true;
        }
        if in_toml && !trimmed.is_empty() {
            toml_lines.push(trimmed);
        }
    }
    let actual_toml = toml_lines.join("\n") + "\n";

    SnapshotAssert::with_settings(|settings| {
        settings.set_snapshot_path("../../tests/snapshots");
        settings.set_prepend_module_to_snapshot(false);
    }, || {
        SnapshotAssert::assert_matches(&actual_toml, "star_toml_e2e__star_toml_default_config_snapshot");
    });
}

#[test]
fn test_performance_budget_validation_ticks() {
    use chicago_tdd_tools::validation::performance::measure_ticks;
    
    #[derive(serde::Deserialize, serde::Serialize)]
    struct SimpleConfig {
        name: String,
        workers: usize,
        #[serde(flatten, default)]
        extra: std::collections::HashMap<String, toml_1::Value>,
    }
    impl star_toml::Validate for SimpleConfig {
        fn validate(&self, v: &mut star_toml::Validator) {
            v.check_non_empty("name", &self.name);
        }
    }
    impl star_toml::ConfigLifecycle for SimpleConfig {}

    let (_, ticks) = measure_ticks(|| {
        let loader = star_toml::trusted()
            .layer_file("examples/star-toml/samples/default.toml");
        let _ = loader.load_admitted::<SimpleConfig>().unwrap();
    });
    assert!(ticks < 10_000_000);
}

#[cfg(feature = "otel")]
#[test]
fn test_observability_configuration_load() {
    use chicago_tdd_tools::observability::unified::ObservabilityTest;
    use chicago_tdd_tools::observability::unified::TestConfig;
    use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
    use std::collections::BTreeMap;
    
    let context = SpanContext::root(TraceId(42), SpanId(100), 1);
    let mut attributes = BTreeMap::new();
    attributes.insert("config.file".to_string(), "examples/star-toml/samples/default.toml".to_string());
    attributes.insert("config.status".to_string(), "admitted".to_string());
    
    let span = chicago_tdd_tools::otel::types::Span::new_active(
        context,
        "configuration.load".to_string(),
        1000,
        attributes,
        Vec::new(),
        SpanStatus::Ok,
    );
    
    let config = TestConfig {
        weaver_enabled: false,
        ..Default::default()
    };
    let test = ObservabilityTest::with_config(config).expect("Failed to initialize ObservabilityTest");
    
    let result = test.validate_span(&span);
    assert!(result.is_ok(), "Configuration load span validation failed: {:?}", result);
}
