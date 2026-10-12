#![cfg(feature = "nato")]
//! Receipted court outcomes: serde-serializable receipts with a stable
//! SHA-256 digest over a canonical string.

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
use sha2::{Digest, Sha256};

use crate::nato::court::{CourtId, CourtOutcome, MetricValue, Pillar, Verdict};
use crate::nato::disclosure::ScopingRecord;

/// How the positive control resolved for this run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlResult {
    /// The control caught its known-bad input; the court is cleared to run.
    MutantCaught {
        /// Id of the injected mutant the control caught.
        mutant_id: String,
        /// Channel through which the mutant was detected.
        detection_channel: String,
    },
    /// The control's known-bad input survived; recorded only when a court
    /// explicitly classifies survival after running.
    MutantSurvived {
        /// Id of the surviving mutant.
        mutant_id: String,
    },
    /// The control failed; the court was refused before running.
    GateRefused {
        /// Human-readable refusal reason.
        reason: String,
    },
}

impl ControlResult {
    /// Whether this control result cleared the court to run.
    pub fn passed(&self) -> bool {
        matches!(self, ControlResult::MutantCaught { .. })
    }
}

/// A receipted court run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CourtReceipt {
    /// Stable id: `cr-<court>-<digest16>`.
    pub receipt_id: String,
    /// Court identifier.
    pub court_id: CourtId,
    /// Capability pillar.
    pub pillar: Pillar,
    /// Canonical court name.
    pub name: String,
    /// SHA-256 of the audited subject.
    pub subject_sha: String,
    /// Byte verdict of the run.
    pub verdict: Verdict,
    /// Named metric values from the run.
    pub metrics: Vec<(String, MetricValue)>,
    /// Positive-control result for the run.
    pub positive_control: ControlResult,
    /// Scoping disclosure for the run.
    pub scoping: ScopingRecord,
    /// SHA-256 over the canonical string (see [`CourtReceipt::canonical`]).
    /// SHA-256 over the canonical string.
    pub digest: String,
    /// Epoch-less ordering slot (0; receipts order by digest).
    pub timestamp_ms: u64,
}

impl CourtReceipt {
    /// Mint a receipt from a court outcome, computing the digest and a
    /// receipt id derived from it.
    pub fn record(
        court_id: CourtId,
        pillar: Pillar,
        name: &str,
        subject_sha: &str,
        outcome: CourtOutcome,
        positive_control: ControlResult,
    ) -> Self {
        let metrics: Vec<(String, MetricValue)> =
            outcome.metrics.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        let digest =
            canonical_digest(CourtId::canonical(court_id), subject_sha, outcome.verdict, &metrics);
        let receipt_id = format!("cr-{}-{}", CourtId::canonical(court_id), &digest[..16]);

        CourtReceipt {
            receipt_id,
            court_id,
            pillar,
            name: name.to_string(),
            subject_sha: subject_sha.to_string(),
            verdict: outcome.verdict,
            metrics,
            positive_control,
            scoping: outcome.scoping,
            digest,
            // Fixed epoch-less counter slot: receipts are ordered by
            // digest, not wall-clock; lanes that need real time inject it.
            timestamp_ms: 0,
        }
    }

    /// Canonical string: `id|sha|verdict|k1=v1;k2=v2`.
    pub fn canonical(&self) -> String {
        canonical_string(
            CourtId::canonical(self.court_id),
            &self.subject_sha,
            self.verdict,
            &self.metrics,
        )
    }

    /// Serialize to pretty JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Canonical string for a court run.
fn canonical_string(
    court: &str,
    subject_sha: &str,
    verdict: Verdict,
    metrics: &[(String, MetricValue)],
) -> String {
    let mut s = String::new();
    s.push_str(court);
    s.push('|');
    s.push_str(subject_sha);
    s.push('|');
    s.push_str(match verdict {
        Verdict::Pass => "pass",
        Verdict::Fail => "fail",
        Verdict::Refused => "refused",
    });
    s.push('|');
    for (i, (k, v)) in metrics.iter().enumerate() {
        if i > 0 {
            s.push(';');
        }
        s.push_str(k);
        s.push('=');
        s.push_str(v.tag());
        s.push(':');
        s.push_str(&v.render());
    }
    s
}

/// SHA-256 hex digest over the canonical string.
pub fn canonical_digest(
    court: &str,
    subject_sha: &str,
    verdict: Verdict,
    metrics: &[(String, MetricValue)],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_string(court, subject_sha, verdict, metrics).as_bytes());
    let bytes = hasher.finalize();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nato::control;
    use crate::nato::court::{
        run_court, CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar,
        Verdict,
    };
    use crate::nato::disclosure::{ScopingRecord, ScopingValidationError};
    use crate::nato::rng::Lcg;

    struct AlwaysPass;
    struct BrokenControl;

    impl AlwaysPass {
        /// Court identifier.
        const ID: CourtId = CourtId::N01;
        /// Capability pillar.
        const PILLAR: Pillar = Pillar::Security;
    }

    impl Court for AlwaysPass {
        fn id(&self) -> CourtId {
            Self::ID
        }

        fn pillar(&self) -> Pillar {
            Self::PILLAR
        }

