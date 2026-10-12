# Reference: CTRT Wire Formats and Machine Codes

Two distinct 352-byte layouts share the `CTRT` magic. This document maps both as they exist in code, plus the complete machine-code table. All multi-byte integers are little-endian.

## Layout A — chained court receipt (chicago-tdd-tools)

Source: `/Users/sac/chicago-tdd-tools/src/ledger/receipt.rs` (module doc, lines 12-37; offsets in code: `B_SUBJECT`…`B_NAME`).

352-byte chained record = 320-byte receipt ([`COURT_RECEIPT_V1_LEN`]) + 32-byte `chain_hash` ([`COURT_RECORD_LEN`]).

### 320-byte receipt record

| Absolute offset | Size | Field | Encoding / constraint |
|---|---|---|---|
| 0..4 | 4 | magic | `b"CTRT"` |
| 4..6 | 2 | version | u16 LE = 1 |
| 6 | 1 | flags | bit0 = positive_control_present, bit1 = fail_closed |
| 7 | 1 | reserved | must be 0 |
| 8..16 | 8 | body_len | u64 LE = 304 |
| 16..320 | 304 | body | body-relative map below |

Body (offsets relative to byte 16):

| Body offset | Size | Field | Encoding / constraint |
|---|---|---|---|
| 0..8 | 8 | reserved | must be 0 |
| 8..40 | 32 | subject_hash | [u8; 32] |
| 40..72 | 32 | parent_receipt_hash | [u8; 32] (genesis `[0u8; 32]`) |
| 72..80 | 8 | timestamp_ms | u64 LE |
| 80..88 | 8 | run_id | u64 LE |
| 88..96 | 8 | total_ticks | u64 LE |
| 96..104 | 8 | wall_ms | u64 LE |
| 104 | 1 | thermal | 0 = Hot, 1 = Warm, 2 = Cold |
| 105 | 1 | outcome | 0 = Pass, 1 = Fail, 2 = Skip, 3 = Error, 4 = Refused |
| 106 | 1 | budget_met | 0 or 1 |
| 107 | 1 | padding | must be 0 |
| 108..140 | 32 | positive_control | [u8; 32] |
| 140..172 | 32 | metadata_digest | [u8; 32] |
| 172..204 | 32 | prev_self_hash | [u8; 32] |
| 204..304 | 100 | name | NUL-padded UTF-8; final 16 bytes of body (body 288..304) are the zero trailer, must be 0; usable name bytes capped at `NAME_MAX = 84` |

Domain separators in this layout:

- Chain law: `chain_hash = BLAKE3(parent_receipt_hash ‖ receipt_bytes[0..304])` — record bytes 320..352 store it; content is `record[..304]` (RM1 offset law, enforced in src/ledger/dossier_wire.rs `ChainRecord::of`).
- Dossier subject: `BLAKE3(u32le(34) || "chicago-ledger-dossier-subject-v1" || merkle_root)` (`SUBJECT_DOMAIN`, src/ledger/signing/subject.rs).
- ML-DSA signing context: `"chicago-ledger-dossier-v1"` (`SIGN_CONTEXT`, src/ledger/signing/signer.rs).

### Dossier envelope (CTT-DOSSIER-v1, JSON)

| Field | Type | Constraint |
|---|---|---|
| `format` | string | exactly `"CTT-DOSSIER-v1"` |
| `court_receipts` | array | 20 receipt JSON objects |
| `merkle_root` | string | 64 lowercase hex chars |
| `key_id` | string | 16 lowercase hex chars (8 bytes, first 8 of BLAKE3 over the encoded public key (`key_id`, src/ledger/signing/keys.rs)) |
| `signature` | string | 6618 lowercase hex chars = 3309 bytes, ML-DSA-65 over `dossier_subject(root)` |
| `sealed_at` | u64 | Unix seconds |

## Layout B — CTRT wire token (affidavit)

Source: `/Users/sac/affidavit/src/wire_ctrt.rs` (`serialize`/`deserialize`, `CTRT_WIRE_SIZE = 352`).

| Offset | Size | Field | Encoding / constraint |
|---|---|---|---|
| 0..4 | 4 | magic | `b"CTRT"` (`CTRT_MAGIC`) |
| 4..6 | 2 | version | u16 LE = 1 (`CTRT_VERSION_1`) |
| 6..38 | 32 | parent_receipt_hash | [u8; 32] |
| 38..70 | 32 | court_id | [u8; 32] |
| 70..102 | 32 | subject_merkle_root | [u8; 32] |
| 102..134 | 32 | compiler_binary_digest | [u8; 32] |
| 134..142 | 8 | execution_ticks | u64 LE |
| 142..150 | 8 | elapsed_cycles | u64 LE |
| 150 | 1 | verdict_status | 0 = Refused, 1 = Accepted, 2 = Pending, 3 = Inconclusive |
| 151..352 | 201 | payload | [u8; 201] (`CTRT_PAYLOAD_SIZE`) |

Decode is constant-flow in the hot path: magic/version/verdict validity is checked by a constant-time mask predicate (`ct_validate_verdict_status`), the verdict rides as a raw `u8` (`verdict_raw`) validated by the return code and decoded into `VerdictStatus`. Digest: `digest() = BLAKE3(serialize())`. A full 352-byte tamper matrix (1-bit and 0xFF inversion at every offset) asserts every mutation either fails decode or changes the digest.

