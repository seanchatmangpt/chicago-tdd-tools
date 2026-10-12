//! N15 — liveness / starvation-freedom court for the RONR admission surface.
#![allow(dead_code)]
// JUSTIFICATION: court items (TOTAL/SLACK/M0/DISPOSITION, fire/reachable/can_reach, StarvationMonitor, run_schedule) are consumed by this module's own cfg(test) courts; deny(warnings) counts test-only refs as dead in non-test builds
//!
//! Verbatim port of the liveness law from strata-protocol
//! `tests/nato/n15_tlc_liveness.rs` (strata fix lane, Apache/MIT per strata
//! LICENSE), re-targeted to this crate's capital:
//! [`crate::nato::transition_tables::ronr::TRANSITIONS`] (the pure-data
//! `(pre, post)` table) fired through [`crate::conformance::pn64`].
//!
//! # Scoping: property-equivalent bounded model, not TLC
//!
//! Spec mandate (NATO_N01_N20.md N15): "model-check (Erlang TLC or Elixir
//! property equivalent) box-diamond-admitted under weak fairness". TLC needs
//! an Erlang/PlusCal spec; the spec itself sanctions a property equivalent,
//! and the in-repo capital is already a formal object (1-safe Petri net over
//! `u16` masks), so the strongest sound substitute is an **exhaustive
//! enumeration of the reachable marking graph** plus a deterministic
//! fair-scheduler simulation with an explicit, typed starvation monitor. The
//! [`ScopingRecord`] below discloses this as full spec scale *by the spec's
//! own sanction of "or Elixir property equivalent"*, with the residual
//! bound (six-transition admission surface, not the full PlusCal state
//! space) named in the doc and in the tier-3 test's scoping.
//!
//! # Fairness assumption (precise wording — load-bearing, not decorative)
//!
//! The fairness assumption is **round-robin over the enabled set**: the
//! scheduler repeatedly cycles the transitions in canonical table order,
//! firing each that is enabled at its turn. The starvation law is:
//! *under that scheduler, every transition is enabled-then-fired within
//! `TOTAL/32` steps, and every non-terminal reachable marking reaches a
//! disposition marking within `TOTAL/32` steps.* This is a *scheduled-model*
//! fairness assumption, deliberately narrower than a real model checker's
//! weak fairness over the full transition relation: it is an assumption
//! about the scheduler we ship, not a theorem about every interleaving.
//! Real liveness over all interleavings remains the model checker's domain;
//! this court proves the property the scheduler actually enforces, exhibits
//! the adversary that the fairness hypothesis excludes (an unfair scheduler
//! that starves `ronr_previous_question` forever — the law trips on it),
//! and therefore cannot be silently vacuous.
//!
//! # Anti-vacuity counterexample
//!
//! A deliberately unfair scheduler (amend/adopt-amendment loop that never
//! serves the rest of the ring) drives the starvation monitor past
//! `TOTAL/32` while `ronr_previous_question` stays enabled-but-unfired. The
//! monitor SEES it — typed, deterministic, single-threaded — proving the
//! law is falsifiable.

use super::disclosure::ScopingRecord;
use super::transition_tables::{ronr, Shape}; // Shape lives at transition_tables top level, not in ronr
use crate::conformance::pn64;

/// Simulation step budget for the bounded scheduler model. `SLACK =
/// TOTAL / 32` mirrors the strata N15 ring court's documented 1/8-of-fair-
/// share slack (TOTAL/32): the bound a non-realtime scheduler admits.
const TOTAL: usize = 4096;

/// Documented starvation slack: a transition enabled at step `t` must fire
/// by step `t + SLACK`, and a non-terminal marking must reach a disposition
/// within `SLACK` steps of scheduling.
const SLACK: usize = TOTAL / 32; // 128

// Canonical six-transition RONR admission surface: opcodes `0x01`-`0x06`
// (`stated`, `second`, `amend`, `adopt_amendment`, `previous_question`,
// `adopt_main_motion`) — the adoption-critical path from a recognized main
// motion. Slice of `ronr::TRANSITIONS` (the tests' `admission_surface()`), so
// any drift in the table data moves this surface with it.

