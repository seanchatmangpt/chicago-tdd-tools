//! Pure shared combinatorial-maximalism kernel. Manufactured; do not edit.
use std::collections::BTreeSet;

use super::evidence::EVIDENCE_OBLIGATIONS;
use super::external_contracts::EXTERNAL_CONTRACTS;
use super::facets::FACETS;
use super::profiles::PROFILES;
use super::projection_axes::PROJECTION_AXES;
use super::realizations::REALIZATIONS;
use super::types::{
    Authority, AuthorizedPlan, BrokerIntent, CandidatePlan, CompositionRequest,
    ConsequenceReceipt, EvidenceSet, ExecutionGrant, ObservedConsequence, Projection, Refusal,
    Standing, VerifiedPlan, Visibility,
};

const MAX_PROJECTIONS: usize = 512;
const NO_DEPENDENCY: &str = "NONE";

/// Manufacture a deterministic candidate lattice without side effects.
pub fn compose(request: CompositionRequest<'_>) -> Result<CandidatePlan, Refusal> {
    let profile = PROFILES.iter().find(|item| item.id == request.profile).ok_or(Refusal::UnknownProfile)?;
    let selected_ids: Vec<&'static str> = if profile.include_all {
        REALIZATIONS.iter().map(|item| item.id).collect()
    } else {
        profile.selections.split(',').filter(|item| !item.is_empty()).collect()
    };

    let unique: BTreeSet<&'static str> = selected_ids.iter().copied().collect();
    if unique.len() != selected_ids.len() {
        return Err(Refusal::DuplicateRealization);
    }

    let mut realizations = Vec::with_capacity(selected_ids.len());
    for id in &selected_ids {
        let realization = REALIZATIONS.iter().find(|item| item.id == *id).ok_or(Refusal::UnknownRealization)?;
        if realization.visibility == Visibility::External && (!request.include_external || !profile.external_allowed) {
            return Err(Refusal::ExternalBoundaryNotAdmitted);
        }
        if realization.dependency != NO_DEPENDENCY && !unique.contains(realization.dependency) {
            return Err(Refusal::DependencyNotClosed);
        }
        if realization.authority == Authority::Brokered && !realization.receipt_required {
            return Err(Refusal::AuthorityLeak);
        }
        realizations.push(realization);
    }

    for facet in FACETS {
        if !realizations.iter().any(|item| item.facet == facet.id) {
            return Err(Refusal::MissingFacet);
        }
    }

    let mut external_contracts = Vec::new();
    for realization in &realizations {
        if realization.visibility == Visibility::External {
            let contract = EXTERNAL_CONTRACTS
                .iter()
                .find(|item| item.source_realization == realization.id)
                .ok_or(Refusal::ExternalContractMissing)?;
            if !contract.receipt_required || !contract.replay_required || contract.private_dependency_leak {
                return Err(Refusal::ExternalContractMissing);
            }
            external_contracts.push(contract);
        }
    }

    let projection_count = realizations.len().saturating_mul(PROJECTION_AXES.len());
    if projection_count > MAX_PROJECTIONS {
        return Err(Refusal::ProjectionOverflow);
    }
    let mut projections = Vec::with_capacity(projection_count);
    for realization in &realizations {
        for axis in PROJECTION_AXES {
            projections.push(Projection { realization_id: realization.id, axis_id: axis.id });
        }
    }

    let fingerprint = fingerprint(profile.id, &realizations);
    Ok(CandidatePlan {
        profile,
        realizations,
        projections,
        external_contracts,
        fingerprint,
        standing: Standing::Candidate,
    })
}

/// Promote a candidate only when every required proof obligation is observed.
pub fn verify(candidate: CandidatePlan, evidence: EvidenceSet) -> Result<VerifiedPlan, Refusal> {
    let required_bits = required_evidence_bits();
    if evidence.bits() & required_bits != required_bits {
        return Err(Refusal::EvidenceIncomplete);
    }
    Ok(VerifiedPlan { candidate, standing: Standing::Verified })
}

