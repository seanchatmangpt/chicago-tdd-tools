//! Lint: `ctt_unused_result_silenced`
//!
//! `let _ = fallible_fn()` silently discards `Result` or `Option`. Every
//! fallible call must be propagated (`?`) or explicitly handled. This is a
//! LateLintPass — it runs post-type-checking and knows the actual return type.

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_hir::{LetStmt, PatKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty;
use rustc_session::{declare_lint, impl_lint_pass};

declare_lint! {
    /// Detects `let _ = <expr>` where the expression returns `Result` or `Option`.
    ///
    /// ### Why is this bad?
    ///
    /// CTT requires every fallible call to be handled. `let _ = expr` is a silent
    /// discard that hides errors. Use `?` to propagate, or an `if let Err(e) = …`
    /// block to handle explicitly.
    ///
    /// ### Example (bad)
    ///
    /// ```rust
    /// let _ = file.write_all(b"data"); // silently discards IO error
    /// ```
    ///
    /// ### Example (good)
    ///
    /// ```rust
    /// file.write_all(b"data")?;
    ///
    /// if let Err(e) = file.write_all(b"data") {
    ///     alert_warning!("write failed: {e}");
    /// }
    /// ```
    pub UNUSED_RESULT_SILENCED,
    Warn,
    "`let _ = expr` silently discards a `Result` or `Option` — propagate with `?` or handle explicitly"
}

pub struct UnusedResultSilenced;

impl_lint_pass!(UnusedResultSilenced => [UNUSED_RESULT_SILENCED]);

impl<'tcx> LateLintPass<'tcx> for UnusedResultSilenced {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx LetStmt<'tcx>) {
        // Match `let _ = <expr>` — wildcard pattern, no type annotation, no `else`.
        if !matches!(local.pat.kind, PatKind::Wild) {
            return;
        }
        if local.ty.is_some() || local.els.is_some() {
            return;
        }
        let Some(init) = local.init else { return };

        // Skip macro-generated locals (test!(), fixture_test!(), etc.)
        if local.span.from_expansion() {
            return;
        }

        let ty = cx.typeck_results().expr_ty(init);
        if is_result_or_option(cx, ty) {
            span_lint_and_help(
                cx,
                UNUSED_RESULT_SILENCED,
                local.span,
                "`let _ = …` silently discards a `Result` or `Option`",
                None,
                "use `expr?` to propagate, or `if let Err(e) = expr { … }` to handle",
            );
        }
    }
}

fn is_result_or_option<'tcx>(cx: &LateContext<'tcx>, ty: ty::Ty<'tcx>) -> bool {
    let ty = ty.peel_refs();
    match ty.kind() {
        ty::TyKind::Adt(def, _) => {
            let did = def.did();
            cx.tcx.is_diagnostic_item(rustc_span::sym::Result, did)
                || cx.tcx.is_diagnostic_item(rustc_span::sym::Option, did)
        }
        _ => false,
    }
}