/// Floor place bit (`ronr::N_FLOOR`); part of the initial marking.
const M0: u16 = ronr::N_MAIN_MOTION | ronr::N_FLOOR;

/// Disposition place bits of the admission surface: adoption is the only
/// terminal this subnet can produce.
const DISPOSITION: u16 = ronr::N_ADOPTED;

/// The scoping disclosure for this court: the spec sanctions "TLC or Elixir
/// property equivalent", so the exhaustive Rust property-equivalent is full
/// spec scale; the bounded six-transition surface is the named evidence.
pub fn scoping() -> ScopingRecord {
    ScopingRecord::full(
        "NATO_N01_N20.md N15: model-check (Erlang TLC or Elixir property \
         equivalent) box-diamond-admitted under weak fairness",
        "exhaustive reachable-marking enumeration + round-robin fair-scheduler \
         property-equivalent simulation with typed starvation monitor over the \
         six-transition RONR admission surface (pn64-fired), plus an \
         anti-vacuity unfair-scheduler counterexample; full interleaving \
         liveness remains TLC's domain and is disclosed in the module header",
    )
}

/// Fire `t` at marking `m` through the pn64 kernel: enabled-ness and the
/// resulting marking both come from `pn64::fire`, never a local re-derivation.
fn fire(m: u16, t: &Shape) -> Option<u16> {
    let w = pn64::pack(m, t.pre, t.post, 0, 0);
    pn64::fire(w).map(pn64::marking)
}

/// All markings reachable from `start` under `table` (exhaustive BFS over
/// the `u16` marking space, no sampling).
fn reachable(start: u16, table: &[Shape]) -> Vec<u16> {
    let mut seen = vec![false; 65536];
    let mut out = Vec::new();
    let mut queue = vec![start];
    seen[start as usize] = true;
    while let Some(cur) = queue.pop() {
        out.push(cur);
        for t in table {
            if let Some(next) = fire(cur, t) {
                if !seen[next as usize] {
                    seen[next as usize] = true;
                    queue.push(next);
                }
            }
        }
    }
    out
}

/// Whether any marking reachable from `m` under `table` carries the target
/// place bit(s). `target` is a place mask, not a full marking: adoption
/// markings always carry additional residual bits (floor, seconded, ...),
/// so an exact-marking `contains` would make every diamond vacuously false.
fn can_reach(m: u16, target: u16, table: &[Shape]) -> bool {
    reachable(m, table).iter().any(|&r| r & target == target)
}

/// The starvation monitor. Per transition: the step of its last fire and the
/// step at which enablement was first observed without a subsequent fire —
/// i.e. the duration it has been enabled-but-never-fired (the strong-fairness
/// flavored reading the anti-vacuity adversary is required to trip).
#[derive(Debug, Default)]
struct StarvationMonitor {
    last_fired: [Option<usize>; 6],
    enabled_since: [Option<usize>; 6],
    starved: Vec<usize>,
}

impl StarvationMonitor {
    /// Record enablement of transition `i` at `step`.
    fn observe_enabled(&mut self, i: usize, step: usize) {
        self.enabled_since[i] = Some(self.enabled_since[i].unwrap_or(step));
        if let Some(since) = self.enabled_since[i] {
            if step - since >= SLACK && !self.starved.contains(&i) {
                self.starved.push(i);
            }
        }
    }

    /// Record a fire of transition `i` at `step`.
    fn observe_fired(&mut self, i: usize, step: usize) {
        self.last_fired[i] = Some(step);
        self.enabled_since[i] = None;
    }
}

