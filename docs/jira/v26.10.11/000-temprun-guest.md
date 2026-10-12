# chicago-tdd-tools: temprun-guest integration

- Standing: PARTIAL_ALIVE
- Created: 2026-10-11 (v26.10.11 Phase-W lane W13)
- Scope: integrate the CTT WASM guest (`crates/ctt-wasm-guest`) into the strata
  temprun surface: guest artifact, ABI, gen_statem adapter, and sealed PQ
  receipts, per PRD §1-4.

## Evidence
- PRD §1-4 directives:
  - Zero ambient authority — the guest imports nothing (`imports == ∅`); all
    capability flows in explicitly through the ABI.
  - Reduction budgets — guest execution is bounded per reduction; the host
    enforces the budget, the guest cannot exceed it.
  - Trajectory enforcement — session state advances only through the
    init → stepping → yielding → sealed trajectory; out-of-order transitions
    are refused.
  - PQ receipts — post-quantum sealed receipts minted at the sealed boundary
    and persisted host-side.
- Three immediate next steps (PRD):
  1. WASM guest embed — `ctt-wasm-guest` builds to `wasm32` and is loadable by
     the temprun loader.
  2. OTP `gen_statem` adapter — `temprun_worker` models the session as
     `init → stepping → yielding → sealed`.
  3. Fail-closed trap verification — N09/N11 courts trap against a live
     session and fail closed on any violation.

## Work to complete
CTT-side deliverables (landed):
- `crates/ctt-wasm-guest` guest crate present in-tree.
- Guest artifact + ABI contract exported for the temprun loader to consume.

Strata-side deliverables (landed):
- `temprun` loader wires the guest artifact:
  `/Users/sac/strata/temprun/src/bin/temprun_step.rs`.
- `temprun_worker` `gen_statem` adapter landed in strata:
  `/Users/sac/strata/stratus/lib/stratus/temprun/worker.ex`.
- Sealed PQ receipt persistence landed host-side:
  `/Users/sac/strata/stratus/lib/stratus/valve/receipt_store.ex`.
- Live-session courts landed: `/Users/sac/strata/stratus/test/strata_n09_single_bit_flip_court.exs`
  and `/Users/sac/strata/stratus/test/strata_n11_live_session_tear_court.exs`.

Strata-side deliverables (remaining):
- Guest-ABI re-base in `temprun_step.rs` — REOPENED (doc-vs-code divergence,
  witnessed this lane 2026-10-11): the file's own ABI-mapping doc-comment
  table (`/Users/sac/strata/temprun/src/bin/temprun_step.rs:86-92`) claims
  the new i32 out-buffer ABI (`ctt_session_step(handle, quota, out_status,
  out_cap) -> i32`, `ctt_session_seal(handle, out_buf, out_cap, out_len) ->
  i32`), but the code still speaks the old packed-i64 ABI: step is
  `get_typed_func::<i32, i64>` unpacking `errno << 56 | word`
  (~`:321-340`) and seal is `get_typed_func::<(), i64>` returning
  `ptr << 32 | len` (~`:359`). The doc-comment describes the landed
  chicago-tdd-tools guest surface; the bridge code implements the retired
  one. Strata-side work order — the strata 1ac493d receipt does NOT close
  this item: 1ac493d landed worker.ex/receipt_store/courts, not an
  out-buffer ABI migration of temprun_step.rs. No strata edits from this
  lane (see Constraints).
- WASM artifact build (`wasm32` cdylib) + zero-import (`imports == ∅`)
  proof run.
- `mix compile` of stratus + `elixirc` of the court `.exs` files.
- Live N09/N11 court runs against a live session (the `.exs` files are
  landed but not yet executed).
- Guest ABI test suite: `cargo test --features wasm-guest,conformance,ledger
  --test guest_abi_tests` is GREEN on this tree as of 2026-10-11 (R17 fix):
  13 passed, 0 failed. The R13-era 2/13 RED (quota-0 idle step) is closed —
  the quota-0 byte-2 behavior is the RS1 trace law (X2: quota-0 on an idle
  session latches Faulted(TraceMalformed) and reports Violation, status byte
  2), and the test now asserts byte 2. The prior "RED / 13-does-not-
  reproduce" receipt is stale; treat the suite as green under the full
  feature set.
