//! Lint: `ctt_no_println_in_production`
//!
//! `println!`, `eprintln!`, `print!`, `eprint!`, and `dbg!` write unstructured output.
//! CTT enforces structured logging via `alert_info!`, `alert_warning!`, etc.
//!
//! Fires only outside `#[cfg(test)]` modules.

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::{Item, ItemKind, MacCall};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::Symbol;

declare_lint! {
    /// Detects `println!`, `eprintln!`, `print!`, `eprint!`, or `dbg!` in production code.
    ///
    /// ### Why is this bad?
    ///
    /// Unstructured output bypasses CTT's alert logging infrastructure. Use
    /// `alert_info!`, `alert_warning!`, `alert_critical!`, or `alert_debug!`
    /// from `chicago_tdd_tools::prelude::*` instead.
    ///
    /// ### Example (bad)
    ///
    /// ```rust
    /// pub fn process(id: u32) {
    ///     println!("processing {id}");
    /// }
    /// ```
    ///
    /// ### Example (good)
    ///
    /// ```rust
    /// use chicago_tdd_tools::prelude::*;
    /// pub fn process(id: u32) {
    ///     alert_info!("processing {id}");
    /// }
    /// ```
    pub NO_PRINTLN_IN_PRODUCTION,
    Warn,
    "use `alert_info!` / `alert_warning!` instead of `println!` / `eprintln!` in production code"
}

const PRINT_MACROS: &[(&str, &str)] = &[
    ("println", "`alert_info!`"),
    ("print",   "`alert_info!`"),
    ("eprintln","`alert_warning!`"),
    ("eprint",  "`alert_warning!`"),
    ("dbg",     "`alert_debug!`"),
];

/// Track cfg(test) depth with a counter so nested test mods are handled correctly.
#[derive(Default)]
pub struct NoPrintln {
    cfg_test_depth: u32,
}

impl_lint_pass!(NoPrintln => [NO_PRINTLN_IN_PRODUCTION]);

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

impl EarlyLintPass for NoPrintln {
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
        if self.cfg_test_depth > 0 {
            return; // inside #[cfg(test)]
        }
        if mac.path.span.from_expansion() {
            return; // macro-generated code
        }

        let macro_name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.name.as_str().to_owned())
            .unwrap_or_default();

        if let Some(&(_, suggestion)) =
            PRINT_MACROS.iter().find(|(name, _)| *name == macro_name.as_str())
        {
            span_lint_and_help(
                cx,
                NO_PRINTLN_IN_PRODUCTION,
                mac.path.span,
                format!("`{macro_name}!` in production code — use structured logging instead"),
                None,
                format!("replace with {suggestion} from `chicago_tdd_tools::prelude::*`"),
            );
        }
    }
}
