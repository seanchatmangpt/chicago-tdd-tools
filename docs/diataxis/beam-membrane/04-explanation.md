# Explanation: Why a Port Subprocess, Not a NIF

> Grounded in `/Users/sac/strata/stratus/lib/stratus/temprun/worker.ex`,
> `/Users/sac/strata/temprun/src/bin/temprun_step.rs`, and the landed
> chicago-tdd-tools guest ABI (`src/conformance/guest.rs`).

## The choice

The guest (a WASM session executing bounded steps under a fuel/quota
budget) runs **out-of-process** under the `temprun_step` host binary,
reached over a raw-mode port with hand-rolled 4-byte LE length-prefixed
frames on stdin/stdout (worker.ex:2-14). The obvious alternative — wrap
wasmtime in a NIF — was not taken. Three reasons, in decreasing weight.

## 1. Dirty-scheduler contention

A WASM step is exactly the workload dirty NIFs exist for but handle
badly at fleet scale: multi-millisecond CPU-bound execution with
quota consumption measured in reductions' worth of guest compute. A NIF
occupies a scheduler thread for its whole duration; dirty schedulers have
a fixed small pool, so N concurrent sessions contend on `N : dirty_pool`
instead of scaling with the scheduler count. The port design inverts
this: each session's guest compute happens on the subprocess's own
thread, and the BEAM-side worker process only wakes for frame I/O — a
few microseconds of pattern matching per round-trip. The worker's
`step/1` is a `:gen_statem.call` that parks the caller; the scheduler
never blocks on guest execution.

## 2. Memory isolation

The guest artifact is untrusted input (the "normative guest artifact",
worker.ex:23-24). Under a NIF, a wasmtime fault, an allocation failure,
or any C-level memory error in the host embedding corrupts or takes down
the whole BEAM VM — every session, the valve, and the registry with it.
Under the port design, the blast radius is one subprocess: port death
arrives as `{port, {:exit_status, _}}` or `{:EXIT, port, _}` and is
handled as an ordinary fail-closed event (worker.ex:326-340) — the
session seals with `:bridge_down`, the supervisor tree is untouched
(`:one_for_one`, worker child `:temporary`, application.ex:11-27). The
kernel does not share an address space with the workload it evaluates.

## 3. Preemption across reduction slices

BEAM preemption is cooperative: a process yields at reduction boundaries.
A NIF — dirty or not — is opaque to the reduction counter; a long guest
step cannot be preempted mid-flight, only bounded at its edges. The port
design makes preemption a **wire-level** property instead: the `Step`
frame carries an explicit u32 quota, the bridge enforces the budget
host-side, and the Yielded status lane (0) hands control back with the
21-byte record reporting `quota_remaining` and `steps_done`
(worker.ex:38-44, 470-481). The `:yielding` state with its 5s timeout is
the BEAM-side echo of that handoff — the yield is enacted by the state
machine, not narrated by a timer loop (worker.ex:95-99). The granularity
of preemption becomes the step, chosen by the caller, instead of
whatever the guest happens to do between traps.

## What the membrane costs

Honest accounting of the trade:

- One `Port.command/2` round-trip per step (copy in, copy out) —
  negligible at step granularity, material only if steps were
  microsecond-scale.
- A framing codec both sides must agree on byte-for-byte (see
  `03-reference.md`).
- Process-per-session lifecycle: spawn cost per `start_link/1`, paid
  once per session, aligned with `:temporary` semantics (a session is an
  admitted transition, not a service).

## Current membrane status: OPEN ABI re-base

The membrane is landed on the BEAM side and bridge side, but **not yet
exercised end-to-end against this repo's guest**, because the two ABI
surfaces disagree (worker.ex:53-63, `temprun_step.rs` "Guest ABI (v1)"
table):

- Bridge v1 spec: `ctt_session_step(quota: i32) -> i64`, packing
  `errno << 56 | word`.
- chicago-tdd-tools landed guest ABI: `ctt_session_step(handle: u64,
  quota: u32, out_status: *mut u8, out_cap: u32) -> i32` with a
  host-written out-buffer.

The wire framing tracks the bridge (canonical). The reconcile item
belongs to the strata build lane. Until it lands, the honest claim is:
**framing, lifecycle, supervision posture, and fail-closed lanes are
landed; the guest ABI hop is an open re-base.** Every other claim this
doc makes is readable in the cited files.

## See Also

- `01-tutorial.md` — session lifecycle in practice
- `03-reference.md` — the wire the two sides must agree on
- `/Users/sac/strata/temprun/src/bin/temprun_step.rs`
