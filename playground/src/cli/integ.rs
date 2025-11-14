//! Integ noun commands
//!
//! Commands for integration features: testcontainers

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::integration;

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
fn stat(verbose: usize) -> Result<Status> {
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
fn contain(image: Option<String>) -> Result<ExecutionResult> {
    integration::testcontainers::example_container_basic().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_ports().map_err(|e| e.to_string())?;
    integration::testcontainers::example_container_env().map_err(|e| e.to_string())?;

    Ok(ExecutionResult {
        executed: vec!["contain".to_string()],
        success: true,
        message: "Testcontainers demo executed successfully".to_string(),
    })
}


