# Tutorial: Author a Zero-Import WASM Guest

Worked example: `crates/ctt-wasm-guest` — the landed temprun-hostable guest cdylib in this
repo. Every command below is runnable from the repo root. All file references are grounded in
landed code; no TODOs.

## What you are building

A `cdylib` whose compiled `.wasm` module has an **empty import section** (law: `imports == ∅`).
It runs as pure computation over caller-provided linear memory: no host functions, no
`wasm-bindgen`, no clock, no RNG, no filesystem. A fuel-limited runtime cannot service host
calls, so a conforming guest imports nothing — and any host that can instantiate it can be
proven to host it.

The guest ABI surface lives in the parent crate at `src/conformance/guest.rs` (feature
`wasm-guest`); `crates/ctt-wasm-guest/src/lib.rs` re-exports it with
`pub use chicago_tdd_tools::conformance::guest::*` so the `#[no_mangle] extern "C"` functions
are exported from the compiled `.wasm`. The zero-ambient law is stated in the ABI's module
header: no `std::env`, `std::fs`, `std::time`, `std::process`; every input arrives through
linear-memory buffers, every output through out-parameters. The only capability the guest
needs from the host is linear memory itself.

## Prerequisites

```sh
rustup target add wasm32-unknown-unknown
```

## Step 1: The ABI surface

Six exported functions (all defined in `src/conformance/guest.rs`, re-exported by the guest
crate):

| Export | C signature | Purpose |
|---|---|---|
| `ctt_guest_abi_version` | `() -> u32` | Probe; returns `ABI_VERSION = 1` |
| `ctt_session_new` | `(net_buf: *const u8, net_len: u32, out_handle: *mut u64) -> i32` | Parse, validate, and compile a wire-format model buffer into a `ConformanceSession` in a table slot |
| `ctt_session_attach` | `(handle: u64, stream_ptr: *const u8, stream_len: u32) -> i32` | Bind a flat LE `u64` OCEL64 event stream (length must be a multiple of 8, nonempty) |
| `ctt_session_step` | `(handle: u64, quota: u32, out_status: *mut u8, out_cap: u32) -> i32` | Replay at most `quota` events; write the fixed 21-byte step status record |
| `ctt_session_seal` | `(handle: u64, out_buf: *mut u8, out_cap: u32, out_len: *mut u32) -> i32` | Write the 32-byte BLAKE3 digest of the 34-byte seal preimage (requires `ledger`) |
| `ctt_session_drop` | `(handle: u64) -> i32` | Free the slot; the generation counter makes ABA reuse observable as `ERR_BAD_HANDLE` |

Key wire facts (module header of `guest.rs`):

- Model buffer: `b"CTTG"` magic, version `0x01`, LE `u16` model length, reserved fields zero,
  then the POWL DSL bytes.
- Step status record: 21 bytes — outcome byte (0 = Yielded, 1 = Complete, 2 = Violation),
  `remaining_events_le32`, `steps_executed_le32`, `events_total_le32`, `final_marking_le64`.
- Panic semantics: `catch_unwind` is deliberately not used; a Rust panic lowers to a Wasm
  trap — that IS fail-closed. Every anticipated fallible path returns a typed `i32` status.
- Handle: `u64 = (generation:u32) << 32 | slot:u32`; `MAX_SESSIONS = 8` slots, fixed static
  table, no allocation after init.

## Step 2: Build the artifact

```sh
just guest-build
```

which runs:

```sh
timeout 300s cargo build --release -p ctt-wasm-guest --target wasm32-unknown-unknown
```

Artifact: `target/wasm32-unknown-unknown/release/ctt_wasm_guest.wasm` — the same path as the
`DEFAULT_GUEST_ARTIFACT` constant in `src/nato/guest_audit.rs`, so the audit court finds it
without configuration.

## Step 3: Census the import section (W4 court)

```sh
just guest-audit
```

which runs:

```sh
timeout 60s cargo test --features nato --lib nato::guest_audit -- --nocapture
```

