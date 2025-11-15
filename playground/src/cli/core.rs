//! Core noun commands
//!
//! Demonstrates clap-noun-verb best practices through core feature examples.
//! Commands for core features: fixtures, builders, assertions, macros, state, type_level, const_assert, alert

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::core::{
    alert, assertions, async_fixtures, builders, const_assert, fixtures, macros, state, type_level,
};

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

#[derive(Serialize)]
pub struct CoreFeatureStatus {
    /// Available core features
    pub features: Vec<String>,
    /// Available example demonstrations
    pub examples: Vec<String>,
}

#[derive(Serialize)]
pub struct CoreExecutionResult {
    /// Names of examples that executed successfully
    pub executed: Vec<String>,
    /// Whether all examples executed without errors
    pub success: bool,
    /// Summary message about execution
    pub message: String,
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb] macro)
// ============================================================================

/// Show core features status
///
/// Displays information about all available core features and examples.
/// Use -v for basic verbose output, -vv for detailed information, -vvv for debug output.
///
/// # Examples
/// ```text
/// playg core stat            # Shows features in JSON format
/// playg core stat -v         # Shows features with verbose output
/// playg core stat --format yaml  # Shows features in YAML format
/// ```
#[verb]
fn stat(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<CoreFeatureStatus> {
    if verbose > 0 {
        eprintln!("📋 Core Features Status");
    }

    Ok(CoreFeatureStatus {
        features: vec![
            "fixtures".to_string(),
            "async".to_string(),
            "builders".to_string(),
            "assert".to_string(),
            "macros".to_string(),
            "state".to_string(),
            "type_level".to_string(),
            "const_assert".to_string(),
            "alert".to_string(),
        ],
        examples: vec![
            "fixtures".to_string(),
            "builders".to_string(),
            "assert".to_string(),
            "macros".to_string(),
            "state".to_string(),
            "type_level".to_string(),
            "const_assert".to_string(),
            "alert".to_string(),
        ],
    })
}

/// List available core examples
///
/// Shows all core example modules that can be executed with `playg core exec`.
#[verb]
fn list() -> Result<Vec<String>> {
    Ok(vec![
        "fixtures".to_string(),
        "builders".to_string(),
        "assert".to_string(),
        "macros".to_string(),
        "state".to_string(),
        "type_level".to_string(),
        "const_assert".to_string(),
        "alert".to_string(),
    ])
}

/// Execute one or more core examples
///
/// Run core feature examples by name. You can execute multiple examples in one command.
///
/// # Arguments
/// * `names` - Space-separated example names (e.g., "fixtures builders assert")
///
/// # Options
/// * `-o, --output` - Optional output file for results
/// * `-v, --verbose` - Increase verbosity level
///
/// # Examples
/// ```text
/// playg core exec "fixtures"
/// playg core exec "fixtures builders assert"
/// playg core exec "fixtures builders" --output results.json
/// playg core exec "assert" -vv
/// ```
#[verb]
fn exec(
    #[arg(index = 0, value_name = "NAMES")]
    names: String,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<CoreExecutionResult> {
    let mut executed = Vec::new();
    let mut errors = Vec::new();

    if verbose > 0 {
        eprintln!("🚀 Executing core examples...");
    }

    let name_list: Vec<String> = names.split_whitespace().map(|s| s.to_string()).collect();
    for name in name_list {
        if verbose > 1 {
            eprintln!("  Running: {}", name);
        }

        if let Err(e) = execute_core_example(&name) {
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
        format!("Executed {} example(s) successfully", executed.len())
    } else {
        format!("Executed {} example(s), {} error(s)", executed.len(), errors.len())
    };

    if verbose > 0 {
        eprintln!();
        eprintln!("📊 Summary: {}", message);
    }

    Ok(CoreExecutionResult {
        executed,
        success,
        message,
    })
}

fn execute_core_example(name: &str) -> std::result::Result<(), String> {
    match name {
        "fixtures" => {
            fixtures::example_basic_fixture().map_err(|e| e.to_string())?;
            fixtures::example_fixture_with_data();
            fixtures::example_fixture_metadata().map_err(|e| e.to_string())?;
            fixtures::example_fixture_isolation().map_err(|e| e.to_string())?;
            Ok(())
        }
        "builders" => {
            builders::example_basic_builder().map_err(|e| e.to_string())?;
            builders::example_business_builder().map_err(|e| e.to_string())?;
            #[cfg(feature = "fake-data")]
            {
                builders::example_fake_data_builder().map_err(|e| e.to_string())?;
            }
            builders::example_builder_hashmap();
            Ok(())
        }
        "assert" => {
            assertions::example_result_assertions();
            assertions::example_predicate_assertions();
            assertions::example_range_assertions();
            assertions::example_assertion_macros();
            assertions::example_assertion_builder();
            Ok(())
        }
        "macros" => {
            // Macros are demonstrated in test module, not as standalone examples
            // This is a placeholder - macros are compile-time constructs
            Ok(())
        }
        "state" => {
            state::example_type_state_pattern();
            state::example_advanced_state_pattern();
            Ok(())
        }
        "type_level" => {
            type_level::example_size_validated_array();
            Ok(())
        }
        "const_assert" => {
            const_assert::example_const_assertions();
            Ok(())
        }
        "alert" => {
            alert::example_alert_macros();
            #[cfg(feature = "logging")]
            {
                alert::example_alert_logger().map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        _ => Err(format!("Unknown example: {}", name)),
    }
}

