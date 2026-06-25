//! Universal BLAKE3 receipt chain validation tests.
//!
//! These tests validate the BLAKE3 receipt chain protocol used by bcinr-powl
//! and any other project that claims BLAKE3 chaining. The doctrine is:
//!
//! > When any project says "BLAKE3 receipts can be replayed", these tests
//! > must prove it — not the project's own unit tests.
//!
//! Run with:
//!   cargo test --test blake3_receipt_tests --features receipt-validation

#![cfg(feature = "receipt-validation")]

use chicago_tdd_tools::observability::receipt::{
    Blake3ChainValidator, Blake3ReceiptEntry, ChainError, ReceiptChainBuilder, RawReceiptEntry,
};

// ─── test 1: synthetic chain round-trips ─────────────────────────────────────

/// Build a 4-entry chain with ReceiptChainBuilder and assert that
/// Blake3ChainValidator replays it without error.
///
/// This is the canonical BLAKE3 replay proof: the same inputs produce the same
/// chain hashes, deterministically, on every run.
#[test]
fn blake3_chain_builder_round_trips() {
    let entries = ReceiptChainBuilder::new()
        .add_entry(1, 0b0001, 0x00)
        .add_entry(2, 0b0011, 0x00)
        .add_entry(3, 0b0111, 0x01)
        .add_entry(4, 0b1111, 0x81)
        .build();

    assert_eq!(entries.len(), 4);
    Blake3ChainValidator::assert_chain_valid(&entries);
}

// ─── test 2: tamper evidence ──────────────────────────────────────────────────

/// A valid 3-entry chain is tamper-evident: the validator confirms that
/// each entry's prev_hash links to the prior entry's stored_hash.
#[test]
fn blake3_chain_tamper_evident() {
    let entries = ReceiptChainBuilder::new()
        .add_entry(10, 0xFF00FF00, 0x00)
        .add_entry(11, 0x00FF00FF, 0x00)
        .add_entry(12, 0xAAAAAAAA, 0x80)
        .build();

    Blake3ChainValidator::assert_tamper_evident(&entries);
}

// ─── test 3: mutation breaks the chain ───────────────────────────────────────

/// Mutate a field in entry[1] and confirm that validate_chain returns
/// HashMismatch. This is the "tamper-detection" test: any byte mutation
/// must be detectable by the validator.
#[test]
fn blake3_chain_mutation_detected() {
    let mut entries = ReceiptChainBuilder::new()
        .add_entry(1, 100, 0x00)
        .add_entry(2, 200, 0x00)
        .add_entry(3, 300, 0x00)
        .build();

    // Corrupt entry[1]'s op_trace byte — flip bit 0.
    entries[1].op_trace_le[0] ^= 0x01;

    let result = Blake3ChainValidator::validate_chain(&entries);
    assert!(
        matches!(result, Err(ChainError::HashMismatch { index: 1, .. })),
        "mutation at entry[1] must produce HashMismatch at index 1, got: {result:?}"
    );
}

// ─── test 4: mutation to entry[0] breaks all downstream entries ──────────────

/// Mutating entry[0]'s content must produce a HashMismatch at entry[0],
/// because the stored_hash no longer matches the recomputed hash.
#[test]
fn blake3_chain_entry0_mutation_breaks_chain() {
    let mut entries = ReceiptChainBuilder::new()
        .add_entry(1, 0xDEAD, 0x00)
        .add_entry(2, 0xBEEF, 0x00)
        .add_entry(3, 0xCAFE, 0x00)
        .build();

    // Corrupt entry[0]'s topo_tag.
    entries[0].topo_tag ^= 0xFF;

    let result = Blake3ChainValidator::validate_chain(&entries);
    assert!(
        matches!(result, Err(ChainError::HashMismatch { index: 0, .. })),
        "mutation at entry[0] must be caught at index 0"
    );
}

// ─── test 5: prev_hash mismatch is detected ──────────────────────────────────

/// Manually construct an entry with a wrong prev_hash and confirm
/// PrevHashMismatch is returned before the hash computation.
#[test]
fn blake3_chain_prev_hash_mismatch_detected() {
    let mut entries = ReceiptChainBuilder::new()
        .add_entry(1, 1, 0x00)
        .add_entry(2, 2, 0x00)
        .build();

    // Corrupt entry[1]'s prev field.
    entries[1].prev[0] ^= 0x01;

    let result = Blake3ChainValidator::validate_chain(&entries);
    assert!(
        matches!(result, Err(ChainError::PrevHashMismatch { index: 1, .. })),
        "bad prev_hash must produce PrevHashMismatch at index 1"
    );
}

// ─── test 6: empty chain is rejected ─────────────────────────────────────────

