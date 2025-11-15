//! Chicago TDD Tools Playground CLI
//!
//! Comprehensive playground demonstrating all features of chicago-tdd-tools.
//! This serves as both a validation suite and a reference implementation.

use clap_noun_verb::Result;

// Import CLI modules to trigger linkme auto-discovery
use playground::cli;

fn main() -> Result<()> {
    // Initialize logging from environment
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    clap_noun_verb::run()
}
