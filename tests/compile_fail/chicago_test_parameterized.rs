//! Negative space: `#[chicago_test]` refuses parameterized test fns (audit W8).

use chicago_tdd_tools::chicago_test;

#[chicago_test(ticket = "docs/jira/CC-9999_does_not_exist.md", scaffold_fn = "scaffold_impl")]
fn rejects_parameterized_fns(x: u32) {
    assert_eq!(x, x);
}

fn main() {}
