//! Conformance subsystem integration tests (lane C12, plan P5).
//!
//! Real collaborators, no mocks (Chicago): every test drives a real
//! [`ConformanceSession`] over a real POWL DSL file written into a unique
//! `tempfile` directory (cleaned by drop), replaying real OCEL64 event
//! words. Gated on the `conformance` feature like `tests/ocel_tests.rs`.
//!
//! Event-word layout (session lane contract): activity id in bits [7:0],
//! tick in bits [15:8]; ticks must advance exactly +1 (mod 256) from the
//! second event on; tid = DFS atom index of the compiled linear net.
#![cfg(feature = "conformance")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — assertions on real replayed state.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use chicago_tdd_tools::conformance::error::ConformanceError;
use chicago_tdd_tools::conformance::report::{ConformanceReport, Deviation};
use chicago_tdd_tools::conformance::session::{
    ConformanceSession, Phase, ReportOutcome, YieldStatus,
};
use chicago_tdd_tools::core::receipt::{
    EnvironmentFingerprint, TestOutcome, TestReceipt, TimingMeasurement,
};

// ---------------------------------------------------------------------------
// Fixtures: real POWL DSL files in unique temp dirs (cleaned on drop)
// ---------------------------------------------------------------------------

/// Write `src` as a `.powl` file in a unique temp dir; returns (path, guard).
fn write_powl_fixture(name: &str, src: &str) -> (PathBuf, tempfile::TempDir) {
    let dir = tempfile::Builder::new()
        .prefix("ctdd_conformance_tests_")
        .tempdir()
        .expect("unique temp dir");
    let path = dir.path().join(name);
    std::fs::write(&path, src).expect("write fixture");
    (path, dir)
}

const THREE_ATOM_POWL: &str = "\
activity handle = log.handle
activity fork = robot.fork
activity approve = log.approve

# linear order under test
sequence {
    handle
    fork
    approve
}
";

const SEVEN_ATOM_POWL: &str = "\
activity a0 = sys.a0
activity a1 = sys.a1
activity a2 = sys.a2
activity a3 = sys.a3
activity a4 = sys.a4
activity a5 = sys.a5
activity a6 = sys.a6

sequence {
    a0
    a1
    a2
    a3
    a4
    a5
    a6
}
";

/// OCEL64 event word: activity id in bits [7:0], tick in bits [15:8].
fn event(tid: u8, tick: u8) -> u64 {
    u64::from(tid) | (u64::from(tick) << 8)
}

/// Compile a fixture file, panicking with the loader/validator diagnostic.
fn session_from_file(path: &std::path::Path) -> ConformanceSession {
    match ConformanceSession::from_powl_file(path) {
        Ok(s) => s,
        Err(e) => panic!("from_powl_file({path:?}) failed: {e}"),
    }
}

/// A freshly compiled 3-atom session with no stream attached.
/// The temp dir guard is intentionally dropped after load — the loader reads
/// the file synchronously in `from_powl_file`, so the file is not needed
/// after compilation.
fn three_atom() -> ConformanceSession {
    let (path, _dir) = write_powl_fixture("three_atom.powl", THREE_ATOM_POWL);
    session_from_file(&path)
}

/// The golden 3-event lawful trajectory.
fn golden_stream() -> Vec<u64> {
    vec![event(0, 1), event(1, 2), event(2, 3)]
}

// ---------------------------------------------------------------------------
// 1. Golden trajectory: Complete + conforming
// ---------------------------------------------------------------------------

