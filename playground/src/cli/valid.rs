//! Valid noun commands
//!
//! Commands for validation features: coverage, guards, jtbd, performance

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::validation;

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

/// Show validation features status
#[verb]
fn stat(verbose: usize) -> Result<Status> {
    Ok(Status {
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
#[verb]
fn exec(
    names: String,
    output: Option<PathBuf>,
    verbose: usize,
) -> Result<ExecutionResult> {
    let mut executed = Vec::new();
    let mut errors = Vec::new();

    let name_list: Vec<String> = names.split_whitespace().map(|s| s.to_string()).collect();
    for name in name_list {
        if let Err(e) = execute_valid_example(&name) {
            errors.push(format!("{}: {}", name, e));
        } else {
            executed.push(name.clone());
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Executed {} check(s) successfully", executed.len())
    } else {
        format!("Executed {} check(s), {} error(s)", executed.len(), errors.len())
    };

    Ok(ExecutionResult {
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

