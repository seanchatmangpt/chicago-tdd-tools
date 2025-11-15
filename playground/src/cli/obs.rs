//! Observability noun commands
//!
//! Demonstrates clap-noun-verb best practices through observability examples.
//! Commands for observability features: otel, weaver

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::observability;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

#[derive(Serialize)]
pub struct ObservabilityFeatureStatus {
    /// Available observability features
    pub features: Vec<String>,
    /// Available example demonstrations
    pub examples: Vec<String>,
}

#[derive(Serialize)]
pub struct ObservabilityExecutionResult {
    /// Names of examples that executed successfully
    pub executed: Vec<String>,
    /// Whether all examples executed without errors
    pub success: bool,
    /// Summary message about execution
    pub message: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Collect available observability features based on enabled crate features
fn collect_observability_features() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(feature = "otel")]
    features.push("otel".to_string());
    #[cfg(feature = "weaver")]
    features.push("weav".to_string());

    features
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb] macro)
// ============================================================================

/// Show observability features status
///
/// Displays available observability features (OTEL, Weaver).
/// Use -v for detailed verbose output.
#[verb]
fn stat(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<ObservabilityFeatureStatus> {
    if verbose > 0 {
        eprintln!("📋 Observability Features Status");
    }

    let features = collect_observability_features();
    Ok(ObservabilityFeatureStatus {
        examples: features.clone(),
        features,
    })
}

/// List available observability demos
///
/// Shows all observability examples that can be executed.
#[verb]
fn list() -> Result<Vec<String>> {
    Ok(collect_observability_features())
}

/// Run OTEL (OpenTelemetry) demo
///
/// Executes OpenTelemetry validation and instrumentation examples.
/// Demonstrates span creation, attributes, and metrics.
#[verb]
#[cfg(feature = "otel")]
fn otel() -> Result<ObservabilityExecutionResult> {
    observability::otel::example_otel_span_basic().map_err(|e| e.to_string())?;
    observability::otel::example_otel_span_attributes().map_err(|e| e.to_string())?;
    observability::otel::example_otel_metric().map_err(|e| e.to_string())?;
    observability::otel::example_otel_helper();
    Ok(ObservabilityExecutionResult {
        executed: vec!["otel".to_string()],
        success: true,
        message: "OTEL demo executed successfully".to_string(),
    })
}

/// Run Weaver live validation demo
///
/// Executes the Weaver live validation demo with optional configuration.
/// Configuration can be provided via command-line arguments or environment variables.
/// Weaver validates OTEL instrumentation against semantic conventions.
#[verb]
#[cfg(feature = "weaver")]
fn weav(
    #[arg(long)]
    report_dir: Option<PathBuf>,

    #[arg(long)]
    registry: Option<PathBuf>,

    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<ObservabilityExecutionResult> {
    observability::weaver::example_weaver_basic();
    observability::weaver::example_weaver_custom_config();
    observability::weaver::example_weaver_availability();

    Ok(ObservabilityExecutionResult {
        executed: vec!["weav".to_string()],
        success: true,
        message: "Weaver demo executed successfully".to_string(),
    })
}

