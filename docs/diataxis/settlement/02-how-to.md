# How To: Typed Refusals and Invariant-Mask Inspection

This guide covers the two troubleshooting surfaces you will hit when a merger
replay refuses: the typed `ConformanceError` surface, and the forbidden-mask
inspection machinery (`set_forbidden_mask` / `forbidden_reachable`).

## The landed refusal surface

`src/conformance/error.rs` is the whole typed refusal enum for replay. The
replay-class variants you will actually meet in merger courts:

| Variant | Fires when | Landed court that pins it |
|---|---|---|
| `Skipped { step }` | cursor's expected atom is declared but not enabled; the stream offers a different declared atom | `phase4_violation_skipped_latches_frozen` (step 2), `phase6_violation_lcr_gate_skipped_latches_frozen` (step 5), `phase7_landed_fixtures_loop_repetition_faults_skipped` (step 1) |
| `OutOfOrder { step, prev_tick, tick }` | a step's tick regresses | pinned in `tests/conformance_tests.rs` |
| `Undeclared { step, tid }` | a step references a transition id not in the model | pinned in `tests/conformance_tests.rs` |
| `ForbiddenBit { step, marking, mask }` | post-fire marking intersects the forbidden mask | `session.rs` unit test `set_forbidden_mask(0b0010)` |
| `TraceMalformed { len }` | zero-length attach, or attach into a terminal phase | `attach_stream` eager refusals |
| `TooManyPlaces { limit: 16 }` | model's atom count exceeds the linear net's 15-transition budget | phases 1/2/9/10 courts in `tests/merger_conformance_tests.rs` |
| `MalformedSource` / `UnknownActivity` / `DuplicateActivity` | loader-time parse/declaration refusals | loader tests |

## Divergence note: E-M07 penny-discrepancy / DvP refusals are NOT landed in error.rs

The E-M07 contract specified penny-discrepancy and DvP-cash-leg-missing as
typed refusals. As landed, **they do not exist as `ConformanceError`
variants**. Grep `src/conformance/error.rs` for `penny`/`dvp`: zero matches.
The grep-verified facts:

- The refusal REASONS live as fixture-attribute markers and model comments:
  the phase-7 violation fixture and the abort branch of
  `phase7_settlement.powl` carry `reason=penny-discrepancy` and
  `reason=dvp-cash-leg-missing` as replay-event attributes, and the model
  header names a "SettlementPennyDiscrepancy-family invariants module (E-M15)"
  — an invariants-lane scope, not the net-structural refusal enum.
- The phase-8 court (`phase8_violation_over_issuance_pins_structural_equality`)
  documents the same pattern explicitly: the count law lives in event
  attributes, attributes are UNSUPPORTED at the packed session tier, so the
  structural replay is name-conforming and the divergence is pinned on the
  real JSON, loud, for the invariants-lane court.
- Consequence for troubleshooting: if you expected a typed
  `PennyDiscrepancy`/`Dvp` variant and got a clean `Complete`, you are not
  seeing a bug — the numeric laws are checked post-replay on real JSON
  attributes, not net-structural. Document accordingly.

If a future lane lands these as typed variants, update this table; until then
treat E-M07's refusal surface as the attributes + invariants-lane pattern above.

## Troubleshooting playbook

### Symptom: `Skipped { step: N }` on a fixture you expected to pass

1. Check the model for a `loop` block over the atom at step N. The linear
   session lowering admits each atom exactly once; looped traces fault on
   second firing (phase 5 golden and both phase 7 fixtures are landed examples
   of this law, pinned as courts, not bugs).
2. Check whether the violation fixture dropped an atom (phase 4: drops
   `multilateral_compression`; phase 3: drops `p3_cet1_gate`; phase 6: drops
   the LCR gate). The step number is the frozen cursor position.
3. Confirm the frozen state: `session.phase()` is `Phase::Faulted(err)`,
   `marking()` is `1 << N`, `steps_done() == N`, and `resume(...)` re-reports
   the same error without advancing. Anything else is a real kernel bug.

### Symptom: `TooManyPlaces { limit: 16 }` on a model that looks fine

The linear session net is `MAX_PETRI_PLACES - 1` = 15 transitions. Phase 1
(20 atoms), phase 2 (22 DFS atoms), phase 9 (35 atoms), and phase 10 (18
atoms) all refuse by design; their lawful replay path is the Wave-4
chained-session harness (E-M09 cap rule), not a single session. Do not raise
the budget to make a test pass — the refusal IS the court.

### Symptom: import refuses your fixture with `MissingField { field: "ocel:activity" }`

The landed fixtures use the raw `ocel:activity` convention; the contract wire
form is `TestActivity` variant + `attributes["powl"]`. Either rewrite the
fixture to the contract wire form or use the raw-remap path
(`powl_fixture_bridge` + session) as the landed tests do.

## Invariant-mask inspection

### Installing a forbidden-mask policy

```rust
use chicago_tdd_tools::conformance::session::ConformanceSession;

let mut s = ConformanceSession::from_powl_file(model_path)?;
// Any post-fire marking that intersects this mask classifies
// ConformanceError::ForbiddenBit. Masked to 16 bits (u64 input, & 0xFFFF).
s.set_forbidden_mask(0b0010);
```

Landed semantics (session.rs:203): the mask is u64 in, truncated to the
16-bit marking encoding, and **read on every step** (mid-stream install between `Yielded` and `resume` changes subsequent-step classification) — the
session latches, so set the mask right after compile, before
`attach_stream`/`step_budget`.

### Checking reachability of a forbidden marking

`StateSpace::forbidden_reachable` (`src/conformance/state.rs:228`) does a BFS
over the lowered net's markings from the initial marking, returning the first
forbidden marking found (any marking whose bits intersect the mask) or `None`:

```rust
let ss = /* StateSpace for the lowered net */;
match ss.forbidden_reachable()? {
    Some(marking) => panic!("DvP trap violated: {marking:#06b} reachable"),
    None => { /* the trap holds */ }
}
```

Details that matter:

- A zero mask short-circuits to `None` (no policy installed).
- BFS caps at `max_markings`; exceeding it is `ModelTooLarge` — a typed
  refusal, never a silent truncation.
- The phase-7 DvP trap is modeled so the forbidden marking (cash marked,
  securities unmarked) is unreachable **by construction** in POWL (the commit
  is one atomic transition); `forbidden_reachable` is the fail-closed checker
  over that construction, and `deadlock_free()` (`state.rs:217`) must hold on
  the good path. Phase-7's model header states both checks explicitly.
- The in-session `is_conforming()` gate itself routes through the same
  short-circuit (`state.rs:200`: `Ok(self.forbidden_reachable()?.is_none())`).

### Latch/freeze discipline (why "just step again" never works)

A faulted session is frozen: marking, cursor, and step count never advance,
and `resume` re-reports the same violation. This is deliberate — a refusal is
a terminal court verdict on the stream, not a retryable condition. To re-run,
build a fresh session (`from_powl_file`), re-install any mask, and re-attach.