/// Bind verified construction to one explicit bounded grant.
pub fn authorize(verified: VerifiedPlan, grant: ExecutionGrant) -> Result<AuthorizedPlan, Refusal> {
    if !is_exact_head(&grant.exact_head) {
        return Err(Refusal::ExactHeadInvalid);
    }
    if !grant.receipt_required {
        return Err(Refusal::ReceiptRequired);
    }
    for realization in &verified.candidate.realizations {
        if realization.visibility == Visibility::External && !grant.allow_external {
            return Err(Refusal::AuthorityLeak);
        }
        if realization.authority == Authority::Brokered && !grant.allow_brokered_actuation {
            return Err(Refusal::AuthorityLeak);
        }
        if has_effect(realization.effect, "network") && !grant.allow_network {
            return Err(Refusal::AuthorityLeak);
        }
        if has_effect(realization.effect, "storage") && !grant.allow_storage {
            return Err(Refusal::AuthorityLeak);
        }
        if has_effect(realization.effect, "process") && !grant.allow_process {
            return Err(Refusal::AuthorityLeak);
        }
        if has_effect(realization.effect, "filesystem") && !grant.allow_filesystem {
            return Err(Refusal::AuthorityLeak);
        }
    }
    Ok(AuthorizedPlan { verified, grant, standing: Standing::Authorized })
}

/// Emit a broker intent. This function performs no side effect.
#[must_use]
pub fn broker_intent(authorized: AuthorizedPlan) -> BrokerIntent {
    let realization_ids = authorized
        .verified
        .candidate
        .realizations
        .iter()
        .map(|item| item.id)
        .collect();
    BrokerIntent {
        profile_id: authorized.verified.candidate.profile.id,
        plan_fingerprint: authorized.verified.candidate.fingerprint,
        exact_head: authorized.grant.exact_head,
        realization_ids,
    }
}

/// Admit an observed consequence and manufacture its causal receipt.
pub fn admit_consequence(
    intent: &BrokerIntent,
    observed: ObservedConsequence,
) -> Result<ConsequenceReceipt, Refusal> {
    if observed.plan_fingerprint != intent.plan_fingerprint
        || observed.exact_head != intent.exact_head
        || !observed.consequence_ok
        || !observed.receiver_verified
    {
        return Err(Refusal::ConsequenceMismatch);
    }
    if !is_blake3_hex(&observed.artifact_digest_blake3) {
        return Err(Refusal::ReceiptDigestInvalid);
    }
    Ok(ConsequenceReceipt {
        profile_id: intent.profile_id,
        plan_fingerprint: intent.plan_fingerprint,
        exact_head: intent.exact_head.clone(),
        artifact_digest_blake3: observed.artifact_digest_blake3,
        standing: Standing::Receipted,
    })
}

/// Independently compare a replay observation with a receipt.
#[must_use]
pub fn replay_matches(receipt: &ConsequenceReceipt, observed: &ObservedConsequence) -> bool {
    receipt.plan_fingerprint == observed.plan_fingerprint
        && receipt.exact_head == observed.exact_head
        && receipt.artifact_digest_blake3 == observed.artifact_digest_blake3
        && observed.consequence_ok
        && observed.receiver_verified
}

/// Required evidence mask for the current ontology.
#[must_use]
pub const fn required_evidence_bits() -> u64 {
    if EVIDENCE_OBLIGATIONS.len() >= 64 {
        u64::MAX
    } else {
        (1_u64 << EVIDENCE_OBLIGATIONS.len()) - 1
    }
}

fn has_effect(effect: &str, expected: &str) -> bool {
    effect.split('+').any(|item| item == expected)
}

fn is_exact_head(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn fingerprint(profile_id: &str, realizations: &[&super::types::Realization]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in profile_id.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    for realization in realizations {
        for byte in realization.id.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    for axis in PROJECTION_AXES {
        for byte in axis.id.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}
