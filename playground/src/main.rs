//! Chicago TDD Tools Playground CLI
//!
//! Comprehensive playground demonstrating all features of chicago-tdd-tools.
//! This serves as both a validation suite and a reference implementation.

#![allow(clippy::module_name_repetitions)]
#![allow(non_upper_case_globals)]
#![allow(clippy::useless_comparison)]
#![allow(unused_comparisons)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::useless_vec)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unused_self)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::single_match_else)]
#![allow(clippy::unused_variables)]
#![allow(dead_code)]
#![allow(clippy::eq_op)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::float_cmp)]
#![allow(clippy::redundant_closure)]
#![allow(unused_doc_comments)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::missing_const_for_fn)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(clippy::unused_unit)]

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
