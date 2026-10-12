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
    VerifiedCourtReceipt, COURT_RECORD_LEN,
};
use blake3::Hasher;

/// Typed refusal of the NATO seal-admission gate (F6 law).
///
/// Two distinct laws govern the 352-byte wire record, and this boundary is
/// now explicit:
///
/// * **Evidence law** (`from_bytes_verified`): a structurally valid record
///   is evidence, whatever its outcome — a `Fail` receipt verifies and
///   enters the ledger as evidence.
/// * **Seal law** ([`seal_wire`] / [`seal_gate`]): only an admitted outcome
///   may be sealed. Mirroring `crate::nato::dossier_court::dossier_admits`
///   (which demands `Verdict::Pass` AND a caught positive control), the
///   guest wire lane demands at minimum `CourtOutcome::Pass` — guest
///   receipts carry no positive-control bytes (all-zero, flag clear), so
///   the gate refuses every non-`Pass` outcome with [`SealError::NotAdmitted`].
#[derive(Debug)]
pub enum SealError {
    /// The 352-byte record failed `from_bytes_verified` (evidence law).
    InvalidRecord(crate::ledger::error::LedgerError),
    /// The record verified but its outcome is not admitted for sealing
    /// (seal law): only `CourtOutcome::Pass` seals.
    NotAdmitted {
        /// The refusing outcome.
        outcome: CourtOutcome,
    },
}

impl core::fmt::Display for SealError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidRecord(e) => write!(f, "invalid receipt record: {e}"),
            Self::NotAdmitted { outcome } => {
                write!(f, "receipt outcome {outcome:?} is not admitted for sealing")
            }
        }
    }
}

impl core::error::Error for SealError {}

/// The seal-admission gate over a typed receipt.
///
/// `Pass` admits; every other outcome is refused with
/// [`SealError::NotAdmitted`]. The gate does NOT invalidate `Fail` receipts
/// as evidence — it only refuses them at the sealing boundary.
///
/// # Errors
///
/// [`SealError::NotAdmitted`] for any outcome other than
/// [`CourtOutcome::Pass`]; never `Ok` for non-`Pass` evidence.
pub const fn seal_gate(receipt: &CourtReceiptV1) -> Result<(), SealError> {
    match receipt.metrics.outcome {
        CourtOutcome::Pass => Ok(()),
        // JUSTIFICATION (clippy::match_same_arms): the three refusal arms are
        // deliberately enumerated so the gate's law names every refused
        // outcome explicitly rather than hiding them in a wildcard.
        #[allow(clippy::match_same_arms)]
        CourtOutcome::Fail | CourtOutcome::Skip | CourtOutcome::Error | CourtOutcome::Refused => {
            Err(SealError::NotAdmitted { outcome: receipt.metrics.outcome })
        }
    }
}

/// Verifies a 352-byte wire record, applies the seal-admission gate
/// ([`seal_gate`]), and returns the verified receipt — the host-side sealing
/// boundary: evidence law first, seal law second.
///
/// # Errors
///
/// [`SealError::InvalidRecord`] when `from_bytes_verified` refuses; [`SealError::NotAdmitted`]
/// when a verified record's outcome is not `Pass`.
pub fn seal_wire(bytes: &[u8; COURT_RECORD_LEN]) -> Result<VerifiedCourtReceipt, SealError> {
    let verified =
        VerifiedCourtReceipt::from_bytes_verified(bytes).map_err(SealError::InvalidRecord)?;
    seal_gate(&verified.receipt())?;
    Ok(verified)
}

/// Fixed receipt name carried by every guest-produced receipt
/// (well under [`crate::ledger::receipt::NAME_MAX`]; NUL padding is applied by [`CourtReceiptV1::encode`]).
const GUEST_RECEIPT_NAME: &str = "ctt-guest-v1";

