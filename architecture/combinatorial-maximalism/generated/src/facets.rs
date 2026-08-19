//! Manufactured facet registry. Do not edit.
use super::types::Facet;

pub const FACETS: &[Facet] = &[
    Facet { order: 1, id: "facet.observation", title: "Observation", phase: "observe", description: "Admit exact observations and environment evidence before composition." },
    Facet { order: 2, id: "facet.admission", title: "Admission", phase: "admit", description: "Convert observations into bounded admitted facts and constraints." },
    Facet { order: 3, id: "facet.construction", title: "Construction", phase: "construct", description: "Explore reversible candidate combinations in the graph domain." },
    Facet { order: 4, id: "facet.verification", title: "Verification", phase: "verify", description: "Prove state and consequence across the verification ladder." },
    Facet { order: 5, id: "facet.authorization", title: "Authorization", phase: "authorize", description: "Separate verified candidates from explicit execution grants." },
    Facet { order: 6, id: "facet.actuation", title: "Actuation", phase: "actuate", description: "Route every side effect through a bounded broker intent." },
    Facet { order: 7, id: "facet.receipt", title: "Receipt", phase: "receipt", description: "Bind observed consequences to causal BLAKE3 receipts." },
    Facet { order: 8, id: "facet.replay", title: "Replay", phase: "replay", description: "Independently reproduce or compare consequences from receipts." },
];
