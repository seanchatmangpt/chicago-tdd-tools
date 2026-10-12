# How To: Operate the Replay Nullifier Filter

Operations for the CTRT anti-replay filter in `/Users/sac/affidavit/src/replay_nullifier.rs` and its CLI wiring in `/Users/sac/affidavit/src/handlers.rs` (`pub fn emit`, `format == "ctrt"` branch).

## The real API

```rust
use affidavit::replay_nullifier::{
    compute_nullifier, ReplayNullifierRegistry, ReplayNullifierError,
};

let mut registry = ReplayNullifierRegistry::new(epoch_key, window_capacity);
let n = registry.check_and_admit(&receipt_wire)?;   // Err on replay
registry.admit_nullifier(n)?;                        // precomputed nullifier
registry.is_seen(&n);                                // non-mutating probe
registry.len();                                      // dense-cache count
registry.rotate_epoch(new_epoch_key);                // explicit rotation
```

Fixed constants (from source):

- `NULLIFIER_SIZE = 32` — nullifier is `[u8; 32]`, `N = BLAKE3_keyed(K_epoch, ReceiptWire)` (`compute_nullifier`).
- `NULLIFIER_KEY_SIZE = 32` — keyed-BLAKE3 epoch key.
- Bit filter capacity: `new(capacity)` allocates `max(capacity, 64) * 8` bits with a two-hash Bloom projection (nullifier bytes `[0..8]` and `[8..16]`, LE, mod `num_bits`).
- Error codes: `ReplayDetected → 0xE101`; `InvalidPersistenceStorage` and `CorruptLedger → 0xE102` (`machine_code()`).

## Where the state lives

The persistence path **is** `.affi/nullifiers.bin` — relative to the process working directory, created by the `affi receipt emit --format ctrt` handler (handlers.rs:160-171):

```rust
let nullifier_dir = std::path::Path::new(".affi");
let nullifier_path = nullifier_dir.join("nullifiers.bin");
let epoch_key: [u8; 32] = [0x26u8; 32];
let mut registry = ReplayNullifierRegistry::new(epoch_key, 10_000);
```

Spec divergence: the registry itself has no persistence; `.affi/nullifiers.bin` is a handler-side convention — a flat append-only log of 32-byte nullifiers, no header, no checksum, no length field.

On each `affi receipt emit --format ctrt`:

1. If the file exists, it is read and each 32-byte chunk is `admit_nullifier`-ed (read errors are swallowed with `if let Ok` — see "Crash-corruption recovery" below).
2. The new receipt's nullifier is computed and `check_and_admit`-ed. A replay exits with `Anti-replay refusal: ReplayDetected: <hex>` (machine code 0xE101).
3. The nullifier is appended (32 bytes, `OpenOptions::append(true)`), and the wire receipt is written to `receipt_<first 8 nullifier bytes hex>.ctrt`.

## Capacity and rotation

- **Window capacity** is the second constructor argument (the CLI uses `10_000`). When `dense_cache.len() >= window_capacity` on the next admit, the registry auto-rotates: `epoch_generation += 1`, new key = `BLAKE3_derive_key("affidavit/ctrt-epoch-rotation/v1", K_epoch || u64le(generation))`, then filter and cache clear. This is replay protection after eviction, not data retention — evicted nullifiers are forgotten by design.
- **Explicit rotation**: `rotate_epoch(new_epoch_key)` swaps the key, zeroes the generation, and clears both structures. After rotation the same receipt wire computes a *different* nullifier and is admitted again — rotation bounds replay memory to one epoch.
- Never rotate the key without changing it: rotating with the same key re-opens the epoch's receipts to replay.

## Crash-corruption recovery

`nullifiers.bin` has no framing beyond 32-byte records. Recovery rules:

- **Trailing partial record** (crash mid-append): `chunks_exact(32)` in the load loop silently drops the incomplete tail. No operator action needed; the next append re-aligns only if the tail is a whole multiple — a partial tail makes every subsequent chunk misaligned, so the correct repair is to truncate the file to a multiple of 32 before further emits.
- **Truncate-and-rebuild is always safe**: `rm .affi/nullifiers.bin` — the registry regenerates on the next emit. Cost: replay memory for the dropped window is lost; previously emitted `.ctrt` files can be re-presented. Re-populate by re-hashing the receipts if you have them: `compute_nullifier(&epoch_key, &wire)` for each retained receipt, append the 32-byte values.
- **Bit rot / hostile content**: the typed errors `InvalidPersistenceStorage` and `CorruptLedger` (0xE102) exist for this class, but the current CLI handler does not map read failures onto them — it skips unreadable state. Spec divergence: persistence-error handling in the handler is fail-open on read, fail-closed on replay. If you need 0xE102 semantics, use the library API directly and surface the error yourself.

## Diagnosing a replay refusal

```console
$ affi receipt emit build artifact-1 '{}' --format ctrt
emitted CTRT wire receipt -> receipt_1f2a3b4c5d6e7f80.ctrt (nullifier: 1f2a…)
$ affi receipt emit build artifact-1 '{}' --format ctrt
Error: Anti-replay refusal: ReplayDetected: 1f2a3b4c…
```

The hex in the error is the full 32-byte nullifier. Same wire bytes in the same epoch always produce the same nullifier; `--format ctrt` emits are deduplicated on `(type, object, payload)` because those three strings determine the wire's `parent_hash`, `court_id`, and `subject_merkle_root` (see 03-reference.md).
