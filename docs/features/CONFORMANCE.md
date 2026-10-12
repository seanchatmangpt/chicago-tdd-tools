# Conformance Checking (POWL Replay)

Load a POWL process model (fail-closed, positioned parse errors), validate it,
compile it to a PN64 net, attach a stream of packed event words, and replay it
cooperatively against a branchless hot path. Every deviation is classified as a
typed value, never a string. Gated behind feature `conformance`
(Cargo.toml:216, wired at src/lib.rs:188-189 as
`#[cfg(feature = "conformance")] pub mod conformance;`).

Flat re-exports: `src/conformance/mod.rs:40-52`; canonical-type table in the
module docs: `src/conformance/mod.rs:3-16`.

## Error Model

One error type for the whole subsystem:
`error::ConformanceError` (src/conformance/error.rs:12-180, re-exported at
mod.rs:40).

| Variant | Fields | Cite |
|---|---|---|
| `MalformedSource` | `line, col, reason` | error.rs:18 |
| `UnknownActivity` | `name, line, col` | error.rs:29 |
| `DuplicateActivity` | `name, line, col` | error.rs:40 |
| `InvalidJson` | `message` | error.rs:51 |
| `Io` | `path, message` | error.rs:58 |
| `ArityBelowMinimum` | — | error.rs:70 |
| `DependencyCycle` | `nodes` | error.rs:81 |
| `ModelTooLarge` | `found, limit` | error.rs:88 |
| `UnboundedLoop` | — | error.rs:97 |
| `UnknownNodeRef` | `side, id` | error.rs:101 |
| `WeightedArc` | `weight` | error.rs:110 |
| `TooManyPlaces` | `limit` | error.rs:117 |
| `Skipped` | `step` | error.rs:127 |
| `OutOfOrder` | `step, prev_tick, tick` | error.rs:134 |
| `Undeclared` | `step, tid` | error.rs:145 |
| `ForbiddenBit` | `step, marking, mask` | error.rs:154 |
| `TraceMalformed` | `len` | error.rs:165 |
| `BcinrValidation` | `message` | error.rs:176 |

Loader errors carry source positions (`line`/`col`); replay errors carry step
indices; marking/tick fields carry the exact values at refusal.

## Loading a POWL Model (DSL)

The canonical POWL surface is `loader::PowlModel` — the retired `model.rs`
POWL variant is test fixtures only (mod.rs:8) and is not documented here.

| Member | Notes | Cite |
|---|---|---|
| `MAX_POWL_NODES` | `pub const ...: usize = 64` | loader.rs:50 |
| `PowlModel` | `{ activities, declaration_order, root }`, pub fields | loader.rs:61 |
| `ActivityDecl` | `{ source, event }` | loader.rs:72 |
| `PowlNode` | `Sequence(Vec<_>) \| Xor(Vec<_>) \| Parallel(Vec<_>) \| Loop{body,redo,max:u8} \| Activity(String) \| Silent` | loader.rs:88-109 |
| `node_count()` | `(&self) -> usize` | loader.rs:622 |
| `xor_arity_profile()` | `(&self) -> Vec<usize>` | loader.rs:629 |
| `from_powl_str(&str)` | parse from string; node cap enforced | loader.rs:641 |
| `from_powl_file(impl AsRef<Path>)` | parse from file | loader.rs:657 |
| `from_powl_json(&str)` | parse from JSON | loader.rs:674 |
| `ParsedNet` | `{ places, transitions, arcs }` name-based mini-net | loader.rs:693 |
| `ParsedNet::from_net_str(&str)` | line-based `place p1` / `transition t1` / `arc p1 -> t1`, `#` comments | loader.rs:720 |

### DSL Grammar

Transcribed from the real parser (loader.rs:385-583; scanner loader.rs:258-342
— tokens `{ } = .` plus Word/Num; numbers 0..=255, overflow refuses
loader.rs:272-274; `#` comments to EOL loader.rs:274-279; reserved words
`activity|sequence|xor|parallel|loop|redo|max|silent` loader.rs:349-351):