- `--features wasm-guest` alone (no conformance/ledger) was blocked by
  `-D dead_code`: the test-local helper `seal_preimage`
  (`tests/guest_abi_tests.rs:266`) was only exercised by the two
  `#[cfg(feature = "ledger")]` seal tests, so under wasm-guest-only the
  helper was unused. Lane R20 gated the helper `#[cfg(feature = "ledger")]`
  (matching its only users — no coverage deleted); post-fix measured:
  wasm-guest-only 12 passed / 0 failed (exit 0), full feature set still
  13 passed / 0 failed (exit 0). CLOSED.

## Acceptance
- Guest artifact builds for `wasm32` with `imports == ∅` — VERIFIED
  (2026-10-11, A47 audit): the built module binary has no import section
  (zero imports, read from the wasm bytes, not asserted).
- `temprun` loader wires and instantiates the guest artifact.
- `temprun_worker` `gen_statem` adapter lands in strata with the
  init → stepping → yielding → sealed trajectory.
- N09/N11 courts pass against a live session, and fail closed on a trapped
  trajectory violation.
- Sealed PQ receipts persist BEAM-side (readable back from the strata store,
  not only emitted in-process).

## Constraints
- The strata checkout is dirty on unmerged fix/kg-42-status-code — integration
  lands in strata only after strata preconditions clear.
- CTT side produces the artifact + ABI only; no strata edits from this lane.

## History
- 2026-10-11 | OPEN | lane W13 work order written | CTT-side artifact + ABI
  landed in-tree; strata-side wiring pending E14's patch plan.
- 2026-10-11 | OPEN | lane S8 refresh (A-jira staleness finding) | strata-side
  wiring landed (temprun_step, worker.ex, receipt_store.ex, live courts);
  CTT-side guest surface landed; remaining: ABI re-base + artifact build +
  live-court execution.
- 2026-10-11 | PARTIAL_ALIVE | lane R13 status refresh | ABI re-base closed:
  guest surface in `src/conformance/guest.rs` is the typed `i32` out-buffer
  ABI (verified from `#[no_mangle]` exports `ctt_session_new/attach/step/seal`
  returning `i32` with `OK`/`ERR_*` status constants); strata bridge landed
  at strata 1ac493d. Guest ABI test suite measured RED on this tree
  (`--features wasm-guest`: compile fail on dead_code; `--all-features`:
  2/13 pass, 1 real assertion failure at `tests/guest_abi_tests.rs:513` +
  ABI_LOCK poison cascade) — suite green is not witnessed, prior "13 green"
  receipt does not reproduce. Remaining: fix ABI/test contract mismatch,
  wasm32 build + imports==∅ proof, stratus compile, live N09/N11 court runs.
- 2026-10-11 | PARTIAL_ALIVE | lane R17/R20/A47 refresh | guest_abi_tests
  GREEN 13/13 under `--features wasm-guest,conformance,ledger` (R17: the
  quota-0 byte-2 behavior is the RS1 trace law X2, test contract updated);
  lane R20 gated the ledger-only helper `seal_preimage`
  (`#[cfg(feature = "ledger")]`) so `--features wasm-guest` alone compiles:
  12/0 there, 13/0 full — both witnessed this session, exit 0. A47 audit:
  wasm32 build + imports==∅ VERIFIED from the module binary's import
  section. ABI re-base item REOPENED: `temprun_step.rs` doc-comment table
  (:86-92) claims the i32 out-buffer ABI while code still speaks
  packed-i64 (errno<<56|word :321, ptr<<32|len :359) — strata-side work
  order; strata 1ac493d does not close it. Remaining: strata ABI re-base,
  stratus compile, live N09/N11 court runs.
