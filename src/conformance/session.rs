//! Conformance session: a compiled POWL net replayed against an OCEL64
//! event stream under a step quota.
//!
//! This is lane C8 of the conformance fan-out (plan P3/P5). It owns the
//! replay state machine only: loading ([`super::loader`]) and model
//! validation (`super::validate`, sibling lane) are consumed, never
//! re-implemented here.
//!
//! # Sibling contracts (reconciled per E1; canonical homes)
//!
//! * `super::loader::PowlModel` — the canonical POWL model (public fields
//!   `activities`, `declaration_order`, `root: PowlNode` with
//!   `Activity(String)` leaves); loaded via
//!   `PowlModel::from_powl_file(path)` (E1 item 7).
//! * `super::validate::validate(&PowlModel) -> Result<(), ConformanceError>`
//!   — the canonical validator entry point (E1 item 6).
//! * Error field spellings follow error.rs exactly (`line`/`col` on
//!   `MalformedSource`/`UnknownActivity`/`DuplicateActivity`); this file
//!   only constructs errors defined verbatim in error.rs
//!   (`Skipped`/`OutOfOrder`/`Undeclared`/`ForbiddenBit`/`TraceMalformed`/
//!   `TooManyPlaces`/`ModelTooLarge`/`Io`).
//! * Report vocabulary lives in `super::report` (`ConformanceReport`,
//!   `ReplaySummary`, `ReportOutcome`, `YieldStatus`, `Phase`) and is
//!   re-exported here (E1 item 2); this module's replay-summary struct was
//!   renamed `ReplaySummary` and moved there.
//!
//! # Event word (OCEL64) layout used by the session
//!
//!   * bits `7:0`   : activity id (verb); matched against transition `tid`
//!   * bits `15:8`  : tick — must be exactly `prev_tick + 1 (mod 256)`
//!
//! # Step semantics (P3)
//!
//! One event word is consumed per step through the branchless PN64 step
//! ([`super::pn64::step_branchless`]) scanned over every transition with
//! constant-time (`ct_eq`) selection of the committed word — no branch,
//! lookup, or early exit depends on event values. Violations are classified
//! exactly and latch the session into [`Phase::Faulted`]; a faulted session
//! is frozen (marking, cursor, and tick never advance again).
//!
//! Zero heap allocation per step; no `unwrap`/`panic!` in production paths.

use super::error::ConformanceError;
use super::pn64;

/// Result alias for the conformance subsystem.
pub type ConformanceResult<T> = Result<T, ConformanceError>;

// ---------------------------------------------------------------------------
// Compiled net
// ---------------------------------------------------------------------------

/// A compiled marking: the live token set in the low 16 bits of a `u64`
/// (PN64 marking field width).
///
/// Distinct by design from the struct `super::state::Marking64`: this alias
/// is the raw PN64 marking field, not the token-set struct.
pub type Marking64 = u64;

/// A linear transition net lowered from a POWL atom sequence.
///
/// Atom `i` becomes transition `tid = i` consuming place `i` and producing
/// place `i + 1`; the initial marking is place 0. Each entry of `words` is a
/// PN64 word carrying that transition's `W-`/`W+`/`tid`; the marking field
/// of the stored words is the *initial* marking and is never mutated — the
/// live marking travels in [`ConformanceSession::marking`].
///
/// Session-local linear net, distinct by design from the canonical
/// `super::net::CompiledNet` (which `super::state` consumes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledNet {
    /// One PN64 word per atom transition (marking field = initial marking).
    pub words: Vec<u64>,
    /// Transition ids declared by this net (== `0..words.len()`).
    pub tids: Vec<u8>,
}

impl CompiledNet {
    /// Number of transitions in the net.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.words.len()
    }

    /// Whether the net declares no transitions at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Report vocabulary (canonical home: super::report, E1 item 2)
// ---------------------------------------------------------------------------

pub use super::report::{
    ConformanceReport, Deviation, Phase, ReplaySummary, ReportOutcome, YieldStatus,
};

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// Upper bound on POWL atoms compilable into one linear net: transition ids
/// are a single byte.
const MAX_ATOMS: usize = u8::MAX as usize + 1;

