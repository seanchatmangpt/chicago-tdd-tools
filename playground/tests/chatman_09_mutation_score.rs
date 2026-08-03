//! Capability 09 — in-crate mutation testing (`MutationTester`,
//! `MutationScore`) on a small admission predicate.
//!
//! The subject under mutation is a map-encoded admission configuration
//! (required/forbidden masks + a state). The "test suite" being scored is a
//! checker that re-derives the expected admission decision through the real
//! `AdmissionTable8` and compares — mutants that change the decision must be
//! caught.

mod chatman_common;

use std::collections::HashMap;

use chicago_tdd_tools::test;
use chicago_tdd_tools::testing::mutation::{MutationOperator, MutationScore, MutationTester};
use praxis_graphlaw::chatman::admission8::{AdmissionTable8, ConstraintMask};

/// The admission predicate under test, over a map-encoded scenario:
/// keys "required", "forbidden", "state" hold u8 values as decimal strings.
fn admission_predicate(data: &HashMap<String, String>) -> Option<bool> {
    let get = |k: &str| data.get(k)?.parse::<u8>().ok();
    let (required, forbidden, state) = (get("required")?, get("forbidden")?, get("state")?);
    let table = AdmissionTable8::from_masks(
        vec!["c".to_string()],
        ConstraintMask(required),
        ConstraintMask(forbidden),
        ConstraintMask(0),
        ConstraintMask(0),
    )
    .ok()?;
    Some(table.lookup(state).admit)
}

/// Baseline scenario: state 0b0000_0011 with required 0b01, forbidden 0b1000_0000
/// -> admitted.
fn baseline() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("required".to_string(), "1".to_string());
    m.insert("forbidden".to_string(), "128".to_string());
    m.insert("state".to_string(), "3".to_string());
    m
}

test!(mutation_score_on_admission_predicate, {
    // Arrange: the expected decision on unmutated data, and a mutant set
    // chosen so every mutant flips or destroys the decision.
    let expected = admission_predicate(&baseline());
    assert_eq!(expected, Some(true), "baseline scenario must admit");

    let mutations = vec![
        // state loses the required bit -> decision flips to refused.
        MutationOperator::ChangeValue("state".to_string(), "2".to_string()),
        // state gains the forbidden bit -> decision flips to refused.
        MutationOperator::ChangeValue("state".to_string(), "131".to_string()),
        // required becomes unsatisfiable for state 3 -> refused.
        MutationOperator::ChangeValue("required".to_string(), "4".to_string()),
        // the state key disappears -> predicate cannot even evaluate.
        MutationOperator::RemoveKey("state".to_string()),
        // numeric drift on the state -> loses bit 0 (3 - 1 = 2) -> refused.
        MutationOperator::NumericDelta("state".to_string(), -1),
    ];
    let total = mutations.len();

    // Act: apply each mutant and count the ones the checker catches.
    let mut tester = MutationTester::new(baseline());
    let mut caught = 0usize;
    for mutation in mutations {
        let mutated = tester.apply_mutation(mutation);
        if admission_predicate(&mutated) != expected {
            caught += 1;
        }
    }

    // Assert: a checker this tight catches every mutant (score 100%).
    let score = MutationScore::calculate(caught, total);
    assert!(
        score.is_acceptable(),
        "mutation score below threshold: caught {caught}/{total}"
    );
    assert_eq!(caught, total, "every mutant must flip the admission decision");
});

test!(surviving_mutant_is_visible_in_the_score, {
    // Arrange: one deliberately equivalent mutant — changing the forbidden
    // mask to another value that still doesn't intersect state 3 leaves the
    // decision unchanged, so the checker cannot catch it.
    let expected = admission_predicate(&baseline());
    let mut tester = MutationTester::new(baseline());

    // Act
    let mutated =
        tester.apply_mutation(MutationOperator::ChangeValue("forbidden".to_string(), "64".to_string()));
    let survived = admission_predicate(&mutated) == expected;

    // Assert: the mutant survives, and the score reports it honestly.
    assert!(survived, "equivalent mutant must survive by construction");
    let score = MutationScore::calculate(0, 1);
    assert!(!score.is_acceptable(), "0/1 must not be an acceptable score");
});
