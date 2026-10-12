#![cfg(all(feature = "wasm-guest", feature = "ledger"))]
//! Guest-side receipt sealing WITHOUT ML-DSA signing.
//!
//! Division of authority: the guest **produces**, the host **seals**.
//!
//! * The guest WASM module builds a typed [`CourtReceiptV1`] from its replay
//!   summary, encodes the 352-byte chained record, and hands those bytes
//!   across the boundary.
//! * The HOST re-decodes with
//!   [`crate::ledger::receipt::VerifiedCourtReceipt::from_bytes_verified`]
//!   and applies the ML-DSA-65 signature host-side (keys live in
//!   `src/ledger/signing/keys.rs`, which uses `std::fs` and can therefore
//!   never run inside the guest).
//!
//! Constraints enforced by construction in this module: no `std::fs`, no
//! wall-clock time — every field is derived from guest-visible inputs, so two
//! identical replays produce byte-identical wire records.

use crate::ledger::receipt::{
    ChainedCourtReceipt, CourtOutcome, CourtReceiptV1, ReceiptMetrics, ThermalClass,
    COURT_RECORD_LEN,
};
use blake3::Hasher;

/// Fixed receipt name carried by every guest-produced receipt
/// (well under [`NAME_MAX`]; NUL padding is applied by [`CourtReceiptV1::encode`]).
const GUEST_RECEIPT_NAME: &str = "ctt-guest-v1";

/// ABI outcome byte → court outcome.
///
/// The guest ABI tags are the [`CourtOutcome`] wire tags (0=Pass, 1=Fail,
/// 2=Skip, 3=Error, 4=Refused). Any out-of-range byte fails closed to
/// [`CourtOutcome::Error`]: the guest never silently converts an unknown
/// verdict into `Pass`.
#[must_use]
pub const fn outcome_from_abi(byte: u8) -> CourtOutcome {
    // JUSTIFICATION (clippy::match_same_arms): the identical arms are the
    // POINT — 3 (Error) and the wildcard both fail closed to `Error`: the
    // explicit tag row documents the wire tag, the wildcard is the
    // fail-closed catch-all for any out-of-range byte.
    #[allow(clippy::match_same_arms)]
    match byte {
        0 => CourtOutcome::Pass,
        1 => CourtOutcome::Fail,
        2 => CourtOutcome::Skip,
        3 => CourtOutcome::Error,
        4 => CourtOutcome::Refused,
        _ => CourtOutcome::Error,
    }
}

/// Builds a guest-produced court receipt from the replay summary.
///
/// Metrics mapping (all guest-visible, no wall clock):
///
/// * `timestamp_ms = 0`, `run_id = 0`, `wall_ms = 0` — time is a host
///   concern; the guest records none.
/// * `total_ticks = steps as u64` — ticks observed are the replayed steps.
/// * `thermal = Warm` (nominal), `budget_met = true` (the guest only
///   constructs a receipt after its replay completed).
/// * `outcome` decoded from the ABI outcome byte via [`outcome_from_abi`]
///   (fail-closed on unknown bytes).
///
/// The trajectory digest lands in the `metadata_digest` field; the positive
/// control is absent (all-zero bytes, flag clear); the name is the fixed
/// `"ctt-guest-v1"`. `parent_receipt_hash` and `prev_self_hash` are all
/// zeros at construction — threading a chain is the caller's job via
/// [`guest_chain_hash`].
#[must_use]
pub fn guest_receipt(
    subject_hash: [u8; 32],
    steps: u32,
    total_events: u32,
    outcome_byte: u8,
    trajectory_digest: [u8; 32],
) -> CourtReceiptV1 {
    // NAME_MAX is 84; the fixed name is 12 bytes, so this cannot fail.
    // Constructed directly (not through `CourtReceiptV1::new`) to keep the
    // guest path infallible; every field satisfies the constructor's
    // validation by construction.
    let _ = total_events;
    CourtReceiptV1 {
        subject_hash,
        parent_receipt_hash: [0u8; 32],
        metrics: ReceiptMetrics {
            timestamp_ms: 0,
            run_id: 0,
            total_ticks: u64::from(steps),
            wall_ms: 0,
            thermal: ThermalClass::Warm,
            outcome: outcome_from_abi(outcome_byte),
            budget_met: true,
        },
        positive_control: [0u8; 32],
        metadata_digest: trajectory_digest,
        prev_self_hash: [0u8; 32],
        name: GUEST_RECEIPT_NAME.to_owned(),
        fail_closed: true,
    }
}

/// Computes the guest-side chain hash over `prev` and `receipt`, reusing the
/// landed chain law: `BLAKE3(prev ‖ encoded_receipt_bytes[0..304])`.
///
/// This is the same hash [`CourtReceiptV1::chain_hash`] computes when the
/// receipt's `parent_receipt_hash` equals `prev`; it is exposed separately so
/// a guest can chain receipts without mutating the receipt struct.
#[must_use]
pub fn guest_chain_hash(prev: [u8; 32], receipt: &CourtReceiptV1) -> [u8; 32] {
    let encoded = receipt.encode();
    let mut hasher = Hasher::new();
    hasher.update(&prev);
    hasher.update(&encoded[..304]);
    hasher.finalize().into()
}