#[test]
fn golden_trajectory_is_complete_and_conforming() {
    let mut s = three_atom();
    s.attach_stream(&golden_stream()).expect("golden stream attaches");
    match s.step_budget(16) {
        YieldStatus::Complete(summary) => {
            // ReplaySummary fields (replay lane).
            assert_eq!(summary.steps_executed, 3);
            assert_eq!(summary.events_total, 3);
            assert_eq!(summary.outcome, ReportOutcome::Complete);
            assert_eq!(summary.final_marking, 1 << 3, "token exits the chain");
        }
        other => panic!("expected Complete, got {other:?}"),
    }
    assert_eq!(*s.phase(), Phase::Complete);
    // Report-lane split: the laws-type verdict comes from
    // `conformance_report()` (computed, never asserted by the caller).
    let report = s.conformance_report();
    assert!(report.is_conforming(), "clean replay conforms");
    assert_eq!(report.open_deviations(), 0);
    // Same law on a freshly built report (accessor-based construction).
    let report = ConformanceReport::new(3, 3, Vec::new());
    assert!(report.is_conforming(), "clean replay conforms");
    assert_eq!(report.open_deviations(), 0);
}

// ---------------------------------------------------------------------------
// 1b. SX8-F1: prefix exhaustion is Complete but NOT conforming
// ---------------------------------------------------------------------------

/// A lawful prefix (2 of 3 events) exhausts the stream: the replay latches
/// `Phase::Complete` and the summary outcome is `Complete` (the trace played
/// lawfully), but the token rests at place 2 — not the terminal place 3 — so
/// the structured verdict records `Unfinished` and `is_conforming()` is
/// `false`.
#[test]
fn sx8f1_prefix_trace_is_complete_but_not_conforming() {
    let mut s = three_atom();
    s.attach_stream(&[event(0, 1), event(1, 2)]).expect("prefix stream attaches");
    match s.step_budget(16) {
        YieldStatus::Complete(summary) => {
            assert_eq!(summary.outcome, ReportOutcome::Complete);
            assert_eq!(summary.steps_executed, 2);
            assert_eq!(summary.final_marking, 1 << 2);
        }
        other => panic!("expected Complete, got {other:?}"),
    }
    assert_eq!(*s.phase(), Phase::Complete);
    let report = s.conformance_report();
    assert!(
        report
            .deviations()
            .iter()
            .any(|d| matches!(d, Deviation::Unfinished { steps_done: 2, total_events: 2 })),
        "expected Unfinished deviation on the exhausted prefix, got {report:?}"
    );
    assert!(!report.is_conforming(), "a lawful prefix is unfinished, never conforming");
}

/// The `assert_conforms!` macro path must panic on a prefix trace: the
/// session's own verdict is not conforming even though the replay completed
/// without violation and the receipt is algebraically closed.
#[test]
#[should_panic = "conformance failed"]
fn sx8f1_assert_conforms_panics_on_prefix_trace() {
    let mut s = three_atom();
    s.attach_stream(&[event(0, 1), event(1, 2)]).expect("prefix stream attaches");
    let _ = s.step_budget(16);
    chicago_tdd_tools::assert_conforms!(s, pass_receipt());
}

// ---------------------------------------------------------------------------
// 2. Skipped-state refusal (Skipped / Undeclared)
// ---------------------------------------------------------------------------

#[test]
fn skipped_tick_gap_refuses_as_skipped() {
    let mut s = three_atom();
    // tick 1 -> 3: neither prev (repeat) nor prev+1 => Skipped.
    s.attach_stream(&[event(0, 1), event(1, 3)]).expect("stream attaches");
    match s.step_budget(16) {
        YieldStatus::Violation(ConformanceError::Skipped { step }) => {
            assert_eq!(step, 1);
        }
        other => panic!("expected Skipped, got {other:?}"),
    }
    assert!(matches!(s.phase(), Phase::Faulted(ConformanceError::Skipped { .. })));
}

