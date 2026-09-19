//! Typed semantic court for the repository-wide `CHICAGO_ALIVE` standing.
//!
//! This module intentionally does not reuse [`crate::reconcile::Standing`]. That
//! older enum is scoped to the chicago-claims structural/mutation MLP. The types
//! here model the ecosystem standing boundary where `CHICAGO_ALIVE` sits between
//! `PARTIAL_ALIVE` and `ALIVE`.
//!
//! The court is pure: it accepts evidence and derives standing. It has no process,
//! network, filesystem, credential, broker, or actuation authority. Exact-head
//! BLAKE3 receiver receipts are independently implemented by
//! `scripts/verify_chicago_alive.py`.

use serde::{Deserialize, Serialize};

/// Repository/ecosystem standing values understood by the Chicago court.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChicagoStanding {
    /// No admitted evidence is available.
    Unknown,
    /// Meaningful bounded execution exists, but Chicago closure is incomplete.
    PartialAlive,
    /// Exact subject survived an admitted executable world with the full Chicago evidence boundary.
    ChicagoAlive,
    /// Exact admitted real target produced the claimed observed consequence.
    Alive,
    /// Required subject/world structure or admission is unavailable.
    Blocked,
    /// The executable subject is known to be build-broken.
    BuildBroken,
    /// The capability is outside the declared support boundary.
    Unsupported,
    /// Evidence contradicts or violates a hard admission/authority invariant.
    Refused,
}

/// Kind of executable world used by one standing trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorldKind {
    /// A bounded synthetic world whose physics are explicitly admitted.
    Synthetic,
    /// A protocol/provider emulator with declared correspondence bounds.
    Emulated,
    /// A real non-production tenant used as a bounded test world.
    TestTenant,
    /// The exact admitted real target boundary required for `ALIVE`.
    RealTarget,
}

/// Whether a collaborator is the real implementation or a test double.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CollaboratorKind {
    /// The implementation that owns the claimed behavior.
    Real,
    /// A fake/mock/stub supplying behavior in place of the owning implementation.
    TestDouble,
}

/// Runtime receipt replay state supplied by the trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReplayState {
    /// Receipt replay reproduced the admitted consequence.
    Match,
    /// Replay diverged from the admitted consequence.
    Mismatch,
    /// Replay has not yet executed.
    NotRun,
}

/// Exact software subject identity under trial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectEvidence {
    /// Stable receiver-facing subject reference, normally `repo@sha`.
    #[serde(rename = "ref")]
    pub subject_ref: String,
    /// Whether the subject identity is exact rather than floating or approximate.
    pub exact: bool,
    /// Exact lowercase 40-hex Git commit identity.
    pub commit_sha: String,
}

/// Admitted executable world identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldEvidence {
    /// Stable world identity.
    #[serde(rename = "ref")]
    pub world_ref: String,
    /// Whether the world has crossed the admission boundary.
    pub admitted: bool,
    /// Class of world in which execution occurred.
    pub kind: WorldKind,
}

/// Evidence for one collaboration edge in the exercised implementation topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollaboratorEvidence {
    /// Stable collaborator/edge identifier.
    pub id: String,
    /// Real implementation or test double.
    pub kind: CollaboratorKind,
    /// Whether this collaborator supplies behavior required by the claim.
    pub load_bearing: bool,
    /// Evidence proving which collaborator executed.
    pub evidence_ref: String,
}

/// Observed authority properties for the trial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityEvidence {
    /// Whether the governed authority path itself was observed.
    pub path_observed: bool,
    /// Whether any direct actuation bypassed the broker boundary.
    pub direct_actuation: bool,
    /// Whether production authority was exercised during this trial.
    pub production_authority_used: bool,
}

/// Observable execution/consequence facts for one trial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEvidence {
    /// Exact subject execution was observed in the admitted world.
    pub execution_observed: bool,
    /// A world consequence was independently observed.
    pub consequence_observed: bool,
    /// The real target identity was exact when a real-target crown is claimed.
    pub real_target_identity_exact: bool,
    /// The exact real target produced the claimed observed consequence.
    pub real_target_consequence_observed: bool,
}

