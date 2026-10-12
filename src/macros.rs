//! Macros Module
//!
//! Backward-compatibility shim. All macros are defined with `#[macro_export]`
//! in `core::macros` and therefore live at the crate root; this module
//! re-exports them **by name** so path-based access resolves for consumers.
//!
//! Resolution note (why the re-exports below are shaped the way they are): in
//! a macro invocation path only the FINAL segment is resolved in the macro
//! namespace; every earlier segment must resolve in the type/module
//! namespace. So `chicago_tdd_tools::macros::test!(...)` requires a macro
//! named `test` directly inside `macros` (not inside the submodule
//! `macros::test`), while `chicago_tdd_tools::macros::assert::assert_ok!(...)`
//! resolves through the `assert` submodule.
//!
//! (Glob re-exports such as `pub use crate::core::macros::test::*;` do NOT
//! re-export `macro_rules!` items at all — the previous shim modules were
//! empty and every path-based invocation failed with E0433.)

// Crate-root aliases, re-exported one level down. Covers the
// `chicago_tdd_tools::macros::test!(...)` / `::assert_ok!(...)` surface.
pub use crate::{
    alert_critical, alert_debug, alert_info, alert_success, alert_warning, assert_approx_eq,
    assert_contains, assert_eq_msg, assert_err, assert_fail, assert_guard_constraint,
    assert_in_range, assert_json_eq, assert_matches, assert_not_contains, assert_ok, assert_subset,
    assert_superset, assert_within_tick_budget, async_test, async_test_with_timeout, fixture_test,
    fixture_test_with_timeout, otel_test, param_test, performance_test, test,
};

/// Assertion macros
///
/// Re-exports assertion macros from `core::macros::assert` by name.
pub mod assert {
    pub use crate::{
        assert_approx_eq, assert_contains, assert_eq_msg, assert_err, assert_fail,
        assert_guard_constraint, assert_in_range, assert_json_eq, assert_matches,
        assert_not_contains, assert_ok, assert_subset, assert_superset, assert_within_tick_budget,
    };
}

/// Test macros
///
/// Re-exports test macros from `core::macros::test` by name.
pub mod test {
    // param_test and otel_test exist in both feature states (compile_error!
    // fallback variants), so they are safe to re-export unconditionally.
    pub use crate::{
        async_test, async_test_with_timeout, fixture_test, fixture_test_with_timeout, otel_test,
        param_test, performance_test, test,
    };
}