#[test]
fn undeclared_activity_refuses_as_undeclared() {
    let mut s = three_atom();
    // tid 9 is not compiled into the 3-atom net.
    s.attach_stream(&[event(0, 1), event(9, 2)]).expect("stream attaches");
    match s.step_budget(16) {
        YieldStatus::Violation(ConformanceError::Undeclared { step, tid }) => {
            assert_eq!(step, 1);
            assert_eq!(tid, 9);
        }
        other => panic!("expected Undeclared, got {other:?}"),
    }
    // Latch + frozen-state law: the session is Faulted and its cursor,
    // steps, and marking are frozen bit-exact after the refusal.
    assert!(matches!(s.phase(), Phase::Faulted(_)));
    assert_eq!(s.steps_done(), 1);
    let marking_after = s.marking();
    let phase_after = s.phase().clone();
    let _ = s.step_budget(16);
    assert_eq!(s.marking(), marking_after);
    assert_eq!(s.phase(), &phase_after);
}

// ---------------------------------------------------------------------------
// 3. Out-of-order refusal (repeated tick)
// ---------------------------------------------------------------------------

#[test]
fn repeated_tick_refuses_as_out_of_order_and_latches() {
    let mut s = three_atom();
    s.attach_stream(&[event(0, 1), event(1, 1)]).expect("stream attaches");
    match s.step_budget(16) {
        YieldStatus::Violation(ConformanceError::OutOfOrder { prev_tick, tick, .. }) => {
            assert_eq!(prev_tick, 1);
            assert_eq!(tick, 1);
        }
        other => panic!("expected OutOfOrder, got {other:?}"),
    }
    // Latch: frozen state, re-replay reports the same violation.
    assert_eq!(s.steps_done(), 1);
    assert_eq!(s.remaining_events(), 1);
    assert!(matches!(s.resume(5), YieldStatus::Violation(_)));
    assert!(matches!(s.phase(), Phase::Faulted(ConformanceError::OutOfOrder { .. })));
}

// ---------------------------------------------------------------------------
// 4. Budget yield loop: remaining decreases monotonically 5 -> 3 -> 1 -> 0
// ---------------------------------------------------------------------------

#[test]
fn budget_yield_loop_remaining_is_monotonic() {
    let (path, _dir) = write_powl_fixture("seven.powl", SEVEN_ATOM_POWL);
    let mut s = session_from_file(&path);
    let stream: Vec<u64> = (0..7).map(|i| event(i as u8, i as u8 + 1)).collect();
    s.attach_stream(&stream).expect("7-event stream attaches");

    let mut previous = 7usize;
    for expected in [5usize, 3, 1] {
        match s.step_budget(2) {
            YieldStatus::Yielded { remaining_events } => {
                assert_eq!(remaining_events, expected);
            }
            other => panic!("expected Yielded({expected}), got {other:?}"),
        }
        assert_eq!(s.remaining_events(), expected);
        assert!(expected < previous, "remaining must decrease monotonically");
        previous = expected;
    }
    // Final quota drains the trace: 0 remaining, Complete.
    match s.step_budget(2) {
        YieldStatus::Complete(report) => {
            assert_eq!(report.steps_executed, 7);
            assert_eq!(report.outcome, ReportOutcome::Complete);
        }
        other => panic!("expected Complete, got {other:?}"),
    }
    assert_eq!(s.remaining_events(), 0);
}

// ---------------------------------------------------------------------------
// 5. Budget exhaustion yields, never violates
// ---------------------------------------------------------------------------

