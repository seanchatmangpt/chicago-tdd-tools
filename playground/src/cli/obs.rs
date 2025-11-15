//! Obs noun commands
//!
//! Commands for observability features: otel, weaver

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::observability;
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

/// Show observability features status
#[verb]
fn stat(
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level")]
    verbose: usize,
    #[arg(short = 'f', long, default_value = "json", help = "Output format: json, yaml, toml, table, tsv")]
    format: String,
) -> Result<Status> {
    let mut features = Vec::new();
    let mut examples = Vec::new();

    #[cfg(feature = "otel")]
    {
        features.push("otel".to_string());
        examples.push("otel".to_string());
    }
    #[cfg(feature = "weaver")]
    {
        features.push("weav".to_string());
        examples.push("weav".to_string());
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

/// List available observability demos
#[verb]
fn list(
    #[arg(short = 'f', long, default_value = "json", help = "Output format: json, yaml, toml, table, tsv")]
    format: String,
) -> Result<Vec<String>> {
    let mut examples = Vec::new();

    #[cfg(feature = "otel")]
    {
        examples.push("otel".to_string());
    }
    #[cfg(feature = "weaver")]
    {
        examples.push("weav".to_string());
    }

    // Format and print output
    if let Ok(fmt) = OutputFormat::from_str(&format) {
        if let Ok(formatted) = fmt.serialize(&examples) {
            println!("{}", formatted);
        }
    }

    Ok(examples)
}

/// Run OTEL demo
#[verb]
#[cfg(feature = "otel")]
fn otel() -> Result<ExecutionResult> {
    observability::otel::example_otel_span_basic().map_err(|e| e.to_string())?;
    observability::otel::example_otel_span_attributes().map_err(|e| e.to_string())?;
    observability::otel::example_otel_metric().map_err(|e| e.to_string())?;
    observability::otel::example_otel_helper();
    Ok(ExecutionResult {
        executed: vec!["otel".to_string()],
        success: true,
        message: "OTEL demo executed successfully".to_string(),
    })
}

/// Run weaver demo
///
/// Executes the Weaver live validation demo with optional configuration.
/// Configuration can be provided via command-line arguments or environment variables.
#[verb]
#[cfg(feature = "weaver")]
fn weav(
    #[arg(long, value_name = "DIR", help = "Weaver report directory")]
    report_dir: Option<PathBuf>,
    #[arg(long, value_name = "FILE", help = "Weaver registry path")]
    registry: Option<PathBuf>,
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level")]
    verbose: usize,
) -> Result<ExecutionResult> {
    observability::weaver::example_weaver_basic();
    observability::weaver::example_weaver_custom_config();
    observability::weaver::example_weaver_availability();

    Ok(ExecutionResult {
        executed: vec!["weav".to_string()],
        success: true,
        message: "Weaver demo executed successfully".to_string(),
    })
}

