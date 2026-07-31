use chicago_tdd_combinatorial_maximalism::{
    admit_consequence, authorize, broker_intent, compose, replay_matches, required_evidence_bits,
    verify, CompositionRequest, EvidenceSet, ExecutionGrant, ObservedConsequence, Refusal,
    Standing, EXTERNAL_CONTRACTS, FACETS, PROFILES, PROJECTION_AXES, REALIZATIONS,
};
use std::collections::BTreeSet;

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn ontology_projection_is_total_and_unique() {
    assert_eq!(FACETS.len(), 8);
    assert_eq!(REALIZATIONS.len(), 35);
    assert_eq!(PROFILES.len(), 4);
    assert_eq!(PROJECTION_AXES.len(), 10);
    assert_eq!(FACETS.iter().map(|item| item.id).collect::<BTreeSet<_>>().len(), FACETS.len());
    assert_eq!(REALIZATIONS.iter().map(|item| item.id).collect::<BTreeSet<_>>().len(), REALIZATIONS.len());
}

#[test]
fn core_profile_manufactures_the_full_projection_product() {
    let plan = compose(CompositionRequest { profile: "profile.core-local", include_external: false }).unwrap_or_else(|error| panic!("{}", error.code()));
    assert_eq!(plan.standing, Standing::Candidate);
    assert_eq!(plan.realizations.len(), 12);
    assert_eq!(plan.projections.len(), plan.realizations.len() * PROJECTION_AXES.len());
    assert!(plan.external_contracts.is_empty());
}

#[test]
fn external_profile_refuses_without_boundary_admission() {
    let error = compose(CompositionRequest { profile: "profile.external-integration", include_external: false }).expect_err("external profile must refuse");
    assert_eq!(error, Refusal::ExternalBoundaryNotAdmitted);
}

#[test]
fn evidence_authority_receipt_and_replay_are_separate_states() {
    let candidate = compose(CompositionRequest { profile: "profile.external-integration", include_external: true }).unwrap_or_else(|error| panic!("{}", error.code()));
    let verified = verify(candidate, EvidenceSet::from_bits(required_evidence_bits())).unwrap_or_else(|error| panic!("{}", error.code()));
    assert_eq!(verified.standing, Standing::Verified);
    let grant = ExecutionGrant {
        exact_head: HEAD.to_string(),
        allow_external: true,
        allow_brokered_actuation: true,
        allow_network: true,
        allow_storage: true,
        allow_process: true,
        allow_filesystem: false,
        receipt_required: true,
    };
    let authorized = authorize(verified, grant).unwrap_or_else(|error| panic!("{}", error.code()));
    assert_eq!(authorized.standing, Standing::Authorized);
    let intent = broker_intent(authorized);
    let observed = ObservedConsequence {
        plan_fingerprint: intent.plan_fingerprint,
        exact_head: intent.exact_head.clone(),
        artifact_digest_blake3: DIGEST.to_string(),
        consequence_ok: true,
        receiver_verified: true,
    };
    let receipt = admit_consequence(&intent, observed.clone()).unwrap_or_else(|error| panic!("{}", error.code()));
    assert_eq!(receipt.standing, Standing::Receipted);
    assert!(replay_matches(&receipt, &observed));
}

#[test]
fn every_external_realization_has_a_closed_contract() {
    for realization in REALIZATIONS.iter().filter(|item| matches!(item.visibility, chicago_tdd_combinatorial_maximalism::Visibility::External)) {
        let contract = EXTERNAL_CONTRACTS.iter().find(|item| item.source_realization == realization.id);
        assert!(contract.is_some(), "missing contract for {}", realization.id);
        let contract = contract.unwrap_or_else(|| unreachable!());
        assert!(contract.receipt_required);
        assert!(contract.replay_required);
        assert!(!contract.private_dependency_leak);
    }
}