#[test]
fn exhausted_budget_yields_and_does_not_violate() {
    let mut s = three_atom();
    s.attach_stream(&golden_stream()).expect("attach");

    // Zero quota: nothing steps, nothing violates.
    match s.step_budget(0) {
        YieldStatus::Yielded { remaining_events } => assert_eq!(remaining_events, 3),
        other => panic!("expected Yielded, got {other:?}"),
    }
    assert_eq!(s.steps_done(), 0);
    assert!(!matches!(s.phase(), Phase::Faulted(_)), "quota exhaustion is not a violation");

    // Partial quota: yields with the lawful remainder, still no violation.
    match s.step_budget(2) {
        YieldStatus::Yielded { remaining_events } => assert_eq!(remaining_events, 1),
        other => panic!("expected Yielded, got {other:?}"),
    }
    // Resume finishes cleanly.
    match s.resume(4) {
        YieldStatus::Complete(report) => assert_eq!(report.steps_executed, 3),
        other => panic!("expected Complete, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 6. Forbidden activity refusal
// ---------------------------------------------------------------------------

#[test]
fn forbidden_marking_refuses_as_forbidden_bit() {
    let mut s = three_atom();
    // After firing tid 0 the marking is place 1 (bit 1); forbid it.
    s.set_forbidden_mask(0b0010);
    s.attach_stream(&[event(0, 1)]).expect("attach");
    match s.step_budget(16) {
        YieldStatus::Violation(ConformanceError::ForbiddenBit { step, marking, mask }) => {
            assert_eq!(step, 0);
            assert_eq!(marking, 0b0010);
            assert_eq!(mask, 0b0010);
        }
        other => panic!("expected ForbiddenBit, got {other:?}"),
    }
    // Frozen before the violating step committed.
    assert_eq!(s.marking(), 1);
    assert_eq!(s.steps_done(), 0);
}

// ---------------------------------------------------------------------------
// 7. POWL model refusals via file (UnknownActivity, TooLarge/TooManyPlaces)
// ---------------------------------------------------------------------------

#[test]
fn unknown_activity_in_powl_file_refuses() {
    let src = "activity handle = log.handle\n\
               sequence {\n    handle\n    ghost\n}\n";
    let (path, _dir) = write_powl_fixture("unknown_activity.powl", src);
    let err =
        ConformanceSession::from_powl_file(&path).expect_err("undeclared reference must refuse");
    match err {
        ConformanceError::UnknownActivity { name, .. } => assert_eq!(name, "ghost"),
        other => panic!("expected UnknownActivity, got {other:?}"),
    }
}

/// At-limit admission (T4): exactly 15 atoms need 16 places — the PN64
/// marking-field budget — so the cap is inclusive (`from_model` admits).
/// Companion refusal: `oversized_powl_model_refuses_at_place_budget` (16 atoms).
#[test]
fn fifteen_atom_sequence_compiles_at_place_budget() {
    let mut src = String::from("# 15 atoms: at the 16-place budget\n");
    for i in 0..15 {
        src.push_str(&format!("activity big{i} = sys.big{i}\n"));
    }
    src.push_str("\nsequence {\n");
    for i in 0..15 {
        src.push_str(&format!("    big{i}\n"));
    }
    src.push_str("}\n");
    let (path, _dir) = write_powl_fixture("at_place_budget.powl", &src);
    let s = session_from_file(&path);
    assert_eq!(s.net().len(), 15);
    // The full chain replays to Complete + conforming: a 15-step proof at the
    // place budget.
    let stream: Vec<u64> = (0..15).map(|i| event(i as u8, i as u8 + 1)).collect();
    let mut s = s;
    s.attach_stream(&stream).expect("15-event stream attaches");
    match s.step_budget(32) {
        YieldStatus::Complete(summary) => {
            assert_eq!(summary.steps_executed, 15);
            assert_eq!(summary.final_marking, 1 << 15);
        }
        other => panic!("expected Complete, got {other:?}"),
    }
    assert!(s.conformance_report().is_conforming(), "at-budget chain conforms");
}

/// At-limit parse (T4): a 64-node model (`MAX_POWL_NODES`) parses Ok; the
/// session then refuses it at the 16-place budget, not the 64-node budget.
#[test]
fn sixty_four_node_model_parses_and_refuses_at_place_budget() {
    let count = chicago_tdd_tools::conformance::loader::MAX_POWL_NODES - 1;
    let mut src = String::new();
    for i in 0..count {
        src.push_str(&format!("activity n{i} = sys.n{i}\n"));
    }
    src.push_str("sequence { ");
    for i in 0..count {
        if i > 0 {
            src.push(' ');
        }
        src.push_str(&format!("n{i}"));
    }
    src.push_str(" }");
    let model = chicago_tdd_tools::conformance::loader::PowlModel::from_powl_str(&src)
        .expect("64-node model parses");
    assert_eq!(model.node_count(), chicago_tdd_tools::conformance::loader::MAX_POWL_NODES);
    let err = ConformanceSession::from_model(&model)
        .expect_err("64 atoms need 65 places, past the 16-place budget");
    assert!(
        matches!(err, ConformanceError::TooManyPlaces { .. }),
        "expected TooManyPlaces, got {err:?}"
    );
}

#[test]
fn oversized_powl_model_refuses_at_place_budget() {
    // 16 atoms need 17 places; the PN64 marking field budget is 16.
    let mut src = String::from("# 16 atoms: one past the place budget\n");
    for i in 0..16 {
        src.push_str(&format!("activity big{i} = sys.big{i}\n"));
    }
    src.push_str("\nsequence {\n");
    for i in 0..16 {
        src.push_str(&format!("    big{i}\n"));
    }
    src.push_str("}\n");
    let (path, _dir) = write_powl_fixture("too_large.powl", &src);
    let err = ConformanceSession::from_powl_file(&path).expect_err("oversized model must refuse");
    assert!(
        matches!(err, ConformanceError::TooManyPlaces { .. }),
        "expected TooManyPlaces, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// 8. Exception-covers-deviation closure
// ---------------------------------------------------------------------------

#[test]
fn exception_closes_deviation_and_restores_conformance() {
    let mut report = ConformanceReport::new(
        4,
        4,
        vec![
            Deviation::VerbMismatch { step: 1, tid: 3, expected: "assemble".to_string() },
            Deviation::NotEnabled { step: 2, tid: 7, marking: 0b1010 },
        ],
    );
    assert_eq!(report.open_deviations(), 2);
    assert!(!report.is_conforming());

    report.explain(0).expect("in-range index");
    assert_eq!(report.open_deviations(), 1, "exactly one closes");
    assert!(!report.is_conforming());

    report.explain(1).expect("in-range index");
    assert_eq!(report.open_deviations(), 0);
    assert!(report.is_conforming(), "fully explained finished trajectory conforms");
}

// ---------------------------------------------------------------------------
// 9. Tampered / misordered trace is not conforming
// ---------------------------------------------------------------------------

#[test]
fn misordered_trace_refuses_and_is_not_conforming() {
    let mut s = three_atom();
    // Tampered: tid 0 replayed while the chain sits at place 1 — declared,
    // lawful tick, but out of the model's linear order.
    s.attach_stream(&[event(0, 1), event(0, 2)]).expect("stream attaches");
    match s.step_budget(16) {
        YieldStatus::Violation(ConformanceError::Skipped { step }) => {
            assert_eq!(step, 1);
        }
        other => panic!("expected Skipped for misordered replay, got {other:?}"),
    }
    // Report-lane split: the latched session's own verdict is not conforming.
    // Two open deviations: the mapped violation (Skipped now carries its real
    // step in the report vocabulary) plus the automatic Unfinished recorded
    // because steps (1) < total_events (2).
    let session_report = s.conformance_report();
    assert_eq!(session_report.open_deviations(), 2);
    assert!(
        session_report
            .deviations()
            .iter()
            .any(|d| matches!(d, Deviation::Skipped { step: 1 })),
        "the Skipped violation must surface with its real step, got {session_report:?}"
    );
    assert!(!session_report.is_conforming(), "tampered trace never conforms");
    // Same law on a directly built report.
    let report = ConformanceReport::new(1, 2, vec![Deviation::Skipped { step: 1 }]);
    // Same shape as the session report above: the Skipped deviation plus
    // the automatic Unfinished (steps 1 < total_events 2) = 2 open deviations.
    assert_eq!(report.open_deviations(), 2);
    assert!(!report.is_conforming(), "tampered trace never conforms");
}

// ---------------------------------------------------------------------------
// 10/11. Macro-level positive + #[should_panic] negative
// ---------------------------------------------------------------------------
// `assert_conforms!(session, receipt)` asserts BOTH halves of the seam:
// `session.conformance_report().is_conforming()` and
// `receipt.is_algebraically_closed()`. A passing replay needs a passing
// (algebraically closed) receipt alongside it.

/// A signed, passing `TestReceipt` (algebraically closed: signature verifies,
/// outcome is not Fail/Error).
fn pass_receipt() -> TestReceipt {
    let mut receipt = TestReceipt::new(
        "golden_trajectory".to_string(),
        "code-hash".to_string(),
        EnvironmentFingerprint::capture(),
        vec!["trajectory conforms".to_string()],
        TimingMeasurement::new(0, 0, "ambient".to_string(), true, 0),
        Vec::new(),
        TestOutcome::Pass,
    );
    receipt.sign();
    receipt
}

#[test]
fn assert_conforms_macro_accepts_golden_trajectory() {
    let mut s = three_atom();
    s.attach_stream(&golden_stream()).expect("golden stream attaches");
    let _ = s.step_budget(16);
    chicago_tdd_tools::assert_conforms!(s, pass_receipt());
    assert_eq!(s.steps_done(), 3);
}

#[test]
#[should_panic = "conformance"]
fn assert_conforms_macro_panics_on_violation() {
    let mut s = three_atom();
    // Tampered stream: second event repeats tid 0 out of order — the session
    // verdict is not conforming even with a closed receipt.
    s.attach_stream(&[event(0, 1), event(0, 2)]).expect("stream attaches");
    let _ = s.step_budget(16);
    chicago_tdd_tools::assert_conforms!(s, pass_receipt());
}

// ---------------------------------------------------------------------------
// 12. ConformanceReceipt roundtrip — SKIPPED (receipt.rs has not landed)
// ---------------------------------------------------------------------------
// The receipt lane's `ConformanceReceipt` (serde roundtrip test) is
// intentionally absent: the source file had not landed at the time this
// suite was written. Add `test_conformance_receipt_roundtrip` here once
// `src/conformance/receipt.rs` lands, asserting
// `serde_json::to_string -> from_str == original` on a receipt built from a
// completed replay.

// ---------------------------------------------------------------------------
// Shared-fixture sanity: loader accepts the doc'd DSL shape used above
// ---------------------------------------------------------------------------

#[test]
fn fixture_dsl_declares_activities_in_order() {
    let (path, _dir) = write_powl_fixture("decls.powl", THREE_ATOM_POWL);
    let model = chicago_tdd_tools::conformance::loader::PowlModel::from_powl_file(&path)
        .expect("golden fixture parses");
    assert_eq!(
        model.declaration_order,
        vec!["handle".to_string(), "fork".to_string(), "approve".to_string()]
    );
    // Session compiles it into a 3-transition linear net.
    let s = ConformanceSession::from_model(&model).expect("compiles");
    assert_eq!(s.net().len(), 3);
}

// ---------------------------------------------------------------------------
// 13. SK8 regressions: self-loop sentinel + model-relative trajectory identity
// ---------------------------------------------------------------------------

/// The unchanged-word refusal law holds at the public boundary: a declared
/// activity replayed out of the model's linear order (its pre-place holds no
/// token) is refused as `Skipped` even though the refusal predicate no longer
/// relies on word identity. Companion to the in-module self-loop tests: a
/// self-loop firing (W- == W+) also leaves the word unchanged but must COMMIT.
#[test]
fn unchanged_word_refusal_holds_at_public_boundary() {
    let mut s = three_atom();
    // tid 1 is declared but place 1 holds no token yet => Skipped.
    s.attach_stream(&[event(1, 1)]).expect("stream attaches");
    match s.step_budget(16) {
        YieldStatus::Violation(ConformanceError::Skipped { step }) => assert_eq!(step, 0),
        other => panic!("expected Skipped, got {other:?}"),
    }
    // Frozen before any commit.
    assert_eq!(s.marking(), 1);
    assert_eq!(s.steps_done(), 0);
}

/// Model-relative trajectory identity: `tid` is the DFS index of the atom in
/// the compiled net (positional), never absolute. The same activity byte is
/// lawful in one net and refused in another compiled from a different model.
#[test]
fn trajectory_identity_is_model_relative_not_absolute() {
    // One-atom model: only tid 0 exists; event tid 1 is Undeclared.
    let src_single = "activity only = sys.only\n\nsequence {\n    only\n}\n";
    let (path, _dir) = write_powl_fixture("single.powl", src_single);
    let mut single = session_from_file(&path);
    single.attach_stream(&[event(1, 1)]).expect("stream attaches");
    match single.step_budget(16) {
        YieldStatus::Violation(ConformanceError::Undeclared { step, tid }) => {
            assert_eq!((step, tid), (0, 1));
        }
        other => panic!("expected Undeclared, got {other:?}"),
    }

    // Two-atom model: tid 1 is declared, but positionally second — firing it
    // first is out of the model's linear order, so it is still refused.
    let src_pair = "activity first = sys.first\n\
                    activity second = sys.second\n\n\
                    sequence {\n    first\n    second\n}\n";
    let (path2, _dir2) = write_powl_fixture("pair.powl", src_pair);
    let mut pair = session_from_file(&path2);
    pair.attach_stream(&[event(1, 1)]).expect("stream attaches");
    match pair.step_budget(16) {
        YieldStatus::Violation(ConformanceError::Skipped { step }) => assert_eq!(step, 0),
        other => panic!("expected Skipped, got {other:?}"),
    }
}

// ─── T20: error-variant coverage gaps ───────────────────────────────────────
//
// BcinrValidation (cross-cutting pass-through) previously had zero coverage.
// Note: `ConformanceError::DependencyCycle` has zero raise sites anywhere in
// `src/` — flagged dead code (see T20 report); no reachability test exists
// because none is possible.

#[test]
fn t20_bcincr_validation_display_is_populated() {
    let e = ConformanceError::BcinrValidation { message: "witness diagnostic".into() };
    let s = e.to_string();
    assert!(!s.is_empty());
    assert!(s.contains("BCINR validation failed"), "{s}");
    assert!(s.contains("witness diagnostic"), "{s}");
}

#[test]
fn t20_bcincr_validation_reachable_from_report_explain() {
    use chicago_tdd_tools::conformance::report::Deviation;
    // Real public API: explain() with an out-of-range deviation index raises
    // the cross-cutting BcinrValidation pass-through.
    let mut report = ConformanceReport::new(
        2,
        2,
        vec![
            Deviation::NotEnabled { step: 1, tid: 0, marking: 0b10 },
            Deviation::NotEnabled { step: 2, tid: 1, marking: 0b01 },
        ],
    );
    report.explain(0).expect("index 0 is in range");
    match report.explain(9) {
        Err(ConformanceError::BcinrValidation { message }) => {
            assert!(message.contains("out of range"), "{message}");
        }
        other => panic!("expected BcinrValidation, got {other:?}"),
    }
}

#[test]
fn t20_bcincr_validation_reachable_from_net_compile_duplicate_place() {
    use chicago_tdd_tools::conformance::net::NetSpec;
    // Real public API: compiling a net with a duplicated place name raises
    // BcinrValidation with the validator's diagnostic.
    let spec = NetSpec::builder()
        .place("p0")
        .place("p0")
        .transition("t", "act")
        .add_pre("p0")
        .build();
    let err = spec.compile().expect_err("duplicate place name must be refused");
    match err {
        ConformanceError::BcinrValidation { message } => {
            assert!(message.contains("duplicate place name"), "{message}");
        }
        other => panic!("expected BcinrValidation, got {other:?}"),
    }
}
