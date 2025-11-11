//! Test noun commands
//!
//! Commands for testing features: property, mutation, snapshot, concurrency, cli, generator, parameterized

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::testing;

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

/// Show testing features status
#[verb]
fn stat(verbose: usize) -> Result<Status> {
    let mut features = vec!["gen".to_string()];
    let mut examples = vec!["gen".to_string()];

    #[cfg(feature = "property-testing")]
    {
        features.push("prop".to_string());
        examples.push("prop".to_string());
    }
    #[cfg(feature = "mutation-testing")]
    {
        features.push("mut".to_string());
        examples.push("mut".to_string());
    }
    #[cfg(feature = "snapshot-testing")]
    {
        features.push("snap".to_string());
        examples.push("snap".to_string());
    }
    #[cfg(feature = "concurrency-testing")]
    {
        features.push("conc".to_string());
        examples.push("conc".to_string());
    }
    #[cfg(feature = "cli-testing")]
    {
        features.push("cli".to_string());
        examples.push("cli".to_string());
    }
    #[cfg(feature = "parameterized-testing")]
    {
        features.push("param".to_string());
        examples.push("param".to_string());
    }

    Ok(Status { features, examples })
}

/// List available test examples
#[verb]
fn list() -> Result<Vec<String>> {
    let mut examples = vec!["gen".to_string()];

    #[cfg(feature = "property-testing")]
    {
        examples.push("prop".to_string());
    }
    #[cfg(feature = "mutation-testing")]
    {
        examples.push("mut".to_string());
    }
    #[cfg(feature = "snapshot-testing")]
    {
        examples.push("snap".to_string());
    }
    #[cfg(feature = "concurrency-testing")]
    {
        examples.push("conc".to_string());
    }
    #[cfg(feature = "cli-testing")]
    {
        examples.push("cli".to_string());
    }
    #[cfg(feature = "parameterized-testing")]
    {
        examples.push("param".to_string());
    }

    Ok(examples)
}

/// Execute multiple test examples
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
        match execute_test_example(&name) {
            Ok(_) => executed.push(name.clone()),
            Err(e) => errors.push(format!("{}: {}", name, e)),
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

fn execute_test_example(name: &str) -> std::result::Result<(), String> {
    match name {
        "gen" => {
            testing::generator::example_test_generator();
            testing::generator::example_compile_time_array();
            testing::generator::example_array_pattern();
            Ok(())
        }
        #[cfg(feature = "property-testing")]
        "prop" => {
            testing::property::example_property_generator();
            testing::property::example_property_validation();
            Ok(())
        }
        #[cfg(feature = "mutation-testing")]
        "mut" => {
            testing::mutation::example_mutation_test();
            Ok(())
        }
        #[cfg(feature = "snapshot-testing")]
        "snap" => {
            testing::snapshot::example_snapshot_test();
            Ok(())
        }
        #[cfg(feature = "concurrency-testing")]
        "conc" => {
            testing::concurrency::example_concurrency_test();
            Ok(())
        }
        #[cfg(feature = "cli-testing")]
        "cli" => {
            testing::cli::example_cli_test();
            Ok(())
        }
        #[cfg(feature = "parameterized-testing")]
        "param" => {
            testing::parameterized::example_parameterized_test();
            Ok(())
        }
        _ => {
            #[cfg(not(feature = "property-testing"))]
            if name == "prop" {
                return Err("Property testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "mutation-testing"))]
            if name == "mut" {
                return Err("Mutation testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "snapshot-testing"))]
            if name == "snap" {
                return Err("Snapshot testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "concurrency-testing"))]
            if name == "conc" {
                return Err("Concurrency testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "cli-testing"))]
            if name == "cli" {
                return Err("CLI testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "parameterized-testing"))]
            if name == "param" {
                return Err("Parameterized testing feature not enabled".to_string());
            }
            Err(format!("Unknown example: {}", name))
        }
    }
}

