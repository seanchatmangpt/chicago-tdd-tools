//! Lint: `ctt_no_direct_log`
//!
//! Direct calls to `log::error!`, `log::warn!`, `log::info!`, `log::debug!`, or
//! `log::trace!` bypass CTT's alert macros. Use `alert_critical!`, `alert_warning!`,
//! `alert_info!`, `alert_debug!` from `chicago_tdd_tools::prelude::*` instead.

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::{Item, ItemKind, MacCall};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::Symbol;

declare_lint! {
    /// Detects direct `log::error!`, `log::warn!`, `log::info!`, `log::debug!`, or
    /// `log::trace!` calls in production code.
    ///
    /// ### Why is this bad?
    ///
    /// CTT wraps the `log` crate with structured `alert_*!` macros that add
    /// severity semantics, span context, and consistent formatting. Direct `log::`
    /// calls bypass this layer.
    ///
    /// ### Example (bad)
    ///
    /// ```rust
    /// log::warn!("retrying: {}", attempt);
    /// ```
    ///
    /// ### Example (good)
    ///
    /// ```rust
    /// use chicago_tdd_tools::prelude::*;
    /// alert_warning!("retrying: {}", attempt);
    /// ```
    pub NO_DIRECT_LOG,
    Warn,
    "use `alert_*!` macros instead of calling `log::*!` directly"
}

const LOG_MACROS: &[(&str, &str)] = &[
    ("error", "`alert_critical!`"),
    ("warn",  "`alert_warning!`"),
    ("info",  "`alert_info!`"),
    ("debug", "`alert_debug!`"),
    ("trace", "`alert_debug!`"),
];

#[derive(Default)]
pub struct NoDirectLog {
    cfg_test_depth: u32,
}

impl_lint_pass!(NoDirectLog => [NO_DIRECT_LOG]);

fn is_cfg_test_mod(item: &Item) -> bool {
    if !matches!(item.kind, ItemKind::Mod(..)) {
        return false;
    }
    item.attrs.iter().any(|attr| {
        attr.has_name(Symbol::intern("cfg"))
            && attr
                .meta_item_list()
                .is_some_and(|list| list.iter().any(|mi| mi.has_name(Symbol::intern("test"))))
    })
}

impl EarlyLintPass for NoDirectLog {
    fn check_item(&mut self, _cx: &EarlyContext<'_>, item: &Item) {
        if is_cfg_test_mod(item) {
            self.cfg_test_depth += 1;
        }
    }

    fn check_item_post(&mut self, _cx: &EarlyContext<'_>, item: &Item) {
        if is_cfg_test_mod(item) {
            self.cfg_test_depth = self.cfg_test_depth.saturating_sub(1);
        }
    }

    fn check_mac(&mut self, cx: &EarlyContext<'_>, mac: &MacCall) {
        if self.cfg_test_depth > 0 || mac.path.span.from_expansion() {
            return;
        }

        // Match `log::warn!(...)` — a two-segment path where the first is "log"
        let segments = &mac.path.segments;
        if segments.len() != 2 {
            return;
        }
        if segments[0].ident.name.as_str() != "log" {
            return;
        }
        let leaf = segments[1].ident.name.as_str().to_owned();
        if let Some(&(_, suggestion)) = LOG_MACROS.iter().find(|(name, _)| *name == leaf.as_str())
        {
            span_lint_and_help(
                cx,
                NO_DIRECT_LOG,
                mac.path.span,
                format!("`log::{leaf}!` — use CTT alert macros for consistent structured logging"),
                None,
                format!("replace with {suggestion} from `chicago_tdd_tools::prelude::*`"),
            );
        }
    }
}
