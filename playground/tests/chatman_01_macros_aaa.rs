//! Capability 01 — AAA test macros (`test!`, `fixture_test!`, `async_test!`)
//! exercised against the Chatman ABI: Refusal construction, name()
//! round-trips, and envelope hashing determinism.
//!
//! Playground style: every test body is explicitly Arrange / Act / Assert.

mod chatman_common;

use chatman_common::{all_refusal_examples, envelope};
use chicago_tdd_tools::{async_test, fixture_test, test};
use praxis_graphlaw::chatman::abi::{InvocationEnvelope, Refusal, ALL_REFUSAL_NAMES};

test!(refusal_construction_carries_context, {
    // Arrange: a refusal naming its concrete offender.
    let refusal = Refusal::SnapshotNotFound("snap:missing-42".to_string());

    // Act: render through the thiserror Display impl.
    let rendered = refusal.to_string();

    // Assert: both the taxonomy prefix and the offender survive.
    assert!(rendered.contains("snapshot not found"), "got {rendered:?}");
    assert!(rendered.contains("snap:missing-42"), "got {rendered:?}");
});

test!(refusal_name_round_trips_against_governed_list, {
    // Arrange: one example per variant, in declaration order.
    let examples = all_refusal_examples();

    // Act: map each to its static name.
    let names: Vec<&'static str> = examples.iter().map(Refusal::name).collect();

    // Assert: exactly the 29-name governed contract, in order.
    assert_eq!(names.len(), ALL_REFUSAL_NAMES.len());
    assert_eq!(names.as_slice(), ALL_REFUSAL_NAMES.as_slice());
});

test!(envelope_hash_is_deterministic_across_runs, {
    // Arrange: the same logical envelope built twice.
    let a = envelope("determinism");
    let b = envelope("determinism");

    // Act
    let (ha, hb) = (a.envelope_hash(), b.envelope_hash());

    // Assert: byte-identical, 64 lowercase hex chars.
    assert_eq!(ha, hb);
    assert_eq!(ha.len(), 64);
    assert!(ha.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
});

test!(envelope_hash_ignores_handle_permutation, {
    // Arrange: two envelopes differing only in handle vector order.
    let mut a = envelope("permute");
    let mut b = envelope("permute");
    a.input_handles.nodes = vec!["<urn:x>".into(), "<urn:y>".into()];
    b.input_handles.nodes = vec!["<urn:y>".into(), "<urn:x>".into()];

    // Act + Assert: handle order is not semantic (ABI contract).
    assert_eq!(a.envelope_hash(), b.envelope_hash());
});

test!(envelope_hash_distinguishes_identities, {
    // Arrange: envelopes differing in exactly one identity field.
    let a = envelope("alpha");
    let b = InvocationEnvelope {
        operator_id: praxis_graphlaw::chatman::abi::OperatorId::new("op:other"),
        ..envelope("alpha")
    };

    // Act + Assert: a single scalar identity change changes the digest.
    assert_ne!(a.envelope_hash(), b.envelope_hash());
});

// fixture_test! provides a ready TestFixture (tokio-backed, 1s timeout).
fixture_test!(fixture_test_macro_works_with_chatman_types, fixture, {
    // Arrange: fixture counter proves fixture creation; envelope under test.
    let counter = fixture.test_counter();
    let env = envelope("fixture");

    // Act
    let hash = env.envelope_hash();

    // Assert
    assert!(counter < u64::MAX);
    assert_eq!(hash, envelope("fixture").envelope_hash());
});

// async_test! wraps a tokio runtime with a 1s timeout.
async_test!(async_test_macro_hashes_envelope, {
    // Arrange
    let env = envelope("async");

    // Act: hashing is pure; awaiting a ready future exercises the async path.
    let hash = std::future::ready(env.envelope_hash()).await;

    // Assert
    assert_eq!(hash.len(), 64);
});