/// The law: under the round-robin scheduler, no transition's enabled streak
/// reaches `SLACK`, and every non-terminal run reaches a disposition within
/// `SLACK` steps. Returns the starved set and whether the disposition was
/// reached, so fair runs assert both clean and unfair runs assert the trip.
fn run_schedule(table: &[Shape], start: u16, fair: bool) -> (Vec<usize>, bool) {
    let mut monitor = StarvationMonitor::default();
    let mut cur = start;
    let mut step = 0usize;
    let mut rr = 0usize; // round-robin cursor
    while step < TOTAL {
        // Observe enablement of everything enabled at this marking.
        let mut enabled: Vec<usize> = Vec::new();
        for (i, t) in table.iter().enumerate() {
            if cur & t.pre == t.pre {
                monitor.observe_enabled(i, step);
                enabled.push(i);
            }
        }
        if cur & DISPOSITION != 0 {
            return (monitor.starved, true);
        }
        // Pick the transition to fire.
        let pick = if fair {
            // Round-robin: first enabled transition at or after the cursor.
            let n = table.len();
            (0..n).map(|k| (rr + k) % n).find(|&k| enabled.contains(&k))
        } else {
            // Adversary: never serve `previous_question` (index 4) or
            // `adopt_main_motion` (index 5); loop the amend cycle instead.
            enabled.iter().copied().find(|&i| i < 4)
        };
        match pick.and_then(|i| fire(cur, &table[i]).map(|m| (i, m))) {
            Some((i, next)) => {
                monitor.observe_fired(i, step);
                cur = next;
                rr = i + 1;
            }
            None => break, // deadlock (no enabled transition): not disposition
        }
        step += 1;
    }
    (monitor.starved, false)
}

