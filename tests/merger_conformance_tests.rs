//! Merger Matrix conformance tests (lane W1, v26.10.11).
//!
//! Drives the contract pipeline over the landed `examples/merger_matrix/`
//! fixtures: authored OCEL 2.0 JSON -> `import_ocel_json`
//! (`src/observability/ocel/import_json.rs`) -> `to_ocel64`
//! (`src/observability/ocel/ocel64_bridge.rs`) -> session-word remap ->
//! `ConformanceSession::from_powl_file` + `attach_stream` + `step_budget`.
//!
//! # Session-word remap law (E-M14 AUTHORITATIVE BLOCK)
//!
//! For event `i` of `K` (cases concatenated in `BTreeMap` lexicographic case
//! order, timestamp-sorted within a case — the deterministic traversal
//! `to_ocel64` uses):
//!
//! ```text
//! session_word[i] = tid_i | (i+1) << 8        // higher bits zero
//! tid_i           = DFS atom index of the event's POWL activity name
//! ```
//!
//! Ticks are exact `+1` from 1..K (`K <= 255` per attached stream). The POWL
//! name rides `ocel:attributes["powl"]` in the contract wire form (the
//! `ocel:activity` field must deserialize into a `TestActivity` variant for
//! import to succeed).
//!
//! # Honest scoping (divergences between landed fixtures and the contract,
//! asserted as typed courts, not papered over)
//!
//! * The landed fixtures carry POWL names in `ocel:activity` (their own
//!   convention) and no `attributes["powl"]`, so `import_ocel_json` refuses
//!   every landed fixture file with a typed `ImportError::MissingField { field:
//!   "ocel:activity" }`. `import_refuses_landed_fixtures_typed` pins that law
//!   so the divergence stays loud; the landed fixtures are driven into the
//!   session through a raw-JSON remap that reads the activity name directly
//!   (the fixtures' own convention, same tid law).
//! * Phase 1 (20 atoms) and phase 2 (21 atoms) exceed the linear net's
//!   16-place budget (`MAX_PETRI_PLACES - 1`); their models refuse to compile
//!   with `TooManyPlaces`, so no golden replay exists for them — the refusal
//!   itself is the court.
//! * Phase 5's landed golden carries 12 events against the model's 11 DFS
//!   atoms: `act_vote` and `act_sweep` each fire a second time in the landed
//!   trace (not a wholesale double-fire). The linear lowering requires each
//!   atom once, so the landed golden faults `Skipped` mid-trace. The
//!   model-derived golden (each DFS atom once) is the conforming trajectory.
//! * `phase5_syndicated_debt_violation.ocel.json` is a coherent alternative
//!   trajectory (xor branch swap to `act_refused` plus 2 attribute edits plus
//!   5 dropped events, 7 events vs the golden's 12), not a single-mutation
//!   diff of the golden.
//! * Phase 7's landed fixtures use the model-declared names (`pacs_intake`,
//!   `payment_elimination`, `settlement_prepare`, `settlement_commit`,
//!   `settlement_seal`); no `intake_pacs008` name appears in the landed
//!   fixture — that earlier claim was stale.
//! * Master/placeholder phases 3, 6, 8-10 are NOT tested: their models are not
//!   landed (only `phase3_pending`/`phase6_pending`/`phase10_pending` marker
//!   events inside `master_trace.ocel.json`).
//!
//! Real collaborators, no mocks (Chicago): real POWL files, real OCEL JSON, a
//! real import/pack/unpack boundary, a real replay. No `println!`.
#![cfg(all(feature = "conformance", feature = "ocel-ingest-json"))]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — assertions on real replayed state.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use chicago_tdd_tools::conformance::error::ConformanceError;
use chicago_tdd_tools::conformance::loader::{PowlModel, PowlNode};
use chicago_tdd_tools::conformance::session::ConformanceSession;
use chicago_tdd_tools::conformance::session::YieldStatus;
use chicago_tdd_tools::observability::ocel::import_json::import_ocel_json;
use chicago_tdd_tools::observability::ocel::ocel64_bridge::{from_ocel64, to_ocel64};
use chicago_tdd_tools::observability::ocel::powl_fixture_bridge::session_words_for_atoms;

// ---------------------------------------------------------------------------
// Fixture roots
// ---------------------------------------------------------------------------

const MODELS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/merger_matrix/models");
const CONF_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/merger_matrix/conformance");

/// Landed phase fixtures as (phase tag, model stem, golden fixture stem).
/// Phase 1/2 are listed for the import-refusal and compile-refusal courts; no
/// golden replay exists for them (the model refuses to compile).
const PHASES: [&str; 5] = [
    "phase1_intake",
    "phase2_antitrust",
    "phase4_novation",
    "phase5_syndicated_debt",
    "phase7_settlement",
];

fn model_path(stem: &str) -> PathBuf {
    PathBuf::from(MODELS_DIR).join(format!("{stem}.powl"))
}

fn fixture_path(stem: &str) -> PathBuf {
    PathBuf::from(CONF_DIR).join(format!("{stem}.ocel.json"))
}

// ---------------------------------------------------------------------------
// Remap helpers (the E-M14 law, applied to both wire forms)
// ---------------------------------------------------------------------------

