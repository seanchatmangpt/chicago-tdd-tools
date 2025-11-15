//! Obs noun commands
//!
//! Commands for observability features: otel, weaver

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

use crate::observability;

#[derive(Serialize)]
pub struct Status {
    pub features: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Serialize)]
pub struct ExecutionResult {
    pub executed: Vec<String>,
    pub success: bool,
    pub message: String,
}

/// Show observability features status
#[verb]
fn stat(#[arg(short = 'v', action = "count")] verbose: usize) -> Result<Status> {
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

    Ok(Status { features, examples })
}

/// List available observability demos
#[verb]
fn list() -> Result<Vec<String>> {
    let mut examples = Vec::new();

    #[cfg(feature = "otel")]
    {
        examples.push("otel".to_string());
    }
    #[cfg(feature = "weaver")]
    {
        examples.push("weav".to_string());
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
    #[arg(short = 'v', action = "count", help = "Verbosity level")] verbose: usize,
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
