# Reference: merger-guests ABI As Landed

This is the reference for `crates/merger-guests` **as landed on disk**. Where
an earlier specification (E-M-series plan, operator summary) disagrees with
the code, the code wins and the divergence is flagged.

## Divergences from the operator's spec (read first)

- **"7 guests" is wrong — the landed crate has 8 leaf kernels.** Landed
  leaves: `hhi_calc`, `netting_engine`, `simm_matrix`, `collateral_lp`,
  `vote_reconciler`, `netting_settler`, `fractional_shares`, `pn64_audit`
  (8 `mgi_*` entry points; 10-slot leaf-id table with 5/6 reserved; tests in
  `lib.rs` pin `sorted == [1,2,3,4,7,8,9,10]`).
- **`S_MONEY` is 10^6, not 10^2.** `fixed.rs:31`: `S_MONEY: i128 =
  1_000_000` (micro-dollars). A spec stating 10^2 (cents) diverges.
- **`S_SHARES` is 10^6, not 10^0.** `fixed.rs:37`: `S_SHARES: i128 =
  1_000_000` (millionths of a share). A spec stating whole shares (10^0)
  diverges.
- Landed scales, verbatim from `fixed.rs:31-37`: `S_MONEY=10^6`,
  `S_RATIO=10^8`, `S_HHI=10^4`, `S_SHARES=10^6`.

## Scales and tags (`crates/merger-guests/src/fixed.rs`)

| Constant | Value | Unit |
|---|---|---|
| `S_MONEY` | 10^6 | dollars in micro-units |
| `S_RATIO` | 10^8 | unit-interval ratios in 1e-8 steps |
| `S_HHI` | 10^4 | HHI points (DOJ/FTC 2023 bands: <1500, 1500-2500, >2500) |
| `S_SHARES` | 10^6 | share counts in millionths |

Tags: `TAG_MONEY=1`, `TAG_RATIO=2`, `TAG_HHI=3`, `TAG_SHARES=4`;
`scale_of(tag) -> Option<i128>` maps tag → multiplier, `None` for anything
else (`ERR_BAD_SCALE`). `Fix = i64`; intermediates are `i128`; all ops
checked; **no floating point anywhere** — the `no_floats_anywhere` test at the
bottom of `fixed.rs` greps the file for float tokens and fails the build if
any appear.

## ABI shape

Stateless, uniform, zero-import:

```text
i32 mgi_<leaf>_compute(in: *const u8, in_len: u32,
                       out: *mut u8, out_cap: u32) -> i32
```

- `0` = ok; **positive** = guest errno; **negative** = reserved for the host
  transport, never returned by a leaf.
- No session table, no handle, no global state — each call is a pure function
  of its input buffer, so any host can replay any call byte-for-byte.
- Zero-import law: the built wasm module MUST have an empty import section
  (core+alloc only, no `wasm-bindgen`, no uuid/async/otel — pinned by the
  `zero_import_law_is_documented_dependency_set` manifest test and
  `tests/zero_import.rs`).
- ABI probe: `mgi_abi_version() -> u32`, stable at `MGI_ABI_VERSION = 1`.

## MGI1 header (16 bytes, LE, packed)

| Offset | Size | Field | Law |
|---|---|---|---|
| 0 | 4 | `magic` | ASCII `MGI1` else `ERR_BAD_MAGIC` |
| 4 | 1 | `leaf_id` | stable leaf identity; leaf refuses a foreign id (`ERR_BAD_MAGIC`) |
| 5 | 1 | `scale_tag` | one of the four tags else `ERR_BAD_SCALE` |
| 6 | 2 | `reserved` | must be zero (`ERR_RESERVED_NOT_ZERO`) |
| 8 | 8 | `dims` | `u64` element count; bounds checked per leaf |

Parse law order: magic → reserved → scale → (leaf checks dims/leaf_id).
`Mgi1Header::{encode, to_bytes}` is the inverse of `parse_header`.

## Leaf-id table (as landed)

| leaf_id | module | entry point | lane |
|---|---|---|---|
| 1 | hhi_calc | `mgi_hhi_calc_compute` | MG2 |
| 2 | netting_engine | `mgi_netting_engine_compute` | MG2 |
| 3 | simm_matrix | `mgi_simm_matrix_compute` | MG3 |
| 4 | collateral_lp | `mgi_collateral_lp_compute` | MG3 |
| 5 | — reserved — | — | — |
| 6 | — reserved — | — | — |
| 7 | vote_reconciler | `mgi_vote_reconciler_compute` | MG4 |
| 8 | netting_settler | `mgi_netting_settler_compute` | MG4 |
| 9 | fractional_shares | `mgi_fractional_shares_compute` | MG4 |
| 10 | pn64_audit | `mgi_pn64_audit_compute` | MG4 |

## errno table 0..=19 (`crates/merger-guests/src/errno.rs`)

Codes 0-13 are the landed `ctt_*` guest semantics (do not renumber; hosts key
on these). Codes 14-19 are merger-guest extensions appended in the next free
range.