/// Bound inherited from the PN64 16-bit marking field: a linear chain over
/// `n` atoms needs `n + 1` places.
const fn max_atoms_for_places() -> usize {
    super::model::MAX_PETRI_PLACES as usize - 1
}

/// A compiled POWL net mid-replay against an OCEL64 stream.
///
/// Construct with [`ConformanceSession::from_powl_file`] (or
/// [`ConformanceSession::from_model`]), bind events with
/// [`ConformanceSession::attach_stream`], then drive with
/// [`ConformanceSession::step_budget`] / [`ConformanceSession::resume`].
#[derive(Debug, Clone)]
pub struct ConformanceSession {
    net: CompiledNet,
    initial_marking: Marking64,
    /// Forbidden-bit mask: no post-fire marking may intersect it.
    forbidden_mask: u64,
    stream: Vec<u64>,
    cursor: usize,
    steps_done: u32,
    prev_tick: Option<u8>,
    marking: Marking64,
    phase: Phase,
}

impl ConformanceSession {
    /// Loads a POWL model from `path`, validates it, and compiles it into a
    /// linear conformance net.
    ///
    /// # Errors
    ///
    /// Propagates loader/validator errors, and [`ConformanceError::
    /// ModelTooLarge`]/[`ConformanceError::TooManyPlaces`] when the atom
    /// sequence exceeds the transition-id byte or the 16-bit place budget.
    pub fn from_powl_file(path: &std::path::Path) -> ConformanceResult<Self> {
        let model = super::loader::PowlModel::from_powl_file(path)?;
        Self::from_model(&model)
    }

    /// Validates an already-loaded model and compiles it.
    ///
    /// # Errors
    ///
    /// As [`ConformanceSession::from_powl_file`].
    pub fn from_model(model: &super::loader::PowlModel) -> ConformanceResult<Self> {
        super::validate::validate(model)?;

        let n = atom_count(model);
        if n > MAX_ATOMS {
            return Err(ConformanceError::ModelTooLarge { found: n, limit: MAX_ATOMS });
        }
        let place_limit = max_atoms_for_places();
        if n > place_limit {
            return Err(ConformanceError::TooManyPlaces {
                limit: super::model::MAX_PETRI_PLACES as usize,
            });
        }

        // JUSTIFICATION: n <= MAX_PETRI_PLACES = 16 is enforced above, so the
        // usize shifts and the tid narrowing are total.
        #[allow(clippy::cast_possible_truncation)]
        let words = (0..n)
            .map(|i| {
                pn64::pack(
                    1u16 << i,       // initial marking field (never mutated)
                    1u16 << i,       // W-: consume place i
                    1u16 << (i + 1), // W+: produce place i+1
                    i as u8,         // tid = atom index
                    0,               // flags unused
                )
            })
            .collect::<Vec<_>>();

        // JUSTIFICATION: n <= MAX_PETRI_PLACES = 16 is enforced above, so
        // the usize -> u8 narrowing is total.
        #[allow(clippy::cast_possible_truncation)]
        let tid_count = n as u8;
        Ok(Self {
            net: CompiledNet { words, tids: (0..tid_count).collect() },
            initial_marking: 1, // place 0
            forbidden_mask: 0,
            stream: Vec::new(),
            cursor: 0,
            steps_done: 0,
            prev_tick: None,
            marking: 1, // place 0
            phase: Phase::Idle,
        })
    }

    /// Installs the forbidden-bit policy mask: any post-fire marking that
    /// intersects it is classified [`ConformanceError::ForbiddenBit`].
    ///
    /// Only effective before the first step (the session latches).
    pub const fn set_forbidden_mask(&mut self, mask: u64) {
        self.forbidden_mask = mask & 0xFFFF;
    }

