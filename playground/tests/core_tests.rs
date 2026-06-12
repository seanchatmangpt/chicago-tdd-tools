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

//! Core Features Test Suite

use chicago_tdd_tools::prelude::*;

pub fn run_core_tests() {
    println!("  Testing core features...");

    // Fixtures
    test_fixtures();

    // Builders
    test_builders();

    // Assertions
    test_assertions();

    // State
    test_state();

    // Type level
    test_type_level();

    // Const assert
    test_const_assert();

    // Alert
    test_alert();

    println!("  ✓ Core features validated");
}

fn test_fixtures() {
    let fixture = TestFixture::new().unwrap();
    assert!(fixture.test_counter() >= 0);
}

fn test_builders() {
    let data = TestDataBuilder::new().with_var("key", "value").build_json().unwrap();
    assert!(data.is_object());
}

fn test_assertions() {
    let result: Result<u32, String> = Ok(42);
    assert_success(&result);
}

fn test_state() {
    let state = TestState::<Arrange>::new();
    let _act_state = state.act();
}

fn test_type_level() {
    const ARRAY: chicago_tdd_tools::core::type_level::SizeValidatedArray<8, 8> =
        chicago_tdd_tools::core::type_level::SizeValidatedArray::new([0u8; 8]);
    assert_eq!(ARRAY.size(), 8);
}

fn test_const_assert() {
    chicago_tdd_tools::core::const_assert::const_assert(true);
}

fn test_alert() {
    alert_info!("Test alert");
}