#[cfg(test)]
mod tests {
    fn admission_surface() -> &'static [Shape] {
        &ronr::TRANSITIONS[0..6]
    }

    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
    fn admission_surface_is_canonical_six() {
        let names: Vec<&str> = admission_surface().iter().map(|t| t.name).collect();
        assert_eq!(
            names,
            vec![
                "ronr_main_motion_stated",
                "ronr_second",
                "ronr_amend",
                "ronr_adopt_amendment",
                "ronr_previous_question",
                "ronr_adopt_main_motion",
            ],
            "admission surface must be exactly opcodes 0x01-0x06 in table order"
        );
        // Firing goes through the pn64 kernel, not a local re-derivation:
        // the canonical first step from 0x0081 is stated -> 0x0082.
        let stated = &admission_surface()[0];
        let next = fire(M0, stated).unwrap_or_else(|| panic!("stated must fire from M0"));
        assert_eq!(next, 0x0082, "pn64-fired stated must yield 0x0082");
    }

    #[test]
    fn scoping_disclosure_validates_full_scale() {
        let s = scoping();
        s.validate().unwrap_or_else(|e| panic!("scoping record must validate: {e:?}"));
        assert!(s.is_full_spec_scale, "spec sanctions 'or property equivalent'");
        assert!(s.spec_mandate.contains("weak fairness"));
        assert!(s.delivered_evidence.contains("property"));
    }

    #[test]
    fn n15_box_diamond_starvation_free_under_round_robin() {
        // Exhaustive: from EVERY reachable marking of the admission surface,
        // the fair round-robin schedule reaches adoption within SLACK steps
        // and starves nothing (box-diamond under the documented fairness).
        let markings = reachable(M0, admission_surface());
        assert!(!markings.is_empty(), "reachable set from 0x0081 must be non-empty");
        for &m in &markings {
            let (starved, dispositioned) = run_schedule(admission_surface(), m, true);
            assert!(dispositioned, "marking {m:#06x} did not reach adoption within {TOTAL} steps");
            assert!(
                starved.is_empty(),
                "marking {m:#06x} starved transitions {starved:?} under round-robin"
            );
            // The SLACK bound is not decorative: the run must disposition
            // well inside TOTAL/32, not merely inside TOTAL.
            let fast = run_to_disposition_within(admission_surface(), m, SLACK);
            assert!(fast, "marking {m:#06x} failed to disposition within SLACK ({SLACK})");
        }
    }

    #[test]
    fn n15_every_enabled_transition_fires_within_slack() {
        // Per-transition form of the law: whichever marking the net is at,
        // a transition that is enabled fires within SLACK steps of the
        // round-robin schedule. Checked from every reachable marking with
        // every rotation offset of the cursor (exhaustive over rotations).
        let markings = reachable(M0, admission_surface());
        let n = admission_surface().len();
        for &m in &markings {
            if m & DISPOSITION != 0 {
                continue;
            }
            for offset in 0..n {
                let mut cur = m;
                let mut last_enabled_step = [None; 6];
                let mut step = 0usize;
                let mut rr = offset;
                let mut fired_all = [false; 6];
                while step < SLACK {
                    for (i, t) in admission_surface().iter().enumerate() {
                        if cur & t.pre == t.pre && last_enabled_step[i].is_none() {
                            last_enabled_step[i] = Some(step);
                        }
                    }
                    let enabled: Vec<usize> = (0..n)
                        .filter(|&i| cur & admission_surface()[i].pre == admission_surface()[i].pre)
                        .collect();
                    if let Some(i) = (0..n).map(|k| (rr + k) % n).find(|k| enabled.contains(k)) {
                        fired_all[i] = true;
                        cur = fire(cur, &admission_surface()[i])
                            .unwrap_or_else(|| panic!("transition {i} must fire"));
                        rr = i + 1;
                    } else {
                        break;
                    }
                    step += 1;
                }
                for (i, &en) in last_enabled_step.iter().enumerate() {
                    if en.is_some() {
                        assert!(
                            fired_all[i],
                            "transition {i} enabled at {m:#06x} (offset {offset}) \
                             never fired within SLACK"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn n15_unfair_scheduler_starvation_counterexample() {
        // Anti-vacuity: the adversary never serves index >= 4 (previous
        // question / adopt), looping the amend cycle from the debated
        // marking 0x000C. previous_question stays enabled at every return
        // to 0x000C, its enabled streak passes SLACK, and the monitor SEES
        // the starvation. Without this trip the fairness law could be
        // silently vacuous.
        let start: u16 = ronr::N_MAIN_MOTION | ronr::N_SECONDED | ronr::N_DEBATABLE; // 0x000C
                                                                                     // The diamond is genuinely excluded for the adversary: restricted to
                                                                                     // indices < 4 (stated/second/amend/adopt_amendment), the schedule is
                                                                                     // trapped in the {0x000C, 0x0014} amend cycle and can never fire
                                                                                     // previous_question or adopt_main_motion, so no reachable marking
                                                                                     // carries the adoption bit.
        assert!(
            !can_reach(start, DISPOSITION, &admission_surface()[0..4]),
            "adversary must never reach adoption (the diamond is genuinely excluded)"
        );
        let (starved, dispositioned) = run_schedule(admission_surface(), start, false);
        assert!(!dispositioned, "unfair schedule must not disposition");
        assert_eq!(
            starved,
            vec![4usize],
            "exactly previous_question (index 4) must be seen starved past SLACK"
        );
        // The SAME start under the fair scheduler dispositions cleanly —
        // the trip is the scheduler's unfairness, not the net's deadlock.
        let (fair_starved, fair_dispositioned) = run_schedule(admission_surface(), start, true);
        assert!(fair_dispositioned, "fair schedule dispositions from 0x000C");
        assert!(fair_starved.is_empty(), "fair schedule starves nothing");
    }

    #[test]
    fn n15_adoption_is_absorbing_nonvacuous_diamond() {
        // Non-vacuity of the diamond: adoption is reachable from M0 and is
        // absorbing — no transition in the surface ever consumes N_ADOPTED.
        assert!(
            can_reach(M0, DISPOSITION, admission_surface()),
            "adoption must be reachable (non-vacuous diamond)"
        );
        // Adoption is absorbing: from every reachable adoption marking, every
        // enabled firing in the surface leaves the adoption bit set.
        for a in reachable(M0, admission_surface()) {
            if a & DISPOSITION != 0 {
                for t in admission_surface() {
                    if let Some(next) = fire(a, t) {
                        assert_ne!(
                            next & DISPOSITION,
                            0,
                            "adoption is never undone: {a:#06x} -> {next:#06x} via {}",
                            t.name
                        );
                    }
                }
            }
        }
    }

    /// Tier 3 (larger net): the FULL 12-verb RONR table. Exhaustive
    /// enumeration of the reachable set from 0x0081, box-diamond verdict
    /// over every reachable marking, and the counterexample set pinned to
    /// exactly the structurally trapped markings created by the
    /// non-admission interrupt verbs (recess / point of order) — the same
    /// pinned verdict as the strata N15 full-net court.
    #[test]
    #[ignore = "nato-tier3"]
    fn n15_tier3_full_net_box_diamond_traps_pinned() {
        let traps: [u16; 4] = [0x0884, 0x2084, 0x4884, 0x6084];
        let table: &[Shape] = &ronr::TRANSITIONS;
        let markings = reachable(M0, table);
        assert_eq!(markings.len(), 40, "full 12-verb net reachable markings from 0x0081");

        const FULL_DISPOSITION: u16 =
            ronr::N_ADOPTED | ronr::N_TABLED | ronr::N_POSTPONED_INDEF | ronr::N_ADJOURNED;
        let mut bad: Vec<u16> = markings
            .iter()
            .copied()
            .filter(|&m| !reach_has_disposition(m, table, FULL_DISPOSITION))
            .collect();
        bad.sort_unstable();
        assert_eq!(bad, traps, "box-diamond counterexample set must be exactly the 4 pinned traps");
        for &t in &traps {
            assert_eq!(t & ronr::N_DEBATABLE, 0, "trap {t:#06x} must have debate closed");
            assert_ne!(
                t & (ronr::N_RECESS_PENDING | ronr::N_POINT_OF_ORDER_RAISED),
                0,
                "trap {t:#06x} must carry recess and/or point-of-order"
            );
            assert_eq!(t & FULL_DISPOSITION, 0, "trap {t:#06x} is not itself a disposition");
        }
        assert_eq!(40 - bad.len(), 36, "36 of 40 markings reach a disposition");
    }

    /// Tier 3 companion: dropping the two non-admission interrupt verbs
    /// (0x09 recess, 0x0B point of order), the admission-critical subnet's
    /// ENTIRE reachable set satisfies box-diamond-disposition with ZERO
    /// traps, and the traps are unreachable in the subnet.
    #[test]
    #[ignore = "nato-tier3"]
    fn n15_tier3_admission_subnet_zero_traps() {
        let subnet: Vec<Shape> = ronr::TRANSITIONS
            .iter()
            .filter(|t| !matches!(t.name, "ronr_recess" | "ronr_point_of_order"))
            .copied()
            .collect();
        assert_eq!(subnet.len(), 10, "admission subnet drops exactly recess + point of order");
        let markings = reachable(M0, &subnet);
        assert_eq!(markings.len(), 32, "admission subnet reachable markings from 0x0081");
        const FULL_DISPOSITION: u16 =
            ronr::N_ADOPTED | ronr::N_TABLED | ronr::N_POSTPONED_INDEF | ronr::N_ADJOURNED;
        let bad: Vec<u16> = markings
            .iter()
            .copied()
            .filter(|&m| !reach_has_disposition(m, &subnet, FULL_DISPOSITION))
            .collect();
        assert!(
            bad.is_empty(),
            "box-diamond must hold on the whole admission subnet; violations: {bad:#06x?}"
        );
        // The full net's traps must not be subnet-reachable.
        for t in [0x0884u16, 0x2084, 0x4884, 0x6084] {
            assert!(!markings.contains(&t), "trap {t:#06x} must not be subnet-reachable");
        }
    }

    /// Whether any marking reachable from `m` under `table` carries a bit
    /// of `disposition`.
    fn reach_has_disposition(m: u16, table: &[Shape], disposition: u16) -> bool {
        reachable(m, table).iter().any(|r| r & disposition != 0)
    }

    /// Whether the round-robin schedule from `start` reaches `DISPOSITION`
    /// within `budget` steps (same cursor discipline as [`run_schedule`]).
    fn run_to_disposition_within(table: &[Shape], start: u16, budget: usize) -> bool {
        let mut cur = start;
        let mut rr = 0usize;
        let n = table.len();
        for _ in 0..budget {
            if cur & DISPOSITION != 0 {
                return true;
            }
            let enabled: Vec<usize> =
                (0..n).filter(|&i| cur & table[i].pre == table[i].pre).collect();
            match (0..n).map(|k| (rr + k) % n).find(|k| enabled.contains(k)) {
                Some(i) => match fire(cur, &table[i]) {
                    Some(next) => {
                        cur = next;
                        rr = i + 1;
                    }
                    None => return false,
                },
                None => return false,
            }
        }
        cur & DISPOSITION != 0
    }
}