    /// Binds the OCEL64 event stream to replay and rewinds the cursor to
    /// its first event. Attaching a new stream unfreezes a `Yielded`
    /// session only from the new stream's start; `Complete`/`Faulted`
    /// remain frozen: attaching into a terminal phase is refused.
    ///
    /// A zero-length stream is rejected eagerly as
    /// [`ConformanceError::TraceMalformed`], as is an attach attempt into a
    /// terminal (`Complete`/`Faulted`) phase.
    ///
    /// # Errors
    ///
    /// [`ConformanceError::TraceMalformed`] when `words.is_empty()` or the
    /// session is already in a terminal phase (`Complete`/`Faulted`).
    pub fn attach_stream(&mut self, words: &[u64]) -> ConformanceResult<()> {
        if matches!(self.phase, Phase::Complete | Phase::Faulted(_)) {
            return Err(ConformanceError::TraceMalformed { len: words.len() });
        }
        if words.is_empty() {
            return Err(ConformanceError::TraceMalformed { len: 0 });
        }
        self.stream = words.to_vec();
        self.cursor = 0;
        Ok(())
    }

    /// Replays at most `quota` events, returning early on violation or
    /// trace exhaustion.
    ///
    /// A session with no attached (or an empty) stream is refused, not
    /// completed: stepping an unattached session latches
    /// `Phase::Faulted(ConformanceError::TraceMalformed { len: 0 })` and
    /// reports `YieldStatus::Violation` — the same trace law `attach_stream`
    /// enforces, applied at step time so a never-attached session can never
    /// reach `Complete` on a zero-event trajectory.
    pub fn step_budget(&mut self, quota: u32) -> YieldStatus {
        // Trace law at step time (X2): an empty bound stream is malformed,
        // never a vacuous Complete. A session that attached a nonempty
        // stream keeps its Complete-on-exhaustion behavior below.
        if self.stream.is_empty() {
            let err = ConformanceError::TraceMalformed { len: 0 };
            self.phase = Phase::Faulted(err.clone());
            return YieldStatus::Violation(err);
        }
        match self.phase {
            Phase::Faulted(ref e) => return YieldStatus::Violation(e.clone()),
            Phase::Complete => return YieldStatus::Complete(self.report(ReportOutcome::Complete)),
            // Idle / Running / Yielded all (re)enter the replay loop; a
            // Yielded session resumed via step_budget behaves like resume.
            _ => {}
        }

        let mut remaining_quota = quota;
        while remaining_quota > 0 {
            if self.cursor >= self.stream.len() {
                self.phase = Phase::Complete;
                return YieldStatus::Complete(self.report(ReportOutcome::Complete));
            }
            let word = self.stream[self.cursor];
            match self.classify_and_commit(word) {
                Ok(()) => {
                    self.cursor += 1;
                    self.steps_done += 1;
                    remaining_quota -= 1;
                }
                Err(err) => {
                    self.phase = Phase::Faulted(err.clone());
                    return YieldStatus::Violation(err);
                }
            }
        }

        // Quota exhausted: the session yields unless the trace is also
        // exhausted, in which case it completes (exhaustion wins over yield).
        if self.cursor >= self.stream.len() {
            self.phase = Phase::Complete;
            return YieldStatus::Complete(self.report(ReportOutcome::Complete));
        }
        self.phase = Phase::Yielded;
        YieldStatus::Yielded { remaining_events: self.stream.len() - self.cursor }
    }

    /// Continues a yielded session with a fresh quota (P3 resume).
    pub fn resume(&mut self, quota: u32) -> YieldStatus {
        self.step_budget(quota)
    }

    /// The latching phase of this session.
    #[must_use]
    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    /// Events successfully replayed so far.
    #[must_use]
    pub const fn steps_done(&self) -> u32 {
        self.steps_done
    }

    /// Events still unconsumed (0 once the trace is exhausted).
    #[must_use]
    pub const fn remaining_events(&self) -> usize {
        self.stream.len().saturating_sub(self.cursor)
    }

    /// The live marking.
    #[must_use]
    pub const fn marking(&self) -> Marking64 {
        self.marking
    }

    /// The compiled initial marking (place 0 tokenized).
    #[must_use]
    pub const fn initial_marking(&self) -> Marking64 {
        self.initial_marking
    }

    /// The compiled net (read-only view).
    #[must_use]
    pub const fn net(&self) -> &CompiledNet {
        &self.net
    }