/// Depth-first, left-to-right atom names of a loaded model — the exact
/// sequence the session lane's `atom_count` walks (`PowlNode` leaves).
fn dfs_atoms(node: &PowlNode, out: &mut Vec<String>) {
    match node {
        PowlNode::Activity(name) => out.push(name.clone()),
        PowlNode::Silent => {}
        PowlNode::Sequence(children) | PowlNode::Xor(children) | PowlNode::Parallel(children) => {
            for c in children {
                dfs_atoms(c, out);
            }
        }
        PowlNode::Loop { body, redo, .. } => {
            dfs_atoms(body, out);
            dfs_atoms(redo, out);
        }
    }
}

/// Load the phase model and return its DFS atom sequence.
fn phase_atoms(stem: &str) -> Vec<String> {
    let model = PowlModel::from_powl_file(&model_path(stem));
    // For the golden-pipeline tests the model MUST compile; callers that test
    // the TooManyPlaces refusal call the loader directly.
    let model = model.unwrap_or_else(|e| panic!("{stem}: model must load: {e}"));
    let mut atoms = Vec::new();
    dfs_atoms(&model.root, &mut atoms);
    atoms
}

/// Production path (RM3 bridge): the landed fixture's activity names in the
/// bridge's deterministic `(timestamp_ns, event_id)` order. No test-local
/// traversal law: the bridge owns the order and the fail-closed parse.
fn fixture_names(stem: &str) -> Vec<String> {
    let text =
        std::fs::read_to_string(fixture_path(stem)).unwrap_or_else(|e| panic!("read {stem}: {e}"));
    chicago_tdd_tools::observability::ocel::powl_fixture_bridge::fixture_activity_names(&text)
        .unwrap_or_else(|e| panic!("{stem}: fixture bridge: {e}"))
}

/// `session_word[i] = tid_i | (i+1) << 8` over explicit names via the RM3
/// bridge (contract-wire streams whose names ride the rich log's
/// `attributes["powl"]`).
fn session_words(names: &[String], atoms: &[String]) -> Vec<u64> {
    session_words_for_atoms(names, atoms).unwrap_or_else(|e| panic!("session-word mapping: {e}"))
}

/// Contract wire form: `ocel:activity` is a `TestActivity` variant name; the
/// POWL name rides `ocel:attributes["powl"]`. One case, timestamps `0..K`
/// (deltas of 1 satisfy `checked_deltas`), one same-type object per event so
/// the packed tier roundtrips losslessly.
fn contract_case_json(atoms: &[String], case: &str) -> String {
    let mut events = serde_json::Map::new();
    for (i, atom) in atoms.iter().enumerate() {
        events.insert(
            format!("e{i:0>3}"), // lexicographic == numeric order
            serde_json::json!({
                "ocel:activity": "TestStarted",
                "ocel:timestamp": i,
                "ocel:caseId": case,
                "ocel:objectTypes": [{"id": format!("o{i:0>3}"), "type": "Artifact"}],
                "ocel:attributes": {"powl": atom},
            }),
        );
    }
    serde_json::json!({
        "ocel:version": "2.0",
        "ocel:events": events,
    })
    .to_string()
}

/// The full contract pipeline on a generated case: import -> pack -> unpack ->
/// packed-tier identity asserts -> remap (rich-log `attributes["powl"]`) ->
/// replay to `Complete` with a conforming report.
fn run_contract_pipeline(stem: &str, case: &str) {
    let atoms = phase_atoms(stem);
    let json = contract_case_json(&atoms, case);

    let log = import_ocel_json(&json)
        .unwrap_or_else(|e| panic!("{stem}: contract JSON must import: {e}"));
    assert_eq!(log.events.len(), atoms.len(), "{stem}: all events import");

    let (words, vocab) =
        to_ocel64(&log).unwrap_or_else(|e| panic!("{stem}: pack must succeed: {e}"));
    assert_eq!(words.len(), atoms.len(), "{stem}: one word per event (1 object each)");

    let back = from_ocel64(&words, &vocab);
    assert_eq!(back.len(), atoms.len(), "{stem}: unpack count identity");
    // Packed-tier identity (the lossy fields are UNSUPPORTED by design):
    // count, per-case relative timeline, object names/types, empty attributes.
    for (i, ev) in back.iter().enumerate() {
        assert_eq!(ev.timestamp_ns, i as u64, "{stem}: relative timeline identity");
        assert_eq!(ev.objects.len(), 1, "{stem}: object slot identity");
        assert_eq!(ev.objects[0].0, format!("o{i:0>3}"), "{stem}: object name identity");
        assert!(
            back.iter().all(|e| e.attributes.is_empty()),
            "{stem}: attributes UNSUPPORTED at the packed tier"
        );
    }

    // Remap reads the RICH log (attributes survive only there).
    let mut rows: Vec<(String, u64, String)> = log
        .events
        .values()
        .map(|ev| {
            let powl = ev
                .attributes
                .get("powl")
                .and_then(serde_json::Value::as_str)
                .expect("contract wire carries attributes[powl]")
                .to_string();
            (ev.case_id.clone(), ev.timestamp_ns, powl)
        })
        .collect();
    rows.sort();
    let names: Vec<String> = rows.into_iter().map(|(_, _, n)| n).collect();

    let words = session_words(&names, &atoms);
    assert_eq!(words.len(), atoms.len());

    let mut session = ConformanceSession::from_powl_file(&model_path(stem))
        .unwrap_or_else(|e| panic!("{stem}: session compile: {e}"));
    session.attach_stream(&words).unwrap_or_else(|e| panic!("{stem}: attach: {e}"));
    match session.step_budget(atoms.len() as u32) {
        YieldStatus::Complete(report) => {
            assert_eq!(report.steps_executed, atoms.len() as u32);
            assert_eq!(report.events_total, atoms.len());
        }
        other => panic!("{stem}: expected Complete, got {other:?}"),
    }
    let cr = session.conformance_report();
    assert!(cr.is_conforming(), "{stem}: report must be conforming: {cr:?}");
}