        fn name(&self) -> &'static str {
            "N01"
        }
        fn positive_control(&self) -> ControlProbe {
            || true
        }
        fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
            CourtOutcome {
                verdict: Verdict::Pass,
                metrics: vec![("count", MetricValue::U64(42)), ("ratio", MetricValue::F64(0.5))],
                scoping: ScopingRecord::full("mandate", "evidence"),
            }
        }
    }

    impl BrokenControl {
        /// Court identifier.
        const ID: CourtId = CourtId::N02;
        /// Capability pillar.
        const PILLAR: Pillar = Pillar::Timing;
    }

    impl Court for BrokenControl {
        fn id(&self) -> CourtId {
            Self::ID
        }

        fn pillar(&self) -> Pillar {
            Self::PILLAR
        }

        fn name(&self) -> &'static str {
            "N02"
        }
        fn positive_control(&self) -> ControlProbe {
            || false
        }
        fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
            // Must never be reached when the control fails.
            CourtOutcome {
                verdict: Verdict::Pass,
                metrics: Vec::new(),
                scoping: ScopingRecord::full("mandate", "evidence"),
            }
        }
    }

    #[test]
    fn positive_control_refusal_short_circuits_run() {
        let receipt = run_court(&BrokenControl, &CandidateRef("abc".into()), "sha-x");
        assert_eq!(receipt.verdict, Verdict::Refused);
        assert!(!receipt.positive_control.passed());
        assert!(matches!(receipt.positive_control, ControlResult::GateRefused { .. }));
        assert!(receipt.metrics.is_empty());
        assert!(receipt.scoping.validate().is_ok());
    }

    #[test]
    fn passing_control_runs_the_court() {
        let receipt = run_court(&AlwaysPass, &CandidateRef("abc".into()), "sha-y");
        assert_eq!(receipt.verdict, Verdict::Pass);
        assert!(receipt.positive_control.passed());
        assert_eq!(receipt.metrics.len(), 2);
        assert_eq!(receipt.pillar, Pillar::Security);
        assert_eq!(receipt.court_id, CourtId::N01);
    }

    #[test]
    fn receipt_json_roundtrip() {
        let receipt = run_court(&AlwaysPass, &CandidateRef("abc".into()), "sha-z");
        let json = receipt.to_json().expect("serialize");
        let back = CourtReceipt::from_json(&json).expect("deserialize");
        assert_eq!(back, receipt);
        assert_eq!(back.digest, receipt.digest);
    }

    #[test]
    fn digest_is_stable_and_discriminating() {
        let a = canonical_digest("N01", "sha", Verdict::Pass, &[]);
        let b = canonical_digest("N01", "sha", Verdict::Pass, &[]);
        let c = canonical_digest("N01", "sha", Verdict::Fail, &[]);
        let d = canonical_digest(
            "N01",
            "sha",
            Verdict::Pass,
            &[("k".to_string(), MetricValue::U64(1))],
        );
        let e = canonical_digest(
            "N01",
            "sha",
            Verdict::Pass,
            &[("k".to_string(), MetricValue::Str("1".to_string()))],
        );
        assert_eq!(a, b, "same inputs must give same digest");
        assert_ne!(a, c, "verdict must change the digest");
        assert_ne!(a, d, "metrics must change the digest");
        assert_ne!(d, e, "metric type must change the digest");
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn scoping_validator() {
        let full = ScopingRecord::full("mandate", "evidence");
        assert!(full.validate().is_ok());

        let partial_ok = ScopingRecord::partial("mandate", "evidence", "bound");
        assert!(partial_ok.validate().is_ok());

        let partial_no_evidence = ScopingRecord {
            spec_mandate: "mandate".into(),
            delivered_evidence: String::new(),
            is_full_spec_scale: false,
            bound: "bound".into(),
        };
        assert_eq!(
            partial_no_evidence.validate(),
            Err(ScopingValidationError::MissingDeliveredEvidence)
        );

        let partial_no_bound = ScopingRecord {
            spec_mandate: "mandate".into(),
            delivered_evidence: "evidence".into(),
            is_full_spec_scale: false,
            bound: String::new(),
        };
        assert_eq!(partial_no_bound.validate(), Err(ScopingValidationError::MissingBound));
    }

    #[test]
    fn lcg_is_deterministic() {
        let mut a = Lcg::new(12345);
        let mut b = Lcg::new(12345);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut c = Lcg::new(1);
        let first = c.sample(10);
        let mut d = Lcg::new(1);
        assert_eq!(first, d.sample(10));
        assert!(c.sample(10) < 10);
        assert_eq!(c.sample(0), 0);
        // Different seeds diverge.
        let mut e = Lcg::new(2);
        let mut f = Lcg::new(3);
        assert_ne!(e.next_u64(), f.next_u64());
    }

    #[test]
    fn control_helpers_classify() {
        let pass: ControlProbe = || true;
        let fail: ControlProbe = || false;
        assert!(control::run_probe(pass).passed());
        assert!(!control::run_probe(fail).passed());
        assert!(control::probe_passed(&control::run_probe(pass)));
        let built = control::build_control_result(false, "m1", "survived");
        assert!(matches!(built, ControlResult::GateRefused { .. }));
    }
}