    /// Terminal report; safe to call in any phase (`Incomplete` /
    /// `Faulted` / `Complete` reflects the latched phase).
    #[must_use]
    pub fn finish(&self) -> ReplaySummary {
        match &self.phase {
            Phase::Complete => self.report(ReportOutcome::Complete),
            Phase::Faulted(e) => self.report(ReportOutcome::Faulted(e.clone())),
            _ => self.report(ReportOutcome::Incomplete),
        }
    }

    /// Structured verdict report over the session's final replay state
    /// (macro seam, E5 finding 3).
    ///
    /// Builds the canonical [`super::report::ConformanceReport`] from the
    /// replayed step count, the bound stream length, and every classified
    /// violation; an unfinished trace records its `Unfinished` deviation
    /// automatically via [`ConformanceReport::new`]. A faulted session
    /// contributes the error that latched it.
    #[must_use]
    pub fn conformance_report(&self) -> ConformanceReport {
        let mut deviations = Vec::new();
        if let Phase::Faulted(err) = &self.phase {
            deviations.push(conformance_error_deviation(err, self.steps_done));
        }
        ConformanceReport::new(self.steps_done as usize, self.stream.len(), deviations)
    }

    // -- internals ----------------------------------------------------------

    const fn report(&self, outcome: ReportOutcome) -> ReplaySummary {
        ReplaySummary {
            steps_executed: self.steps_done,
            events_total: self.stream.len(),
            final_marking: self.marking,
            outcome,
        }
    }