/// Replay a raw (name, tick) stream and assert the exact violation variant,
/// the `Phase::Faulted` latch, and the frozen post-latch state.
fn assert_violation(
    stem: &str,
    atoms: &[String],
    names: &[String],
    expected: &ConformanceError,
    expected_marking: u64,
    expected_steps: u32,
) {
    let words = session_words(names, atoms);
    // RM2 Fix 4 (operator v26.10.12 spec §4): violation fixtures replay
    // against the NORMATIVE BASE model — no *_violation.powl exists; the
    // court is (base model, violation trace).
    let base_stem = stem.strip_suffix("_violation").unwrap_or(stem);
    let mut session = ConformanceSession::from_powl_file(&model_path(base_stem))
        .unwrap_or_else(|e| panic!("{base_stem}: session compile: {e}"));
    session.attach_stream(&words).unwrap_or_else(|e| panic!("{stem}: attach: {e}"));

    // The first budgeted run reports the violation.
    match session.step_budget(words.len() as u32) {
        YieldStatus::Violation(err) => {
            assert_eq!(&err, expected, "{stem}: exact violation variant");
        }
        other => panic!("{stem}: expected Violation, got {other:?}"),
    }
    // Latch: phase Faulted carrying the same error.
    match session.phase() {
        chicago_tdd_tools::conformance::session::Phase::Faulted(err) => {
            assert_eq!(err, expected, "{stem}: faulted latch carries the error");
        }
        other => panic!("{stem}: expected Faulted phase, got {other:?}"),
    }
    // Frozen state: marking, cursor, steps never advance again.
    assert_eq!(session.marking(), expected_marking, "{stem}: frozen marking");
    assert_eq!(session.steps_done(), expected_steps, "{stem}: frozen steps");
    assert_eq!(
        session.remaining_events(),
        words.len() - expected_steps as usize,
        "{stem}: frozen cursor"
    );
    // resume re-reports the same violation (never advances).
    match session.resume(5) {
        YieldStatus::Violation(err) => assert_eq!(&err, expected),
        other => panic!("{stem}: resume after fault must re-report, got {other:?}"),
    }
    let cr = session.conformance_report();
    assert!(!cr.is_conforming(), "{stem}: faulted report is not conforming");
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

/// Landed fixtures (all golden + violation files + master) carry POWL names in
/// `ocel:activity` instead of the contract's `TestActivity`-variant +
/// `attributes["powl"]` wire form, so import refuses every one of them with a
/// typed missing-field error. This court keeps the divergence loud (fail-closed
/// at the import boundary is correct behavior; the fixtures are the side that
/// diverges from the E-M14 AUTHORITATIVE wire form).
#[test]
fn import_refuses_landed_fixtures_typed() {
    let mut stems: Vec<String> =
        PHASES.iter().flat_map(|p| [format!("{p}"), format!("{p}_violation")]).collect();
    stems.push("master_trace".to_string());
    for stem in &stems {
        let text = std::fs::read_to_string(fixture_path(stem))
            .unwrap_or_else(|e| panic!("read {stem}: {e}"));
        let err = match import_ocel_json(&text) {
            Ok(_) => panic!("{stem}: import must refuse the landed wire form"),
            Err(err) => err,
        };
        assert!(
            matches!(
                &err,
                chicago_tdd_tools::observability::ocel::import_json::ImportError::MissingField {
                    field: "ocel:activity",
                    ..
                }
            ),
            "{stem}: expected typed ocel:activity refusal, got {err:?}"
        );
        // The refusal names the offending event id in its subject.
        if let chicago_tdd_tools::observability::ocel::import_json::ImportError::MissingField {
            subject,
            ..
        } = &err
        {
            assert!(!subject.is_empty());
        }
    }
}

/// Phase 1 (20 atoms) and phase 2 (21 atoms) exceed the linear session net's
/// 16-place budget (`MAX_PETRI_PLACES - 1` = 15 transitions); the model loader
/// refuses with a typed `TooManyPlaces`. No golden replay exists for these
/// phases under the landed kernel.
#[test]
fn phase1_phase2_models_refuse_place_budget() {
    use chicago_tdd_tools::conformance::session::ConformanceSession as CS;
    // Phase 2's declared activity block has 21 entries, but the DFS atom walk
    // yields 22: the divestiture xor lowers to a real xor node whose branch
    // contributes one extra leaf atom. The load-bearing property either way is
    // the place-budget refusal (both counts exceed the 15-transition budget).
    for (stem, atom_count) in [("phase1_intake", 20usize), ("phase2_antitrust", 22)] {
        let atoms = {
            let model = PowlModel::from_powl_file(&model_path(stem))
                .unwrap_or_else(|e| panic!("{stem}: model must load: {e}"));
            let mut a = Vec::new();
            dfs_atoms(&model.root, &mut a);
            a
        };
        assert_eq!(atoms.len(), atom_count, "{stem}: declared atom count");
        let err = CS::from_powl_file(&model_path(stem)).expect_err("compile must refuse");
        assert!(
            matches!(err, ConformanceError::TooManyPlaces { limit: 16 }),
            "{stem}: expected TooManyPlaces(16), got {err:?}"
        );
    }
}

/// Phase 4: full contract pipeline over a generated contract-wire case, plus
/// the landed golden (raw remap) replaying to Complete.
#[test]
fn phase4_golden_pipeline_and_landed_golden_complete() {
    run_contract_pipeline("phase4_novation", "p4-golden");

    let atoms = phase_atoms("phase4_novation");
    let names = fixture_names("phase4_novation");
    assert_eq!(names, atoms, "phase4 landed golden == full DFS atom sequence");
    let mut session = ConformanceSession::from_powl_file(&model_path("phase4_novation"))
        .unwrap_or_else(|e| panic!("phase4 session compile: {e}"));
    let words = session_words(&names, &atoms);
    session.attach_stream(&words).unwrap_or_else(|e| panic!("attach: {e}"));
    match session.step_budget(words.len() as u32) {
        YieldStatus::Complete(report) => {
            assert_eq!(report.steps_executed, 5);
            assert_eq!(report.final_marking, 1 << 5);
        }
        other => panic!("phase4 landed golden: expected Complete, got {other:?}"),
    }
}

/// Phase 4 negative: the landed violation fixture drops
/// `multilateral_compression` (atom 2); replay faults `Skipped` at step 2 and
/// latches frozen.
#[test]
fn phase4_violation_skipped_latches_frozen() {
    let atoms = phase_atoms("phase4_novation");
    let names = fixture_names("phase4_novation_violation");
    assert_violation(
        "phase4_novation_violation",
        &atoms,
        &names,
        &ConformanceError::Skipped { step: 2 },
        1 << 2, // two atoms committed
        2,
    );
}

/// Phase 5 golden (model-derived, each DFS atom once) runs the full contract
/// pipeline to a conforming Complete.
#[test]
fn phase5_golden_pipeline_completes() {
    run_contract_pipeline("phase5_syndicated_debt", "p5-golden");
}

/// Phase 5 landed golden double-fires the loop atoms (`act_vote`,
/// `act_sweep`); the linear lowering admits each atom exactly once, so the
/// second `act_vote` faults `Skipped` at step 4. Court for the loop-repetition
/// law.
#[test]
fn phase5_landed_golden_loop_repetition_faults_skipped() {
    let atoms = phase_atoms("phase5_syndicated_debt");
    let names = fixture_names("phase5_syndicated_debt");
    assert_violation(
        "phase5_syndicated_debt",
        &atoms,
        &names,
        &ConformanceError::Skipped { step: 4 },
        1 << 4,
        4,
    );
}

/// Phase 5 negative: the landed violation fixture replaces the admission
/// branch (`act_admitted`, atom 5) with `act_refused` (atom 6) — declared but
/// not enabled at the cursor -> `Skipped` at step 5, latched frozen.
#[test]
fn phase5_violation_skipped_latches_frozen() {
    let atoms = phase_atoms("phase5_syndicated_debt");
    let names = fixture_names("phase5_syndicated_debt_violation");
    assert_violation(
        "phase5_syndicated_debt_violation",
        &atoms,
        &names,
        &ConformanceError::Skipped { step: 5 },
        1 << 5,
        5,
    );
}

/// Phase 7 golden (model-derived) runs the full contract pipeline to a
/// conforming Complete.
#[test]
fn phase7_golden_pipeline_completes() {
    run_contract_pipeline("phase7_settlement", "p7-golden");
}

/// Phase 7's landed fixtures replay the protocol's LOOPED shape — the
/// intake/elimination block fires twice — but the linear session lowering
/// admits each atom exactly once, so the second `pacs_intake` faults
/// `Skipped` at step 1. Golden and violation both pin the loop-repetition
/// law as a typed court. (B6 converged the fixture vocabulary to the
/// model's declarations, so the earlier `Undeclared` divergence no longer
/// exists on disk; the honest fault is `Skipped { step: 1 }`.)
#[test]
fn phase7_landed_fixtures_loop_repetition_faults_skipped() {
    let atoms = phase_atoms("phase7_settlement");
    for stem in ["phase7_settlement", "phase7_settlement_violation"] {
        let names = fixture_names(stem);
        let words = session_words(&names, &atoms);
        let mut session = ConformanceSession::from_powl_file(&model_path("phase7_settlement"))
            .unwrap_or_else(|e| panic!("phase7 session compile: {e}"));
        session.attach_stream(&words).unwrap_or_else(|e| panic!("attach: {e}"));
        match session.step_budget(words.len() as u32) {
            YieldStatus::Violation(ConformanceError::Skipped { step: 1 }) => {}
            other => panic!("{stem}: expected Skipped at step 1, got {other:?}"),
        }
        assert!(matches!(
            session.phase(),
            chicago_tdd_tools::conformance::session::Phase::Faulted(
                ConformanceError::Skipped { .. }
            )
        ));
        assert_eq!(session.steps_done(), 1, "faulted after exactly one commit");
        assert_eq!(session.marking(), 1 << 1, "frozen after the first intake");
    }
}

// ---------------------------------------------------------------------------
// Phase 10: dossier sealing (lane MX7, E-M10)
// ---------------------------------------------------------------------------

/// Expected DFS atom sequence of `phase10_sealing.powl`: 10 dossier chapters
/// as ONE parallel block (Stage A), deontic closure (Stage B), master merkle
/// root, pillar certs 1-5 as one parallel block, ML-DSA-65 master seal
/// (Stage C, all CTT-host-side).
const PHASE10_ATOMS: [&str; 18] = [
    "dossier_chapter_c01",
    "dossier_chapter_c02",
    "dossier_chapter_c03",
    "dossier_chapter_c04",
    "dossier_chapter_c05",
    "dossier_chapter_c06",
    "dossier_chapter_c07",
    "dossier_chapter_c08",
    "dossier_chapter_c09",
    "dossier_chapter_c10",
    "deontic_closure",
    "master_merkle_root",
    "pillar_cert_1",
    "pillar_cert_2",
    "pillar_cert_3",
    "pillar_cert_4",
    "pillar_cert_5",
    "master_seal",
];

/// Phase 10 model loads and validates (grammar + node budget + declarations),
/// and its composition is exactly the E-M10 shape: 18 declared activities in
/// the parallel(10) -> closure -> root -> parallel(5) -> seal order.
#[test]
fn phase10_model_loads_and_composition_shape_holds() {
    let model = PowlModel::from_powl_file(&model_path("phase10_sealing"))
        .unwrap_or_else(|e| panic!("phase10 model must load: {e}"));
    assert_eq!(model.activities.len(), 18, "phase10 declares 18 activities");

    let mut atoms = Vec::new();
    dfs_atoms(&model.root, &mut atoms);
    let expected: Vec<String> = PHASE10_ATOMS.iter().map(|s| s.to_string()).collect();
    assert_eq!(atoms, expected, "phase10 DFS composition == E-M10 shape");
}

/// Phase 10's 18-atom composition exceeds the linear session net's 15-atom
/// place budget (MAX_PETRI_PLACES - 1), so the session compiler refuses with
/// a typed `TooManyPlaces(16)`. The refusal is the court: no in-session
/// golden replay exists for phase 10; the actual ML-DSA-65 master seal
/// executes host-side via `just emit-dossier` (E-M10 CTT-host-side boundary).
#[test]
fn phase10_session_compile_refuses_place_budget() {
    use chicago_tdd_tools::conformance::session::ConformanceSession as CS;
    let err = CS::from_powl_file(&model_path("phase10_sealing")).expect_err("compile must refuse");
    assert!(
        matches!(err, ConformanceError::TooManyPlaces { limit: 16 }),
        "phase10: expected TooManyPlaces(16), got {err:?}"
    );
}

/// The CTT-host-side boundary is documented in the model source itself: the
/// header names the E-M10 boundary law and `just emit-dossier` as the host
/// executor, and every sealing (Stage C) activity carries a CTT-HOST-SIDE
/// marker on its declaration line. The test asserts the documented boundary,
/// not an in-session seal execution.
#[test]
fn phase10_ctt_host_side_boundary_is_documented() {
    let text = std::fs::read_to_string(model_path("phase10_sealing"))
        .unwrap_or_else(|e| panic!("read phase10 model: {e}"));
    assert!(
        text.contains("CTT-HOST-SIDE BOUNDARY"),
        "phase10: header must document the CTT-host-side boundary"
    );
    assert!(
        text.contains("just emit-dossier"),
        "phase10: host executor (just emit-dossier) must be named"
    );
    // The seven sealing steps: merkle root, 5 pillar certs, master seal.
    for step in [
        "master_merkle_root",
        "pillar_cert_1",
        "pillar_cert_2",
        "pillar_cert_3",
        "pillar_cert_4",
        "pillar_cert_5",
        "master_seal",
    ] {
        let line = text
            .lines()
            .find(|l| l.contains(&format!("activity {step} ")))
            .unwrap_or_else(|| panic!("phase10: declaration for {step}"));
        assert!(
            line.contains("CTT-HOST-SIDE"),
            "phase10: {step} declaration must carry the CTT-HOST-SIDE marker"
        );
    }
    // Ingestion and closure stay guest-side (no host-side marker).
    for guest_step in ["dossier_chapter_c01", "deontic_closure"] {
        let line = text
            .lines()
            .find(|l| l.contains(&format!("activity {guest_step} ")))
            .unwrap_or_else(|| panic!("phase10: declaration for {guest_step}"));
        assert!(!line.contains("CTT-HOST-SIDE"), "phase10: {guest_step} must stay guest-side");
    }
}

/// The phase 10 seal fixture (p10-seal-1) is a landed OCEL 2.0 trace whose
/// activity sequence is exactly the model's DFS composition — one event per
/// atom, case-namespaced `p10-` per E-M09 — including the CTT-host-side
/// boundary attributes on the sealing events.
#[test]
fn phase10_seal_fixture_matches_composition() {
    let atoms = phase_atoms("phase10_sealing");
    let names = fixture_names("phase10_seal");
    assert_eq!(names.len(), 18, "phase10 fixture: 18 events");
    assert_eq!(names, atoms, "phase10 fixture activities == DFS composition");

    let text = std::fs::read_to_string(fixture_path("phase10_seal"))
        .unwrap_or_else(|e| panic!("read phase10 fixture: {e}"));
    let doc: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse phase10 fixture: {e}"));
    for event in doc["ocel:events"].as_object().expect("events object").values() {
        assert_eq!(
            event["ocel:caseId"].as_str().expect("caseId"),
            "p10-seal-1",
            "phase10 fixture: single namespaced case"
        );
    }
    // The sealing events carry the CTT-host-side boundary on the wire.
    for step in ["master_merkle_root", "pillar_cert_3", "master_seal"] {
        let ev = doc["ocel:events"]
            .as_object()
            .unwrap()
            .values()
            .find(|e| e["ocel:activity"].as_str() == Some(step))
            .unwrap_or_else(|| panic!("phase10 fixture: event for {step}"));
        assert_eq!(
            ev["ocel:attributes"]["boundary"].as_str(),
            Some("CTT-host-side"),
            "phase10 fixture: {step} carries the CTT-host-side boundary attribute"
        );
    }
}

// ---------------------------------------------------------------------------
// Phase 9 (E-M09, lane MX6): master composition driver
// ---------------------------------------------------------------------------

/// Total POWL node count (activities + structural blocks), the quantity
/// `MAX_POWL_NODES` bounds (loader.rs:47). Matches the phase-7 header
/// arithmetic (9 activities + sequence + loop + parallel + xor = 14).
fn powl_nodes(node: &PowlNode) -> usize {
    match node {
        PowlNode::Activity(_) | PowlNode::Silent => 1,
        PowlNode::Sequence(children) | PowlNode::Xor(children) | PowlNode::Parallel(children) => {
            1 + children.iter().map(powl_nodes).sum::<usize>()
        }
        PowlNode::Loop { body, redo, .. } => 1 + powl_nodes(body) + powl_nodes(redo),
    }
}

/// The phase9 master composition loads and validates: one root, declared
/// atoms as authored, and inside the loader-wide 64-node cap (39 nodes —
/// the full N01..N20 court fan-out fits, so E-M09's collapsed single-activity
/// cap rule is NOT needed).
#[test]
fn phase9_composition_loads_within_node_budget() {
    let model = PowlModel::from_powl_file(&model_path("phase9_conformance"))
        .unwrap_or_else(|e| panic!("phase9 model must load: {e}"));
    let mut atoms = Vec::new();
    dfs_atoms(&model.root, &mut atoms);
    assert_eq!(atoms.len(), 35, "phase9: declared activity count");
    assert_eq!(powl_nodes(&model.root), 39, "phase9: node budget 39/64");
    assert!(powl_nodes(&model.root) <= 64, "phase9: under MAX_POWL_NODES");
    // Every declaration is bound and every atom declared: DFS atoms are a
    // subset of the declaration block and vice versa (no dead declarations).
    for atom in &atoms {
        assert!(model.activities.contains_key(atom), "phase9: {atom} declared");
    }
    assert_eq!(model.activities.len(), atoms.len(), "phase9: no dead declarations");
}

/// The landed phase9 audit fixture drives the composition: its raw activity
/// stream (the fixtures' own `ocel:activity` convention, p9-namespaced case)
/// is exactly the full DFS atom sequence of the model, in audit-loop order.
#[test]
fn phase9_landed_audit_fixture_matches_composition() {
    let atoms = phase_atoms("phase9_conformance");
    let names = fixture_names("phase9_audit");
    assert_eq!(names, atoms, "phase9 fixture drives the composition in DFS order");
    // Case-id namespacing law (E-M09): every fixture row rides the p9- case.
    let text = std::fs::read_to_string(fixture_path("phase9_audit")).expect("read phase9 fixture");
    assert!(text.contains("\"ocel:caseId\": \"p9-audit-1\""));
    assert!(!text.contains("p9-audit-\"") && text.matches("p9-audit-1").count() == 35);
}

/// Honest harness boundary: the composition compiles as a `PowlModel` but
/// refuses as a single linear `ConformanceSession` (35 atoms > 15-transition
/// place budget) with the same typed `TooManyPlaces` refusal as phases 1/2.
/// The lawful replay path is the Wave-4 chained-session harness (E-M09 cap
/// rule); no single-session golden replay is claimed.
#[test]
fn phase9_session_compile_refuses_place_budget() {
    use chicago_tdd_tools::conformance::session::ConformanceSession as CS;
    let err = CS::from_powl_file(&model_path("phase9_conformance"))
        .expect_err("phase9 single-session compile must refuse");
    assert!(
        matches!(err, ConformanceError::TooManyPlaces { limit: 16 }),
        "phase9: expected TooManyPlaces(16), got {err:?}"
    );
}

/// The violation fixtures of the un-compilable phases (1, 2) are documented,
/// not replayed: their models refuse the place budget, so no linear-net
/// replay exists. Pinned in `phase1_phase2_models_refuse_place_budget`; the
/// fixture files exist and are import-refused with the rest
/// (`import_refuses_landed_fixtures_typed`).
#[test]
fn untestable_phase_violation_fixtures_exist() {
    for stem in ["phase1_intake_violation", "phase2_antitrust_violation"] {
        let text = std::fs::read_to_string(fixture_path(stem))
            .unwrap_or_else(|e| panic!("read {stem}: {e}"));
        assert!(text.contains("\"ocel:version\": \"2.0\""));
    }
}

/// Phase 3 (E-M03, capital/RWA chain): 8-atom linear net, raw-remap pattern.
/// Golden replays to Complete; the violation (CET1 gate skipped) faults
/// `Skipped` at the gate's step and latches frozen.
mod phase3 {
    use super::{assert_violation, fixture_names, model_path, phase_atoms, session_words};
    use chicago_tdd_tools::conformance::error::ConformanceError;
    use chicago_tdd_tools::conformance::session::ConformanceSession;
    use chicago_tdd_tools::conformance::session::YieldStatus;

    /// Atom count of the phase3 net (place budget: compiles, unlike phases
    /// 1/2 — the 16-place session net admits 8 atoms).
    const P3_ATOMS: usize = 8;
    /// Index of `p3_cet1_gate` in the DFS atom sequence (0-based).
    const CET1_GATE_STEP: usize = 3;

    /// Phase 3 golden: the landed fixture's activity names are exactly the
    /// model's DFS atom sequence; the raw remap replays to a conforming
    /// `Complete` with the terminal marking.
    #[test]
    fn phase3_golden_raw_remap_completes() {
        let atoms = phase_atoms("phase3_capital");
        assert_eq!(atoms.len(), P3_ATOMS, "phase3 atom count");
        let names = fixture_names("phase3_capital");
        assert_eq!(names, atoms, "phase3 landed golden == full DFS atom sequence");

        let mut session = ConformanceSession::from_powl_file(&model_path("phase3_capital"))
            .unwrap_or_else(|e| panic!("phase3 session compile: {e}"));
        let words = session_words(&names, &atoms);
        session.attach_stream(&words).unwrap_or_else(|e| panic!("phase3 attach: {e}"));
        match session.step_budget(words.len() as u32) {
            YieldStatus::Complete(report) => {
                assert_eq!(report.steps_executed, P3_ATOMS as u32);
                assert_eq!(report.final_marking, 1 << P3_ATOMS);
            }
            other => panic!("phase3 golden: expected Complete, got {other:?}"),
        }
        let cr = session.conformance_report();
        assert!(cr.is_conforming(), "phase3 golden report must be conforming: {cr:?}");
    }

    /// Phase 3 negative: the violation fixture skips `p3_cet1_gate` (atom 3);
    /// the cursor expects the CET1 receipt, the stream offers `p3_dscr_gate`
    /// (declared but not enabled) -> `Skipped { step: 3 }`, latched frozen.
    #[test]
    fn phase3_violation_cet1_gate_skip_latches_frozen() {
        let atoms = phase_atoms("phase3_capital");
        assert_eq!(atoms[CET1_GATE_STEP], "p3_cet1_gate", "gate index pinned");
        let names = fixture_names("phase3_capital_violation");
        assert_eq!(names.len(), P3_ATOMS - 1, "violation drops exactly the gate");
        assert_violation(
            "phase3_capital_violation",
            &atoms,
            &names,
            &ConformanceError::Skipped { step: CET1_GATE_STEP as u32 },
            1 << CET1_GATE_STEP, // three atoms committed
            CET1_GATE_STEP as u32,
        );
    }
}

// ---------------------------------------------------------------------------
// Phase 8 (lane MX5): Statutory Merger Share Exchange & DGCL Authorization
// ---------------------------------------------------------------------------
//
// Model: examples/merger_matrix/models/phase8_statutory.powl — 6-place/
// 5-transition spine PlanIngest → ContinuityGate → ShareExchange →
// DGCLAuthorize → CertIssue with the four invariants as structure (ordering/
// no-bypass strict sequence, exact share-count equality as a bounded
// conservation loop, one-shot certificate outside the loop, fail-closed
// refusal absorption on the terminal xor branch).

/// Phase 8 golden (model-derived, each DFS atom once — the linear lowering)
/// runs the full contract pipeline to a conforming Complete.
#[test]
fn phase8_golden_pipeline_completes() {
    run_contract_pipeline("phase8_statutory", "p8-golden");
}

/// Phase 8 landed golden: activity names == the full DFS atom sequence, replay
/// reaches Complete with all 6 atoms executed at final marking bit 5, and the
/// certificate fires exactly once (one-shot law).
#[test]
fn phase8_landed_golden_replays_complete_one_shot_cert() {
    let atoms = phase_atoms("phase8_statutory");
    assert_eq!(
        atoms,
        [
            "plan_ingest",
            "continuity_gate",
            "share_exchange",
            "dgcl_authorize",
            "cert_issue",
            "refusal_absorb"
        ],
        "phase8 DFS spine (declaration order)"
    );
    let names = fixture_names("phase8_statutory");
    assert_eq!(names, atoms, "phase8 landed golden == full DFS atom sequence");
    assert_eq!(
        names.iter().filter(|n| *n == "cert_issue").count(),
        1,
        "one-shot certificate: exactly one cert_issue event"
    );

    let words = session_words(&names, &atoms);
    let mut session = ConformanceSession::from_powl_file(&model_path("phase8_statutory"))
        .unwrap_or_else(|e| panic!("phase8 session compile: {e}"));
    session.attach_stream(&words).unwrap_or_else(|e| panic!("attach: {e}"));
    match session.step_budget(words.len() as u32) {
        YieldStatus::Complete(report) => {
            assert_eq!(report.steps_executed, 6);
            assert_eq!(report.final_marking, 1 << 6);
        }
        other => panic!("phase8 landed golden: expected Complete, got {other:?}"),
    }
    assert!(session.conformance_report().is_conforming());
}

/// Phase 8 negative (zero over-issuance, the structural-equality law): the
/// violation fixture carries a `share_exchange` event with exchanged_shares
/// (40,000,001) != outstanding_shares (40,000,000) — one share over-issued.
/// The count law lives in the event attributes, and attributes are
/// UNSUPPORTED at the packed session tier, so the structural replay of the
/// violation skeleton is name-conforming; the divergence is pinned here on the
/// real JSON (golden equality vs violation mismatch) so it stays loud for the
/// invariants-lane court (E-M15), matching the phase-7 trace-metric scoping.
#[test]
fn phase8_violation_over_issuance_pins_structural_equality() {
    let read_share_exchange = |stem: &str| {
        let text = std::fs::read_to_string(fixture_path(stem))
            .unwrap_or_else(|e| panic!("read {stem}: {e}"));
        let doc: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {stem}: {e}"));
        doc["ocel:events"]
            .as_object()
            .expect("events object")
            .values()
            .find(|ev| ev["ocel:activity"] == "share_exchange")
            .unwrap_or_else(|| panic!("{stem}: share_exchange event must exist"))
            .clone()
    };

    let golden = read_share_exchange("phase8_statutory");
    assert_eq!(
        golden["ocel:attributes"]["outstanding_shares"],
        golden["ocel:attributes"]["exchanged_shares"],
        "golden: issued == outstanding (zero over-issuance holds)"
    );

    let violation = read_share_exchange("phase8_statutory_violation");
    let outstanding = violation["ocel:attributes"]["outstanding_shares"]
        .as_u64()
        .expect("outstanding_shares numeric");
    let exchanged = violation["ocel:attributes"]["exchanged_shares"]
        .as_u64()
        .expect("exchanged_shares numeric");
    assert_eq!(outstanding, 40_000_000, "violation: outstanding count");
    assert_eq!(exchanged, outstanding + 1, "violation: exactly one share over-issued");
    assert_eq!(
        violation["ocel:attributes"]["violation"], "over-issuance",
        "violation: typed marker rides the event"
    );

    // The violation skeleton is name-identical to the golden — the fault is
    // purely in the attributes, so the structural net admits it; the count law
    // is enforced at the invariants lane, not net-structural.
    let golden_names = fixture_names("phase8_statutory");
    let violation_names = fixture_names("phase8_statutory_violation");
    assert_eq!(golden_names, violation_names, "violation keeps the golden skeleton");
    let atoms = phase_atoms("phase8_statutory");
    let words = session_words(&violation_names, &atoms);
    let mut session = ConformanceSession::from_powl_file(&model_path("phase8_statutory"))
        .unwrap_or_else(|e| panic!("phase8 session compile: {e}"));
    session.attach_stream(&words).unwrap_or_else(|e| panic!("attach: {e}"));
    match session.step_budget(words.len() as u32) {
        YieldStatus::Complete(_) => {}
        other => panic!("phase8 violation skeleton: expected structural Complete, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Phase 6: Tri-Party Collateral & Treasury Rebalancing (lane MX4, E-M06)
// ---------------------------------------------------------------------------

/// Phase 6 golden (model-derived, each DFS atom once) runs the full contract
/// pipeline to a conforming Complete.
#[test]
fn phase6_golden_pipeline_completes() {
    run_contract_pipeline("phase6_treasury", "p6-golden");
}

/// Phase 6 landed golden (raw remap): the fixture carries exactly the model's
/// 13-atom DFS sequence and replays to Complete with the terminal marking
/// `1 << 13`.
#[test]
fn phase6_landed_golden_completes() {
    let atoms = phase_atoms("phase6_treasury");
    let names = fixture_names("phase6_treasury");
    assert_eq!(names, atoms, "phase6 landed golden == full DFS atom sequence");
    let mut session = ConformanceSession::from_powl_file(&model_path("phase6_treasury"))
        .unwrap_or_else(|e| panic!("phase6 session compile: {e}"));
    let words = session_words(&names, &atoms);
    session.attach_stream(&words).unwrap_or_else(|e| panic!("attach: {e}"));
    match session.step_budget(words.len() as u32) {
        YieldStatus::Complete(report) => {
            assert_eq!(report.steps_executed, 13);
            assert_eq!(report.final_marking, 1 << 13);
        }
        other => panic!("phase6 landed golden: expected Complete, got {other:?}"),
    }
}

/// Phase 6 negative: the landed violation fixture skips the normative LCR
/// gate (`act_assert_lcr_ge_128`, atom 5) — replay faults `Skipped` at step 5
/// and latches frozen. The gate can never be bypassed silently (A05).
#[test]
fn phase6_violation_lcr_gate_skipped_latches_frozen() {
    let atoms = phase_atoms("phase6_treasury");
    let names = fixture_names("phase6_treasury_violation");
    assert_violation(
        "phase6_treasury_violation",
        &atoms,
        &names,
        &ConformanceError::Skipped { step: 5 },
        1 << 5, // five atoms committed, frozen before the gate
        5,
    );
}
