# Tutorial: Drive a Temprun Session from a Saga Step
> **FRESHNESS NOTE (2026-10-12):** the W-ABI2 bridge rebase has landed (strata
> 59ed80c) — the 'OPEN ABI re-base' framing and the 352-byte Sealed wire rows
> below are superseded; Sealed now carries a 32-byte digest only.

> Landed reality as of stratus v26.10.x. Every surface here is read from
> `/Users/sac/strata/stratus/lib/stratus/temprun/worker.ex` and
> `/Users/sac/strata/stratus/lib/stratus/application.ex`.

## What you will build

A Reactor-style saga step (`use Reactor.Step`, `run/3`) that starts a
`Stratus.Temprun.Worker`, drives the guest to completion, and seals the
session — all through the worker's real client API.

## Before you start

- A `temprun_step` host binary: `:step_bin` opt, or `STRATUS_TEMPRUN_STEP_BIN`
  (absolute path), or the documented default `priv/temprun_step` relative to
  the `:stratus` app dir (worker.ex:147-148, 489-510).
- Guest artifact bytes for `:model` (required, non-empty binary).
- A positive integer `:quota` (required).

## 1. Start the worker

```elixir
{:ok, worker} =
  Stratus.Temprun.Worker.start_link(model: wasm_bytes, quota: 10_000)
```

`init/1` validates opts first: missing/empty `:model` or a non-positive
`:quota` is `{:stop, :badarg}`; a missing bridge binary is
`{:stop, :bridge_down}` — fail-closed at boot, never a half-open session
(worker.ex:231-257). On success `init/1` sends the `New` frame immediately
and the worker holds in `:init` until the handle arrives.

## 2. The `:gen_statem` lifecycle

The worker is a `:gen_statem` with `callback_mode: :handle_event_function`.
Trajectory:

```
:init -> :stepping -> :yielding -> :sealed
```

- `:init` — `New` frame sent, waiting for the `Newed` reply carrying the
  u64 session handle. A `step/1` here refuses `{:error, :not_initialized}`,
  constant-time, state preserved (worker.ex:266-269).
- `:stepping` — handle in hand. `step/1` sends `Step(handle, quota)` and
  parks the caller in `pending`.
- `:yielding` — a reply on the Yielded status lane (0) hibernates the
  worker with a 5s `{:timeout, :yield}` (worker.ex:169, 432-436). The next
  `step/1` returns it to `:stepping` via
  `{:next_event, :internal, {:retry_step, from}}` (worker.ex:290-296).
- `:sealed` — terminal. Either `seal/1` succeeded or a refusal sealed the
  session. Every later call is refused with the recorded refusal atom
  (worker.ex:359-361); `registers/1` stays readable for observability
  (worker.ex:349-357).

Only the Yielded lane (0) drives a transition. Complete (1) stays live and
sealable; Violation (2) / Trap (3) keep the session live but latch the
host-side Faulted phase — the next `Step` returns an `Error` frame, which
takes the fail-closed path (worker.ex:156-159, 437-441).

## 3. Framed step calls from a saga step

```elixir
defmodule MyApp.DriveGuest do
  use Reactor.Step

  @max_steps 4096

  @impl true
  def run(arguments, _context, _options) do
    with {:ok, worker} <-
           Stratus.Temprun.Worker.start_link(
             model: arguments.model,
             quota: arguments.quota
           ),
         {:ok, word} <- drive(worker, 0),
         {:ok, %{digest: digest}} <- Stratus.Temprun.Worker.seal(worker) do
      {:ok, %{word: word, digest: digest}}
    end
  end

  defp drive(_worker, n) when n > @max_steps, do: {:error, :sealed}

  defp drive(worker, n) do
    case Stratus.Temprun.Worker.step(worker) do
      # Yielded (0): quota consumed; the caller re-arms with the next step.
      {:ok, %{status: 0}} -> drive(worker, n + 1)
      # Complete (1)
      {:ok, %{status: 1, word: word}} -> {:ok, word}
      # Violation (2) / Trap (3): keep stepping; the latched Faulted phase
      # makes the next Step return an Error frame -> fail-closed refusal.
      {:ok, %{status: s}} when s in [2, 3] -> drive(worker, n + 1)
      {:error, refusal} -> {:error, refusal}
    end
  end
end
```

Two bounds to keep straight:

- The worker's 5s yield timeout only returns the worker from `:yielding` to
  `:stepping` — it does not advance the guest.
- The Yielded loop terminates only when the caller re-arms. Bind your
  saga's retry budget to your own bound (`@max_steps` above), not the
  worker's timeout.

## 4. Seal

```elixir
{:ok, %{digest: <<_::256>>, wire: wire}} = Stratus.Temprun.Worker.seal(worker)
```

`seal/1` returns `{:ok, sealed_receipt()}`: `%{digest: <<_::256>>,
wire: <<_::2816>> | nil}` — the 32-byte BLAKE3 digest plus the 352-byte
receipt wire **iff the guest produced one** (status == 0 in the Sealed
reply, worker.ex:444-456). After seal, every call refuses with `:sealed`
(or the recorded refusal); `registers/1` remains readable post-seal.

## 5. Verify

Real checks on a live node:

```elixir
# Handle present once the Newed reply landed
%{handle: h} = Stratus.Temprun.Worker.registers(worker)
true = is_integer(h) and h != 0

# Out-of-trajectory step before init completes: constant-time atom
{:error, :not_initialized} = Stratus.Temprun.Worker.step(worker)

# After seal: registers stay inspectable
{:ok, _} = Stratus.Temprun.Worker.seal(worker)
%{receipt: %{digest: d}, refusal: nil} = Stratus.Temprun.Worker.registers(worker)
true = byte_size(d) == 32
```

## Divergence note (OPEN, disclosed in worker.ex:53-63)

The bridge's v1 spec assumes guest ABI `ctt_session_step(quota: i32) ->
i64` (`errno << 56 | word`); chicago-tdd-tools' landed guest ABI
(`src/conformance/guest.rs`) uses `ctt_session_step(handle: u64,
quota: u32, out_status: *mut u8, out_cap: u32) -> i32` with a host-written
out-buffer. The wire framing tracks the bridge (canonical). These surfaces
must be reconciled before end-to-end against a real guest; the reconcile
item belongs to the strata build lane. Until then, `step/1` against a
`temprun_step` built from this repo's guest is an OPEN ABI re-base, not a
landed end-to-end surface.

## See Also

- `02-how-to.md` — bridge-failure recovery runbook
- `03-reference.md` — full wire protocol
- `/Users/sac/strata/stratus/lib/stratus/temprun/worker.ex`
- `/Users/sac/strata/stratus/lib/stratus/application.ex`
