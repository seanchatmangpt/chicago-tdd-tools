//! Chicago TDD Tools Playground CLI
//!
//! Comprehensive playground demonstrating all features of chicago-tdd-tools.
//! This serves as both a validation suite and a reference implementation.

use clap_noun_verb::Result;
use std::sync::Arc;

// Import CLI modules to trigger linkme auto-discovery
use playground::cli;

/// Application state shared across all commands
#[derive(Debug, Clone)]
pub struct AppState {
    /// Application version
    pub version: String,
    /// Verbose mode flag
    pub verbose: bool,
    /// Output format preference
    pub output_format: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            verbose: false,
            output_format: "json".to_string(),
        }
    }
}

fn main() -> Result<()> {
    // Initialize application state
    let state = Arc::new(AppState::default());

    // Run with application context
    clap_noun_verb::run()
}