/// One adversarial falsifier outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FalsifierEvidence {
    /// Stable falsifier identifier.
    pub id: String,
    /// Whether the system survived the falsifier as specified.
    pub passed: bool,
    /// Evidence for the falsifier execution.
    pub evidence_ref: String,
}

/// Bounded relationship between the executable world and the claimed target world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrespondenceEvidence {
    /// Whether correspondence is explicitly bounded rather than universalized.
    pub bounded: bool,
    /// Stable target-world identity.
    pub target_ref: String,
    /// Stable contract/revision identity defining correspondence.
    pub contract_ref: String,
    /// Explicit known divergences that remain outside the claim boundary.
    pub known_divergences: Vec<String>,
}

/// Complete evidence input consumed by the pure Chicago standing court.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChicagoTrial {
    /// Exact software subject.
    pub subject: SubjectEvidence,
    /// Admitted executable world.
    pub world: WorldEvidence,
    /// Observed authority path.
    pub authority: AuthorityEvidence,
    /// Observed execution and consequence.
    pub trial: ExecutionEvidence,
    /// Collaboration topology exercised by the trial.
    pub collaborators: Vec<CollaboratorEvidence>,
    /// Adversarial falsifiers executed against the subject/world pair.
    pub falsifiers: Vec<FalsifierEvidence>,
    /// Bounded correspondence contract to the target world.
    pub correspondence: CorrespondenceEvidence,
    /// Consequence receipts emitted by the exercised system.
    pub receipt_refs: Vec<String>,
    /// Runtime replay state for those receipts.
    pub replay: ReplayState,
}

/// One typed court finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChicagoFinding {
    /// Stable machine-readable finding code.
    pub code: String,
    /// Human-readable bounded explanation.
    pub detail: String,
}

/// Derived result of evaluating one [`ChicagoTrial`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChicagoDecision {
    /// Mechanically derived standing.
    pub standing: ChicagoStanding,
    /// Admission/structural defects that prevent evaluation closure.
    pub blockers: Vec<ChicagoFinding>,
    /// Contradictory or prohibited evidence that refuses promotion.
    pub refusals: Vec<ChicagoFinding>,
    /// Missing but non-contradictory evidence that leaves the subject partial.
    pub gaps: Vec<ChicagoFinding>,
    /// Real-world evidence still needed to graduate a Chicago crown to `ALIVE`.
    pub missing_for_alive: Vec<String>,
    /// The semantic court never directly actuates.
    pub direct_actuation: bool,
}

fn finding(code: &str, detail: impl Into<String>) -> ChicagoFinding {
    ChicagoFinding {
        code: code.to_owned(),
        detail: detail.into(),
    }
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn exact_sha40(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sort_findings(findings: &mut [ChicagoFinding]) {
    findings.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.detail.cmp(&right.detail))
    });
}