/// Session/ABI outcome byte → court outcome (the ABI-lane translation).
///
/// F1 law: the seal-preimage outcome byte (`ReportOutcome` lane) and the
/// [`CourtOutcome`] wire-tag lane are TWO DIFFERENT ONTOLOGIES that collide on
/// the integers: 0/1/2 mean Incomplete/Complete/Faulted on the session lane
/// but Pass/Fail/Skip on the court wire-tag lane. Decoding a session byte
/// with the court-tag decoder silently minted `Fail` receipts for `Complete`
/// sessions. The honest translation:
///
/// * `0` (Incomplete) → [`CourtOutcome::Skip`] — nothing failed; the run
///   did not finish, so the receipt skips rather than lies.
/// * `1` (Complete)   → [`CourtOutcome::Pass`]
/// * `2` (Faulted)    → [`CourtOutcome::Fail`]
/// * any other byte fails closed to [`CourtOutcome::Error`].
///
/// Use this fn ONLY on bytes produced by the session/ABI lane
/// (`ctt_session_seal` outcome byte, `guest_receipt`'s `outcome_byte`).
/// Use [`outcome_from_court_tag`] ONLY on [`CourtOutcome`] wire-tag bytes
/// (record byte 105 of the 352-byte chained record).
#[must_use]
pub const fn outcome_from_abi(byte: u8) -> CourtOutcome {
    // JUSTIFICATION (clippy::match_same_arms): the identical arms are the
    // POINT — 3 (Error) and the wildcard both fail closed to `Error`: the
    // wildcard is the fail-closed catch-all for any out-of-range byte.
    #[allow(clippy::match_same_arms)]
    match byte {
        0 => CourtOutcome::Skip,
        1 => CourtOutcome::Pass,
        2 => CourtOutcome::Fail,
        3 => CourtOutcome::Error,
        _ => CourtOutcome::Error,
    }
}