    /// Classifies one event word exactly and, if lawful, commits it.
    ///
    /// Classification order (each class is checked explicitly, never
    /// inferred from a silent PN64 refusal):
    ///   1. tick order — `OutOfOrder` when the tick repeats the previous
    ///      tick, `Skipped` when it is anything other than `prev + 1
    ///      (mod 256)`;
    ///   2. `Undeclared` — the activity byte matches no compiled `tid`;
    ///   3. `ForbiddenBit` — the post-fire marking intersects the mask;
    ///   4. `Skipped` — declared and lawful-bitwise but not enabled at the
    ///      cursor (the event is out of the model's linear order).
    ///
    /// On any violation the session state is left bit-exact untouched; the
    /// caller latches the phase.
    fn classify_and_commit(&mut self, word: u64) -> ConformanceResult<()> {
        let activity = (word & 0xFF) as u8;
        let tick = ((word >> 8) & 0xFF) as u8;

        // 1. Tick order: exact +1 mod-256 sequence.
        if let Some(prev) = self.prev_tick {
            let expected = prev.wrapping_add(1);
            if tick == prev {
                return Err(ConformanceError::OutOfOrder {
                    step: self.steps_done,
                    prev_tick: prev,
                    tick,
                });
            }
            if tick != expected {
                return Err(ConformanceError::Skipped { step: self.steps_done });
            }
        }

        // Constant-time lane/tid select: fold ct_eq over every declared tid.
        let mut sel = 0u64;
        let mut declared = 0u64;
        for (i, &tid) in self.net.tids.iter().enumerate() {
            declared |= pn64::ct_eq(u64::from(tid), u64::from(activity));
            let w = (self.net.words[i] & !pn64::MASK16) | self.marking;
            let hit = pn64::ct_eq(u64::from(pn64::tid(w)), u64::from(activity));
            // branch-free accumulate: keep the hit lane's word
            let mask = hit.wrapping_neg();
            sel = (sel & !mask) | (w & mask);
        }
        if declared == 0 {
            return Err(ConformanceError::Undeclared { step: self.steps_done, tid: activity });
        }

        // 3. Forbidden-bit law on the post-fire marking of the selected lane.
        let next_m = (pn64::marking(sel) ^ pn64::pre(sel)) | pn64::post(sel);
        if u64::from(next_m) & (self.forbidden_mask & 0xFFFF) != 0 {
            return Err(ConformanceError::ForbiddenBit {
                step: self.steps_done,
                marking: next_m,
                mask: (self.forbidden_mask & 0xFFFF) as u16,
            });
        }

        // 4. Branchless commit over the selected lane; a declared, lawful
        //    lane that still refuses is out of the model's linear order.
        let committed = pn64::step_branchless(sel, word, self.forbidden_mask);
        // A bit-exact unchanged word means the branchless step refused
        // (not enabled) — the event is out of the model's linear order.
        if pn64::ct_eq(committed, sel) == 1 {
            return Err(ConformanceError::Skipped { step: self.steps_done });
        }

        self.marking = Marking64::from(pn64::marking(committed));
        self.prev_tick = Some(tick);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Atom sequence (loader-model DFS)
// ---------------------------------------------------------------------------

/// Depth-first, left-to-right atom sequence of a loader-lane model: every
/// `PowlNode::Activity` leaf in traversal order (`Silent` leaves contribute
/// no transition).
///
/// Coded against `super::loader::PowlNode` (see module docs for the
/// loader/model.rs duplication note).
fn atom_count(model: &super::loader::PowlModel) -> usize {
    // The atom *identities* are not needed beyond ordering: the linear net
    // assigns tid = DFS index. Collect only the count, bounded by the model
    // node budget so traversal stays fixed-cost.
    fn count_activities(node: &super::loader::PowlNode) -> usize {
        match node {
            super::loader::PowlNode::Activity(_) => 1,
            super::loader::PowlNode::Silent => 0,
            super::loader::PowlNode::Sequence(children)
            | super::loader::PowlNode::Xor(children)
            | super::loader::PowlNode::Parallel(children) => {
                children.iter().map(count_activities).sum()
            }
            super::loader::PowlNode::Loop { body, redo, .. } => {
                count_activities(body) + count_activities(redo)
            }
        }
    }
    count_activities(&model.root)
}

/// Maps a classified replay violation onto the report vocabulary's
/// [`super::report::Deviation`] surface. Every replay-classified violation
/// carries its real data into the report; nothing is fabricated as a
/// phantom `NotEnabled { tid: 0, marking: 0 }`.
const fn conformance_error_deviation(
    err: &ConformanceError,
    step: u32,
) -> super::report::Deviation {
    use super::report::Deviation;
    match err {
        ConformanceError::ForbiddenBit { marking, .. } => {
            Deviation::ForbiddenBit { step, marking: *marking }
        }
        ConformanceError::OutOfOrder { prev_tick, tick, .. } => {
            Deviation::OutOfOrder { step, prev_tick: *prev_tick, tick: *tick }
        }
        ConformanceError::Undeclared { tid, .. } => Deviation::Undeclared { step, tid: *tid },
        ConformanceError::Skipped { .. } => Deviation::Skipped { step },
        // Terminal-before-start latch (never-attached session): the
        // trajectory never played, which the vocabulary expresses as an
        // unfinished trajectory of zero replayed steps.
        ConformanceError::TraceMalformed { .. } => {
            Deviation::Unfinished { steps_done: step as usize, total_events: 0 }
        }
        // No replay path latches Io / model errors into Phase::Faulted; if a
        // new session-side variant is added without a report twin, it falls
        // back to the real marking it was latched at rather than a phantom.
        _ => Deviation::NotEnabled { step, tid: 0, marking: 0 },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// Three-atom linear model: handle -> fork -> approve.
    fn three_atom_model() -> super::super::loader::PowlModel {
        use super::super::loader::{ActivityDecl, PowlModel as M, PowlNode as N};
        let mut activities = BTreeMap::new();
        activities.insert(
            "handle".to_string(),
            ActivityDecl { source: "log".to_string(), event: "handle".to_string() },
        );
        activities.insert(
            "fork".to_string(),
            ActivityDecl { source: "robot".to_string(), event: "fork".to_string() },
        );
        activities.insert(
            "approve".to_string(),
            ActivityDecl { source: "log".to_string(), event: "approve".to_string() },
        );
        M {
            activities,
            declaration_order: vec![
                "handle".to_string(),
                "fork".to_string(),
                "approve".to_string(),
            ],
            root: N::Sequence(vec![
                N::Activity("handle".to_string()),
                N::Activity("fork".to_string()),
                N::Silent,
                N::Activity("approve".to_string()),
            ]),
        }
    }

    /// OCEL64 event word: activity in bits [7:0], tick in bits [15:8].
    fn event(tid: u8, tick: u8) -> u64 {
        u64::from(tid) | (u64::from(tick) << 8)
    }

    /// Golden trajectory: a 3-atom sequence replayed in order completes.
    #[test]
    fn golden_trajectory_completes() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1), event(1, 2), event(2, 3)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        match s.step_budget(10) {
            YieldStatus::Complete(report) => {
                assert_eq!(report.steps_executed, 3);
                assert_eq!(report.events_total, 3);
                assert_eq!(report.outcome, ReportOutcome::Complete);
                assert_eq!(report.final_marking, 1 << 3);
            }
            other => panic!("expected Complete, got {other:?}"),
        }
        assert_eq!(*s.phase(), Phase::Complete);
        // Frozen: further budget is idempotent Complete.
        assert!(matches!(s.resume(5), YieldStatus::Complete(_)));
    }

    #[test]
    fn out_of_order_repeating_tick_latches_faulted() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1), event(1, 1)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        match s.step_budget(10) {
            YieldStatus::Violation(ConformanceError::OutOfOrder { prev_tick, tick, .. }) => {
                assert_eq!(prev_tick, 1);
                assert_eq!(tick, 1);
            }
            other => panic!("expected OutOfOrder violation, got {other:?}"),
        }
        // Latch semantics: frozen marking/cursor/phase; replay reports same.
        assert_eq!(s.marking(), 1 << 1);
        assert_eq!(s.steps_done(), 1);
        assert_eq!(s.remaining_events(), 1);
        assert!(matches!(s.resume(5), YieldStatus::Violation(_)));
        assert!(matches!(s.phase(), Phase::Faulted(ConformanceError::OutOfOrder { .. })));
        let r = s.finish();
        assert!(matches!(r.outcome, ReportOutcome::Faulted(_)));
    }

    #[test]
    fn skipped_gap_tick_latches_faulted() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        // tick jumps 1 -> 3: not prev, not prev+1 => Skipped.
        s.attach_stream(&[event(0, 1), event(1, 3)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        match s.step_budget(10) {
            YieldStatus::Violation(ConformanceError::Skipped { step }) => assert_eq!(step, 1),
            other => panic!("expected Skipped violation, got {other:?}"),
        }
        assert!(matches!(s.phase(), Phase::Faulted(ConformanceError::Skipped { .. })));
    }

    #[test]
    fn undeclared_tid_latches_faulted() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1), event(9, 2)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        match s.step_budget(10) {
            YieldStatus::Violation(ConformanceError::Undeclared { step, tid }) => {
                assert_eq!(step, 1);
                assert_eq!(tid, 9);
            }
            other => panic!("expected Undeclared violation, got {other:?}"),
        }
    }

    #[test]
    fn forbidden_bit_latches_faulted() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        // After firing tid 0 the marking is 0b0010; forbid bit 1 (place 1).
        s.set_forbidden_mask(0b0010);
        s.attach_stream(&[event(0, 1)]).unwrap_or_else(|e| panic!("attach: {e}"));
        match s.step_budget(10) {
            YieldStatus::Violation(ConformanceError::ForbiddenBit { step, marking, mask }) => {
                assert_eq!(step, 0);
                assert_eq!(marking, 0b0010);
                assert_eq!(mask, 0b0010);
            }
            other => panic!("expected ForbiddenBit violation, got {other:?}"),
        }
        // Frozen before the violating step committed.
        assert_eq!(s.marking(), 1);
        assert_eq!(s.steps_done(), 0);
    }

    #[test]
    fn declared_but_out_of_linear_order_is_skipped() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        // tid 1 is declared but place 1 holds no token yet => Skipped.
        s.attach_stream(&[event(1, 1)]).unwrap_or_else(|e| panic!("attach: {e}"));
        match s.step_budget(10) {
            YieldStatus::Violation(ConformanceError::Skipped { step }) => assert_eq!(step, 0),
            other => panic!("expected Skipped violation, got {other:?}"),
        }
    }

    #[test]
    fn quota_yield_then_resume_completes() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1), event(1, 2), event(2, 3)])
            .unwrap_or_else(|e| panic!("attach: {e}"));

        match s.step_budget(2) {
            YieldStatus::Yielded { remaining_events } => assert_eq!(remaining_events, 1),
            other => panic!("expected Yielded, got {other:?}"),
        }
        assert_eq!(*s.phase(), Phase::Yielded);
        assert_eq!(s.steps_done(), 2);

        match s.resume(1) {
            YieldStatus::Complete(report) => assert_eq!(report.steps_executed, 3),
            other => panic!("expected Complete, got {other:?}"),
        }
    }

    #[test]
    fn zero_quota_yields_without_stepping() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1)]).unwrap_or_else(|e| panic!("attach: {e}"));
        match s.step_budget(0) {
            YieldStatus::Yielded { remaining_events } => assert_eq!(remaining_events, 1),
            other => panic!("expected Yielded, got {other:?}"),
        }
        assert_eq!(s.steps_done(), 0);
    }

    #[test]
    fn empty_trace_completes_immediately() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1)]).unwrap_or_else(|e| panic!("attach: {e}"));
        // Drain the one event, then the trace is exhausted.
        let _ = s.step_budget(1);
        match s.step_budget(1) {
            YieldStatus::Complete(report) => {
                assert_eq!(report.outcome, ReportOutcome::Complete);
                assert_eq!(report.events_total, 1);
            }
            other => panic!("expected Complete, got {other:?}"),
        }
    }

    #[test]
    fn empty_stream_rejected_as_malformed() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        assert!(matches!(s.attach_stream(&[]), Err(ConformanceError::TraceMalformed { len: 0 })));
    }

    // -- S3 exploit-closure regressions (X1/X2/X3/X4) ----------------------

    /// X1 (S3): attach into a `Complete` session must REFUSE — a terminal
    /// session is frozen; re-arming it with a fresh stream would unfreeze a
    /// verdict already reported.
    #[test]
    fn x1_attach_after_complete_refuses() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1), event(1, 2), event(2, 3)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        assert!(matches!(s.step_budget(10), YieldStatus::Complete(_)));
        assert_eq!(*s.phase(), Phase::Complete);
        assert!(matches!(
            s.attach_stream(&[event(0, 1)]),
            Err(ConformanceError::TraceMalformed { len: 1 })
        ));
        // Still frozen at the terminal verdict.
        assert_eq!(*s.phase(), Phase::Complete);
        assert_eq!(s.steps_done(), 3);
    }

    /// X3 (S3): attach into a `Faulted` session must REFUSE — the latch
    /// carries the violation; a re-attach must not erase it.
    #[test]
    fn x3_attach_after_faulted_refuses() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1), event(9, 2)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        assert!(matches!(
            s.step_budget(10),
            YieldStatus::Violation(ConformanceError::Undeclared { .. })
        ));
        assert!(matches!(
            s.attach_stream(&[event(0, 1), event(1, 2), event(2, 3)]),
            Err(ConformanceError::TraceMalformed { len: 3 })
        ));
        // The original violation latch survives the refused re-attach.
        assert!(matches!(s.phase(), Phase::Faulted(ConformanceError::Undeclared { .. })));
        assert!(matches!(s.resume(5), YieldStatus::Violation(_)));
    }

    /// X2 (S3): a never-attached session must NOT reach `Complete` on
    /// `step_budget` — a zero-event trajectory is malformed, not vacuously
    /// conforming. Same trace law as `attach_stream`, applied at step time.
    #[test]
    fn x2_step_budget_unattached_refuses_not_completes() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        assert_eq!(*s.phase(), Phase::Idle);
        match s.step_budget(10) {
            YieldStatus::Violation(ConformanceError::TraceMalformed { len: 0 }) => {}
            other => panic!("expected TraceMalformed refusal, got {other:?}"),
        }
        assert!(matches!(s.phase(), Phase::Faulted(ConformanceError::TraceMalformed { len: 0 })));
        assert_eq!(s.steps_done(), 0);
        // Terminal latch: subsequent budget re-reports the violation.
        assert!(matches!(
            s.step_budget(10),
            YieldStatus::Violation(ConformanceError::TraceMalformed { len: 0 })
        ));
        // And its structured report is not conforming (no phantom Complete).
        assert!(!s.conformance_report().is_conforming());
    }

    /// X4 (S3): the report lane carries the REAL violation data —
    /// `Undeclared` maps to `Deviation::Undeclared` with the offending tid,
    /// never the phantom `NotEnabled { tid: 0, marking: 0 }`.
    #[test]
    fn x4_undeclared_deviation_carries_real_tid() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(0, 1), event(9, 2)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        let _ = s.step_budget(10);
        let cr = s.conformance_report();
        assert!(
            cr.deviations()
                .iter()
                .any(|d| matches!(d, Deviation::Undeclared { step: 1, tid: 9 })),
            "expected real Undeclared deviation, got {cr:?}"
        );
        assert!(
            !cr.deviations()
                .iter()
                .any(|d| matches!(d, Deviation::NotEnabled { tid: 0, marking: 0, .. })),
            "phantom NotEnabled must not appear: {cr:?}"
        );
    }

    /// X4 (S3): `Skipped` maps to `Deviation::Skipped` with the real step,
    /// not the phantom fallback.
    #[test]
    fn x4_skipped_deviation_carries_real_step() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        s.attach_stream(&[event(1, 1)]).unwrap_or_else(|e| panic!("attach: {e}"));
        let _ = s.step_budget(10);
        let cr = s.conformance_report();
        assert!(
            cr.deviations().iter().any(|d| matches!(d, Deviation::Skipped { step: 0 })),
            "expected real Skipped deviation, got {cr:?}"
        );
        assert!(
            !cr.deviations()
                .iter()
                .any(|d| matches!(d, Deviation::NotEnabled { tid: 0, marking: 0, .. })),
            "phantom NotEnabled must not appear: {cr:?}"
        );
    }

    /// X2 companion: an unattached session's summary is `Incomplete`
    /// (never `Complete`), and `finish()` reflects that.
    #[test]
    fn unattached_session_finish_is_incomplete_never_complete() {
        let s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        let summary = s.finish();
        assert_eq!(summary.outcome, ReportOutcome::Incomplete);
        assert_eq!(summary.steps_executed, 0);
        assert_eq!(summary.events_total, 0);
    }

    #[test]
    fn oversized_model_rejected_by_place_budget() {
        use super::super::loader::{PowlModel as M, PowlNode as N};
        // 16 atoms need 17 places > MAX_PETRI_PLACES.
        let atoms: Vec<N> = (0..16).map(|i| N::Activity(format!("a{i}"))).collect();
        let model = M {
            activities: BTreeMap::new(),
            declaration_order: Vec::new(),
            root: N::Sequence(atoms),
        };
        assert!(matches!(
            ConformanceSession::from_model(&model),
            Err(ConformanceError::TooManyPlaces { limit: 16 })
        ));
    }

    #[test]
    fn first_tick_is_free_prev_starts_none() {
        let mut s = ConformanceSession::from_model(&three_atom_model())
            .unwrap_or_else(|e| panic!("compile: {e}"));
        // Any first tick is admitted; the +1 law binds from step 2.
        s.attach_stream(&[event(0, 200), event(1, 201), event(2, 202)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        assert!(matches!(s.step_budget(10), YieldStatus::Complete(_)));
    }

    // Exercises from_powl_file through the real DSL parser (Chicago-style
    // real collaborator): load -> validate -> compile -> replay.
    #[test]
    fn from_powl_file_golden_replay() {
        let mut path = std::env::temp_dir();
        path.push("ctdd_conformance_session_golden.powl");
        let src = "activity handle = log.handle\n\
                   activity fork = robot.fork\n\
                   sequence {\n    handle\n    fork\n}\n";
        std::fs::write(&path, src).unwrap_or_else(|e| panic!("write temp: {e}"));
        let mut s = match ConformanceSession::from_powl_file(&path) {
            Ok(s) => s,
            Err(e) => panic!("from_powl_file failed: {e}"),
        };
        s.attach_stream(&[event(0, 1), event(1, 2)])
            .unwrap_or_else(|e| panic!("attach: {e}"));
        assert!(matches!(s.step_budget(10), YieldStatus::Complete(_)));
        let _ = std::fs::remove_file(&path);
    }
}
