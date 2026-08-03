//! Capability 02 — the CTT assertion family on Chatman types:
//! `assert_ok!` / `assert_err!` / `assert_fail!` (typed payload),
//! `assert_matches!` with guards on Refusal variants,
//! `assert_contains!` / `assert_subset!` on refusal-name sets vs the
//! 29-name governed list, and `assert_eq_msg!`.

mod chatman_common;

use chatman_common::{all_refusal_examples, canonical_nquads, small_admission_table};
use chicago_tdd_tools::{
    assert_contains, assert_eq_msg, assert_err, assert_fail, assert_matches, assert_ok,
    assert_subset, test,
};
use praxis_graphlaw::chatman::abi::{Receipt, Refusal, ALL_REFUSAL_NAMES};

test!(assert_ok_and_err_on_receipt_construction, {
    // Arrange: sorted vs unsorted canonical material.
    let sorted = canonical_nquads();
    let unsorted = "<urn:z> <urn:p> <urn:o> .\n<urn:a> <urn:p> <urn:o> .";

    // Act
    let good = Receipt::from_canonical_nquads("subj", "witness", "replay", &sorted);
    let bad = Receipt::from_canonical_nquads("subj", "witness", "replay", unsorted);

    // Assert
    assert_ok!(&good);
    assert_err!(&bad);
});

test!(assert_fail_yields_typed_refusal_payload, {
    // Arrange: empty material must refuse with MissingReceipt.
    let result = Receipt::from_canonical_nquads("subj", "witness", "replay", "");

    // Act: assert_fail! returns the typed error payload for inspection.
    let refusal: Refusal = assert_fail!(result, "empty material must refuse");

    // Assert: the payload is the specific taxonomy variant, not a string.
    assert_eq_msg!(refusal.name(), "MissingReceipt", "refusal variant name");
});

test!(assert_matches_with_guard_on_refusal_variants, {
    // Arrange: an admission table refusing state 0 (bit 0 required, unset).
    let table = small_admission_table()?;

    // Act
    let refused = table.admit(0b0000_0000);

    // Assert: pattern match with a guard on the context payload.
    assert_matches!(
        &refused,
        Err(Refusal::HookPatternNotAdmitted(msg)) if msg.contains("0b00000000"),
        "refusal must name the offending state in binary"
    );
    Ok::<(), Refusal>(())
});

test!(refusal_names_are_subset_of_governed_list, {
    // Arrange: names of every constructible variant.
    let names: Vec<&'static str> = all_refusal_examples().iter().map(Refusal::name).collect();

    // Act + Assert: exhaustive coverage in both directions.
    assert_subset!(names, ALL_REFUSAL_NAMES, "every name must be governed");
    assert_subset!(ALL_REFUSAL_NAMES, names, "every governed name must be constructible");
    assert_eq_msg!(names.len(), 29usize, "the taxonomy is a 29-variant contract");
});

test!(assert_contains_on_governed_refusal_names, {
    // Arrange: the governed 29-name list.
    let governed = ALL_REFUSAL_NAMES;

    // Act + Assert: constitutionally load-bearing names are present.
    assert_contains!(governed, "N3ActuationRefused");
    assert_contains!(governed, "WarmPathRequired");
    assert_contains!(governed, "Triple8UniverseOverflow");
    assert_contains!(governed, "WitnessNotAuthority", "witnesses attest, never authorize");
});

test!(receipt_verify_detects_tampered_material, {
    // Arrange: a valid receipt whose canonical material is then mutated.
    let receipt = Receipt::from_canonical_nquads("subj", "wit", "replay", &canonical_nquads())?;
    let mut tampered = receipt.clone();
    tampered.canon_nquads.push_str("\n<urn:evil> <urn:p> <urn:o> <urn:g> .");

    // Act
    let ok = receipt.verify();
    let bad = tampered.verify();

    // Assert: identity is computed, never trusted.
    assert_ok!(&ok);
    assert_matches!(&bad, Err(Refusal::ValidationFailed(msg)) if msg.contains("mismatch"));
    Ok::<(), Refusal>(())
});