> **ABI v2 collision (open, code-owned).** The `ctt_*` guest ABI is now v2
> (`src/conformance/guest.rs`, probe returns 2), which defines `ERR_OOB_READ = 14` and
> `ERR_OOB_WRITE = 15` — numeric values this merger-guest extension range already occupies
> (`ERR_BAD_SCALE`/`ERR_DIM`). Also at v2, `ERR_CAP_TOO_SMALL` (11) is **pure capacity**
> again: the v1 out-span OOB overload moved to codes 14/15, so this table's row 11 below is
> correct for the merger guests but no longer matches the ctt v2 meaning. Renumbering the
> merger-guest range past 16 is a work-order item outside this doc-sync lane.

| Code | Constant | Meaning |
|---|---|---|
| 0 | `OK` | ok |
| 1 | `ERR_NULL_PTR` | required pointer/out-pointer was null |
| 2 | `ERR_BAD_MAGIC` | buffer does not start with `MGI1` |
| 3 | `ERR_BAD_VERSION` | wire version byte is not 1 |
| 4 | `ERR_TRUNCATED` | buffer shorter than declared layout requires |
| 5 | `ERR_TRAILING` | bytes present after declared payload |
| 6 | `ERR_RESERVED_NOT_ZERO` | a must-be-zero field is non-zero |
| 7 | `ERR_BAD_MODEL` | model DSL failed parse/validate/compile |
| 8 | `ERR_TABLE_FULL` | all `MAX_SESSIONS` slots occupied |
| 9 | `ERR_BAD_HANDLE` | handle free, out of range, or generation-stale |
| 10 | `ERR_STREAM_EMPTY` | attach of a zero-length stream |
| 11 | `ERR_CAP_TOO_SMALL` | out-buffer capacity below the record's fixed length (pure capacity; merger guests have no OOB span law — see the v2 collision note above) |
| 12 | `ERR_SEAL_UNAVAILABLE` | sealing without the `ledger` feature |
| 13 | `ERR_STREAM_ALIGNMENT` | stream byte length not a multiple of 8 |
| 14 | `ERR_BAD_SCALE` | scale_tag not one of the four documented scales |
| 15 | `ERR_DIM` | input dimensionality mismatch with leaf arity |
| 16 | `ERR_OVERFLOW` | fixed-point op left the i64 value domain |
| 17 | `ERR_FUEL` | reduction quota exhausted before kernel finished |
| 18 | `ERR_NO_SOLUTION` | solver found no feasible solution |
| 19 | `ERR_LENGTH` | element count outside the leaf's documented bounds |

## Landed leaf: netting_engine (leaf_id 2)

Wire (after the 16-byte header, `TAG_MONEY`): `dims` rows of 24 bytes —
`debtor_idx: u64`, `creditor_idx: u64`, `amount: i64` (S_MONEY micro-dollars,
signed). `dims == 0` or `dims > MAX_OBLIGATIONS = 10_000` → `ERR_DIM`; byte
length must be exactly `16 + 24*dims` (`ERR_TRUNCATED` / `ERR_TRAILING`).

Output: header echo, then `u32 n_rows`, `u32 n_parties`, `n_rows × 24 B` net
rows, `n_parties × 16 B` positions. Below that → `ERR_CAP_TOO_SMALL`.

Semantics: bilateral aggregation by `(debtor, creditor)` pair (zero-net pairs
drop), then multilateral per-party signed positions. Conservation invariant:
`Σ positions == 0` in i128, fail-closed `ERR_OVERFLOW` — documented as
unreachable-by-construction, retained as a wire-invariant assertion
(`Σnet == Σgross`, sign-conserved). Outputs stably sorted for byte-for-byte
replay.

## Budgets (as landed in test constants)

- **Netting fuel cap** (`netting_engine.rs:133`): `fuel_cap(rows) = rows*4 +
  64` accounted steps. Test-pinned: `fuel_cap(0) == 64`,
  `fuel_cap(10_000) == 40_064`; a legit small input never trips fuel.
- **Fault-injection fuel tier** (`tests/fault_injection.rs:1391`): the
  fuel-stub cap is `2*MAX_SHARES + 16`, which dominates the 10_000-step worst
  case — `ERR_FUEL` is unreachable-by-construction for lawful max-dimension
  inputs; the court asserts a max-dimension lawful input still succeeds.
- **Stack budget: no landed stack-budget constant exists** in
  `crates/merger-guests`. The landed budget surfaces are fuel (above) and the
  domain caps (`MAX_OBLIGATIONS = 10_000`, per-leaf dimension bounds, i64
  value domain with i128 scratch). If a spec quotes a stack budget for this
  crate, that quota is not landed here.

## Session-side constants (the replay kernel the guests mirror)

- Linear session net budget: `MAX_PETRI_PLACES - 1` = 15 transitions;
  `TooManyPlaces { limit: 16 }` beyond that (`src/conformance/session.rs:111`).
- POWL loader node cap: `MAX_POWL_NODES = 64` (`src/conformance/loader.rs:47`).
- Forbidden mask: `set_forbidden_mask(u64)`, truncated to 16-bit markings;
  read on every step — mid-stream install changes classification of subsequent steps (`session.rs:223`).
