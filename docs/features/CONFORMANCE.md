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
mod.rs:30).

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
POWL variant is test fixtures only (mod.rs:7) and is not documented here.

| Member | Notes | Cite |
|---|---|---|
| `MAX_POWL_NODES` | `pub const ...: usize = 64` | loader.rs:47 |
| `PowlModel` | `{ activities, declaration_order, root }`, pub fields | loader.rs:58 |
| `ActivityDecl` | `{ source, event }` | loader.rs:69 |
| `PowlNode` | `Sequence(Vec<_>) \| Xor(Vec<_>) \| Parallel(Vec<_>) \| Loop{body,redo,max:u8} \| Activity(String) \| Silent` | loader.rs:85-105 |
| `node_count()` | `(&self) -> usize` | loader.rs:666 |
| `xor_arity_profile()` | `(&self) -> Vec<usize>` | loader.rs:673 |
| `from_powl_str(&str)` | parse from string; node cap enforced | loader.rs:685 |
| `from_powl_file(impl AsRef<Path>)` | parse from file | loader.rs:704 |
| `from_powl_json(&str)` | parse from JSON | loader.rs:718 |
| `ParsedNet` | `{ places, transitions, arcs }` name-based mini-net | loader.rs:737 |
| `ParsedNet::from_net_str(&str)` | line-based `place p1` / `transition t1` / `arc p1 -> t1`, `#` comments | loader.rs:764 |

### DSL Grammar

Transcribed from the real parser (loader.rs:398-611; scanner loader.rs:261-355
— tokens `{ } = .` plus Word/Num; numbers 0..=255, overflow refuses
loader.rs:315-323; `#` comments to EOL loader.rs:279-285; reserved words
`activity|sequence|xor|parallel|loop|redo|max|silent` loader.rs:356-358):