```text
document      := activity_decl* root_expr EOF
                 (loader.rs:385-414; empty/missing root refuses
                  "missing root expression" loader.rs:392)
activity_decl := `activity` NAME `=` SRC `.` EVENT
                 (loader.rs:415-478; duplicate → DuplicateActivity :473)
root_expr     := `sequence` `{` children `}`
               | `xor`      `{` children `}`
               | `parallel` `{` children `}`
               | `loop`     `{` expr `}` [`redo` `{` expr `}`] [`max` NUM]
                 (loader.rs:508-534; default max = 255 :531)
               | `silent`    (loader.rs:535)
               | NAME        (must be a declared activity, else
                              UnknownActivity loader.rs:536-549)
children      := expr* until matching `}` (loader.rs:551-565)
```

### Golden Example

Verbatim from the compile-checked sample
(examples/conformance_quickstart.rs:20-31):

```text
# Three-atom linear process: handle -> fork -> approve
activity handle  = log.handle
activity fork    = robot.fork
activity approve = ledger.approve

sequence {
    handle
    fork
    approve
}
```

## Session Lifecycle

`session::ConformanceSession` (session.rs:121), flat re-exported at mod.rs:46.
Deliberate distinctions: `session::Marking64 = u64` (session.rs:65) is an
alias, distinct from the `state::Marking64` struct (state.rs:20); likewise
`session::CompiledNet` (session.rs:78) is the session-local linear net, while
the canonical `CompiledNet` is `net::CompiledNet` (mod.rs:13-14).

| Member | Notes | Cite |
|---|---|---|
| `ConformanceResult<T>` | `Result<T, ConformanceError>` | session.rs:54 |
| `Marking64 = u64` alias | distinct from `state::Marking64` | session.rs:65 |
| `from_powl_file(&Path)` | load + validate + compile in one call | session.rs:156 |
| `from_model(&PowlModel)` | from an already-loaded model | session.rs:166 |
| `set_forbidden_mask(u64)` | masked to 0xFFFF; read on every step | session.rs:223 |
| `attach_stream(&[u64])` | empty stream -> `TraceMalformed{len:0}` | session.rs:240 (refusal :244-245) |
| `step_budget(u32) -> YieldStatus` | never `Err`; violations latch `Phase::Faulted` and re-report via `YieldStatus::Violation` | session.rs:268 |
| `resume(u32)` | continue after a yield | session.rs:316 |
| `phase()` | `&Phase` | session.rs:322 |
| `steps_done()` | `u32` | session.rs:328 |
| `remaining_events()` | `usize` | session.rs:334 |
| `marking()` | `Marking64` | session.rs:340 |
| `initial_marking()` | `Marking64` | session.rs:346 |
| `net()` | `&CompiledNet` | session.rs:359 |
| `finish()` | `-> ReplaySummary` | session.rs:366 |
| `conformance_report()` | `-> ConformanceReport` (macro seam) | session.rs:389 |

### Event-Word Layout and Tick Law

Event words: activity id in bits [7:0], tick in bits [15:8]
(session.rs:28-29 module docs). Ticks must advance exactly `prev + 1 (mod 256)`
(session.rs:421-434; tests/conformance_tests.rs:8-9). Classification order
(`classify_and_commit`, session.rs:417, classified inline :421-434), each class
checked explicitly, never inferred from a silent PN64 refusal:

1. Tick order: `OutOfOrder` when `tick == prev`, `Skipped` when not
   `prev + 1 (mod 256)` — the tick law is checked first.
