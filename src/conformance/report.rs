//! Conformance reporting: deviations, exception closure, verdicts.
//!
//! Documented law (exception-covers-deviation closure): a deviation is
//! *explained* when and only when it is covered by a registered exception.
//! `ConformanceReport::explain` registers that coverage; `open_deviations`
//! counts the uncovered remainder; `is_conforming` requires zero uncovered
//! deviations AND a finished trajectory.

use serde::{Deserialize, Serialize};

use crate::conformance::error::ConformanceError;

/// One observed departure from the specification during replay.
///
/// Step numbers are 1-based where they refer to trajectory positions;
/// `tid` values are transition ids; `marking` is the raw marking bitset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Deviation {
    /// A step fired the right transition but under the wrong activity name.
    VerbMismatch {
        /// Trajectory step at which the mismatch was observed.
        step: u32,
        /// Transition id that fired.
        tid: u8,
        /// The activity name the specification expected.
        expected: String,
    },
    /// A step's transition was not enabled by the observed marking.
    NotEnabled {
        /// Trajectory step at which the step was attempted.
        step: u32,
        /// Transition id that was not enabled.
        tid: u8,
        /// The observed marking bits.
        marking: u16,
    },
    /// The observed marking intersected a forbidden-bit mask.
    ForbiddenBit {
        /// Trajectory step at which the forbidden marking was produced.
        step: u32,
        /// The observed marking bits.
        marking: u16,
    },
    /// A step's tick regressed relative to the previous accepted tick.
    OutOfOrder {
        /// Trajectory step at which the regression was observed.
        step: u32,
        /// The tick of the previous accepted step.
        prev_tick: u8,
        /// The offending tick.
        tick: u8,
    },
    /// The trajectory ended before all recorded events were replayed.
    Unfinished {
        /// Steps actually replayed.
        steps_done: usize,
        /// Total events in the recorded trace.
        total_events: usize,
    },
    /// A step referenced a transition id never declared in the model.
    Undeclared {
        /// Trajectory step at which the undeclared id was offered.
        step: u32,
        /// The undeclared transition id.
        tid: u8,
    },
    /// A declared step was out of the model's linear order (or the tick
    /// stream skipped ahead), so the event was refused.
    Skipped {
        /// Trajectory step at which the refusal occurred.
        step: u32,
    },
}

/// Terminal status of a replay run.
///
/// Canonical home (E1 item 2): moved here from `session.rs` (which was the
/// only consumer); `session.rs` re-exports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportOutcome {
    /// Every recorded event replayed without violation.
    Complete,
    /// Replay latched on a classified violation.
    Faulted(ConformanceError),
    /// The stream ended (or the session stopped) before completion.
    Incomplete,
}

/// Replay summary produced by [`super::session::ConformanceSession`].
///
/// Renamed from session.rs's former local `ConformanceReport` (E1 item 2) —
/// the name `ConformanceReport` stays with the accessor-based verdict type
/// below. Field names are load-bearing: session tests and
/// `tests/conformance_tests.rs` assert `steps_executed` / `events_total` /
/// `final_marking` / `outcome` directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaySummary {
    /// Trajectory steps actually replayed.
    pub steps_executed: u32,
    /// Total events bound to the session's stream.
    pub events_total: usize,
    /// Raw PN64 marking after the final accepted step (session's
    /// `Marking64 = u64` alias; distinct from `super::state::Marking64`).
    pub final_marking: u64,
    /// Terminal status of the replay.
    pub outcome: ReportOutcome,
}

/// Cooperative step result: control returns to the caller each slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YieldStatus {
    /// The step budget was exhausted with events still to replay.
    Yielded {
        /// Events not yet replayed.
        remaining_events: usize,
    },
    /// Replay reached a terminal outcome; carries the replay summary.
    Complete(ReplaySummary),
    /// A violation latched the session; carries the classified error.
    Violation(ConformanceError),
}

/// Replay state-machine phase of a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Constructed, no stream attached.
    Idle,
    /// Stream attached, stepping in progress.
    Running,
    /// A step budget was exhausted mid-stream.
    Yielded,
    /// Replay finished without violation.
    Complete,
    /// A violation latched the session; it is frozen from here on.
    Faulted(ConformanceError),
}

