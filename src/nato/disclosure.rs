#![cfg(feature = "nato")]
//! Scoping disclosure: every court result must state whether it was
//! measured at full spec scale, and if not, what was actually delivered.

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

use serde::{Deserialize, Serialize};

/// Disclosure record carried on every court receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopingRecord {
    /// The mandate the spec asked for (static, machine-checkable).
    pub spec_mandate: String,
    /// The evidence actually delivered (static, machine-checkable).
    pub delivered_evidence: String,
    /// Whether `delivered_evidence` is at full spec scale.
    pub is_full_spec_scale: bool,
    /// The bound that limits the delivered evidence.
    pub bound: String,
}

/// Failure modes of [`ScopingRecord::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopingValidationError {
    /// Not full-scale, but no delivered evidence was named.
    MissingDeliveredEvidence,
    /// Not full-scale, but no bound was named.
    MissingBound,
}

impl ScopingRecord {
    /// Full-scale disclosure: mandate met by the delivered evidence.
    pub fn full(spec_mandate: &'static str, delivered_evidence: &'static str) -> Self {
        ScopingRecord {
            spec_mandate: spec_mandate.into(),
            delivered_evidence: delivered_evidence.into(),
            is_full_spec_scale: true,
            bound: String::new(),
        }
    }

    /// Partial-scale disclosure: evidence below spec scale must name both
    /// what was delivered and the bound that limits it.
    pub fn partial(
        spec_mandate: &'static str,
        delivered_evidence: &'static str,
        bound: &'static str,
    ) -> Self {
        ScopingRecord {
            spec_mandate: spec_mandate.into(),
            delivered_evidence: delivered_evidence.into(),
            is_full_spec_scale: false,
            bound: bound.into(),
        }
    }

    /// Disclosure used when a court is refused before running (no
    /// evidence delivered; refusal reason carried by the control result).
    pub fn refused_scope() -> Self {
        ScopingRecord {
            spec_mandate: "full court evaluation".into(),
            // Non-empty so `validate()` accepts the refusal disclosure: the
            // delivered "evidence" for a refused run is the refusal itself.
            delivered_evidence: "court refused before evaluation (positive control failed)".into(),
            is_full_spec_scale: false,
            bound: "court refused before evaluation (positive control failed)".into(),
        }
    }

    /// Debug validator: a partial disclosure must name non-empty delivered
    /// evidence and a non-empty bound. Full-scale disclosure requires only
    /// a non-empty mandate.
    pub fn validate(&self) -> Result<(), ScopingValidationError> {
        if self.spec_mandate.is_empty() {
            // Mandate is required in all cases; treat as missing evidence
            // of the mandate itself.
            return Err(ScopingValidationError::MissingDeliveredEvidence);
        }
        if !self.is_full_spec_scale {
            if self.delivered_evidence.is_empty() {
                return Err(ScopingValidationError::MissingDeliveredEvidence);
            }
            if self.bound.is_empty() {
                return Err(ScopingValidationError::MissingBound);
            }
        }
        Ok(())
    }
}
