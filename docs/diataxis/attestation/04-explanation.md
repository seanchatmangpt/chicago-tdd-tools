---
title: "Explanation: Deontic Context Binding"
quadrant: explanation
---

# Explanation: Deontic Context Binding

Why a covenant breach fails at verify: `/Users/sac/affidavit/src/deontic.rs` (FR-PQC-06) makes the deontic parameters — jurisdiction, constraint set, lease epoch — *part of the signed message's context*, so they are covered by the ML-DSA-65 signature itself, not checked beside it.

## The Ctx construction

`DeonticContext` (deontic.rs:107) has three fields:

```rust
pub struct DeonticContext {
    pub jurisdiction: String,          // scoping obligation/permission
    pub deontic_constraint_hash: [u8; 32], // digest of the active constraint set
    pub lease_epoch: u64,              // monotonic authority-validity epoch
}
```

`to_context_bytes()` flattens it domain-separated:

```text
Ctx = "STRATA:" || Jurisdiction || DeonticConstraintHash || LeaseEpoch(u64 BE)
```

`construct_deontic_context` (`to_be_bytes()` at deontic.rs:160 — the epoch is big-endian here, a deliberate exception to the little-endian wire fields in 03-reference.md) refuses to build a context longer than `MAX_ML_DSA_CONTEXT_LEN = 255`, returning `ContextTooLong` → machine code 0xE301. The 255-byte cap is the FIPS 204 ML-DSA context limit (Algorithm 2 / §5.4): the context string is hashed into the signing transcript, and the standard forbids longer inputs.

## Why breaches fail at verify

Signing and verification both consume the same `Ctx`:

```rust
sign_with_deontic_context(&seed, msg, &context)?;   // esk.sign_deterministic(msg, ctx)
verify_with_deontic_context(&public, msg, &sig, &context)?; // vk.verify_with_context(msg, ctx, sig)
```

ML-DSA is deterministic in (key, message, context): the context enters `sign_deterministic`/`verify_with_context` as the FIPS 204 context string, so a signature produced under `Ctx_a` is not a valid signature over the message under `Ctx_b` — the internal transcript hash differs. A verifier presenting a different jurisdiction, a constraint set with one flipped bit, or an expired lease epoch therefore gets `false` from the primitive, and `verify_with_deontic_context_raw` converts that into `DeonticError::SignatureOrContextMismatch` → 0xE301. There is no path where altered parameters still verify: the covenant is not a side-condition the code checks, it is inside the crypto.

The test suite pins this (deontic.rs tests): mismatched jurisdiction, 1-bit-mutated constraint hash, and epoch+1 each fail with 0xE301; structurally equal contexts pass.

## Layered refusals

- **Cryptographic binding** (above) catches any parameter divergence between sign-time and verify-time with 0xE301.
- **Structural comparison** is separate: `enforce_deontic_context_match(expected, actual)` compares the `DeonticContext` values directly and returns the distinct `ContextMismatch` → 0xE304 when any field diverges. Use it when you need to tell an operator *which* parameter moved; the cryptographic path only tells you that verification failed.
- **Malformed inputs** fail before the crypto: wrong public-key length (≠ 1952) → `MalformedPublicKey` (0xE301), wrong signature length (≠ 3309) → `MalformedSignature` (0xE301), over-long context → `ContextTooLong` (0xE301).

Adjacent codes in the same module: `verify_ml_kem_decapsulation` → 0xE302 (shared-secret mismatch / implicit rejection), `verify_formal_oracle_consensus` and `verify_formal_oracle_signature_agreement` → 0xE303 (primary vs. formal-oracle disagreement).

## Relation to the dossier seal

Two context mechanisms coexist and must not be confused:

1. The ledger dossier seal (chicago-tdd-tools) signs with the fixed context `chicago-ledger-dossier-v1` over the framed subject `BLAKE3(u32le(34) || "chicago-ledger-dossier-subject-v1" || root)` — domain separation at the *message* layer.
2. Deontic binding (affidavit) parameterizes the *FIPS 204 context string* per lease. It is the mechanism to reach for when authority is scoped: a signature minted under lease epoch 100 stops verifying the moment the verifier asserts epoch 101, which is exactly the fail-closed behavior an expiring authorization requires.

Fail-closed summary: every divergence — in jurisdiction, constraint hash, epoch, key material, signature encoding, or context length — lands in a typed `DeonticError` with a machine code, and the only success path is byte-equal parameters plus a cryptographically valid signature.