/// [`CourtOutcome`] wire-tag byte → outcome (record byte 105 lane).
///
/// The `CourtOutcome` wire tags (0=Pass, 1=Fail, 2=Skip, 3=Error, 4=Refused).
/// Any out-of-range byte fails closed to [`CourtOutcome::Error`]: the guest
/// never silently converts an unknown verdict into `Pass`. This is the court
/// lane decoder — NOT the session/ABI lane ([`outcome_from_abi`]); the two
/// colliding integer spaces are exactly why they are separate named fns.
#[must_use]
pub const fn outcome_from_court_tag(tag: u8) -> CourtOutcome {
    // JUSTIFICATION (clippy::match_same_arms): same fail-closed argument as
    // [`outcome_from_abi`].
    #[allow(clippy::match_same_arms)]
    match tag {
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
///
/// # Panics
///
/// Only if the receipt's `name` violates [`crate::ledger::receipt::NAME_MAX`]/interior-NUL —
/// impossible for the fixed guest name, asserted by
/// `guest_name_within_name_max`; the panic is the loud failure.
#[must_use]
pub fn guest_chain_hash(prev: [u8; 32], receipt: &CourtReceiptV1) -> [u8; 32] {
    // JUSTIFICATION: guest names are the compile-time constant
    // GUEST_RECEIPT_NAME (asserted <= NAME_MAX by
    // `guest_name_within_name_max`), so the encode-boundary name check
    // cannot fire here; a panic is the loud failure, not a silent clamp.
    #[allow(clippy::expect_used)]
    let encoded = receipt.encode().expect("guest receipt name within NAME_MAX");
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
///
/// # Panics
///
/// Only if the receipt's `name` violates the [`crate::ledger::receipt::NAME_MAX`]/interior-NUL
/// encode boundary — impossible for the fixed guest name; the panic is the
/// loud failure, not a silent clamp.
#[must_use]
pub fn encode_receipt_wire(receipt: &CourtReceiptV1) -> [u8; COURT_RECORD_LEN] {
    // JUSTIFICATION: same name-boundary argument as `guest_chain_hash`.
    #[allow(clippy::expect_used)]
    ChainedCourtReceipt::from_receipt(receipt)
        .expect("guest receipt name within NAME_MAX")
        .encode()
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
        // 1 = Complete on the session/ABI lane → CourtOutcome::Pass (F1).
        guest_receipt(SUBJECT, 12, 12, 1, TRAJECTORY)
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
        // Outcome byte 4 is out-of-range on the session/ABI lane, so the
        // mapping fails closed to `Error` (never `Pass`).
        let receipt = guest_receipt(SUBJECT, 7, 9, 4, TRAJECTORY);
        assert_eq!(receipt.subject_hash, SUBJECT);
        assert_eq!(receipt.metrics.total_ticks, 7);
        assert_eq!(receipt.metrics.outcome, CourtOutcome::Error);
        assert_eq!(receipt.metadata_digest, TRAJECTORY);
        assert_eq!(receipt.positive_control, [0u8; 32]);
        assert_eq!(receipt.name, "ctt-guest-v1");
        assert_eq!(receipt.metrics.timestamp_ms, 0);
        assert_eq!(receipt.metrics.run_id, 0);
        assert_eq!(receipt.metrics.wall_ms, 0);
        assert!(receipt.fail_closed);
    }

    /// Unknown outcome bytes fail closed to `Error`, never `Pass`, on BOTH
    /// lanes; each lane's translation is pinned in both directions (byte →
    /// outcome here, outcome → byte via `CourtOutcome::to_u8` /
    /// the session lane's preimage byte table).
    #[test]
    fn outcome_translation_pinned_both_lanes_both_directions() {
        // Session/ABI lane (F1): Incomplete→Skip, Complete→Pass, Faulted→Fail.
        assert_eq!(outcome_from_abi(0), CourtOutcome::Skip);
        assert_eq!(outcome_from_abi(1), CourtOutcome::Pass);
        assert_eq!(outcome_from_abi(2), CourtOutcome::Fail);
        assert_eq!(outcome_from_abi(3), CourtOutcome::Error);
        assert_eq!(outcome_from_abi(4), CourtOutcome::Error, "fail closed");
        assert_eq!(outcome_from_abi(0xFF), CourtOutcome::Error);
        // Court wire-tag lane: Pass/Fail/Skip/Error/Refused = 0..4.
        assert_eq!(outcome_from_court_tag(0), CourtOutcome::Pass);
        assert_eq!(outcome_from_court_tag(1), CourtOutcome::Fail);
        assert_eq!(outcome_from_court_tag(2), CourtOutcome::Skip);
        assert_eq!(outcome_from_court_tag(3), CourtOutcome::Error);
        assert_eq!(outcome_from_court_tag(4), CourtOutcome::Refused);
        assert_eq!(outcome_from_court_tag(5), CourtOutcome::Error, "fail closed");
        // Reverse direction (court lane): outcome -> wire tag round-trips
        // through the canonical encoder.
        for (tag, outcome) in [
            (0u8, CourtOutcome::Pass),
            (1, CourtOutcome::Fail),
            (2, CourtOutcome::Skip),
            (3, CourtOutcome::Error),
            (4, CourtOutcome::Refused),
        ] {
            assert_eq!(outcome.to_u8(), tag);
            assert_eq!(outcome_from_court_tag(tag), outcome);
        }
        // The collision is named: the SAME integer decodes DIFFERENTLY per lane.
        assert_ne!(outcome_from_abi(1), outcome_from_court_tag(1));
        assert_ne!(outcome_from_abi(2), outcome_from_court_tag(2));
    }

    /// F6: a structurally valid `Fail` receipt is still EVIDENCE (it
    /// verifies through `from_bytes_verified`) but is refused at the sealing
    /// boundary by the admission gate — typed refusal, never a sealed Fail.
    #[test]
    fn fail_receipt_validates_but_seal_refuses() {
        let fail_receipt = guest_receipt(SUBJECT, 5, 5, 2, TRAJECTORY); // 2 = Faulted → Fail
        assert_eq!(fail_receipt.metrics.outcome, CourtOutcome::Fail);
        let wire = encode_receipt_wire(&fail_receipt);
        // Evidence law: the record verifies.
        assert!(
            VerifiedCourtReceipt::from_bytes_verified(&wire).is_ok(),
            "a Fail receipt is evidence and must still verify"
        );
        // Seal law: the gate refuses it.
        let err = seal_wire(&wire).expect_err("non-Pass receipts must not seal");
        assert!(
            matches!(err, SealError::NotAdmitted { outcome: CourtOutcome::Fail }),
            "expected NotAdmitted{{Fail}}, got {err}"
        );
        // The Pass lane still seals.
        let pass_wire = encode_receipt_wire(&guest_pass());
        let sealed = seal_wire(&pass_wire).expect("a Pass receipt must seal");
        assert_eq!(sealed.receipt().metrics.outcome, CourtOutcome::Pass);
        // Every other refused outcome is named by the gate.
        for byte in [0u8, 3, 4] {
            let r = guest_receipt(SUBJECT, 1, 1, byte, TRAJECTORY);
            let w = encode_receipt_wire(&r);
            assert!(
                matches!(seal_wire(&w), Err(SealError::NotAdmitted { .. })),
                "byte {byte} must not seal"
            );
        }
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
        assert_eq!(genesis_chain, genesis.chain_hash().expect("valid"));

        let mut child = guest_receipt(SUBJECT, 13, 13, 1, TRAJECTORY);
        child.parent_receipt_hash = genesis_chain;
        child.prev_self_hash = genesis_chain;
        let child_chain = guest_chain_hash(genesis_chain, &child);
        assert_eq!(child_chain, child.chain_hash().expect("valid"));

        let parent_entry = ChainedCourtReceipt::from_receipt(&{
            let mut g = genesis;
            g.parent_receipt_hash = [0u8; 32];
            g
        })
        .expect("valid");
        let child_entry = ChainedCourtReceipt::from_receipt(&child).expect("valid");
        assert_eq!(child_entry.prev_hash(), parent_entry.stored_hash());
        Blake3ChainValidator::assert_chain_valid(&[parent_entry, child_entry]);
    }

    /// Name fits the wire budget with room to spare.
    #[test]
    fn guest_name_within_name_max() {
        assert!(GUEST_RECEIPT_NAME.len() <= NAME_MAX);
    }
}
