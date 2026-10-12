# Reference: PN64 Bit Layout and the Guest ABI (as landed)

Source of truth: `src/conformance/pn64.rs` and `src/conformance/guest.rs` (this repo,
`/Users/sac/chicago-tdd-tools`). This page documents landed reality, including one explicit
divergence from an earlier operator spec.

## PN64: one Petri-net transition in one `u64`

Ported from strata `src/pn64.rs` @ fix/kg-42-status-code (0de13a3). Little-endian bit fields;
one `u64` is the whole state. `src/conformance/pn64.rs:6-11`:

```text
bits [15:0]   marking m    — token presence per place, one bit per place
bits [31:16]  W-           — precondition mask (places this transition consumes)
bits [47:32]  W+           — postcondition mask (places this transition produces)
bits [55:48]  tid          — transition id (8 bits)
bits [63:56]  flags        — flags / invariant byte
```

Constants and accessors (all `const fn`, `no_std`-compatible, no heap):

- `MASK16 = 0xFFFF`; `marking`, `pre`, `post`, `tid`, `flags` field extractors; `pack(mark, pre, post, tid, flags)`.
- `enabled(pn)`: `(m & W-) == W-` (an empty precondition is always enabled).
- `fire(pn)` -> `Option<u64>`: `m' = (m ^ W-) | W+`, all non-marking fields preserved
  bit-exact; `None` when not enabled.
- `ct_eq(a, b)` / `ct_lte(a, b)`: constant-time comparison via the
  `(x | x.wrapping_neg()) >> 63` idiom and the sign-bit trick respectively. `ct_lte` is exact
  whenever the difference does not wrap across 2^63 (documented inexactness outside).

### `step_branchless(pn, ocel, invariant_mask) -> u64`

The branchless firing step used by the timing and disassembly courts. The OCEL64 event word
contributes `activity = ocel & 0xFF`; the verb matches when `(activity & 0xFF) == tid`. Three
validity predicates, each folded to 1/0 arithmetically:

1. Verb match.
2. Token availability: `(m & W-) == W-`.
3. Invariant law (FORBIDDEN-BIT): `(M(t+1) & InvariantMask) == 0` over `mask & 0xFFFF`.

Commit selection is arithmetic (`mask = valid.wrapping_neg()`), never a branch: on any refusal
the returned word is `pn` bit-exact unchanged. Differential-tested against a naive branched
reference over 200,000 pseudorandom words plus structured corners (`pn64.rs` tests).

## Guest ABI (as landed): `i32` return + out-buffer status records

> **Divergence note (explicit).** An earlier operator spec described a return-word scheme in
> which the guest communicates outcome in return words 0-3. That is the **superseded E13
> design**. The landed ABI (`src/conformance/guest.rs`; `ABI_VERSION = 2` by default — the `guest-abi-v1` feature compiles the shim tier, `ABI_VERSION = 1`) is different: every
> ABI function returns an `i32` status (0 = ok, positive = guest errno), and bulk results
> travel through out-parameters written into linear memory (the 21-byte step status record,
> the 32-byte seal digest, the `u64` handle out-parameter). The ABI functions themselves never
> return a negative; a Wasm trap is the negative case, delivered by the runtime for genuinely
> unreachable states. If you are integrating against a doc that says "return words 0-3", that
> doc predates the landed surface — this page and `guest.rs` win.

### Status errno table (0..=16, ABI v2)

`guest.rs:138-156`. Zero is success; positive values are guest errnos; negatives never appear.

| Code | Name | Meaning |
|-----:|------|---------|
| 0 | `OK` | success |
| 1 | `ERR_NULL_PTR` | a required pointer/out-pointer argument was null |
| 2 | `ERR_BAD_MAGIC` | buffer does not start with `CTTG` |
| 3 | `ERR_BAD_VERSION` | wire version byte != 1 |
| 4 | `ERR_TRUNCATED` | buffer shorter than its declared layout requires |
| 5 | `ERR_TRAILING` | bytes present after the declared payload |
| 6 | `ERR_RESERVED_NOT_ZERO` | a must-be-zero field is non-zero |
| 7 | `ERR_BAD_MODEL` | model DSL failed parse/validate/compile |
| 8 | `ERR_TABLE_FULL` | all `MAX_SESSIONS` (8) slots occupied |
| 9 | `ERR_BAD_HANDLE` | handle is free, out of range, or generation-stale |
| 10 | `ERR_STREAM_EMPTY` | attach of a zero-length stream |
| 11 | `ERR_CAP_TOO_SMALL` | out-buffer capacity below the record's fixed length (pure capacity meaning at v2) |
| 12 | `ERR_SEAL_UNAVAILABLE` | sealing requested without the `ledger` feature |
| 13 | `ERR_STREAM_ALIGNMENT` | stream byte length not a multiple of 8 |
| 14 | `ERR_OOB_READ` | an input span (`net_buf`/`stream_ptr`) reaches at/past the installed linear-memory bound |
| 15 | `ERR_OOB_WRITE` | an out-buffer span reaches at/past the installed linear-memory bound (the former v1 `ERR_CAP_TOO_SMALL` overload) |
| 16 | `ERR_DEFERRED_HEAP` | a heap-requiring path was requested under the deferred-heap ABI tier (reserved; not yet returned) |

A test pins the table's stability and uniqueness (`status_table_is_stable_and_unique`) and
asserts every code is non-negative.

