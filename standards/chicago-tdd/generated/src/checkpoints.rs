//! Manufactured from ontology.ttl. Do not edit.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GallCheckpoint {
    pub order: u16,
    pub id: &'static str,
    pub title: &'static str,
    pub phase: &'static str,
    pub depends_on: &'static str,
    pub standing: &'static str,
    pub target_state: &'static str,
    pub receipt_required: bool,
    pub evidence: &'static str,
    pub falsifier: &'static str,
    pub description: &'static str,
}

pub const GALL_CHECKPOINTS: &[GallCheckpoint] = &[
    GallCheckpoint { order: 13, id: "GALL-013", title: "Observation provenance closure", phase: "admit", depends_on: "GALL-012", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "O* identity and the seven-day source window are exact.", falsifier: "Mutate the admitted source window or observation identity.", description: "The admitted observation carries an exact identity and bounded seven-day provenance window." },
    GallCheckpoint { order: 14, id: "GALL-014", title: "Checkpoint totality closure", phase: "admit", depends_on: "GALL-013", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "Exactly ten contiguous checkpoints GALL-013 through GALL-022 exist.", falsifier: "Delete, duplicate, or renumber a checkpoint.", description: "The next Gall checkpoint set is complete, unique, and contiguous." },
    GallCheckpoint { order: 15, id: "GALL-015", title: "Checkpoint dependency closure", phase: "route", depends_on: "GALL-014", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "Each checkpoint depends only on its immediate predecessor.", falsifier: "Redirect a checkpoint dependency to a non-predecessor.", description: "The checkpoint graph is a single acyclic dependency chain." },
    GallCheckpoint { order: 16, id: "GALL-016", title: "State and crown separation", phase: "diagnose", depends_on: "GALL-015", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "All checkpoint plans remain ADMITTED and target only an observed ALIVE state.", falsifier: "Promote authored standing to ALIVE or equate checkpoint completion with crown completion.", description: "Authored plans never overclaim execution standing or crown completion." },
    GallCheckpoint { order: 17, id: "GALL-017", title: "Evidence and falsifier closure", phase: "verify", depends_on: "GALL-016", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "Every checkpoint declares unique evidence and falsifier recipes plus a receipt requirement.", falsifier: "Remove or duplicate an evidence or falsifier recipe.", description: "Every checkpoint has an executable proof obligation and a negative witness." },
    GallCheckpoint { order: 18, id: "GALL-018", title: "Toolchain pin closure", phase: "resolve", depends_on: "GALL-017", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "O* pins ggen, LSP, wasm4pm dependencies, and the Rust toolchain to exact revisions.", falsifier: "Change an admitted pin without changing the execution rail.", description: "The manufacturing toolchain is exact and cross-checked against the real ggen runner." },
    GallCheckpoint { order: 19, id: "GALL-019", title: "Authored/generated disjointness", phase: "render", depends_on: "GALL-018", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "All outputs remain under generated/ and no authored input is an output.", falsifier: "Route a generated output over an authored source or outside the generated root.", description: "Authored law and generated products are path-disjoint." },
    GallCheckpoint { order: 20, id: "GALL-020", title: "Exact-head receiver binding", phase: "receipt", depends_on: "GALL-019", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "Receiver mode requires one exact 40-hex commit identity.", falsifier: "Use LOCAL, an empty value, or a malformed candidate head.", description: "Published receiver evidence is bound to exactly one commit." },
    GallCheckpoint { order: 21, id: "GALL-021", title: "OCEL checkpoint evidence", phase: "observe", depends_on: "GALL-020", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "Emit one deterministic OCEL 2.0 object and event for every checkpoint.", falsifier: "Change the admitted OCEL schema or event/object closure.", description: "Checkpoint execution is represented as replayable object-centric process evidence." },
    GallCheckpoint { order: 22, id: "GALL-022", title: "Receipt replay closure", phase: "replay", depends_on: "GALL-021", standing: "ADMITTED", target_state: "ALIVE", receipt_required: true, evidence: "Two receipt computations over identical admitted inputs are byte-identical.", falsifier: "Inject nondeterminism into the receipt composite.", description: "The complete checkpoint evidence envelope is deterministic under replay." },
];