2. `Undeclared` (activity id not in the model's activity dictionary).
3. `ForbiddenBit` (marking bit set in the forbidden mask).

Violations latch `Phase::Faulted` and re-report via `YieldStatus::Violation`
(session.rs:287-289).

## Report Types

All verdict types are homed in `report.rs` (E1 item-2 reconcile); `session`
re-exports them for path stability.

| Type | Shape | Cite |
|---|---|---|
| `ReportOutcome` | `Complete \| Faulted(ConformanceError) \| Incomplete` | report.rs:80 |
| `ReplaySummary` | `{ steps_executed: u32, events_total: usize, final_marking: u64, outcome: ReportOutcome }` | report.rs:97 |
| `YieldStatus` | `Yielded{remaining_events} \| Complete(ReplaySummary) \| Violation(ConformanceError)` | report.rs:111 |
| `Phase` | `Idle \| Running \| Yielded \| Complete \| Faulted(ConformanceError)` | report.rs:125 |
| `Deviation` | `VerbMismatch{step,tid,expected} :20, NotEnabled{step,tid,marking} :29, ForbiddenBit{step,marking} :38, OutOfOrder{step,prev_tick,tick} :45, Unfinished{steps_done,total_events} :54` | report.rs:18-60 |
| `ConformanceReport` | private fields + accessor law (report.rs:141-143) | report.rs:144 |
| `ConformanceReport::new(steps, total_events, deviations)` | auto-records `Unfinished` when `steps < total_events` | report.rs:163 |
| `steps()` / `total_events()` / `deviations()` | accessors | report.rs:174 / 180 / 186 |
| `open_deviations()` | counts unexplained deviations | report.rs:192 |
| `explain(usize)` | mark a deviation explained; out-of-range refuses | report.rs:205 |
| `is_conforming()` | trajectory finished AND zero open deviations; explaining `Unfinished` cannot un-truncate | report.rs:222 (test :281-302) |

## pn64 Primitives

Branchless, `no_std`-compatible hot path. Bit layout [15:0] m / [31:16] W- /
[47:32] W+ / [55:48] tid / [63:56] flags (pn64.rs:7-11; ported from strata @
0de13a3, pn64.rs:3-4).

| Member | Cite |
|---|---|
| `MASK16` | pn64.rs:18 |
| `marking` / `pre` / `post` / `tid` / `flags` | pn64.rs:23 / 30 / 37 / 44 / 57 |
| `pack(mark, pre, post, tid, flags)` | pn64.rs:65 |
| `enabled` / `fire` | pn64.rs:77 / 85 |
| `ct_eq` / `ct_lte` | pn64.rs:102 / 117 |
| `step_branchless(pn, ocel, invariant_mask)` | pn64.rs:152 |

## Net / State Space / Validation

- `net::NetBuilder` (net.rs:198-250): `CompiledNet::builder()` net.rs:67,
  `place` :205, `forbidden_place` :212, `transition` :220, `add_pre` :232,
  `add_post` :241, `build` :250; `NetSpec::compile()` net.rs:92,
  `compile()` emits `CompiledNet.forbidden_mask` (bit i set per
  `forbidden_place` declaration), feedable to `StateSpace::new` /
  session's `set_forbidden_mask` so declarations are enforced.
  `initial_marking()` net.rs:158.
- `state::StateSpace` (state.rs:41): `new` state.rs:58, `with_max_markings`
  :93 (const), `enabled_mask` :103, `fire` :117, `try_fire` :129, `reach()`
  :160, `is_safe()` :199, `deadlock_free()` :217, `forbidden_reachable()`
  :225; `state::Marking64` struct state.rs:20 with `covers(mask)` :25.
- `validate::validate(&PowlModel)` validate.rs:45, `validate_petri(&PetriNet)`
  validate.rs:99, `validate_bcinr` validate.rs:166 — see Known Boundaries: the
  BCINR bridge is NOT wired.

## Test Macros

Crate-root `#[macro_export]` macros (no `#[macro_use]`; lib.rs:206):

- `conformance_test!` (src/core/macros/conformance_test.rs:47-48) — expands to
  a bare `#[test]` fn.
- `assert_conforms!(session, receipt)` (conformance_test.rs:71-86) — asserts
  BOTH legs: `$session.conformance_report().is_conforming()` AND
  `$receipt.is_algebraically_closed()`
  (`core::receipt::TestReceipt::is_algebraically_closed()`,
  src/core/receipt.rs:319). A conformance claim counts only when the trajectory
  conforms AND the receipt is algebraically closed. Never cite
  `assert_conforms!` as a single-leg assert.

## OCEL64 Ingestion Bridge

All four files live under `src/observability/ocel/`; each carries a file-level
`#![cfg(feature = ...)]` gate, and crate-root visibility is wired in
src/observability/ocel/mod.rs:25-33 (each module declared with its own
`#[cfg]`). Cite the feature spelling each file declares.

### ocel64_bridge (feature `ocel-ingest`)

`src/observability/ocel/ocel64_bridge.rs`, gated
`#![cfg(feature = "ocel-ingest")]` (file line 1).

Word layout: activity [15:0] / src [31:16] / tgt [47:32] / qualifier [55:48]
u8 / tick delta [63:56] u8 (module docs ocel64_bridge.rs:4-12; ported from
strata @ 0de13a3).

Free fns: `activity` :75, `src_obj` :82, `tgt_obj` :89, `qualifier` :96,
`tick` :103, `pack` :111. `BridgeError` :127: `Collision{field,details}` :131,
`CapacityExceeded{field,count,bound}` :139, `TickDeltaOverflow{case,index,delta_ns}`
:151 — tick-delta overflow is REFUSED, never saturated (module docs :8-12);
`NonMonotonicTimestamp{case,index}` :163 (unreachable from `to_ocel64`, which
sorts per case first; enforced in `checked_deltas` :323);
`UnknownQualifier{raw}` :171. Vocabulary mapping: `type_name` :195,
`activity_key` :212 (payloads dropped at the packed tier, :208-212),
`qualifier_for` :250 (total; unknown -> 0), `qualifier_name` :271,
`parse_type_name` :288, `checked_deltas(&str, &[u64])` :316,
`to_ocel64(&OcelLog) -> Result<(Vec<u64>, BridgeVocab), BridgeError>` :403,
`from_ocel64(&[u64], &BridgeVocab) -> Vec<TestOcelEvent>` :496.

### Honest Disclosures (UNSUPPORTED by design, NOT bugs)

Module docs ocel64_bridge.rs:46-63:

- Attributes do not survive the roundtrip: `from_ocel64` always emits an empty
  attribute map (:563).
- `event_id`s are regenerated as `e_{n}` over the whole decoded sequence;
  original event ids are not carried in the word format.
- `case_id`s are regenerated as `case_{n}` per separator-delimited segment;
  the `BridgeVocab` contract has no case-name id space.
- Absolute timestamps are not carried — only per-case tick deltas. The decoded
  timeline is correct up to a per-case time origin.
- Only one qualifier byte exists per word, so the decoded tgt-slot type is the
  src-slot's type. Exact object-type roundtrip holds only for corpora where
  each event's objects of a given type come in even counts (the encoder sorts
  each event's objects by `(qualifier, name)`, so same-type objects are
  adjacent).

