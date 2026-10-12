# How To: Recover a Temprun Port Bridge

> Landed reality from `/Users/sac/strata/stratus/lib/stratus/temprun/worker.ex`
> and `/Users/sac/strata/stratus/lib/stratus/application.ex`.

## The problem

The guest runs out-of-process under the `temprun_step` host binary, reached
over a raw-mode port. The bridge can die at any moment: the binary is
missing at boot, the OS reaps it, stdout goes away mid-session, or it
emits a frame that does not decode. Every one of those must end in a typed
refusal, never a hang, never an exception crossing the boundary
(worker.ex:68, 70-78).

## Typed refusal atoms

The refusal vocabulary is fixed, zero-entropy atoms mirroring
`Stratus.Valve` — no data-dependent strings (worker.ex:81-84):

| atom | when |
|---|---|
| `:bridge_down` | port exit / EOF mid-session, `Error` frame, undecodable frame, over-1-MiB length prefix, missing bridge binary |
| `:sealed` | any call after a successful seal |
| `:not_initialized` | `step/1` or `seal/1` before the `Newed` handle arrives |
| `:malformed_frame` | defensive catch-all for an unknown call in an unexpected state (worker.ex:364-366) |

Calls out of trajectory refuse with a fixed atom, constant-time, state
preserved (worker.ex:79-80).

## Recovery runbook, by failure mode

### 1. Bridge binary missing at boot

`init/1` resolves `:step_bin` opt > `STRATUS_TEMPRUN_STEP_BIN` >
`priv/temprun_step`. A missing default logs a warning and
`{:stop, :bridge_down}` (worker.ex:492-510). Recovery: no restart
hammers the same wall — set `STRATUS_TEMPRUN_STEP_BIN` to the absolute
path (or pass `:step_bin`), then start the worker again. `start_link/1`
fails `{:stop, :bridge_down}` cleanly; nothing half-open survives.

### 2. Port death mid-session (EOF, exit status, EXIT signal)

All three arrive as distinct messages and land in one lane
(worker.ex:326-340):

- `{port, :eof}` — stdout gone.
- `{port, {:exit_status, code}}` — port death as a data message (the
  `:exit_status` port option).
- `{:EXIT, port, reason}` — the exit signal.

Each marks `bridge_dead: true` and calls `seal_with_refusal/2` with
`:bridge_down`. `bridge_dead` matters: `close_bridge/1` sends no `Drop`
frame to a known-dead bridge — writing it would EPIPE-kill the worker
process (worker.ex:541-556). Recovery is automatic and terminal: any
pending caller is replied to FIRST (`reply_pending/2`), the port is torn
down, the worker enters `:sealed` with `refusal: :bridge_down`. A fresh
session is a fresh `start_link/1`.

### 3. Bad frame mid-session

Wrong tag, wrong size, an `Error` (0xFF) frame, or a length prefix over
the 1 MiB cap: same refusal path as port death
(worker.ex:417-422, 462-466). An oversize length is refused **without
buffering the body**, mirroring the bridge's `E_FRAME_TOO_LARGE` lane
(worker.ex:374-406). A zero length prefix refuses too. The frame
accumulator `buf` is cleared on every refusal path so no stale partial
frame survives into another session.

### 4. Stalled bridge (session wedged in `:yielding`)

The `:yielding` state carries a 5s `{:timeout, :yield}`; on expiry the
worker returns to `:stepping` and the next `step/1` re-arms the bridge
round-trip (worker.ex:166-169, 320-322). This is the bounded window: a
stalled bridge cannot wedge the session forever — the next round-trip
hits either a reply or a death message.

## Keeping the tree healthy

### Supervision posture: `:one_for_one` + `:temporary`

From `application.ex:11-27` (disclosed deviation from `:one_for_all`):

- A temprun worker crash must not take the registry and valve down — the
  valve's witness registers (`parent_digest`/`commit_hash`) are session
  state a sibling's bridge failure has no causal claim over.
- The worker is `:temporary` (`child_spec/1` sets `restart: :temporary`,
  worker.ex:192-198): **no restart**. A temprun session is an admitted
  transition, not a service. A dead session re-sealed with `:bridge_down`
  is not resurrected into the same tree slot; a fresh one is started
  explicitly.
- The worker child is only added to the tree when
  `Application.get_env(:stratus, :temprun_worker)` returns opts. Without
  config the child is absent — fail-closed at boot-time composition
  (application.ex:42-47).

### What this means operationally

1. **Do not wrap the worker in `:one_for_all`.** Fault isolation is the
   landed design; a bridge death is session-local.
2. **Do not rely on the supervisor to restart a session.** `:temporary`
   means a dead worker stays dead. Restart is the caller's explicit
   `start_link/1` — re-read `registers/1` on the old pid only for the
   recorded refusal atom (readable post-seal), then start fresh.
3. **Never send to the port yourself.** All writes go through the
   worker's frame codec. Manual `Port.command/2` from another process
   corrupts the frame stream.
4. **Check the refusal, not the state.** `registers/1` is the
   observability surface: `handle`, `receipt`, `refusal`. A sealed
   session's refusal atom is the diagnosis; the recovery action is
   always "start a new session with corrected inputs."

### The one recovery invariant

Every client call is replied to before the port is torn down
(worker.ex:73-74). If you observe a `step/1` or `seal/1` caller hanging,
that is a bug in this membrane, not an operational state — the landed
code replies pending callers first in `seal_with_refusal/2`
(worker.ex:560-566) and in every Sealed decode path.

## See Also

- `01-tutorial.md` — driving a session
- `03-reference.md` — wire protocol
- `/Users/sac/strata/stratus/lib/stratus/application.ex`
