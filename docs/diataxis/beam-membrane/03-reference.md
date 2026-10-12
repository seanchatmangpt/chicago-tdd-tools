# Reference: The Temprun Port Protocol

> Landed reality: `/Users/sac/strata/stratus/lib/stratus/temprun/worker.ex`
> (Port protocol v1) aligned to
> `/Users/sac/strata/temprun/src/bin/temprun_step.rs`. The Φ_lease wire
> carriage is **designed, not landed** — see the last section.

## Framing: 4-byte LITTLE-endian length prefix, hand-rolled

Every frame, both directions:

```
[u32 LE total_len][body]
```

`total_len` counts the tag byte plus payload bytes; `0 < len <= 1 MiB`.

The framing is hand-rolled because the BEAM `{:packet, 4}` port option is
**big-endian** and the bridge speaks little-endian framing. The port is
therefore opened in raw stream mode with no `:packet` option —
`Port.open({:spawn_executable, path}, [:binary, :exit_status, :eof])`
(worker.ex:512-523) — and both sides read/write the u32 LE prefix in
hand-rolled codec functions: `with_frames/3` / `extract_frames/3` on the
BEAM side (worker.ex:374-406), `send_frame/1` packing
`<<byte_size(payload)::little-32, payload::binary>>` (worker.ex:525-529),
and the equivalent reader in `temprun_step.rs` (header comment, lines
10-15).

## Frame tags

Requests (host → guest) and replies (guest → host) share one tag space.

| tag | direction | name | payload |
|---|---|---|---|
| `0x01` | host → guest | `New` | model bytes (normative guest artifact, verbatim) |
| `0x02` | host → guest | `Step` | `handle::little-64, quota::little-32` |
| `0x03` | host → guest | `Seal` | `handle::little-64` |
| `0x04` | host → guest | `Drop` | `handle::little-64` |
| `0x01` | guest → host | `Newed` | `handle::little-64` |
| `0x02` | guest → host | `Stepped` | `handle::little-64, status::u8, record::21 bytes` |
| `0x03` | guest → host | `Sealed` | `handle::little-64, status::u8, dlen::little-16, digest::dlen bytes, wire::352 bytes?` |
| `0x04` | guest → host | `Dropped` | `handle::little-64, status::u8` |
| `0xFF` | guest → host | `Error` | `errno::little-32` |

(`Sealed` note: `temprun_step.rs` line 50 documents the digest field as
34 bytes total — `u16 LE len(=32) || BLAKE3 digest`; the BEAM decoder
reads `dlen` generically and accepts `rest` of `<<>>` or exactly 352
bytes, worker.ex:444-447.)

## The 21-byte step record

Carried in every `Stepped` reply (worker.ex:470-481):

```
offset  size  field
0       8     word            :: little-u64  (guest word, errno high bits stripped)
8       4     quota_remaining :: little-u32
12      4     steps_done      :: little-u32
16      1     phase           :: u8
17      4     reserved        :: little-u32 (=0)
```

Phase values: 0 Idle, 1 Running, 2 Yielded, 3 Complete, 4 Faulted
(latching).

## Status lanes

Mirror `YieldStatus` (worker.ex:38-39, 156-160):

| value | lane | worker behavior |
|---|---|---|
| 0 | Yielded | worker hibernates in `:yielding` (5s timeout), caller re-arms with the next `step/1` |
| 1 | Complete | stays live and sealable — `New → Step(Complete) → Seal` is the happy path |
| 2 | Violation | stays live; host-side Faulted phase latches |
| 3 | Trap | same latch path |

## Errno vocabulary (`Error` frames)

(worker.ex:47-51)

| errno | meaning |
|---|---|
| 1 | trap |
| 2 | unknown handle |
| 3 | missing export |
| 4 | malformed frame |
| 5 | frame over 1 MiB |
| 6 | invalid module |
| 7 | instantiation failure |
| 8 | seal not produced |
| 9 | fuel error |
| 10 | session faulted (terminal) |

Any `Error` frame is fail-closed at the BEAM layer: refusal
`:bridge_down`, session seals with the refusal recorded.

## Stream-decode rules

The BEAM side accumulates raw stream bytes in `buf` and extracts frames
incrementally (worker.ex:379-406):

- fewer than 4 bytes: held for the next `{:data, bytes}` delivery.
- length prefix present but body incomplete: held.
- `len == 0` or `len > 1 MiB`: refused (`:bridge_down`) **without
  buffering the body** — mirroring the bridge's `E_FRAME_TOO_LARGE` /
  `E_BAD_FRAME` lanes.
- any reply that does not decode exactly (wrong tag, wrong size,
  out-of-trajectory reply, oversize frame) is refusal `:bridge_down` and
  the worker seals with the refusal recorded. No exceptions cross this
  boundary (worker.ex:65-68).

## Binary-reference discipline

Model bytes and frames stay refc binaries; pattern-matched fields are
sub-binary references. Nothing larger than the 32-byte digest and the
optional 352-byte receipt wire is ever copied onto the worker heap
(worker.ex:86-91).

## Φ_lease on the wire — DESIGNED, NOT LANDED

**Status: designed, not landed.** There is no lease field anywhere in the
landed port protocol: no frame tag carries lease data, the 21-byte step
record has a `reserved` field but no lease semantics, and `worker.ex`'s
refusal vocabulary contains no lease atom. Do not code against a lease
wire on this membrane.

What exists landed, on a different surface: the Φ_lease **conservation
identity** `ΔS_admitted + Φ_lease ≡ R_receipt` in the stratus merger
lease ledger (`/Users/sac/strata/stratus/lib/stratus/merger/lease_ledger.ex`,
E-M19 conservation audit) — merger-side, not port-side. Any future port
carriage of Φ_lease is design work against that identity; until it lands
in `worker.ex` and `temprun_step.rs` jointly, this membrane's lease wire
is a design placeholder, not a reference surface.

## See Also

- `01-tutorial.md`, `02-how-to.md`
- `/Users/sac/strata/temprun/src/bin/temprun_step.rs` (host bridge, 604 lines)
- `/Users/sac/strata/stratus/lib/stratus/temprun/worker.ex`
