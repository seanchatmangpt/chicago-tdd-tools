#![allow(clippy::module_name_repetitions)]
#![allow(non_upper_case_globals)]
#![allow(unused_comparisons)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::useless_vec)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
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
#![allow(unused_variables)]
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
#![allow(clippy::len_zero)]

//! Validation Features Test Suite

use chicago_tdd_tools::prelude::*;

pub fn run_validation_tests() {
    println!("  Testing validation features...");

    // Coverage
    test_coverage();

    // Guards
    test_guards();

    // JTBD
    test_jtbd();

    // Performance
    test_performance();

    println!("  ✓ Validation features validated");
}

fn test_coverage() {
    let mut report = CoverageReport::new();
    report.add_item("item1".to_string(), true);
    assert_eq!(report.total.get(), 1);
}

fn test_guards() {
    let validator = GuardValidator::new();
    assert!(validator.validate_run_len(5).is_ok());
}

fn test_jtbd() {
    use std::collections::HashMap;
    let mut validator = JtbdValidator::new();
    validator.register_scenario(JtbdScenario {
        name: "test".to_string(),
        setup_context: Box::new(ExecutionContext::default),
        execute: Box::new(|_| ExecutionResult::ok(HashMap::new())),
        validate_result: Box::new(|_, _| true),
        expected_behavior: "test".to_string(),
    });
    let results = validator.validate_all();
    assert_eq!(results.len(), 1);
}

fn test_performance() {
    let counter = TickCounter::start();
    let _ticks = counter.elapsed_ticks();
}
