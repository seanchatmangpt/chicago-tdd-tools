//! CLI: lift `docs/receipts/N*.json` court receipts to Turtle on stdout.
//!
//! ```text
//! cargo run --manifest-path spec-harness/Cargo.toml --bin lift_receipts -- <receipts-dir>
//! ```

use std::process::ExitCode;

use chatman_spec_harness::nato_rdf::lift_receipts_dir;

fn main() -> ExitCode {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/receipts".to_string());
    match lift_receipts_dir(&dir) {
        Ok(turtle) => {
            print!("{turtle}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("REFUSED (nato-rdf-lift): {e}");
            ExitCode::FAILURE
        }
    }
}
