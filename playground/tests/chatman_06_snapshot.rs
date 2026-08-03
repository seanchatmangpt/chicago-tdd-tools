//! Capability 06 — snapshot testing (`SnapshotAssert`, insta-backed):
//! pins the 29-name Refusal contract and the serialized envelope JSON shape.
//!
//! First run: execute with `INSTA_UPDATE=always cargo test --features chatman`
//! (or `cargo insta review`) to accept the snapshots under tests/snapshots/.
//! Thereafter any drift in the refusal taxonomy or the envelope wire shape
//! fails the suite — that is the point.

mod chatman_common;

use chatman_common::envelope;
use chicago_tdd_tools::test;
use chicago_tdd_tools::testing::snapshot::SnapshotAssert;
use praxis_graphlaw::chatman::abi::ALL_REFUSAL_NAMES;

test!(snapshot_pins_the_29_refusal_name_contract, {
    // Arrange: the governed list, one name per line (stable, ordered).
    let contract = ALL_REFUSAL_NAMES.join("\n");

    // Act + Assert: any added/removed/renamed variant breaks this snapshot.
    SnapshotAssert::assert_matches(&contract, "chatman_refusal_names_v26_7_9");
});

test!(snapshot_pins_envelope_json_wire_shape, {
    // Arrange: a fully deterministic envelope (no wall clock, no randomness).
    let env = envelope("snapshot");

    // Act: serialize through serde — this is the wire shape consumers parse.
    let json = serde_json::to_value(&env).map_err(std::io::Error::other)?;

    // Assert: field names, nesting, and id transparency are all pinned.
    SnapshotAssert::assert_json_matches(&json, "chatman_envelope_json_v26_7_9");
    Ok::<(), std::io::Error>(())
});

test!(snapshot_pins_envelope_hash_value, {
    // Arrange + Act: the canonical digest of a fixed envelope is itself a
    // regression surface — hash-scheme drift must be a loud, reviewed event.
    let hash = envelope("snapshot").envelope_hash();

    // Assert
    SnapshotAssert::assert_matches(&hash, "chatman_envelope_hash_v26_7_9");
});