```text
document      := activity_decl* root_expr EOF
                 (loader.rs:398-431; empty/missing root refuses
                  "missing root expression" loader.rs:404-410)
activity_decl := `activity` NAME `=` SRC `.` EVENT
                 (loader.rs:432-509; duplicate → DuplicateActivity :501)
root_expr     := `sequence` `{` children `}`
               | `xor`      `{` children `}`
               | `parallel` `{` children `}`
               | `loop`     `{` expr `}` [`redo` `{` expr `}`] [`max` NUM]
                 (loader.rs:553-578; default max = 255 :567)
               | `silent`    (loader.rs:575)
               | NAME        (must be a declared activity, else
                              UnknownActivity loader.rs:577-583)
children      := expr* until matching `}` (loader.rs:591-611)
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

`session::ConformanceSession` (session.rs:119), flat re-exported at mod.rs:46.
Deliberate distinctions: `session::Marking64 = u64` (session.rs:58) is an
alias, distinct from the `state::Marking64` struct (state.rs:20); likewise
`session::CompiledNet` (session.rs:71) is the session-local linear net, while
the canonical `CompiledNet` is `net::CompiledNet` (mod.rs:13-16).

| Member | Notes | Cite |
|---|---|---|
| `ConformanceResult<T>` | `Result<T, ConformanceError>` | session.rs:47 |
| `Marking64 = u64` alias | distinct from `state::Marking64` | session.rs:58 |
| `from_powl_file(&Path)` | load + validate + compile in one call | session.rs:141 |
| `from_model(&PowlModel)` | from an already-loaded model | session.rs:151 |
| `set_forbidden_mask(u64)` | masked to 0xFFFF, pre-first-step only | session.rs:204 |
| `attach_stream(&[u64])` | empty stream -> `TraceMalformed{len:0}` | session.rs:219 (refusal :221) |
| `step_budget(u32) -> YieldStatus` | never `Err`; violations latch `Phase::Faulted` and re-report via `YieldStatus::Violation` | session.rs:235 |
| `resume(u32)` | continue after a yield | session.rs:275 |
| `phase()` | `&Phase` | session.rs:280 |
| `steps_done()` | `u32` | session.rs:285 |
| `remaining_events()` | `usize` | session.rs:290 |
| `marking()` | `Marking64` | session.rs:295 |
| `initial_marking()` | `Marking64` | session.rs:301 |
| `net()` | `&CompiledNet` | session.rs:307 |
| `finish()` | `-> ReplaySummary` | session.rs:318 |
| `conformance_report()` | `-> ConformanceReport` (macro seam) | session.rs:335 |

### Event-Word Layout and Tick Law

Event words: activity id in bits [7:0], tick in bits [15:8]
(session.rs:28-29 module docs). Ticks must advance exactly `prev + 1 (mod 256)`
(session.rs:377-379; tests/conformance_tests.rs:8-9). Classification order
(`classify_and_commit`, session.rs:372, classified inline :374-395), each class
checked explicitly, never inferred from a silent PN64 refusal:

1. Tick order: `OutOfOrder` when `tick == prev`, `Skipped` when not
   `prev + 1 (mod 256)` — the tick law is checked first.
2. `Undeclared` (activity id not in the model's activity dictionary).
3. `ForbiddenBit` (marking bit set in the forbidden mask).

Violations latch `Phase::Faulted` and re-report via `YieldStatus::Violation`
(session.rs:235).

## Report Types

All verdict types are homed in `report.rs` (E1 item-2 reconcile); `session`
re-exports them for path stability.

| Type | Shape | Cite |
|---|---|---|
| `ReportOutcome` | `Complete \| Faulted(ConformanceError) \| Incomplete` | report.rs:67 |
| `ReplaySummary` | `{ steps_executed: u32, events_total: usize, final_marking: u64, outcome: ReportOutcome }` | report.rs:84-94 |
| `YieldStatus` | `Yielded{remaining_events} \| Complete(ReplaySummary) \| Violation(ConformanceError)` | report.rs:98-108 |
| `Phase` | `Idle \| Running \| Yielded \| Complete \| Faulted(ConformanceError)` | report.rs:112-123 |
| `Deviation` | `VerbMismatch{step,tid,expected} :20, NotEnabled{step,tid,marking} :29, ForbiddenBit{step,marking} :38, OutOfOrder{step,prev_tick,tick} :45, Unfinished{steps_done,total_events} :54` | report.rs:18-60 |
| `ConformanceReport` | private fields + accessor law (report.rs:127-129) | report.rs:131 |
| `ConformanceReport::new(steps, total_events, deviations)` | auto-records `Unfinished` when `steps < total_events` | report.rs:150-160 |
| `steps()` / `total_events()` / `deviations()` | accessors | report.rs:169 / 175 / 181 |
| `open_deviations()` | counts unexplained deviations | report.rs:187 |
| `explain(usize)` | mark a deviation explained; out-of-range refuses | report.rs:200-214 |
| `is_conforming()` | trajectory finished AND zero open deviations; explaining `Unfinished` cannot un-truncate | report.rs:217 (test :295-308) |

## pn64 Primitives

Branchless, `no_std`-compatible hot path. Bit layout [15:0] m / [31:16] W- /
[47:32] W+ / [55:48] tid / [63:56] flags (pn64.rs:6-10; ported from strata @
0de13a3, pn64.rs:3-4).

| Member | Cite |
|---|---|
| `MASK16` | pn64.rs:18 |
| `marking` / `pre` / `post` / `tid` / `flags` | pn64.rs:23 / 30 / 37 / 44 / 57 |
| `pack(mark, pre, post, tid, flags)` | pn64.rs:65 |
| `enabled` / `fire` | pn64.rs:73 / 81 |
| `ct_eq` / `ct_lte` | pn64.rs:98 / 113 |
| `step_branchless(pn, ocel, invariant_mask)` | pn64.rs:148 |

## Net / State Space / Validation

- `net::NetBuilder` (net.rs:195-253): `CompiledNet::builder()` net.rs:67,
  `place` :202, `forbidden_place` :212, `transition` :223, `add_pre` :235,
  `add_post` :244, `build` :253; `NetSpec::compile()` net.rs:97,
  `initial_marking()` net.rs:165.
- `state::StateSpace` (state.rs:41): `new` state.rs:58, `with_max_markings`
  :100 (const), `enabled_mask` :110, `fire` :124, `try_fire` :136, `reach()`
  :168, `is_safe()` :210, `deadlock_free()` :228, `forbidden_reachable()`
  :236; `state::Marking64` struct state.rs:20 with `covers(mask)` :24.
- `validate::validate(&PowlModel)` validate.rs:45, `validate_petri(&PetriNet)`
  validate.rs:102, `validate_bcinr` validate.rs:164 — see Known Boundaries: the
  BCINR bridge is NOT wired.

## Test Macros

Crate-root `#[macro_export]` macros (no `#[macro_use]`; lib.rs:205):

- `conformance_test!` (src/core/macros/conformance_test.rs:47-48) — expands to
  a bare `#[test]` fn.
