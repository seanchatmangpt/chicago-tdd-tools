//! Manufactured refusal catalog. Do not edit.
use super::types::RefusalDefinition;

pub const REFUSALS: &[RefusalDefinition] = &[
    RefusalDefinition { order: 1, code: "CMD-ADM-001", name: "OBSERVATION_NOT_ADMITTED", boundary: "admission" },
    RefusalDefinition { order: 2, code: "CMD-SRC-001", name: "SOURCE_CLOSURE_DRIFT", boundary: "source" },
    RefusalDefinition { order: 3, code: "CMD-FAC-001", name: "FACET_TOTALITY_DRIFT", boundary: "facet" },
    RefusalDefinition { order: 4, code: "CMD-REA-001", name: "REALIZATION_DRIFT", boundary: "realization" },
    RefusalDefinition { order: 5, code: "CMD-DEP-001", name: "DEPENDENCY_NOT_CLOSED", boundary: "dependency" },
    RefusalDefinition { order: 6, code: "CMD-PRF-001", name: "PROFILE_NOT_TOTAL", boundary: "profile" },
    RefusalDefinition { order: 7, code: "CMD-EXT-001", name: "EXTERNAL_CONTRACT_DRIFT", boundary: "external" },
    RefusalDefinition { order: 8, code: "CMD-AUT-001", name: "AUTHORITY_LEAK", boundary: "authority" },
    RefusalDefinition { order: 9, code: "CMD-PRJ-001", name: "PROJECTION_LATTICE_DRIFT", boundary: "projection" },
    RefusalDefinition { order: 10, code: "CMD-EVD-001", name: "EVIDENCE_OBLIGATION_DRIFT", boundary: "evidence" },
    RefusalDefinition { order: 11, code: "CMD-GEN-001", name: "GENERATED_OUTPUT_DRIFT", boundary: "generation" },
    RefusalDefinition { order: 12, code: "CMD-GEN-002", name: "AUTHORED_GENERATED_OVERLAP", boundary: "generation" },
    RefusalDefinition { order: 13, code: "CMD-ACT-001", name: "DIRECT_ACTUATION_REFUSED", boundary: "actuation" },
    RefusalDefinition { order: 14, code: "CMD-PIN-001", name: "EXECUTION_PIN_DRIFT", boundary: "resolution" },
    RefusalDefinition { order: 15, code: "CMD-RCP-001", name: "RECEIPT_COMPONENT_MISSING", boundary: "receipt" },
    RefusalDefinition { order: 16, code: "CMD-RCP-002", name: "EXACT_HEAD_MISMATCH", boundary: "receiver" },
    RefusalDefinition { order: 17, code: "CMD-STD-001", name: "PREMATURE_STANDING", boundary: "standing" },
];