#[test]
fn blake3_chain_empty_rejected() {
    let entries: Vec<RawReceiptEntry> = vec![];
    let result = Blake3ChainValidator::validate_chain(&entries);
    assert_eq!(result, Err(ChainError::Empty));
}

// ─── test 7: single entry chain is valid ─────────────────────────────────────

#[test]
fn blake3_chain_single_entry_valid() {
    let entries = ReceiptChainBuilder::new()
        .add_entry(42, 0xFFFFFFFF, 0x01)
        .build();
    Blake3ChainValidator::assert_chain_valid(&entries);
}

// ─── test 8: replay determinism — same inputs, same hashes ───────────────────

/// BLAKE3 is deterministic: replaying the same events on two independent
/// builders produces identical chain hashes. This is the "replay" property
/// the doctrine requires.
#[test]
fn blake3_replay_is_deterministic() {
    let build = || {
        ReceiptChainBuilder::new()
            .add_entry(1, 0xABCD, 0x00)
            .add_entry(2, 0xEF01, 0x01)
            .add_entry(3, 0x2345, 0x00)
            .build()
    };

    let chain_a = build();
    let chain_b = build();

    assert_eq!(
        chain_a.len(),
        chain_b.len(),
        "replayed chains must have same length"
    );
    for i in 0..chain_a.len() {
        assert_eq!(
            chain_a[i].stored_hash(),
            chain_b[i].stored_hash(),
            "entry {i}: BLAKE3 chain hashes must be identical on replay"
        );
    }
}

// ─── test 9: bcinr-powl 57-byte format round-trip ────────────────────────────

/// Validate the bcinr-powl specific 57-byte entry format using
/// RawReceiptEntry::from_bcinr_powl and chain_from_raw_entries.
///
/// This test constructs synthetic 57-byte entries matching the bcinr-powl
/// layout:
///   [0..8]   run_id LE
///   [8..16]  op_trace LE
///   [16]     topo_tag
///   [17..49] chain_hash
///   [49..57] replay_ptr LE (not part of the hash — ignored here)
///
/// The hash law: BLAKE3(prev_chain_hash ‖ run_id_le ‖ op_trace_le ‖ topo_tag)
#[test]
fn bcinr_powl_57byte_format_round_trips() {
    // Build valid 57-byte entries by computing BLAKE3 chains manually.
    const ENTRY_BYTES: usize = 57;
    let mut raw_entries: Vec<[u8; ENTRY_BYTES]> = Vec::new();
    let mut prev_hash = [0u8; 32];

    for i in 0u64..3 {
        let run_id_le = (i + 1).to_le_bytes();
        let op_trace_le = ((i + 1) * 0b111).to_le_bytes();
        let topo_tag = i as u8;
        let replay_ptr: u64 = i * ENTRY_BYTES as u64;

        let mut h = blake3::Hasher::new();
        h.update(&prev_hash);
        h.update(&run_id_le);
        h.update(&op_trace_le);
        h.update(&[topo_tag]);
        let chain_hash: [u8; 32] = *h.finalize().as_bytes();

        let mut entry = [0u8; ENTRY_BYTES];
        entry[0..8].copy_from_slice(&run_id_le);
        entry[8..16].copy_from_slice(&op_trace_le);
        entry[16] = topo_tag;
        entry[17..49].copy_from_slice(&chain_hash);
        entry[49..57].copy_from_slice(&replay_ptr.to_le_bytes());

        prev_hash = chain_hash;
        raw_entries.push(entry);
    }

    // Parse via chain_from_raw_entries.
    let entries =
        RawReceiptEntry::chain_from_raw_entries(raw_entries.iter());

    assert_eq!(entries.len(), 3);
    Blake3ChainValidator::assert_tamper_evident(&entries);
}

// ─── test 10: overflow flag in topo_tag is preserved ─────────────────────────

/// Verify that entries with the overflow flag (bit 7 of topo_tag) still
/// validate correctly — the full byte is hashed, not just bits 0..6.
#[test]
fn blake3_chain_overflow_flag_preserved_in_hash() {
    let entries_no_overflow = ReceiptChainBuilder::new()
        .add_entry(1, 0xDEAD, 0x00) // no overflow
        .add_entry(2, 0xBEEF, 0x00)
        .build();

    let entries_with_overflow = ReceiptChainBuilder::new()
        .add_entry(1, 0xDEAD, 0x80) // overflow flag set
        .add_entry(2, 0xBEEF, 0x00)
        .build();

    // Both chains are internally valid.
    Blake3ChainValidator::assert_chain_valid(&entries_no_overflow);
    Blake3ChainValidator::assert_chain_valid(&entries_with_overflow);

    // But the hashes differ because topo_tag differs.
    assert_ne!(
        entries_no_overflow[0].stored_hash(),
        entries_with_overflow[0].stored_hash(),
        "overflow flag in topo_tag must change the BLAKE3 hash"
    );
}
