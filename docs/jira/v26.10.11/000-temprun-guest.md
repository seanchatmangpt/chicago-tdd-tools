# chicago-tdd-tools: temprun-guest integration

- Standing: OPEN
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
- Guest-ABI re-base in `temprun_step.rs`: the bridge assumes the
  `errno << 56 | word` i64 ABI; the landed guest surface
  (`src/conformance/guest.rs`, re-exported via `crates/ctt-wasm-guest`) is the
  `(ptr: *const u8, len: u32) -> i32` out-buffer ABI with typed `i32` status
  codes — per A-S1's delta table.
- WASM artifact build (`wasm32` cdylib) + zero-import (`imports == ∅`)
  proof run.
- `mix compile` of stratus + `elixirc` of the court `.exs` files.
- Live N09/N11 court runs against a live session (the `.exs` files are
  landed but not yet executed).

## Acceptance
- Guest artifact builds for `wasm32` with `imports == ∅` (verified from the
  module's import section, not asserted).
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
