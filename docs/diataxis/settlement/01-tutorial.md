# Tutorial — Execute a Phase 7 Settlement Conformance Run

This tutorial runs the landed Phase 7 settlement court end to end. Everything
below uses only files already on disk in this repo; no step requires code that
does not exist.

## What you will do

1. Load `examples/merger_matrix/models/phase7_settlement.powl` as a
   `PowlModel` and read its DFS atom sequence.
2. Read the landed OCEL fixture through the RM3 bridge
   (`powl_fixture_bridge::fixture_activity_names` and
   `session_words_for_atoms`).
3. Replay via `ConformanceSession::{from_powl_file, attach_stream,
   step_budget}` and assert `is_conforming()` on the report.
4. Run the `netting_engine` guest kernel (`crates/merger-guests`) as the
   worked example of the conservation invariant that underwrites the
   settlement math.

## Why

Phase 7 is the financial-settlement phase of the merger matrix: ISO 20022
pacs.008/pacs.009 intake, bounded payment-elimination loops (net/gross <=
0.18x), an atomic two-phase-commit DvP trap (securities may move
PREPARE→COMMIT only when the cash leg commits in the SAME step), then a pn64
seal. The invariants are embedded as POWL structure, so replay is the court:
if a trace skips the intake membrane or fires a loop atom a second time, the
session latches `Phase::Faulted` with a typed `ConformanceError` instead of
passing silently.

## Landed files this tutorial walks

| File | Role |
|---|---|
| `examples/merger_matrix/models/phase7_settlement.powl` | normative base model: 9 activities, 14/64 node budget (`MAX_POWL_NODES`, `src/conformance/loader.rs:47`) |
| `examples/merger_matrix/conformance/phase7_settlement.ocel.json` | golden fixture — looped shape (intake/elimination fires twice) |
| `examples/merger_matrix/conformance/phase7_settlement_violation.ocel.json` | violation fixture — same looped skeleton |
| `tests/merger_conformance_tests.rs` | the landed courts; Phase 7 entries: `phase7_golden_pipeline_completes`, `phase7_landed_fixtures_loop_repetition_faults_skipped` |
| `src/conformance/session.rs` | `ConformanceSession`, `set_forbidden_mask`, latch/freeze semantics |
| `src/conformance/state.rs:225` | `StateSpace::forbidden_reachable` — the DvP trap checker |
| `crates/merger-guests/src/netting_engine.rs` | the conservation kernel (leaf_id 2, `mgi_netting_engine_compute`) |

## Step 1 — Run the landed courts

```bash
cd /Users/sac/chicago-tdd-tools
cargo test --test merger_conformance_tests -- phase7
```

Two tests run, and both pass:

- `phase7_golden_pipeline_completes` — the full contract pipeline (import →
  pack → unpack → packed-tier identity asserts → remap via
  `attributes["powl"]` → replay) over a model-derived case `p7-golden`.
  The report is conforming with `steps_executed == events_total`.
- `phase7_landed_fixtures_loop_repetition_faults_skipped` — **both** landed
  fixtures (golden and violation) replay the protocol's LOOPED shape: the
  intake/elimination block fires twice, but the linear session lowering admits
  each atom exactly once, so the second `pacs_intake` faults
  `ConformanceError::Skipped { step: 1 }`, the phase latches `Faulted`, the
  marking freezes at `1 << 1`, and `resume` re-reports the same violation.

