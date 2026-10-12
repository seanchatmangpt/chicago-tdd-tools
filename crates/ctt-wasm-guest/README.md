# ctt-wasm-guest

Hermetic WASM verification kernel for chicago-tdd-tools: a `cdylib` that
re-exports the conformance guest ABI (`chicago_tdd_tools::conformance::guest`)
so the ABI functions are exported from the compiled `.wasm` module, hostable
by temprun (BEAM).

## Imports == ∅

The built module MUST have an empty import section: no `wasm-bindgen`, no
host functions, no `std::env`/`std::fs`/`std::time`/`std::process`. Every
input arrives through linear-memory buffers; every output is written through
out-parameters into linear memory. Any host that can instantiate the module
can be proven to host it (the `pn_kernel` precedent). Verify with
`wasm-objdump -x` — a non-empty import section is a Gate 0 build failure.

## The four architectural laws

1. **Zero ambient authority** — no imports, no file/clock/RNG/env access.
   The only capability the guest needs is linear memory itself. All
   deterministic replay state lives in a fixed handle table
   (`MAX_SESSIONS = 8` slots, no allocation after init, generation counters
   make ABA reuse observable as `ERR_BAD_HANDLE`).
2. **Preemptive reduction budgets** — the guest never blocks the VM. Each
   ABI call runs at most one bounded slice (`YieldDirective` quota,
   `0 < quota <= MAX_REDUCTION_QUOTA = 1_000_000`; default 2000, matching
   the BEAM's per-process reduction budget), then returns an honest status.
   Panic == trap == fail-closed: `panic = "abort"`, no `catch_unwind`, a
   trapped guest emits no receipt and half-commits nothing.
3. **Trajectory enforcement** — the model is parsed by the real
   `PowlModel::from_powl_str` and compiled by the real
   `ConformanceSession::from_model` (validate included); the ABI is a wire
   shim over the landed kernel, not a second parser. Replay semantics are
   bit-exact: identical (model, stream) → identical outcomes and seal
   preimage.
4. **Receipts** — every completed replay yields a BLAKE3-digested seal
   record. With `ledger` off, sealing is refused (`ERR_SEAL_UNAVAILABLE`)
   rather than downgraded to a weak hash.

## Build

```sh
cargo build --target wasm32-unknown-unknown --release -p ctt-wasm-guest
```

Artifact: `target/wasm32-unknown-unknown/release/ctt_wasm_guest.wasm`

## ABI surface (v1, `ctt_guest_abi_version() == 1`)

Buffers are `(ptr: u32, len: u32)` pairs in guest linear memory,
little-endian, tightly packed. Handles are
`u64 = (generation:u32) << 32 | slot:u32`.

| Symbol | Buffers (in → out) | Status |
|---|---|---|
| `ctt_guest_abi_version() -> u32` | — → version probe (host must call first; refuse to bind on mismatch) | n/a |
| `ctt_session_new(net_buf, net_len, out_handle)` | wire-format `CTTG` buffer → `u64` handle | errno table below |
| `ctt_session_attach(host, stream_ptr, stream_len)` | flat LE u64 OCEL64 event words (zero-length rejected: `ERR_STREAM_EMPTY`) | errno table below |
| `ctt_session_step(handle, quota, out_status, out_cap)` | → 21-byte step status record (1B outcome: 0 Yielded / 1 Complete / 2 Violation; 4B remaining_events; 4B steps_executed; 4B events_total; 8B final_marking) | errno table below |
| `ctt_session_seal(handle, out_buf, out_cap, out_len)` | → 34-byte seal preimage (8B steps + 8B events + 8B final_marking + 1B outcome + 1B violation code + 8B reserved, zero), BLAKE3-digested when `ledger` is on | errno table below |
| `ctt_session_drop(handle)` | — | errno table below |

### Status (errno) table

Positive return = guest errno; negative = trap-class (delivered by the
runtime, never returned by an ABI fn in v1).

| code | name | meaning |
|-----:|---|---|
| 0 | `OK` | success |
| 1 | `ERR_NULL_PTR` | null pointer / out-pointer |
| 2 | `ERR_BAD_MAGIC` | buffer lacks `CTTG` magic |
| 3 | `ERR_BAD_VERSION` | wire version byte != 1 |
| 4 | `ERR_TRUNCATED` | buffer shorter than layout requires |
| 5 | `ERR_TRAILING` | bytes present after the declared payload |
| 6 | `ERR_RESERVED_NOT_ZERO` | must-be-zero field non-zero |
| 7 | `ERR_BAD_MODEL` | model DSL failed parse/validate/compile |
| 8 | `ERR_TABLE_FULL` | all `MAX_SESSIONS` (8) slots occupied |
| 9 | `ERR_BAD_HANDLE` | handle free, out of range, or generation-stale |
| 10 | `ERR_STREAM_EMPTY` | attach of a zero-length stream |
| 11 | `ERR_CAP_TOO_SMALL` | out-buffer capacity below fixed record length |
| 12 | `ERR_SEAL_UNAVAILABLE` | sealing without the `ledger` feature |
| 13 | `ERR_STREAM_ALIGNMENT` | stream byte length not a multiple of 8 |

### Cooperative-yield statuses (`guest_yield`)

Distinct from the ABI errno table — these are the yield-protocol statuses
the host scheduler consumes:

`STATUS_OK = 0`, `STATUS_E_INTERRUPTED = 1` (quota exhausted, reschedule),
`STATUS_E_INVALID = 2`, `STATUS_E_NOMEM = 3`, `STATUS_E_TRAP = 4`,
`STATUS_E_THERMAL = 5` (advisory; the supervisor still owns termination),
`STATUS_E_UNSUPPORTED = 6`, `STATUS_E_UNAUTHORIZED = 7`,
`STATUS_E_TIMEDOUT = 8`, `STATUS_E_INTERNAL = 9`.

## Guest produces, host seals

The guest builds a typed `CourtReceiptV1` from its replay summary and
encodes the 352-byte chained record (`encode_receipt_wire`); it never
signs. The HOST re-decodes via `VerifiedCourtReceipt::from_bytes_verified`
and applies the ML-DSA-65 signature host-side (keys live in
`src/ledger/signing/keys.rs`, which uses `std::fs` and can never run
inside the guest). Guest receipts carry no wall clock:
`timestamp_ms = run_id = wall_ms = 0`; identical replays produce
byte-identical wire records. Chain law: `BLAKE3(prev ‖ encoded[0..304])`.

## temprun integration

The guest executes at most one reduction slice (≤2000 steps default) per
ABI call and returns control. Exceeding thermal or tick ceilings —
preemption, kill, backoff — is the BEAM supervisor's termination call, not
guest behavior. A wasm trap is a conformance failure of the session:
fail-closed, never fail-open.

Landed host surfaces (sibling repo `~/strata` — the BEAM/wasm execution side of this integration): the Rust bridge lives in strata at
`temprun/src/bin/temprun_step.rs` (loads the `.wasm`, drives
`ctt_session_step` slices) and the BEAM supervisor/worker in stratus at
`lib/stratus/temprun/worker.ex`, a `gen_statem` that consumes the raw-mode
little-endian framing of the step status record.

* Parent ABI: `src/conformance/guest.rs` (wire formats, errno table)
* Yield protocol: `src/conformance/guest_yield.rs`
* Receipt sealing: `src/conformance/guest_seal.rs`
* Guest telemetry ABI: `src/conformance/guest_telemetry.rs` — landed,
  gated `all(feature = "wasm-guest", feature = "ocel-ingest")`; converts
  replayed trajectories to packed OCEL64 words via
  `observability::ocel::ocel64_bridge::pack`.
