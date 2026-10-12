//! Integration tests for `#[chicago_test]` (proc_macros/src/chicago_test.rs).
//!
//! Real expansion, Chicago style: the macro runs for real against an existing
//! ticket fixture, the generated test executes its body through
//! `__runtime::catch_scaffold`, and the `chicago_cc_NNN_` name derivation is
//! asserted on the generated symbol. Negative space (async/parameterized
//! refusals) lives in `tests/compile_fail/chicago_test_*.rs`.

use chicago_tdd_tools::chicago_test;

// Happy path: ticket exists (tests/fixtures/CC-0042_acceptance_spec.md),
// so the macro expands instead of refusing at the ticket gate.
#[chicago_test(
    ticket = "tests/fixtures/CC-0042_acceptance_spec.md",
    scaffold_fn = "scaffold_impl"
)]
fn happy_path_body_runs() {
    assert_eq!(2 + 2, 4);
}

// A second expansion used to pin the name-derivation contract
// (extract_ticket_id -> CC-0042 -> slug cc_0042 -> prefix chicago_cc_0042_).
#[chicago_test(
    ticket = "tests/fixtures/CC-0042_acceptance_spec.md",
    scaffold_fn = "scaffold_impl"
)]
fn name_derivation_body() {
    // Scaffold-pending panic inside the body must be absorbed by
    // catch_scaffold (CANDIDATE pass), so this generated test passes.
    panic!("SCAFFOLD PENDING: CC-0042  ticket=fixture  test=name_derivation_body");
}

#[test]
fn generated_test_name_has_chicago_cc_prefix() {
    // Compile-time existence check: the macro generated these exact symbols
    // from the two expansions above (chicago_cc_0042_<original_name>).
    let gen1 = stringify!(chicago_cc_0042_happy_path_body_runs);
    let gen2 = stringify!(chicago_cc_0042_name_derivation_body);
    assert!(gen1.starts_with("chicago_cc_0042_"));
    assert!(gen2.starts_with("chicago_cc_0042_"));

    // Invoke the generated tests directly: catch_scaffold must let a normal
    // body pass and absorb a scaffold-pending panic as CANDIDATE.
    chicago_cc_0042_happy_path_body_runs();
    chicago_cc_0042_name_derivation_body();
}