Divergence note (honest landed reality): the conforming replay path for the
on-disk fixtures is the model-derived pipeline (`p7-golden`), not the raw
landed fixture files. Both raw fixtures pin the loop-repetition law as a typed
court. The earlier fixture-vocabulary `Undeclared` divergence no longer exists
on disk (B6 converged fixture vocabulary to the model's declarations), so the
honest fault is `Skipped { step: 1 }`.

## Step 2 — The bridge path (powl_fixture_bridge)

The production path never hand-walks the fixture JSON. The landed bridge owns
two jobs:

- `fixture_activity_names(&text)` — parses the fixture fail-closed and returns
  activity names in the bridge's deterministic `(timestamp_ns, event_id)`
  order. No test-local traversal law.
- `session_words_for_atoms(&names, &atoms)` — maps names to session words,
  `word[i] = tid_i | (i+1) << 8`, refusing an undeclared name with a typed
  error.

Note the typed import boundary: the landed fixtures carry POWL names in
`ocel:activity` rather than the contract wire form (`TestActivity` variant +
`attributes["powl"]`), so `import_ocel_json` refuses every landed fixture with
`ImportError::MissingField { field: "ocel:activity" }`. This is a pinned court
(`import_refuses_landed_fixtures_typed`) that keeps the divergence loud rather
than a silent pass. The raw-remap path (bridge + session, bypassing
`import_ocel_json`) is how the landed fixtures are replayed.

## Step 3 — Read the model's invariant-as-structure

`phase7_settlement.powl` embeds three stages as structure:

- STAGE A INTAKE: `pacs_intake` heads the sequence (fail-closed membrane).
- STAGE B NETTING: `loop { payment_elimination } redo { silent } max 64`
  bounded loop, followed by `netting_ratio_check` (net/gross <= 0.18x) before
  any settlement move. Efficiency >= 92.1% and volume <= Fed cap are TRACE
  METRICS checked post-replay, not net-structural.
- STAGE C 2PC DvP TRAP: `settlement_prepare` is ONE atomic transition covering
  both legs; the xor is commit-vs-abort. The forbidden marking (cash marked,
  securities unmarked) is unreachable by construction in POWL and checked by
  `StateSpace::forbidden_reachable` on the lowered net; `deadlock_free` must
  hold on the good path. The abort branch carries the seeded refusal reasons
  (`reason=penny-discrepancy`, `reason=dvp-cash-leg-missing`) as replay-event
  attributes.

## Step 4 — The netting_engine guest as the worked example

Phase 7's netting invariant is enforced numerically by the MG2 leaf kernel,
`crates/merger-guests/src/netting_engine.rs` (leaf_id 2, entry point
`mgi_netting_engine_compute`, scale `TAG_MONEY` = `S_MONEY` = 10^6
micro-dollars):

1. **Bilateral pass**: rows with the same `(debtor, creditor)` pair aggregate
   into one net row; pairs netting to zero drop out.
2. **Multilateral pass**: per-party signed position
   `Σ(owed to it) − Σ(it owes)`.
3. **Conservation check**: `Σ positions == 0` is asserted in `i128` after the
   collapse and refuses with `ERR_OVERFLOW` on violation — documented as
   unreachable-by-construction (each delta is `checked_add`-ed from the same
   amounts that passed the per-row i64 domain check), retained as a
   fail-closed wire-invariant assertion (`Σnet == Σgross`, sign-conserved).
4. **Determinism**: outputs stably sorted — net rows by
   `(debtor_idx, creditor_idx)`, positions by `party_idx` — so byte-for-byte
   replay of the same input yields the same output on any host.

Run its courts:

```bash
cargo test -p merger-guests 2>/dev/null || \
  cargo test --test fault_injection --test zero_import -p merger-guests
```

(Use whichever target name your workspace registers for
`crates/merger-guests`; the courts are
`crates/merger-guests/tests/fault_injection.rs` and
`tests/zero_import.rs`.)

## Step 5 — Replay the phase-7 settlement math yourself

Minimal shape of the replay (mirrors `run_contract_pipeline`, abridged):

```rust
use chicago_tdd_tools::conformance::session::{ConformanceSession, YieldStatus};
use chicago_tdd_tools::observability::ocel::powl_fixture_bridge::session_words_for_atoms;

let atoms = /* DFS atoms of phase7_settlement.powl */;
let names = /* fixture_activity_names of the golden fixture */;
let words = session_words_for_atoms(&names, &atoms).unwrap();

let mut session = ConformanceSession::from_powl_file(model_path).unwrap();
session.attach_stream(&words).unwrap();
match session.step_budget(atoms.len() as u32) {
    YieldStatus::Complete(report) => {
        assert_eq!(report.steps_executed, atoms.len() as u32);
        assert!(session.conformance_report().is_conforming());
    }
    other => panic!("expected Complete, got {other:?}"),
}
```

## Checkpoint

- `cargo test --test merger_conformance_tests -- phase7` → 2 passed.
- You can name which of the two landed fixture files produces a conforming
  replay (neither as raw fixtures — both pin the loop-repetition law; the
  conforming path is the model-derived pipeline).
- You can name the errno the netting leaf returns if the conservation
  invariant failed (`ERR_OVERFLOW`, 16) and its fuel cap
  (`4 × rows + 64` accounted steps, `ERR_FUEL` at exhaustion).
