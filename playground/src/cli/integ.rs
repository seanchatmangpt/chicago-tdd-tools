//! Integration noun commands
//!
//! Demonstrates clap-noun-verb best practices through integration testing examples.
//! Commands for integration features: testcontainers

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::integration;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

#[derive(Serialize)]
pub struct IntegrationFeatureStatus {
    /// Available integration features
    pub features: Vec<String>,
    /// Available example demonstrations
    pub examples: Vec<String>,
}

#[derive(Serialize)]
pub struct IntegrationExecutionResult {
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

/// Collect available integration features based on enabled crate features
fn collect_integration_features() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(feature = "testcontainers")]
    features.push("contain".to_string());

    features
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb] macro)
// ============================================================================

/// Show integration features status
///
/// Displays available integration features (Testcontainers).
/// Use -v for detailed verbose output.
#[verb]
fn stat(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<IntegrationFeatureStatus> {
    if verbose > 0 {
        eprintln!("📋 Integration Features Status");
    }

    let features = collect_integration_features();
    Ok(IntegrationFeatureStatus {
        examples: features.clone(),
        features,
    })
}

/// List available integration demos
///
/// Shows all integration examples that can be executed.
#[verb]
fn list() -> Result<Vec<String>> {
    Ok(collect_integration_features())
}

/// Run testcontainers demo
///
/// Executes Docker container-based integration testing examples.
/// Requires Docker to be installed and running.
#[verb]
#[cfg(feature = "testcontainers")]
fn contain(
    #[arg(long)]
    image: Option<String>,
) -> Result<IntegrationExecutionResult> {
    integration::testcontainers::example_container_basic().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_ports().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_env().map_err(|e| e.to_string())?;

    Ok(IntegrationExecutionResult {
        executed: vec!["contain".to_string()],
        success: true,
        message: "Testcontainers demo executed successfully".to_string(),
    })
}


