//! Negative space: `#[chicago_test]` refuses async test fns (audit W8).
//! An async fn under a plain `#[test]` wrapper would silently mis-expand.

use chicago_tdd_tools::chicago_test;

#[chicago_test(ticket = "docs/jira/CC-9999_does_not_exist.md", scaffold_fn = "scaffold_impl")]
async fn rejects_async_fns() {
    assert_eq!(1 + 1, 2);
}

fn main() {}
