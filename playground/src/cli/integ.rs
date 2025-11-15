//! Integ noun commands
//!
//! Commands for integration features: testcontainers

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::integration;
use crate::format_utils::OutputFormat;

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

/// Show integration features status
#[verb]
fn stat(
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level")]
    verbose: usize,
    #[arg(short = 'f', long, default_value = "json", help = "Output format: json, yaml, toml, table, tsv")]
    format: String,
) -> Result<Status> {
    let mut features = Vec::new();
    let mut examples = Vec::new();

    #[cfg(feature = "testcontainers")]
    {
        features.push("contain".to_string());
        examples.push("contain".to_string());
    }

    let status = Status { features, examples };

    // Format and print output
    if let Ok(fmt) = OutputFormat::from_str(&format) {
        if let Ok(formatted) = fmt.serialize(&status) {
            println!("{}", formatted);
        }
    }

    Ok(status)
}

/// List available integration demos
#[verb]
fn list(
    #[arg(short = 'f', long, default_value = "json", help = "Output format: json, yaml, toml, table, tsv")]
    format: String,
) -> Result<Vec<String>> {
    let mut examples = Vec::new();

    #[cfg(feature = "testcontainers")]
    {
        examples.push("contain".to_string());
    }

    // Format and print output
    if let Ok(fmt) = OutputFormat::from_str(&format) {
        if let Ok(formatted) = fmt.serialize(&examples) {
            println!("{}", formatted);
        }
    }

    Ok(examples)
}

/// Run testcontainers demo
#[verb]
#[cfg(feature = "testcontainers")]
fn contain(
    #[arg(long, value_name = "IMAGE", help = "Docker image to use")]
    image: Option<String>,
) -> Result<ExecutionResult> {
    integration::testcontainers::example_container_basic().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_ports().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_env().map_err(|e| e.to_string())?;

    Ok(ExecutionResult {
        executed: vec!["contain".to_string()],
        success: true,
        message: "Testcontainers demo executed successfully".to_string(),
    })
}