/// Encodes the 352-byte chained record handed across the WASM boundary.
///
/// This is the ONLY thing that crosses the boundary. The HOST verifies it
/// with `VerifiedCourtReceipt::from_bytes_verified` and seals it with
/// ML-DSA-65 host-side: **guest produces, host seals**. The chained record's
/// chain hash covers `receipt.parent_receipt_hash` — set that field (or use
/// [`guest_chain_hash`] to preview the linkage) before calling this when the
/// receipt is not genesis.
#[must_use]
pub fn encode_receipt_wire(receipt: &CourtReceiptV1) -> [u8; COURT_RECORD_LEN] {
    ChainedCourtReceipt::from_receipt(receipt).encode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::receipt::VerifiedCourtReceipt;
    use crate::ledger::receipt::NAME_MAX;
    use crate::observability::receipt::Blake3ReceiptEntry;

    const SUBJECT: [u8; 32] = [0xA5; 32];
    const TRAJECTORY: [u8; 32] = [0x5A; 32];

    fn guest_pass() -> CourtReceiptV1 {
        guest_receipt(SUBJECT, 12, 12, 0, TRAJECTORY)
    }

    /// Guest build → wire encode → host verify roundtrip preserves the
    /// receipt, byte for byte.
    #[test]
    fn guest_receipt_roundtrips_through_host_verification() {
        let receipt = guest_pass();
        let wire = encode_receipt_wire(&receipt);
        assert_eq!(wire.len(), 352);

        let verified = VerifiedCourtReceipt::from_bytes_verified(&wire)
            .expect("guest-produced wire record must verify on the host");
        assert_eq!(verified.as_bytes(), wire);
        assert_eq!(verified.receipt(), receipt);
    }

    /// Every guest receipt field maps to its intended source.
    #[test]
    fn guest_receipt_fields_map_from_summary() {
        let receipt = guest_receipt(SUBJECT, 7, 9, 4, TRAJECTORY);
        assert_eq!(receipt.subject_hash, SUBJECT);
        assert_eq!(receipt.metrics.total_ticks, 7);
        assert_eq!(receipt.metrics.outcome, CourtOutcome::Refused);
        assert_eq!(receipt.metadata_digest, TRAJECTORY);
        assert_eq!(receipt.positive_control, [0u8; 32]);
        assert_eq!(receipt.name, "ctt-guest-v1");
        assert_eq!(receipt.metrics.timestamp_ms, 0);
        assert_eq!(receipt.metrics.run_id, 0);
        assert_eq!(receipt.metrics.wall_ms, 0);
        assert!(receipt.fail_closed);
    }

    /// Unknown ABI outcome bytes fail closed to `Error`, never `Pass`.
    #[test]
    fn unknown_outcome_byte_fails_closed() {
        assert_eq!(outcome_from_abi(5), CourtOutcome::Error);
        assert_eq!(outcome_from_abi(0xFF), CourtOutcome::Error);
        assert_eq!(outcome_from_abi(0), CourtOutcome::Pass);
        assert_eq!(outcome_from_abi(1), CourtOutcome::Fail);
        assert_eq!(outcome_from_abi(2), CourtOutcome::Skip);
        assert_eq!(outcome_from_abi(3), CourtOutcome::Error);
    }

    /// Determinism: identical summaries produce byte-identical wire records.
    #[test]
    fn identical_replays_produce_identical_wire() {
        let a = encode_receipt_wire(&guest_pass());
        let b = encode_receipt_wire(&guest_pass());
        assert_eq!(a, b);
    }

    /// Two guest receipts link per the Blake3ReceiptEntry chain law:
    /// child.parent = BLAKE3(genesis.parent ‖ genesis[0..304]), and the
    /// generic chain validator accepts the pair.
    #[test]
    fn guest_chain_hash_links_two_guest_receipts() {
        use crate::ledger::receipt::ChainedCourtReceipt;
        use crate::observability::receipt::Blake3ChainValidator;

        let genesis = guest_pass();
        let genesis_chain = guest_chain_hash([0u8; 32], &genesis);
        assert_eq!(genesis_chain, genesis.chain_hash());

        let mut child = guest_receipt(SUBJECT, 13, 13, 1, TRAJECTORY);
        child.parent_receipt_hash = genesis_chain;
        child.prev_self_hash = genesis_chain;
        let child_chain = guest_chain_hash(genesis_chain, &child);
        assert_eq!(child_chain, child.chain_hash());

        let parent_entry = ChainedCourtReceipt::from_receipt(&{
            let mut g = genesis;
            g.parent_receipt_hash = [0u8; 32];
            g
        });
        let child_entry = ChainedCourtReceipt::from_receipt(&child);
        assert_eq!(child_entry.prev_hash(), parent_entry.stored_hash());
        Blake3ChainValidator::assert_chain_valid(&[parent_entry, child_entry]);
    }

    /// Name fits the wire budget with room to spare.
    #[test]
    fn guest_name_within_name_max() {
        assert!(GUEST_RECEIPT_NAME.len() <= NAME_MAX);
    }
}