/// Structured verdict over a replayed trajectory.
///
/// Field privacy with accessor-based laws: callers cannot construct a
/// self-contradictory report (e.g. claim conformance while deviations are
/// open); the verdict is computed, never asserted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// Every deviation observed during replay, in observation order.
    deviations: Vec<Deviation>,
    /// Parallel to `deviations`: whether a registered exception covers it.
    explained: Vec<bool>,
    /// Steps replayed (the report's own count, independent of `Unfinished`).
    steps: usize,
    /// Total events in the recorded trace.
    total_events: usize,
}

impl ConformanceReport {
    /// Build a report from observed deviations.
    ///
    /// `steps` is the number of trajectory steps replayed; `total_events` is
    /// the trace length the replay was expected to cover. If the replay ended
    /// early (`steps < total_events`), an `Unfinished` deviation is recorded
    /// automatically — a report cannot silently omit truncation.
    #[must_use]
    pub fn new(steps: usize, total_events: usize, deviations: Vec<Deviation>) -> Self {
        let mut deviations = deviations;
        if steps < total_events {
            deviations.push(Deviation::Unfinished { steps_done: steps, total_events });
        }
        let explained = vec![false; deviations.len()];
        Self { deviations, explained, steps, total_events }
    }

    /// Steps replayed.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }

    /// Total events in the recorded trace.
    #[must_use]
    pub const fn total_events(&self) -> usize {
        self.total_events
    }

    /// All observed deviations, in observation order.
    #[must_use]
    pub fn deviations(&self) -> &[Deviation] {
        &self.deviations
    }

    /// Number of deviations not yet covered by a registered exception.
    #[must_use]
    pub fn open_deviations(&self) -> usize {
        self.explained.iter().filter(|e| !**e).count()
    }

    /// Mark the deviation at `deviation_index` as covered by a registered
    /// exception (exception-covers-deviation closure).
    ///
    /// # Errors
    ///
    /// Returns [`ConformanceError::BcinrValidation`] with a diagnostic message
    /// when `deviation_index` is out of range. (Cross-cutting pass-through
    /// variant, used pending a dedicated index error in `error.rs` — which is
    /// outside this lane's file ownership.)
    pub fn explain(&mut self, deviation_index: usize) -> Result<(), ConformanceError> {
        self.explained.get_mut(deviation_index).map_or_else(
            || {
                Err(ConformanceError::BcinrValidation {
                    message: format!("deviation index {deviation_index} out of range"),
                })
            },
            |flag| {
                *flag = true;
                Ok(())
            },
        )
    }

    /// Conforming iff the trajectory finished AND every deviation is covered
    /// by a registered exception (zero open deviations).
    #[must_use]
    pub fn is_conforming(&self) -> bool {
        self.trajectory_finished() && self.open_deviations() == 0
    }

