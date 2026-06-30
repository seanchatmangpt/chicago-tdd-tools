//! Lint: `ctt_no_async_raw_test`
//!
//! `#[tokio::test]` or `#[async_std::test]` without `async_test!()` bypasses CTT's
//! per-test timeouts and AAA tracking. Use `async_test!(name, { … })` instead.

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::{AttrStyle, Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::Symbol;

declare_lint! {
    /// Detects `#[tokio::test]` or `#[async_std::test]` used directly on a function
    /// instead of CTT's `async_test!()` macro.
    ///
    /// ### Why is this bad?
    ///
    /// Same reason as `NO_RAW_TEST` but for async: per-test timeout (1 s), AAA
    /// tracking, and fixture integration are all lost when you write the attribute
    /// directly.
    ///
    /// ### Example (bad)
    ///
    /// ```rust
    /// #[tokio::test]
    /// async fn my_test() {
    ///     assert_eq!(fetch().await, 42);
    /// }
    /// ```
    ///
    /// ### Example (good)
    ///
    /// ```rust
    /// use chicago_tdd_tools::prelude::*;
    /// async_test!(my_test, {
    ///     assert_eq!(fetch().await, 42);
    /// });
    /// ```
    pub NO_ASYNC_RAW_TEST,
    Warn,
    "use `async_test!()` instead of `#[tokio::test]` / `#[async_std::test]`"
}

pub struct NoAsyncRawTest;

impl_lint_pass!(NoAsyncRawTest => [NO_ASYNC_RAW_TEST]);

/// Runtime crate names whose `test` attribute triggers this lint.
const ASYNC_RUNTIMES: &[&str] = &["tokio", "async_std", "actix_rt", "smol"];

impl EarlyLintPass for NoAsyncRawTest {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if !matches!(item.kind, ItemKind::Fn(..)) {
            return;
        }

        for attr in &item.attrs {
            if attr.style != AttrStyle::Outer || attr.span.from_expansion() {
                continue;
            }
            // attr.path() returns SmallVec<[Symbol; 1]> — the path segments as symbols
            let path = attr.path();
            // Match `#[runtime::test]` — two-segment path ending in "test"
            if path.len() != 2 {
                continue;
            }
            let runtime = path[0];
            let leaf = path[1];
            if leaf != Symbol::intern("test") {
                continue;
            }
            if ASYNC_RUNTIMES.iter().any(|r| runtime == Symbol::intern(r)) {
                span_lint_and_help(
                    cx,
                    NO_ASYNC_RAW_TEST,
                    attr.span,
                    format!(
                        "`#[{runtime}::test]` bypasses CTT's `async_test!()` — timeouts and AAA tracking are disabled"
                    ),
                    None,
                    "replace with `async_test!(fn_name, { … })` from `chicago_tdd_tools::prelude::*`",
                );
            }
        }
    }
}
