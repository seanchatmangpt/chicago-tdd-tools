//! Chicago TDD Tools — custom Clippy-style lints.
//!
//! Six semantic enforcement lints:
//!
//! - [`NO_RAW_TEST`]: `#[test]` without `test!()` wrapper bypasses timeouts and AAA tracking.
//! - [`NO_ASYNC_RAW_TEST`]: `#[tokio::test]` without `async_test!()` — same.
//! - [`NO_PRINTLN_IN_PRODUCTION`]: `println!`/`eprintln!` outside tests; use `alert_info!` etc.
//! - [`NO_DIRECT_LOG`]: `log::warn!` etc. directly; use `alert_*!` macros instead.
//! - [`UNUSED_RESULT_SILENCED`]: `let _ = result_fn()` silently discards errors.
//! - [`ASSERT_WITHOUT_MESSAGE`]: `assert_eq!(a, b)` in tests without a failure message.
//!
//! # Integration
//!
//! Add to your workspace `Cargo.toml`:
//!
//! ```toml
//! [workspace.metadata.dylint]
//! libraries = [{ git = "https://github.com/seanchatmangpt/chicago-tdd-tools", pattern = "crates/chicago-lints" }]
//! ```
//!
//! Run: `cargo dylint --all`

#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

dylint_linting::dylint_library!();

mod lints;

#[allow(rustc::internal)]
#[unsafe(no_mangle)]
pub fn register_lints(
    sess: &rustc_session::Session,
    lint_store: &mut rustc_lint::LintStore,
) {
    dylint_linting::init_config(sess);

    lint_store.register_lints(&[
        lints::no_raw_test::NO_RAW_TEST,
        lints::no_async_raw_test::NO_ASYNC_RAW_TEST,
        lints::no_println::NO_PRINTLN_IN_PRODUCTION,
        lints::no_direct_log::NO_DIRECT_LOG,
        lints::unused_result_silenced::UNUSED_RESULT_SILENCED,
        lints::assert_without_message::ASSERT_WITHOUT_MESSAGE,
    ]);

    lint_store.register_early_pass(|| Box::new(lints::no_raw_test::NoRawTest));
    lint_store.register_early_pass(|| Box::new(lints::no_async_raw_test::NoAsyncRawTest));
    lint_store.register_early_pass(|| Box::new(lints::no_println::NoPrintln::default()));
    lint_store.register_early_pass(|| Box::new(lints::no_direct_log::NoDirectLog::default()));
    lint_store.register_late_pass(|_| Box::new(lints::unused_result_silenced::UnusedResultSilenced));
    lint_store.register_early_pass(|| Box::new(lints::assert_without_message::AssertWithoutMessage::default()));
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
    }
}
