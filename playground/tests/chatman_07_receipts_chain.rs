//! Capability 07 — BLAKE3 receipt-chain validation:
//! `ReceiptChainBuilder` for synthetic 3-entry chains, `Blake3ChainValidator`
//! round-trip + tamper detection, and the bridge to the Chatman ABI's own
//! `Receipt` (computed, never asserted).

mod chatman_common;

use chatman_common::canonical_nquads;
use chicago_tdd_tools::observability::receipt::{
    Blake3ChainValidator, ChainError, ReceiptChainBuilder,
};
use chicago_tdd_tools::test;
use praxis_graphlaw::chatman::abi::{Receipt, Refusal};

test!(three_entry_chain_round_trips, {
    // Arrange: a synthetic 3-entry chain; hashes computed by the builder.
    let entries = ReceiptChainBuilder::new()
        .add_entry(1, 0b111, 0)
        .add_entry(2, 0b011, 0)
        .add_entry(3, 0b001, 1)
        .build();

    // Act
    let validated = Blake3ChainValidator::validate_chain(&entries);

    // Assert: forward-chained BLAKE3 verifies end to end.
    assert_eq!(entries.len(), 3);
    assert!(validated.is_ok(), "got {validated:?}");
    Blake3ChainValidator::assert_chain_valid(&entries);
});

test!(tampering_with_middle_entry_breaks_the_chain, {
    // Arrange: valid chain, then flip one content byte in entry 1.
    let mut entries = ReceiptChainBuilder::new()
        .add_entry(1, 0b111, 0)
        .add_entry(2, 0b011, 0)
        .add_entry(3, 0b001, 0)
        .build();
    entries[1].op_trace_le[0] ^= 0xFF;

    // Act
    let validated = Blake3ChainValidator::validate_chain(&entries);

    // Assert: the recomputed hash disagrees with the stored hash.
    assert!(
        matches!(validated, Err(ChainError::HashMismatch { index: 1, .. })),
        "expected HashMismatch at index 1, got {validated:?}"
    );
});

test!(framework_tamper_evidence_helper_agrees, {
    // Arrange
    let entries = ReceiptChainBuilder::new()
        .add_entry(10, 0b101, 0)
        .add_entry(11, 0b110, 0)
        .add_entry(12, 0b010, 0)
        .build();

    // Act + Assert: the framework's own mutate-and-check harness passes.
    Blake3ChainValidator::assert_tamper_evident(&entries);
});

test!(chatman_receipt_digest_is_computed_and_tamper_evident, {
    // Arrange: the ABI Receipt is the same discipline at the N-Quads level.
    let receipt = Receipt::from_canonical_nquads(
        "urn:chatman:invocation:7",
        "urn:witness:ci",
        "replay:snapshot:7",
        &canonical_nquads(),
    )?;

    // Act: verification recomputes the digest from the canonical material.
    receipt.verify()?;

    // Assert: mutating material after construction is detected on verify.
    let mut tampered = receipt.clone();
    tampered.canon_nquads = tampered.canon_nquads.replace("<urn:a>", "<urn:x>");
    assert!(
        matches!(tampered.verify(), Err(Refusal::ValidationFailed(_))),
        "tampered material must fail digest recomputation"
    );
    Ok::<(), Refusal>(())
});
