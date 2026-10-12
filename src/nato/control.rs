#![cfg(feature = "nato")]
//! Positive-control helpers: run a [`ControlProbe`] and classify the result.

// Lint-scope note: NATO court-harness and ledger wire-format modules use
// domain literals, metric casts, and court-assertion panics by design; the
// style-class lints below are allowed at module scope with this
// justification rather than 300+ inline permits.
#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::use_self,
    clippy::missing_const_for_fn,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::explicit_iter_loop,
    clippy::unusual_byte_groupings,
    clippy::unreadable_literal,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::needless_range_loop,
    clippy::needless_continue,
    clippy::unnecessary_wraps,
    clippy::double_must_use,
    clippy::op_ref,
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::or_fun_call,
    clippy::nonminimal_bool,
    clippy::unnecessary_map_or,
    clippy::manual_is_multiple_of,
    clippy::manual_midpoint,
    clippy::identity_op,
    clippy::format_collect,
    clippy::trivially_copy_pass_by_ref
)]

use crate::nato::court::ControlProbe;
use crate::nato::receipt::ControlResult;

/// Run a positive-control probe and classify the outcome.
///
/// A probe returning `true` is classified as `MutantCaught` with an empty
/// detection-channel marker (the probe itself is the channel); `false` is
/// classified as `GateRefused` — the court is not allowed to run.
pub fn run_probe(probe: ControlProbe) -> ControlResult {
    if probe() {
        ControlResult::MutantCaught {
            mutant_id: "positive-control".to_string(),
            detection_channel: "probe".to_string(),
        }
    } else {
        ControlResult::GateRefused {
            reason: "positive control failed: mutant survived the control probe".to_string(),
        }
    }
}

/// Whether the control result permits the court to run.
pub fn probe_passed(control: &ControlResult) -> bool {
    matches!(control, ControlResult::MutantCaught { .. })
}

/// Convenience constructor for a refused control result with a caller
/// supplied reason (used by courts that classify survival explicitly).
pub fn build_control_result(probe_passed: bool, mutant_id: &str, reason: &str) -> ControlResult {
    if probe_passed {
        ControlResult::MutantCaught {
            mutant_id: mutant_id.to_string(),
            detection_channel: "probe".to_string(),
        }
    } else {
        ControlResult::GateRefused { reason: reason.to_string() }
    }
}