    /// The trajectory finished iff no `Unfinished` deviation was recorded.
    fn trajectory_finished(&self) -> bool {
        !self.deviations.iter().any(|d| matches!(d, Deviation::Unfinished { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verb_mismatch(step: u32) -> Deviation {
        Deviation::VerbMismatch { step, tid: 3, expected: "assemble".to_string() }
    }

    fn not_enabled(step: u32) -> Deviation {
        Deviation::NotEnabled { step, tid: 7, marking: 0b1010 }
    }

    #[test]
    fn test_new_finished_complete_trajectory_has_no_synthetic_unfinished() {
        let report = ConformanceReport::new(5, 5, vec![not_enabled(2)]);
        assert_eq!(report.steps(), 5);
        assert_eq!(report.total_events(), 5);
        assert_eq!(report.deviations().len(), 1);
        assert!(report.trajectory_finished(), "complete trajectory is finished");
    }

    #[test]
    fn test_open_deviations_counts_only_unexplained() {
        let report = ConformanceReport::new(4, 4, vec![verb_mismatch(1), not_enabled(2)]);
        assert_eq!(report.open_deviations(), 2);
    }

    #[test]
    fn test_explain_closes_exactly_one_deviation() {
        let mut report = ConformanceReport::new(4, 4, vec![verb_mismatch(1), not_enabled(2)]);
        report.explain(0).expect("in-range index");
        assert_eq!(report.open_deviations(), 1, "only the explained one closes");
        report.explain(1).expect("in-range index");
        assert_eq!(report.open_deviations(), 0);
    }

    #[test]
    fn test_explain_out_of_range_is_typed_error_not_panic() {
        let mut report = ConformanceReport::new(4, 4, vec![verb_mismatch(1)]);
        let err = report.explain(5).expect_err("out-of-range index must be a typed error");
        assert!(
            matches!(err, ConformanceError::BcinrValidation { .. }),
            "unexpected error shape: {err:?}"
        );
        // The failed explain must not have mutated state.
        assert_eq!(report.open_deviations(), 1);
    }

    #[test]
    fn test_is_conforming_requires_zero_open_and_finished() {
        // Finished, all explained => conforming.
        let mut clean = ConformanceReport::new(4, 4, vec![verb_mismatch(1)]);
        clean.explain(0).expect("in-range index");
        assert!(clean.is_conforming());

        // Finished, one open => not conforming.
        let dirty = ConformanceReport::new(4, 4, vec![verb_mismatch(1)]);
        assert!(!dirty.is_conforming());

        // Zero open but truncated => not conforming.
        let mut truncated = ConformanceReport::new(2, 4, vec![]);
        assert_eq!(truncated.open_deviations(), 1, "Unfinished counts as open");
        assert!(!truncated.is_conforming());
        truncated.explain(0).expect("in-range index");
        assert!(
            !truncated.is_conforming(),
            "explaining Unfinished cannot un-truncate the trajectory"
        );
    }

    #[test]
    fn test_perfect_trajectory_conforms_with_no_explanations() {
        let report = ConformanceReport::new(7, 7, vec![]);
        assert_eq!(report.open_deviations(), 0);
        assert!(report.is_conforming());
    }

    #[test]
    fn test_deviation_roundtrips_through_serde() {
        let report = ConformanceReport::new(3, 3, vec![verb_mismatch(1), not_enabled(2)]);
        let json = serde_json::to_string(&report).expect("serializable");
        let back: ConformanceReport = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(report, back);
        // Private fields reconstruct exactly: the verdict is preserved.
        assert_eq!(back.open_deviations(), 2);
        assert!(!back.is_conforming());
    }

    #[test]
    fn test_replay_summary_fields_are_public_and_comparable() {
        let summary = ReplaySummary {
            steps_executed: 3,
            events_total: 3,
            final_marking: 1 << 3,
            outcome: ReportOutcome::Complete,
        };
        assert_eq!(summary.steps_executed, 3);
        assert_eq!(summary.events_total, 3);
        assert_eq!(summary.final_marking, 1 << 3);
        assert_eq!(summary.outcome, ReportOutcome::Complete);
    }

    #[test]
    fn test_report_outcome_carries_faulted_error() {
        let err = ConformanceError::Skipped { step: 2 };
        let outcome = ReportOutcome::Faulted(err.clone());
        assert_eq!(outcome, ReportOutcome::Faulted(err));
        assert_ne!(outcome, ReportOutcome::Complete);
        assert_ne!(outcome, ReportOutcome::Incomplete);
    }

    #[test]
    fn test_yield_status_variants_carry_their_payloads() {
        let yielded = YieldStatus::Yielded { remaining_events: 4 };
        assert_eq!(yielded, YieldStatus::Yielded { remaining_events: 4 });

        let summary = ReplaySummary {
            steps_executed: 1,
            events_total: 2,
            final_marking: 0b11,
            outcome: ReportOutcome::Incomplete,
        };
        assert_eq!(YieldStatus::Complete(summary.clone()), YieldStatus::Complete(summary));
        assert_eq!(
            YieldStatus::Violation(ConformanceError::Skipped { step: 0 }),
            YieldStatus::Violation(ConformanceError::Skipped { step: 0 })
        );
        assert_ne!(
            YieldStatus::Violation(ConformanceError::Skipped { step: 0 }),
            YieldStatus::Violation(ConformanceError::Skipped { step: 1 })
        );
    }

    #[test]
    fn test_phase_latching_states_are_distinguishable() {
        assert_eq!(Phase::Idle, Phase::Idle);
        assert_ne!(Phase::Idle, Phase::Running);
        assert_ne!(Phase::Yielded, Phase::Complete);
        assert_eq!(
            Phase::Faulted(ConformanceError::TraceMalformed { len: 0 }),
            Phase::Faulted(ConformanceError::TraceMalformed { len: 0 })
        );
    }

    #[test]
    fn test_unfinished_deviation_is_inspectable_in_deviations_slice() {
        let report = ConformanceReport::new(2, 5, vec![]);
        assert!(report
            .deviations()
            .iter()
            .any(|d| matches!(d, Deviation::Unfinished { steps_done: 2, total_events: 5 })));
    }
}