Spec divergence: Layout B is the receipt emitted by `affi receipt emit --format ctrt`; Layout A is the ledger chain record behind the NATO dossier. Both claim the `CTRT` magic and 352-byte size but are not interchangeable — no cross-layout decoder exists and none should be assumed.

## Machine-code table (complete, 0xE001–0xE409)

Every `machine_code()` implementation in the two crates, as it exists in code.

### 0xE0xx — CTRT wire decode (`/Users/sac/affidavit/src/wire_ctrt.rs`, `WireCtrtError`)

| Code | Variant |
|---|---|
| 0xE001 | `InvalidLength(usize)` — slice is not exactly 352 bytes |
| 0xE002 | `InvalidMagic([u8; 4])` — magic is not `CTRT` |
| 0xE003 | `UnsupportedVersion(u16)` — version is not 1 |
| 0xE004 | `InvalidVerdictStatus(u8)` — verdict byte > 3 |
| 0xE005 | wire_ctrt | DigestMismatch | computed digest ≠ stored chain hash (deserialize_verified) |

### 0xE1xx — Replay nullifier (`/Users/sac/affidavit/src/replay_nullifier.rs`, `ReplayNullifierError`)

| Code | Variant |
|---|---|
| 0xE101 | `ReplayDetected(String)` — nullifier already observed this epoch |
| 0xE102 | `InvalidPersistenceStorage(String)` / `CorruptLedger(String)` |
| 0xE103 | replay_nullifier | ReplayEpochMismatch | nullifier presented against a rotated epoch |

### 0xE2xx — Trajectory folding (`/Users/sac/affidavit/src/trajectory_folding.rs`, `TrajectoryFoldError`)

| Code | Variant |
|---|---|
| 0xE201 | `ParentLinkageMismatch` — `B.parent_receipt_hash` ≠ BLAKE3(A) |
| 0xE202 | `IncompatibleVerdict` — outcome/verdict mismatch during folding |
| 0xE203 | `InvalidLength` |
| 0xE204 | `BadMagic` |
| 0xE205 | `UnsupportedVersion` |
| 0xE206 | `ReservedNonzero` |
| 0xE207 | `BodyLenMismatch` |
| 0xE208 | `InvalidEnum` |
| 0xE209 | `ControlFlagMismatch` |
| 0xE20A | `DigestMismatch` |
| 0xE20B | `ZeroTrailerViolation` |
| 0xE20C | `InvalidName` |

### 0xE3xx — PQC / deontic (`/Users/sac/affidavit/src/deontic.rs` + `crypto_trust_pqc.rs` + `error.rs`)

| Code | Variants |
|---|---|
| 0xE301 | `DeonticError::SignatureOrContextMismatch`, `ContextTooLong`, `MalformedPublicKey`, `MalformedSignature`; `error::PqcError::Signing`/`Verification`; `crypto_trust_pqc::PqcError::MlDsa`/`MalformedSignature` |
| 0xE302 | `DeonticError::MlKemDecapsulationFailure`; `error::PqcError::Encapsulation`; `crypto_trust_pqc::PqcError::MalformedPublicKey` |
| 0xE303 | `DeonticError::FormalOracleDivergence`; `error::PqcError::Chain`; `crypto_trust_pqc::PqcError::SlhDsa`/`Es256Half` |
| 0xE304 | `DeonticError::ContextMismatch` — structural parameter divergence |

### 0xE4xx — Git Merkle (`/Users/sac/affidavit/src/git_merkle.rs`, `GitMerkleError`)

| Code | Variant |
|---|---|
| 0xE401 | `ObjectNotFound` |
| 0xE402 | `TreeCorruptOrUnparseable` |
| 0xE403 | `Io` |
| 0xE404 | `Odb` |
| 0xE405 | `Decode` |
| 0xE406 | `Receipt` |
| 0xE407 | `InvalidCommit` |
| 0xE408 | `RefNotFound` |
| 0xE409 | `EntryNotFound` |

### PQC naming-delta table

From the module doc of `/Users/sac/affidavit/src/error.rs` (lines 3-13) — names were not mass-renamed; this mapping is the record:

| Code | Operator matrix name | Actual variant(s) |
|---|---|---|
| `0xE301` | Signing / Verification | `error::PqcError::Signing`, `error::PqcError::Verification`; `crypto_trust_pqc::PqcError::MlDsa`, `::MalformedSignature`; `deontic::DeonticError::SignatureOrContextMismatch` family |
| `0xE302` | Encapsulation | `error::PqcError::Encapsulation`; `crypto_trust_pqc::PqcError::MalformedPublicKey`; `deontic::DeonticError::MlKemDecapsulationFailure` |
| `0xE303` | Chain | `error::PqcError::Chain`; `crypto_trust_pqc::PqcError::SlhDsa`, `::Es256Half`; `deontic::DeonticError::FormalOracleDivergence` |

Distinct from the above, the ledger CLI in chicago-tdd-tools has its own process-exit classes (src/ledger/mod.rs `LedgerExitCode`): 0 Ok, 10 MalformedReceipt, 11 SchemaVersionMismatch, 12 ChainBroken, 13 Tampered, 14 AdmissionRefused, 15 DanglingObject, 16 NonMonotonic, 17 EmptyDossier, 18 DigestMismatch, 19 DossierInvalid. These are exit codes, not `machine_code()` values.
