use chicago_tdd_standards_generated::{GALL_STATES, MANUFACTURING_LIFECYCLE, REFUSALS, STANDARDS};
use std::collections::BTreeSet;

#[test]
fn ontology_projection_is_total_and_unique() {
    assert_eq!(STANDARDS.len(), 20);
    assert_eq!(REFUSALS.len(), 12);
    assert_eq!(STANDARDS.iter().map(|row| row.id).collect::<BTreeSet<_>>().len(), STANDARDS.len());
    assert_eq!(REFUSALS.iter().map(|row| row.code).collect::<BTreeSet<_>>().len(), REFUSALS.len());
}

#[test]
fn standing_and_receipt_law_is_closed() {
    assert!(STANDARDS.iter().all(|row| row.standing == "ADMITTED"));
    assert!(STANDARDS.iter().all(|row| row.receipt_required));
    assert!(REFUSALS.iter().any(|row| row.code == "CTDD-ACT-002"));
    assert!(REFUSALS.iter().any(|row| row.code == "CTDD-GEN-003"));
}

#[test]
fn lifecycle_and_gall_state_vocabularies_are_exact() {
    assert_eq!(MANUFACTURING_LIFECYCLE, ["Resolve", "Enrich", "Extract", "Render", "Write", "Receipt"]);
    assert_eq!(GALL_STATES, ["PARTIAL_ALIVE", "ALIVE", "BLOCKED", "BUILD_BROKEN", "UNKNOWN", "UNSUPPORTED"]);
}