Attribute (and event/case identity) preservation is UNSUPPORTED at the packed
tier by design; use the rich `OcelLog` tier when attributes matter.

### idmap_bridge (feature `ocel-ingest`)

`src/observability/ocel/idmap_bridge.rs`, gated
`#![cfg(feature = "ocel-ingest")]` (idmap_bridge.rs:44).

`IdMapError::Collision{index_a,name_a,index_b,name_b}` :51; `IdMap` :69 via
`IdMap::build(&[&str])` :91 (ids 1..=n in first-seen order; duplicate name is
the degenerate collision refusal); `get` :118, `name` :124, `is_bijective()`
 :132, `len` :152, `is_empty` :158; `collision_risk(&[&str]) -> usize` :174.

**Birthday bound** (module docs idmap_bridge.rs:16-33): 16-bit
blake3-truncated collision sentinel; p(n) is approximately
`1 - exp(-n(n-1)/2^17)`; **about 302 names gives ~50% collision probability
(exact crossing n=302)**; fewer than 83 names keeps p below 5%; a collision is
a typed refusal and the digest never participates in id derivation.

### import_json (feature `ocel-ingest-json`)

`src/observability/ocel/import_json.rs`, gated
`#![cfg(feature = "ocel-ingest-json")]` (import_json.rs:22).

