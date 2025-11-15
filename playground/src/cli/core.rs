//! Core noun commands
//!
//! Commands for core features: fixtures, builders, assertions, macros, state, type_level, const_assert, alert

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::core::{
    alert, assertions, async_fixtures, builders, const_assert, fixtures, macros, state, type_level,
};

#[derive(Serialize)]
struct Status {
    features: Vec<String>,
    examples: Vec<String>,
}

#[derive(Serialize)]
struct ExecutionResult {
    executed: Vec<String>,
    success: bool,
    message: String,
}

/// Show core features status
///
/// Displays information about all available core features and examples.
/// Use -v, -vv, or -vvv for more detail.
#[verb]
fn stat(
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level (-v, -vv, -vvv)")]
    verbose: usize,
) -> Result<Status> {
    Ok(Status {
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
/// Run examples by name. You can execute multiple examples in one command.
/// Example names are space-separated.
///
/// Examples:
///   playg core exec fixtures
///   playg core exec "fixtures builders assert"
#[verb]
fn exec(
    #[arg(help = "Space-separated example names to execute")]
    names: String,
    #[arg(short = 'o', long, value_name = "FILE", help = "Write output to file")]
    output: Option<PathBuf>,
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level")]
    verbose: usize,
) -> Result<ExecutionResult> {
    let mut executed = Vec::new();
    let mut errors = Vec::new();

    let name_list: Vec<String> = names.split_whitespace().map(|s| s.to_string()).collect();
    for name in name_list {
        if let Err(e) = execute_core_example(&name) {
            errors.push(format!("{}: {}", name, e));
        } else {
            executed.push(name.clone());
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Executed {} example(s) successfully", executed.len())
    } else {
        format!("Executed {} example(s), {} error(s)", executed.len(), errors.len())
    };

    Ok(ExecutionResult {
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

