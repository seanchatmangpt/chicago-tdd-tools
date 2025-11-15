//! Validation noun commands
//!
//! Demonstrates clap-noun-verb best practices through quality validation examples.
//! Commands for validation features: coverage, guards, jtbd, performance

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::validation;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

#[derive(Serialize)]
pub struct ValidationFeatureStatus {
    /// Available validation features
    pub features: Vec<String>,
    /// Available example demonstrations
    pub examples: Vec<String>,
}

#[derive(Serialize)]
pub struct ValidationExecutionResult {
    /// Names of validation checks that passed
    pub executed: Vec<String>,
    /// Whether all validation checks passed
    pub success: bool,
    /// Summary message about validation results
    pub message: String,
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb] macro)
// ============================================================================

/// Show validation features status
///
/// Displays available validation features: coverage, guards, JTBD, performance.
/// Use -v for detailed verbose output.
#[verb]
fn stat(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<ValidationFeatureStatus> {
    if verbose > 0 {
        eprintln!("📋 Validation Features Status");
    }

    Ok(ValidationFeatureStatus {
        features: vec![
            "cov".to_string(),
            "guard".to_string(),
            "jtbd".to_string(),
            "perf".to_string(),
        ],
        examples: vec![
            "cov".to_string(),
            "guard".to_string(),
            "jtbd".to_string(),
            "perf".to_string(),
        ],
    })
}

/// List available validation checks
///
/// Shows all validation checks that can be executed with `playg valid exec`.
#[verb]
fn list() -> Result<Vec<String>> {
    Ok(vec![
        "cov".to_string(),
        "guard".to_string(),
        "jtbd".to_string(),
        "perf".to_string(),
    ])
}

/// Execute multiple validation checks
///
/// Run validation checks by name. You can execute multiple checks in one command.
///
/// # Arguments
/// * `names` - Space-separated check names (e.g., "cov guard jtbd")
///
/// # Options
/// * `-o, --output` - Optional output file for results
/// * `-v, --verbose` - Increase verbosity level
#[verb]
fn exec(
    #[arg(index = 0, value_name = "NAMES")]
    names: String,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<ValidationExecutionResult> {
    let mut executed = Vec::new();
    let mut errors = Vec::new();

    if verbose > 0 {
        eprintln!("🚀 Executing validation checks...");
    }

    let name_list: Vec<String> = names.split_whitespace().map(|s| s.to_string()).collect();
    for name in name_list {
        if verbose > 1 {
            eprintln!("  Running: {}", name);
        }

        if let Err(e) = execute_valid_example(&name) {
            errors.push(format!("{}: {}", name, e));
            if verbose > 0 {
                eprintln!("  ❌ Error: {}", e);
            }
        } else {
            executed.push(name.clone());
            if verbose > 0 {
                eprintln!("  ✅ {}", name);
            }
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Executed {} check(s) successfully", executed.len())
    } else {
        format!("Executed {} check(s), {} error(s)", executed.len(), errors.len())
    };

    if verbose > 0 {
        eprintln!();
        eprintln!("📊 Summary: {}", message);
    }

    Ok(ValidationExecutionResult {
        executed,
        success,
        message,
    })
}

fn execute_valid_example(name: &str) -> std::result::Result<(), String> {
    match name {
        "cov" => {
            validation::coverage::example_coverage_basic();
            validation::coverage::example_coverage_newtypes();
            validation::coverage::example_coverage_percentage();
            Ok(())
        }
        "guard" => {
            validation::guards::example_guard_basic().map_err(|e| e.to_string())?;
            validation::guards::example_guard_failure();
            Ok(())
        }
        "jtbd" => {
            validation::jtbd::example_jtbd_basic();
            validation::jtbd::example_jtbd_index();
            Ok(())
        }
        "perf" => {
            validation::performance::example_tick_measurement();
            validation::performance::example_tick_budget();
            validation::performance::example_performance_validation().map_err(|e| e.to_string())?;
            Ok(())
        }
        _ => Err(format!("Unknown example: {}", name)),
    }
}