Fail-closed contract (import_json.rs:7-19): accepts
`{"ocel:version": "2.0", "ocel:events": {...}, "ocel:objects": {...}}` shape;
missing `ocel:version` -> `MissingField`; version != "2.0" -> `InvalidVersion`;
missing `ocel:activity` / `ocel:timestamp` (or per-object `ocel:type`) ->
`MissingField` carrying the field name; non-JSON -> `MalformedJson`.
`ImportError` :31 (`MalformedJson{message}` :34, `MissingField{field,subject}`
:40, `InvalidVersion{found}` :48). `import_ocel_json(&str) -> Result<OcelLog,
ImportError>` :67 — refuses the whole document, never a partial import.

### import_xml (feature `ocel-ingest-xml`)

`src/observability/ocel/import_xml.rs`, gated
`#![cfg(feature = "ocel-ingest-xml")]` (import_xml.rs:32).

**Roundtrip-first reader disclosure** (module docs import_xml.rs:3-5): accepts
exactly the shape the ocel-xml exporter emits (the
strata_protocol/src/ocel_xml.rs envelope), NOT a general OCEL 2.0 XML parser.
Accepted envelope example at import_xml.rs:11-17; the escaper inverse covers
exactly `&amp; &lt; &gt; &quot; &apos;` — a numeric or unknown entity is a hard
refusal (:19-20); activities/types deserialize as `TestActivity` /
`TestObjectType` unit variants via serde, payload variants refused (:21-25);
hand-rolled scanner, no XML crate (:26-27). `XmlImportError` :41
(`MalformedXml{offset,reason}` :44, `UnclosedTag{tag,offset}` :52,
`MissingAttribute{tag,attr}`, `InvalidVersion{found}`).
`import_ocel_xml(&str) -> Result<OcelLog, XmlImportError>` :251.

## Tiering Commands

From the Justfile on disk:

```text
just test-conformance     # Justfile:136-137
just test-ocel-ingest     # Justfile:140-141
just verify-hermetic      # Justfile:196-201 (locked + offline build/test matrix over conformance/ledger/nato)
just check-mutation-gate  # Justfile:204-205
```

Runnable pipeline:
`cargo run --example conformance_quickstart --features conformance`
(the full compile-checked walk-through is
examples/conformance_quickstart.rs:37-70: load -> attach -> `step_budget`
loop -> verdict asserts; the `event(tid, tick) = tid | (tick << 8)` helper is
examples/conformance_quickstart.rs:33-36).

## Known Boundaries (honest disclosures)

- **BCINR bridge UNWIRED**: the `powl-bcinr` feature DOES exist
  (Cargo.toml:225, declared as `powl-bcinr = ["conformance"]`), but the bridge
  is compiled out: `src/conformance/bcinr.rs` is `#[cfg(any())]`-gated
  (bcinr.rs:44), so enabling the feature contributes zero items;
  `validate_bcinr` (validate.rs:166) remains a stub returning `Ok(())`.
  Documenting `to_bcinr_powl2` / `validate_bcinr` as functional would be an
  overclaim.
- No `conformance-ocel` aggregation feature exists; none is cited.
- Session's `stream` field is private: there is no public `stream()` getter,
  and `attach_stream` on a `Complete`/`Faulted` session stays frozen
  (session.rs:240-242) — attaching does not unfreeze a terminal phase.
- `session::CompiledNet` and `session::Marking64` are distinct on purpose from
  the canonical `net::CompiledNet` and `state::Marking64` (mod.rs:13-14).
- The POWL DSL is not a general Petri-net text format; the separate
  `ParsedNet::from_net_str` mini-format (loader.rs:693-720) lowers to the
  validated `model::PetriNet` (loader.rs:726-736).
