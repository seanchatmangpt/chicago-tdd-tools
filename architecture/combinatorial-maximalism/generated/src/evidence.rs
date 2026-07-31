//! Manufactured evidence obligations. Do not edit.
use super::types::EvidenceObligation;

pub const EVIDENCE_OBLIGATIONS: &[EvidenceObligation] = &[
    EvidenceObligation { order: 1, id: "evidence.observation-hash", title: "Observation hash", required: true },
    EvidenceObligation { order: 2, id: "evidence.source-closure", title: "Source closure", required: true },
    EvidenceObligation { order: 3, id: "evidence.profile-totality", title: "Profile totality", required: true },
    EvidenceObligation { order: 4, id: "evidence.dependency-closure", title: "Dependency closure", required: true },
    EvidenceObligation { order: 5, id: "evidence.authority-fence", title: "Authority fence", required: true },
    EvidenceObligation { order: 6, id: "evidence.generated-identity", title: "Generated identity", required: true },
    EvidenceObligation { order: 7, id: "evidence.external-contract", title: "External contract", required: true },
    EvidenceObligation { order: 8, id: "evidence.receipt-v2", title: "Receipt v2", required: true },
    EvidenceObligation { order: 9, id: "evidence.exact-head", title: "Exact head", required: true },
    EvidenceObligation { order: 10, id: "evidence.replay-identity", title: "Replay identity", required: true },
    EvidenceObligation { order: 11, id: "evidence.ocel-conformance", title: "OCEL conformance", required: true },
    EvidenceObligation { order: 12, id: "evidence.verifier-report", title: "Verifier report", required: true },
];