The court (`src/nato/guest_audit.rs`) parses the import section **directly from the wasm
bytes** — a hand-rolled LEB128 section walk, no `wasm-tools`/`wasm-objdump` dependency, no
`std::process` in the production path. Its assertion over the real artifact: the import
census must be empty (`receipt.imports == Vec::new()`).

Two honesty properties, both landed:

- **Honest skip**: artifact missing -> typed
  `GuestAuditStatus::Skipped { reason: SkipKind::ArtifactMissing { path } }`, never a silent
  pass. The skipped census fabricates no imports.
- **Negative control**: the test `detects_nonzero_imports_in_real_wasm_bytes` hand-assembles a
  real wasm module with one import (`env::proxy_call`) and the census must report it. A census
  that reported zero imports over those bytes would be lying — the control is what gives the
  zero-verdict its bits.

The receipt (`GuestImportReceipt`) is serde-JSON round-trippable and carries `imports`,
`sections_checked`, `status`, and the tool provenance label.

## Step 4: Drive the guest natively

The ABI module is compiled under `#![cfg(all(feature = "wasm-guest", feature = "conformance"))]`
and is a plain native module on host targets, so the full contract suite runs without a wasm
runtime:

```sh
just guest-test
```

which runs the guest crate's tests plus the fault-injection suite:

```sh
cargo test -p ctt-wasm-guest --features chicago-tdd-tools/wasm-guest
cargo test --features wasm-guest,conformance,ledger --test guest_abi_tests
```

## Step 5: Bounded slices via `step_budget`

The reduction-yield contract (`src/conformance/guest_yield.rs`): the guest **never blocks the
VM**. Each entry runs at most one bounded reduction slice, then returns an honest status.
`ConformanceSession::step_budget(quota)` (`src/conformance/session.rs`) replays at most
`quota` events and returns a `YieldStatus`:

- `Yielded { remaining_events }` — quota exhausted mid-stream; the host reschedules with
  `resume(quota)` (or another `step_budget` call, which resumes a yielded session).
- `Complete(ReplaySummary)` — terminal; carries `steps_executed`, `events_total`,
  `final_marking`, `outcome`.
- `Violation(ConformanceError)` — a classified error latched `Phase::Faulted`; the session is
  frozen from there. An empty bound stream is malformed at step time (the X2 trace law), never
  a vacuous Complete.

Constants: `DEFAULT_REDUCTION_QUOTA = 2000` (BEAM's classical per-process reduction budget),
`MAX_REDUCTION_QUOTA = 1_000_000`. `YieldDirective::new` refuses `0` (`ZeroQuota`) and
over-ceiling (`OverCeiling`) with typed refusals. The BEAM supervisor owns policy;
`next_quota(_)` is a v1 placeholder returning the constant 2000.

## Step 6: Seal

`ctt_session_seal` writes the 32-byte BLAKE3 digest of the fixed 34-byte preimage
(8B LE `steps_executed` | 8B LE `events_total` | 8B LE `final_marking` | 1B outcome |
1B violation code | 8B zero reserved). With the `ledger` feature off, sealing is **refused**
with `ERR_SEAL_UNAVAILABLE` rather than downgraded — a non-cryptographic "receipt" is a
receipt-shaped lie, so v1 ships no fallback digest. Violation codes are the 0..=8 table in
`guest.rs::violation` (NONE, SKIPPED, OUT_OF_ORDER, UNDECLARED, FORBIDDEN_BIT,
TRACE_MALFORMED, TOO_MANY_PLACES, MODEL_TOO_LARGE, OTHER).

## Step 7: What you built

- `ctt_wasm_guest.wasm` built at the audited path via `just guest-build`.
- The W4 import census over the real artifact via `just guest-audit`, with honest-skip and a
  negative control.
- Native ABI contract + fault-injection confidence via `just guest-test`.
- Bounded, resumable replay slices with a 2000-reduction default quota.
- A BLAKE3-sealed, feature-gated replay receipt.

Known, disclosed limitation (from the guest crate manifest): the wasm32-only `uuid` dependency
with the `js` feature does not by itself achieve the empty-import-section law; the honest fix
is making `uuid` optional in the root manifest and gating the two call sites. The disclosure
is recorded in `crates/ctt-wasm-guest/Cargo.toml`.