- `assert_conforms!(session, receipt)` (conformance_test.rs:70-86) — asserts
  BOTH legs: `$session.conformance_report().is_conforming()` AND
  `$receipt.is_algebraically_closed()`
  (`core::receipt::TestReceipt::is_algebraically_closed()`,
  src/core/receipt.rs:319). A conformance claim counts only when the trajectory
  conforms AND the receipt is algebraically closed. Never cite
  `assert_conforms!` as a single-leg assert.

## OCEL64 Ingestion Bridge

All four files live under `src/observability/ocel/`; each carries a file-level
`#![cfg(feature = ...)]` gate, and crate-root visibility is wired in
src/observability/ocel/mod.rs:25-32 (each module declared with its own
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
`UnknownQualifier{raw}` :171. Vocabulary mapping: `type_name` :200,
`activity_key` :217 (payloads dropped at the packed tier, :212-217),
`qualifier_for` :255 (total; unknown -> 0), `qualifier_name` :276,
`parse_type_name` :295, `checked_deltas(&str, &[u64])` :323,
`to_ocel64(&OcelLog) -> Result<(Vec<u64>, BridgeVocab), BridgeError>` :422,
`from_ocel64(&[u64], &BridgeVocab) -> Vec<TestOcelEvent>` :521.

### Honest Disclosures (UNSUPPORTED by design, NOT bugs)

Module docs ocel64_bridge.rs:46-63:

- Attributes do not survive the roundtrip: `from_ocel64` always emits an empty
  attribute map (:588).
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
`#![cfg(feature = "ocel-ingest")]` (idmap_bridge.rs:34).

`IdMapError::Collision{index_a,name_a,index_b,name_b}` :38-52; `IdMap` :59 via
`IdMap::build(&[&str])` :81 (ids 1..=n in first-seen order; duplicate name is
the degenerate collision refusal); `get` :108, `name` :114, `is_bijective()`
:122, `len` :142, `is_empty` :148; `collision_risk(&[&str]) -> usize` :164.

**Birthday bound** (module docs idmap_bridge.rs:13-33): 16-bit
blake3-truncated collision sentinel; p(n) is approximately
`1 - exp(-n(n-1)/2^17)`; **about 302 names gives ~50% collision probability
(exact crossing n=302)**; fewer than 83 names keeps p below 5%; a collision is
a typed refusal and the digest never participates in id derivation.

### import_json (feature `ocel-ingest-json`)

`src/observability/ocel/import_json.rs`, gated
`#![cfg(feature = "ocel-ingest-json")]` (import_json.rs:21).

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
`import_ocel_xml(&str) -> Result<OcelLog, XmlImportError>` :269.

## Tiering Commands

From the Justfile on disk:

```text
just test-conformance     # Justfile:125-126
just test-ocel-ingest     # Justfile:129-130
just verify-hermetic      # Justfile:174-179 (locked + offline build/test matrix over conformance/ledger/nato)
just check-mutation-gate  # Justfile:182-183
```

Runnable pipeline:
`cargo run --example conformance_quickstart --features conformance`
(the full compile-checked walk-through is
examples/conformance_quickstart.rs:38-72: load -> attach -> `step_budget`
loop -> verdict asserts; the `event(tid, tick) = tid | (tick << 8)` helper is
examples/conformance_quickstart.rs:33-37).

## Known Boundaries (honest disclosures)

- **BCINR bridge NOT wired**: the `powl-bcinr` feature does not exist in
  Cargo.toml; `src/conformance/bcinr.rs` is fully
  `#[cfg(feature = "powl-bcinr")]`-gated (bcinr.rs:39-40) and contributes zero
  items under default features; `validate_bcinr` (validate.rs:164-171) is a
  stub returning `Ok(())` until the optional dependency is wired. Documenting
  `to_bcinr_powl2` / `validate_bcinr` as functional would be an overclaim.
- No `conformance-ocel` aggregation feature exists; none is cited.
- Session's `stream` field is private: there is no public `stream()` getter,
  and `attach_stream` on a `Complete`/`Faulted` session stays frozen
  (session.rs:209-212) — attaching does not unfreeze a terminal phase.
- `session::CompiledNet` and `session::Marking64` are distinct on purpose from
  the canonical `net::CompiledNet` and `state::Marking64` (mod.rs:13-16).
- The POWL DSL is not a general Petri-net text format; the separate
  `ParsedNet::from_net_str` mini-format (loader.rs:737-764) lowers to the
  validated `model::PetriNet` (loader.rs:727-733).