/// Derive standing from admitted trial evidence without performing any actuation.
///
/// `required_exact_head`, when present, acts as the receiver's exact-head fence.
/// A mismatch is a typed refusal rather than a downgrade to approximate evidence.
#[must_use]
pub fn evaluate_chicago_alive(
    evidence: &ChicagoTrial,
    required_exact_head: Option<&str>,
) -> ChicagoDecision {
    let mut blockers = Vec::new();
    let mut refusals = Vec::new();
    let mut gaps = Vec::new();
    let mut missing_for_alive = Vec::new();

    if !nonempty(&evidence.subject.subject_ref) {
        blockers.push(finding("CTA-SUB-001", "exact subject ref is missing"));
    }
    if !evidence.subject.exact {
        blockers.push(finding("CTA-SUB-002", "subject identity is not exact"));
    }
    if !exact_sha40(&evidence.subject.commit_sha) {
        blockers.push(finding(
            "CTA-SUB-003",
            "subject commit_sha must be exact lowercase 40-hex",
        ));
    }
    if let Some(required) = required_exact_head {
        if !exact_sha40(required) {
            blockers.push(finding(
                "CTA-SUB-005",
                "required exact head must be lowercase 40-hex",
            ));
        } else if evidence.subject.commit_sha != required {
            refusals.push(finding(
                "CTA-SUB-004",
                format!(
                    "subject commit {} does not match required exact head {required}",
                    evidence.subject.commit_sha
                ),
            ));
        }
    }

    if !nonempty(&evidence.world.world_ref) {
        blockers.push(finding("CTA-WLD-001", "world ref is missing"));
    }
    if !evidence.world.admitted {
        blockers.push(finding("CTA-WLD-002", "world is not admitted"));
    }

    let mut load_bearing_count = 0usize;
    for collaborator in &evidence.collaborators {
        if !collaborator.load_bearing {
            continue;
        }
        load_bearing_count += 1;
        if collaborator.kind != CollaboratorKind::Real {
            refusals.push(finding(
                "CTA-COL-001",
                format!(
                    "load-bearing collaborator is not real: {}",
                    collaborator.id
                ),
            ));
        }
        if !nonempty(&collaborator.evidence_ref) {
            gaps.push(finding(
                "CTA-COL-003",
                format!(
                    "load-bearing collaborator lacks evidence: {}",
                    collaborator.id
                ),
            ));
        }
    }
    if load_bearing_count == 0 {
        gaps.push(finding(
            "CTA-COL-004",
            "no load-bearing collaboration edge was evidenced",
        ));
    }

    if evidence.authority.direct_actuation {
        refusals.push(finding(
            "CTA-AUT-001",
            "direct actuation bypasses the broker boundary",
        ));
    }
    if !evidence.authority.path_observed {
        gaps.push(finding("CTA-AUT-002", "authority path was not observed"));
    }
    if evidence.world.kind != WorldKind::RealTarget && evidence.authority.production_authority_used {
        refusals.push(finding(
            "CTA-AUT-003",
            "production authority was used outside the real target world",
        ));
    }

    if !evidence.trial.execution_observed {
        gaps.push(finding(
            "CTA-EXE-001",
            "exact subject execution was not observed in the admitted world",
        ));
    }
    if !evidence.trial.consequence_observed {
        gaps.push(finding(
            "CTA-CON-001",
            "observable world consequence is missing",
        ));
    }

    if !evidence.receipt_refs.iter().any(|item| nonempty(item)) {
        gaps.push(finding("CTA-RCP-001", "no consequence receipt was supplied"));
    }

    match evidence.replay {
        ReplayState::Match => {}
        ReplayState::Mismatch => refusals.push(finding("CTA-RPL-001", "receipt replay diverged")),
        ReplayState::NotRun => gaps.push(finding("CTA-RPL-002", "receipt replay has not matched")),
    }

    if evidence.falsifiers.is_empty() {
        gaps.push(finding(
            "CTA-FAL-001",
            "no adversarial falsifier was executed",
        ));
    } else {
        for falsifier in &evidence.falsifiers {
            if !falsifier.passed {
                refusals.push(finding(
                    "CTA-FAL-002",
                    format!("falsifier did not pass: {}", falsifier.id),
                ));
            }
            if !nonempty(&falsifier.evidence_ref) {
                gaps.push(finding(
                    "CTA-FAL-003",
                    format!("falsifier lacks evidence: {}", falsifier.id),
                ));
            }
        }
    }

    if !evidence.correspondence.bounded {
        blockers.push(finding(
            "CTA-COR-001",
            "target correspondence is not bounded",
        ));
    }
    if !nonempty(&evidence.correspondence.target_ref) {
        blockers.push(finding(
            "CTA-COR-002",
            "correspondence target ref is missing",
        ));
    }
    if !nonempty(&evidence.correspondence.contract_ref) {
        blockers.push(finding(
            "CTA-COR-003",
            "correspondence contract ref is missing",
        ));
    }

    if evidence.trial.real_target_consequence_observed
        && evidence.world.kind != WorldKind::RealTarget
    {
        refusals.push(finding(
            "CTA-ALV-001",
            "real-target consequence claimed outside REAL_TARGET world",
        ));
    }

    sort_findings(&mut blockers);
    sort_findings(&mut refusals);
    sort_findings(&mut gaps);

    let standing = if !blockers.is_empty() {
        ChicagoStanding::Blocked
    } else if !refusals.is_empty() {
        ChicagoStanding::Refused
    } else if !gaps.is_empty() {
        ChicagoStanding::PartialAlive
    } else if evidence.world.kind == WorldKind::RealTarget
        && evidence.trial.real_target_identity_exact
        && evidence.trial.real_target_consequence_observed
    {
        ChicagoStanding::Alive
    } else {
        if evidence.world.kind != WorldKind::RealTarget {
            missing_for_alive.push("exact real target world".to_owned());
        }
        if !evidence.trial.real_target_identity_exact {
            missing_for_alive.push("exact real target identity".to_owned());
        }
        if !evidence.trial.real_target_consequence_observed {
            missing_for_alive.push("observed real target consequence".to_owned());
        }
        missing_for_alive.sort();
        ChicagoStanding::ChicagoAlive
    };

    ChicagoDecision {
        standing,
        blockers,
        refusals,
        gaps,
        missing_for_alive,
        direct_actuation: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_trial() -> ChicagoTrial {
        ChicagoTrial {
            subject: SubjectEvidence {
                subject_ref: "repo@0123456789abcdef0123456789abcdef01234567".to_owned(),
                exact: true,
                commit_sha: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            },
            world: WorldEvidence {
                world_ref: "urn:gymact:world:test".to_owned(),
                admitted: true,
                kind: WorldKind::Synthetic,
            },
            authority: AuthorityEvidence {
                path_observed: true,
                direct_actuation: false,
                production_authority_used: false,
            },
            trial: ExecutionEvidence {
                execution_observed: true,
                consequence_observed: true,
                real_target_identity_exact: false,
                real_target_consequence_observed: false,
            },
            collaborators: vec![CollaboratorEvidence {
                id: "authority".to_owned(),
                kind: CollaboratorKind::Real,
                load_bearing: true,
                evidence_ref: "urn:evidence:authority".to_owned(),
            }],
            falsifiers: vec![FalsifierEvidence {
                id: "receipt-tamper".to_owned(),
                passed: true,
                evidence_ref: "urn:evidence:falsifier".to_owned(),
            }],
            correspondence: CorrespondenceEvidence {
                bounded: true,
                target_ref: "urn:target:real".to_owned(),
                contract_ref: "urn:contract:v1".to_owned(),
                known_divergences: vec![],
            },
            receipt_refs: vec!["urn:receipt:1".to_owned()],
            replay: ReplayState::Match,
        }
    }

    #[test]
    fn complete_synthetic_trial_is_chicago_alive() {
        let trial = base_trial();
        let decision = evaluate_chicago_alive(&trial, None);
        assert_eq!(decision.standing, ChicagoStanding::ChicagoAlive);
        assert!(!decision.direct_actuation);
    }

    #[test]
    fn exact_real_target_consequence_is_alive() {
        let mut trial = base_trial();
        trial.world.kind = WorldKind::RealTarget;
        trial.trial.real_target_identity_exact = true;
        trial.trial.real_target_consequence_observed = true;
        let decision = evaluate_chicago_alive(&trial, None);
        assert_eq!(decision.standing, ChicagoStanding::Alive);
    }

    #[test]
    fn load_bearing_test_double_refuses_crown() {
        let mut trial = base_trial();
        trial.collaborators[0].kind = CollaboratorKind::TestDouble;
        let decision = evaluate_chicago_alive(&trial, None);
        assert_eq!(decision.standing, ChicagoStanding::Refused);
        assert!(decision.refusals.iter().any(|item| item.code == "CTA-COL-001"));
    }

    #[test]
    fn missing_replay_stays_partial_alive() {
        let mut trial = base_trial();
        trial.replay = ReplayState::NotRun;
        let decision = evaluate_chicago_alive(&trial, None);
        assert_eq!(decision.standing, ChicagoStanding::PartialAlive);
    }

    #[test]
    fn unadmitted_world_is_blocked() {
        let mut trial = base_trial();
        trial.world.admitted = false;
        let decision = evaluate_chicago_alive(&trial, None);
        assert_eq!(decision.standing, ChicagoStanding::Blocked);
    }

    #[test]
    fn exact_head_mismatch_refuses() {
        let trial = base_trial();
        let decision = evaluate_chicago_alive(&trial, Some("ffffffffffffffffffffffffffffffffffffffff"));
        assert_eq!(decision.standing, ChicagoStanding::Refused);
        assert!(decision.refusals.iter().any(|item| item.code == "CTA-SUB-004"));
    }
}
