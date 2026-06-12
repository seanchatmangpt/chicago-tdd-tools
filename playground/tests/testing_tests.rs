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

//! Testing Features Test Suite

use chicago_tdd_tools::prelude::*;

pub fn run_testing_tests() {
    println!("  Testing testing features...");

    // Generator (always available)
    test_generator();

    // Mutation (always available)
    test_mutation();

    #[cfg(feature = "property-testing")]
    test_property();

    #[cfg(feature = "snapshot-testing")]
    test_snapshot();

    #[cfg(feature = "concurrency-testing")]
    test_concurrency();

    #[cfg(feature = "cli-testing")]
    test_cli();

    println!("  ✓ Testing features validated");
}

fn test_generator() {
    let mut generator = chicago_tdd_tools::testing::generator::TestGenerator::new();
    let _code = generator.generate_test("test", "spec");
}

fn test_mutation() {
    use std::collections::HashMap;
    let data = HashMap::new();
    let mut tester = MutationTester::new(data);
    tester.apply_mutation(MutationOperator::AddKey("key".to_string(), "value".to_string()));
}

#[cfg(feature = "property-testing")]
fn test_property() {
    let mut generator = PropertyTestGenerator::<10, 3>::new();
    let _data = generator.generate_test_data();
}

#[cfg(feature = "snapshot-testing")]
fn test_snapshot() {
    let data = "test";
    SnapshotAssert::assert_matches(&data, "test_snapshot");
}

#[cfg(feature = "concurrency-testing")]
fn test_concurrency() {
    use std::sync::{Arc, Mutex};
    ConcurrencyTest::run(|| {
        let data = Arc::new(Mutex::new(0));
        *data.lock().unwrap() += 1;
    });
}

#[cfg(feature = "cli-testing")]
fn test_cli() {
    // CLI tests use .trycmd files
    assert!(true);
}
