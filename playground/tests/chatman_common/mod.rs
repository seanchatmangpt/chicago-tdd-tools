//! Shared helpers for the Chatman Engine validation suite (feature: chatman).
//!
//! Contract-first: everything here is built against the Chatman ABI as landed
//! in `praxis-graphlaw/src/chatman/` (workflow wf_255e0807). No production
//! logic lives here — only deterministic scenario builders and seed derivation.
//!
//! Each `chatman_*` integration-test binary includes this module, so helpers
//! unused by a given binary are expected; hence the module-wide dead_code allow.
#![allow(dead_code)]

use praxis_graphlaw::chatman::abi::{
    GraphSnapshotId, InputHandles, InvocationEnvelope, InvocationId, OperatorId, ProfileId,
    Refusal,
};
use praxis_graphlaw::chatman::admission8::{AdmissionTable8, ConstraintMask};

/// Builds a deterministic invocation envelope whose identities are derived
/// from `label`. Same label, same envelope, byte-identical hash — always.
pub fn envelope(label: &str) -> InvocationEnvelope {
    InvocationEnvelope {
        invocation_id: InvocationId::new(format!("inv:{label}")),
        snapshot_id: GraphSnapshotId::new(format!("snap:{label}")),
        profile_id: ProfileId::new(format!("profile:{label}")),
        operator_id: OperatorId::new(format!("op:{label}")),
        input_handles: InputHandles {
            nodes: vec![format!("<urn:{label}:n1>"), format!("<urn:{label}:n2>")],
            events: vec![format!("ocel:{label}:e1")],
            plan_steps: vec![format!("plan:{label}:s1")],
        },
    }
}

/// Derives a deterministic 32-byte proptest seed from a label, via BLAKE3.
///
/// The BLAKE3 pass is the ABI's own `envelope_hash()` (which routes through
/// `wasm4pm_compat::hash`), so seed derivation shares hash identity with the
/// engine instead of introducing a second hashing scheme.
pub fn seed_from(label: &str) -> [u8; 32] {
    let hex = envelope(label).envelope_hash();
    let bytes = hex.as_bytes();
    assert_eq!(bytes.len(), 64, "envelope_hash must be 64 hex chars");
    let mut seed = [0u8; 32];
    for (i, chunk) in bytes.chunks(2).enumerate() {
        let hi = hex_nibble(chunk[0]);
        let lo = hex_nibble(chunk[1]);
        seed[i] = (hi << 4) | lo;
    }
    seed
}

fn hex_nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => panic!("envelope_hash produced non-lowercase-hex byte {c}"),
    }
}

/// One representative value per `Refusal` variant, in declaration order.
/// Mirrors `ALL_REFUSAL_NAMES` (29 names); the assertion suite checks the
/// name round-trip against the governed list.
pub fn all_refusal_examples() -> Vec<Refusal> {
    let ctx = || "ctx".to_string();
    vec![
        Refusal::ValidationFailed(ctx()),
        Refusal::PlanInfeasible(ctx()),
        Refusal::TraceUnlawful(ctx()),
        Refusal::HookUnpermitted(ctx()),
        Refusal::MissingReceipt(ctx()),
        Refusal::SnapshotNotFound(ctx()),
        Refusal::BoundaryRequestMissingReceipt(ctx()),
        Refusal::Triple8UniverseOverflow(ctx()),
        Refusal::TermNotInTriple8Universe(ctx()),
        Refusal::ProfileSymbolTableMismatch(ctx()),
        Refusal::ProjectionHashMismatch(ctx()),
        Refusal::WarmPathRequired(ctx()),
        Refusal::AdmissionTableMismatch(ctx()),
        Refusal::HookPatternNotAdmitted(ctx()),
        Refusal::OcelEventNotAdmitted(ctx()),
        Refusal::LeastExpressiveRouteViolation(ctx()),
        Refusal::UnsupportedDialect(ctx()),
        Refusal::N3UnavailableByProfile(ctx()),
        Refusal::N3ActuationRefused(ctx()),
        Refusal::RouteDecisionMismatch(ctx()),
        Refusal::GraphSnapshotMismatch(ctx()),
        Refusal::ProfileHashMismatch(ctx()),
        Refusal::AgentOverrideDenied(ctx()),
        Refusal::WitnessNotAuthority(ctx()),
        Refusal::BreedUnpermitted(ctx()),
        Refusal::NondeterministicOperatorRequiresReceipt(ctx()),
        Refusal::ProcessReceiptShadowType(ctx()),
        Refusal::DuplicateCanonicalTapeType(ctx()),
        Refusal::TripleTermInSnapshot(ctx()),
    ]
}

/// A small admission table: requires bit 0, forbids bit 7, sets bit 1 and
/// clears bit 2 on admission. Deterministic by construction.
pub fn small_admission_table() -> Result<AdmissionTable8, Refusal> {
    AdmissionTable8::from_masks(
        vec!["admitted".to_string(), "hot".to_string()],
        ConstraintMask(0b0000_0001), // required: bit 0
        ConstraintMask(0b1000_0000), // forbidden: bit 7
        ConstraintMask(0b0000_0010), // set on admit: bit 1
        ConstraintMask(0b0000_0100), // clear on admit: bit 2
    )
}

/// Sorted canonical N-Quads material for a synthetic 2-quad snapshot.
pub fn canonical_nquads() -> String {
    // Lines are pre-sorted: Receipt::from_canonical_nquads refuses unsorted
    // material by design.
    "<urn:a> <urn:p> <urn:b> <urn:g> .\n<urn:a> <urn:p> <urn:c> <urn:g> .".to_string()
}