> **v1 → v2 (explicit).** At v1, `ERR_CAP_TOO_SMALL` (11) doubled as the out-span OOB
> refusal; v2 (W-ABI2 item 4) splits that overload out into `ERR_OOB_READ` (14) /
> `ERR_OOB_WRITE` (15), so 11 is pure capacity again. The seal preimage also grew from 34 to
> 58 bytes: the v1-exact 26-byte prefix is unchanged, and the v1 reserved-zero field
> (`26..34`) is consumed by the 32-byte module digest (`26..58`). The v1 `reserved == 0` law
> and the v2 digest law are mutually exclusive on those bytes — a v1 parser validating
> `reserved == 0` fails closed on a v2 preimage. The authoritative version discriminator is
> the `ctt_guest_abi_version()` probe (returns 2 by default; 1 under `guest-abi-v1`); a host MUST call it before the first
> `ctt_session_new`.

### Step status record v1 (out-buffer of `ctt_session_step`)

21 bytes total (`STEP_STATUS_LEN`); `out_cap` must be `>= 21`.

```text
offset  size  field
0       1     outcome byte: 0 = Yielded, 1 = Complete, 2 = Violation
1       4     remaining_events_le32  (valid for Yielded; else 0)
5       4     steps_executed_le32    (summary; valid unless Yielded)
9       4     events_total_le32      (summary; valid unless Yielded)
13      8     final_marking_le64     (summary; valid unless Yielded)
```

The three outcomes correspond exactly to `YieldStatus::Yielded { remaining_events }`,
`YieldStatus::Complete(ReplaySummary)`, and `YieldStatus::Violation(ConformanceError)`
(`src/conformance/report.rs:113`). On `Violation`, the violation's classified detail travels
via the sealed summary's violation code, not the step record.

### Other wire records

- **Model buffer** (`ctt_session_new` input): `b"CTTG"` magic (4B), version `0x01` (1B),
  reserved 0 (1B), `model_len_le16` (2B), reserved2 zero (8B), then exactly `model_len` POWL
  DSL bytes (trailing bytes rejected).
- **Stream** (`ctt_session_attach` input): flat LE `u64` OCEL64 words; nonempty; length a
  multiple of 8.
- **Seal preimage** (`ctt_session_seal`, v2): fixed 58 bytes — `8B LE steps_executed |
  8B LE events_total | 8B LE final_marking | 1B outcome (0 Incomplete / 1 Complete /
  2 Faulted) | 1B violation code | 32B module digest in `26..58` (the v1 zero-reserved field
  `26..34` is consumed; v1-exact 26-byte prefix unchanged)`; the out-buffer receives the
  32-byte BLAKE3 digest (`DIGEST_LEN`), only with the `ledger` feature.
- **Violation codes** (seal-preimage byte): 0 `NONE`, 1 `SKIPPED`, 2 `OUT_OF_ORDER`,
  3 `UNDECLARED`, 4 `FORBIDDEN_BIT`, 5 `TRACE_MALFORMED`, 6 `TOO_MANY_PLACES`,
  7 `MODEL_TOO_LARGE`, 8 `OTHER` (`guest.rs::violation`).
- **Handle**: `u64 = (generation:u32) << 32 | slot:u32`; slot < `MAX_SESSIONS = 8`; drop bumps
  the generation so stale handles fail with `ERR_BAD_HANDLE` (a `u32` generation wraps after
  ~2^32 drop cycles of the same slot — measured ~16.7 h at ~14 µs/cycle; widening is a recorded
  ABI v2 work order).

## `guest_yield` STATUS_E table (yield-scheduling surface)

`src/conformance/guest_yield.rs`. `STATUS_OK` is an **alias of the canonical `guest::OK`**
(one zero for both surfaces). The remaining `STATUS_E` values are **yield-scheduling statuses,
NOT ABI errnos**: their numbers overlap the canonical `ERR_*` 1..=9, but they live in a
different namespace and must never cross the wire as ABI errnos. If a yield status ever needs
to appear in an ABI errno position, the table in `guest.rs` is extended there — these are
never renumbered to coincide.

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `STATUS_OK` | slice completed within quota (alias of canonical `OK`) |
| 1 | `STATUS_E_INTERRUPTED` | slice exhausted its quota before completing (host reschedules) |
| 2 | `STATUS_E_INVALID` | guest rejected a malformed/out-of-contract host request |
| 3 | `STATUS_E_NOMEM` | guest asked for a resource the host has not granted (distinct from canonical `ERR_CAP_TOO_SMALL = 11`) |
| 4 | `STATUS_E_TRAP` | trap/panic inside guest code; state poisoned for this slice |
| 5 | `STATUS_E_THERMAL` | thermal/tick ceiling breach, early yield (advisory; supervisor owns termination) |
| 6 | `STATUS_E_UNSUPPORTED` | requested protocol version/capability not implemented |
| 7 | `STATUS_E_UNAUTHORIZED` | caller lacks authority for the requested operation |
| 8 | `STATUS_E_TIMEDOUT` | slice deadline passed before completion |
| 9 | `STATUS_E_INTERNAL` | internal guest error not otherwise classified |

Slice scheduling constants: `DEFAULT_REDUCTION_QUOTA = 2000`,
`MAX_REDUCTION_QUOTA = 1_000_000`; `YieldDirective::new` refuses quota 0 and quota above the
ceiling with `QuotaRefusal::ZeroQuota` / `OverCeiling`. `next_quota(_)` is a v1 placeholder
returning the constant 2000 — the BEAM supervisor owns adaptive policy.
