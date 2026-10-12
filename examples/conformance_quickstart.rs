//! Conformance quickstart: POWL model in, structured verdict out.
//!
//! Requires `--features conformance`:
//!
//! ```text
//! cargo run --example conformance_quickstart --features conformance
//! ```
//!
//! Pipeline: write a `.powl` DSL fixture to a tempfile, load it with
//! [`chicago_tdd_tools::conformance::ConformanceSession::from_powl_file`]
//! (fail-closed parse, positioned errors), attach an OCEL64 event word
//! stream (activity id in bits [7:0], tick in bits [15:8], ticks strictly
//! `prev + 1`), then drive the replay with cooperative `step_budget`
//! slices, printing each [`YieldStatus`] until the trajectory completes,
//! and assert the final report is conforming.

use chicago_tdd_tools::conformance::{ConformanceSession, ReportOutcome, YieldStatus};

/// Minimal POWL DSL model: a three-atom sequence, one activity per system.
const POWL_FIXTURE: &str = r#"
# Three-atom linear process: handle -> fork -> approve
activity handle  = log.handle
activity fork    = robot.fork
activity approve = ledger.approve

sequence {
    handle
    fork
    approve
}
"#;

/// OCEL64 event word: activity id in bits [7:0], tick in bits [15:8].
fn event(tid: u8, tick: u8) -> u64 {
    u64::from(tid) | (u64::from(tick) << 8)
}

fn main() {
    // 1. Persist the DSL fixture so the loader reads a real file path.
    let dir = tempfile::tempdir().expect("tempdir");
    let model_path = dir.path().join("quickstart.powl");
    std::fs::write(&model_path, POWL_FIXTURE).expect("write fixture");

    // 2. Fail-closed load: parse errors are positioned and typed.
    let mut session = ConformanceSession::from_powl_file(&model_path).expect("parse POWL fixture");

    // 3. Attach an OCEL64 word stream: tids 0,1,2 at ticks 1,2,3.
    let words = [event(0, 1), event(1, 2), event(2, 3)];
    session.attach_stream(&words).expect("attach stream");

    // 4. Cooperative reductions: step in bounded slices, printing each
    //    YieldStatus, resuming until the session leaves the Yielded phase.
    let report = loop {
        match session.step_budget(2) {
            YieldStatus::Yielded { remaining_events } => {
                println!("yielded: {remaining_events} event(s) remaining");
            }
            YieldStatus::Complete(report) => break report,
            YieldStatus::Violation(err) => {
                panic!("conformance violation: {err}");
            }
        }
    };

    // 5. Assert the structured verdict: every event replayed, no violation.
    assert_eq!(report.steps_executed, words.len() as u32);
    assert_eq!(report.events_total, words.len());
    assert_eq!(report.outcome, ReportOutcome::Complete);
    println!("verdict: CONFORMING ({} steps)", report.steps_executed);
}
