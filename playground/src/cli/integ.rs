//! Integ noun commands
//!
//! Commands for integration features: testcontainers

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

use crate::integration;

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

/// Show integration features status
#[verb]
fn stat(#[arg(short = 'v', action = "count")] verbose: usize) -> Result<Status> {
    let mut features = Vec::new();
    let mut examples = Vec::new();

    #[cfg(feature = "testcontainers")]
    {
        features.push("contain".to_string());
        examples.push("contain".to_string());
    }

    Ok(Status { features, examples })
}

/// List available integration demos
#[verb]
fn list() -> Result<Vec<String>> {
    let mut examples = Vec::new();

    #[cfg(feature = "testcontainers")]
    {
        examples.push("contain".to_string());
    }

    Ok(examples)
}

/// Run testcontainers demo
#[verb]
#[cfg(feature = "testcontainers")]
fn contain() -> Result<ExecutionResult> {
    integration::testcontainers::example_container_basic().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_ports().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_env().map_err(|e| e.to_string())?;

    Ok(ExecutionResult {
        executed: vec!["contain".to_string()],
        success: true,
        message: "Testcontainers demo executed successfully".to_string(),
    })
}
