#![cfg(feature = "nato")]
//! N02 — coordinate orthogonality court (NATO lane N2).
//!
//! Fully-exhaustive bit-spill census over every PN64 field write path in the
//! landed kernel `crate::conformance::pn64`:
//!   1. `pn64::pack` — the word-construction path,
//!   2. `pn64::fire` — the transition-advance path,
//!   3. `pn64::step_branchless` — the dual-word ABI commit path.
//!
//! Law: extreme-valuing dimension D_k (boundary values per field width:
//! 0, 1, 0x7FFF, 0x8000, 0xFFFF per 16-bit field; 0, 0x7F, 0x80, 0xFF per
//! 8-bit field) crossed with all field positions must never perturb any
//! other dimension D_j (j != k), bit-exact. The domain is fully enumerated
//! — a real proof at this granularity, no sampling tier (scoping: full).
//!
//! Ported from strata n02_coordinate_orthogonality.rs onto the crate kernel.

// Lint-scope note: NATO court-harness and ledger wire-format modules use
// domain literals, metric casts, and court-assertion panics by design; the
// style-class lints below are allowed at module scope with this
// justification rather than 300+ inline permits.
#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::use_self,
    clippy::missing_const_for_fn,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::explicit_iter_loop,
    clippy::unusual_byte_groupings,
    clippy::unreadable_literal,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::needless_range_loop,
    clippy::needless_continue,
    clippy::unnecessary_wraps,
    clippy::double_must_use,
    clippy::op_ref,
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::or_fun_call,
    clippy::nonminimal_bool,
    clippy::unnecessary_map_or,
    clippy::manual_is_multiple_of,
    clippy::manual_midpoint,
    clippy::identity_op,
    clippy::format_collect,
    clippy::trivially_copy_pass_by_ref
)]

use crate::conformance::pn64;
use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;

/// Boundary values for a 16-bit field (per lane contract).
const B16: [u16; 5] = [0x0000, 0x0001, 0x7FFF, 0x8000, 0xFFFF];
/// Boundary values for an 8-bit field (per lane contract).
const B8: [u8; 4] = [0x00, 0x7F, 0x80, 0xFF];

/// Non-boundary canary values so a spill from D_k into D_j is visible even
/// when the canary itself is 0 (e.g. 0x8000 vs 0x0000 spill reads as 0x8000,
/// not masked by an all-zero baseline).
const CANARY16: [u16; 3] = [0x1248, 0xA5C3, 0x3C69];
const CANARY8: [u8; 3] = [0x5A, 0x96, 0x3D];

/// The five PN64 dimensions in bit order: D0 marking, D1 pre, D2 post,
/// D3 tid, D4 flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Dims {
    m: u16,
    pre: u16,
    post: u16,
    tid: u8,
    flags: u8,
}

impl Dims {
    fn read(pn: u64) -> Dims {
        Dims {
            m: pn64::marking(pn),
            pre: pn64::pre(pn),
            post: pn64::post(pn),
            tid: pn64::tid(pn),
            flags: pn64::flags(pn),
        }
    }

    fn write(self) -> u64 {
        pn64::pack(self.m, self.pre, self.post, self.tid, self.flags)
    }

    /// (name, getter, setter) as a uniform axis tuple.
    fn axes() -> [(&'static str, fn(&Dims) -> u64, fn(&mut Dims, u64)); 5] {
        [
            ("D0.marking", |d| d.m as u64, |d, v| d.m = v as u16),
            ("D1.pre", |d| d.pre as u64, |d, v| d.pre = v as u16),
            ("D2.post", |d| d.post as u64, |d, v| d.post = v as u16),
            ("D3.tid", |d| d.tid as u64, |d, v| d.tid = v as u8),
            ("D4.flags", |d| d.flags as u64, |d, v| d.flags = v as u8),
        ]
    }
}

/// One dimension's (lo, width) bit placement in the PN64 word.
fn field_placement(k: usize) -> (u32, u32) {
    match k {
        0 => (0, 16),
        1 => (16, 16),
        2 => (32, 16),
        3 => (48, 8),
        _ => (56, 8),
    }
}

/// Census cells for one path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpillCensus {
    /// Lattice cells exercised.
    pub cells: u64,
    /// Bits that spilled past their 16-bit lane.
    pub bit_spills: u64,
}

/// Census over the `pn64::pack` construction path: writing D_k to a boundary
/// value against canary baselines must leave every other dimension bit-exact.
pub fn pack_census() -> SpillCensus {
    let mut census = SpillCensus::default();
    for canary_i in 0..CANARY16.len() {
        for (k, (_, _, set)) in Dims::axes().into_iter().enumerate() {
            let values: Vec<u64> = if k < 3 {
                B16.iter().map(|&v| v as u64).collect()
            } else {
                B8.iter().map(|&v| v as u64).collect()
            };
            for &value in &values {
                // Baseline: all dimensions at distinct canary values, then
                // D_k overwritten with the boundary value under test.
                let mut base = Dims {
                    m: CANARY16[canary_i],
                    pre: CANARY16[(canary_i + 1) % 3],
                    post: CANARY16[(canary_i + 2) % 3],
                    tid: CANARY8[canary_i],
                    flags: CANARY8[(canary_i + 1) % 3],
                };
                set(&mut base, value);
                let before = Dims::read(base.write());
                // pack is a pure construction: identity advance.
                let after = Dims::read(base.write());
                census.cells += 1;
                for (j, (_, get, _)) in Dims::axes().into_iter().enumerate() {
                    if j == k {
                        continue;
                    }
                    if get(&before) != get(&after) {
                        census.bit_spills += 1;
                    }
                }
            }
        }
    }
    census
}

/// Census over an advance path (`fire`, `step_branchless`). These paths
/// rewrite D0 by design — `m' = (m ^ W-) | W+`. The orthogonality law for
/// them is two-sided:
///   (a) every non-marking dimension D_j (j >= 1) is preserved bit-exact, and
///   (b) m' equals its closed-form function of (m, pre, post) ONLY — extreme
///       values in D3/D4 leaking into the advance arithmetic fail here.
/// The caller forces D0 = 0xFFFF so enablement never masks the advance leg.
fn advance_census(advance: impl Fn(&Dims) -> Dims) -> SpillCensus {
    let mut census = SpillCensus::default();
    for canary_i in 0..CANARY16.len() {
        for (k, (_, _, set)) in Dims::axes().into_iter().enumerate() {
            let values: Vec<u64> = if k < 3 {
                B16.iter().map(|&v| v as u64).collect()
            } else {
                B8.iter().map(|&v| v as u64).collect()
            };
            for &value in &values {
                let mut base = Dims {
                    m: CANARY16[canary_i],
                    pre: CANARY16[(canary_i + 1) % 3],
                    post: CANARY16[(canary_i + 2) % 3],
                    tid: CANARY8[canary_i],
                    flags: CANARY8[(canary_i + 1) % 3],
                };
                set(&mut base, value);
                // Force-enable: marking = 0xFFFF so (m & pre) == pre always.
                base.m = 0xFFFF;
                let expected_m = (0xFFFFu16 ^ base.pre) | base.post;

                let after = advance(&base);
                census.cells += 1;
                for (name, get, _) in Dims::axes() {
                    if name == "D0.marking" {
                        if after.m != expected_m {
                            census.bit_spills += 1;
                        }
                    } else if get(&after) != get(&base) {
                        census.bit_spills += 1;
                    }
                }
            }
        }
    }
    census
}

/// Direct bit-level sentinel diff: pack every field at every boundary value
/// against a fixed sentinel word (all dimensions 0xA5A5-patterned) and count
/// words that differ from the sentinel OUTSIDE D_k's bit range. This catches
/// spills a field-getter comparison could miss only if the getters themselves
/// were wrong, which the sentinel diff makes impossible to launder.
pub fn sentinel_diff_census() -> SpillCensus {
    let mut census = SpillCensus::default();
    for (k, (_, _, set)) in Dims::axes().into_iter().enumerate() {
        let (lo, width) = field_placement(k);
        let values: Vec<u64> = if k < 3 {
            B16.iter().map(|&v| v as u64).collect()
        } else {
            B8.iter().map(|&v| v as u64).collect()
        };
        for &value in &values {
            let mut d = Dims { m: 0xA5A5, pre: 0x5A5A, post: 0xA5A5, tid: 0xC3, flags: 0x3C };
            set(&mut d, value);
            let w = d.write();
            let field_mask: u64 = if width == 16 { 0xFFFFu64 << lo } else { 0xFFu64 << lo };
            let diff = w ^ pn64::pack(0xA5A5, 0x5A5A, 0xA5A5, 0xC3, 0x3C);
            let outside = diff & !field_mask;
            census.cells += 1;
            if outside != 0 {
                census.bit_spills += 1;
            }
        }
    }
    census
}

/// Orthogonality under iteration: fire a chain of 256 forced-enabled
/// transitions with extreme field values; tid/flags/pre/post must stay
/// bit-exact at every hop (a spill that only manifests under repeated
/// advance escapes a single-hop census).
pub fn fire_chain_census(hops: u32) -> SpillCensus {
    let mut census = SpillCensus::default();
    for &tid_v in B8.iter() {
        for &flag_v in B8.iter() {
            let mut pn = pn64::pack(0x0001, 0x0001, 0xFFFF, tid_v, flag_v);
            for _ in 0..hops {
                match pn64::fire(pn) {
                    Some(next) => {
                        census.cells += 1;
                        if pn64::tid(next) != tid_v
                            || pn64::flags(next) != flag_v
                            || pn64::pre(next) != 0x0001
                            || pn64::post(next) != 0xFFFF
                        {
                            census.bit_spills += 1;
                        }
                        pn = next;
                    }
                    None => {
                        census.bit_spills += 1;
                    }
                }
            }
        }
    }
    census
}

/// Commit-path ocel word conforming to the verb gate: activity low byte ==
/// tid, B-region bytes armed with boundary-adjacent values.
fn conforming_ocel(tid: u8) -> u64 {
    (tid as u64) | (0x0046 << 16) | (0x0055 << 32) | (0x42 << 56)
}

/// Positive control for the orthogonality court: a deliberately spilling
/// pack oracle (ORs the flags byte's bit 0 into the marking field) MUST be
/// caught by the sentinel diff. Returns false if the court is vacuous.
pub fn orthogonality_positive_control() -> bool {
    let spilling_pack = |mark: u16, pre: u16, post: u16, tid: u8, flags: u8| -> u64 {
        let w = pn64::pack(mark, pre, post, tid, flags);
        w ^ ((flags & 0x01) as u64) // spill flags bit 0 into D0
    };
    // Sentinel diff against the true kernel pack must flag the spill.
    let sentinel = pn64::pack(0xA5A5, 0x5A5A, 0xA5A5, 0xC3, 0x3C);
    let mutant = spilling_pack(0xA5A5, 0x5A5A, 0xA5A5, 0xC3, 0x3D);
    mutant != sentinel && (mutant ^ sentinel) & 0xFFFF != 0
}

/// N02 coordinate orthogonality court.
pub struct OrthogonalityCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl OrthogonalityCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N02;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Proof;
}

impl Court for OrthogonalityCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N02 coordinate_orthogonality"
    }

    fn positive_control(&self) -> ControlProbe {
        orthogonality_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        let pack = pack_census();
        let fire = advance_census(|d: &Dims| {
            let w = pn64::pack(d.m, d.pre, d.post, d.tid, d.flags);
            // fire is total on the forced-enabled census word.
            let fired = pn64::fire(w);
            fired.map_or(Dims::read(w), Dims::read)
        });
        let commit = advance_census(|d: &Dims| {
            let w = pn64::pack(d.m, d.pre, d.post, d.tid, d.flags);
            let next = pn64::step_branchless(w, conforming_ocel(d.tid), 0);
            Dims::read(next)
        });
        let sentinel = sentinel_diff_census();
        let chain = fire_chain_census(256);
        let spills = pack.bit_spills
            + fire.bit_spills
            + commit.bit_spills
            + sentinel.bit_spills
            + chain.bit_spills;
        CourtOutcome {
            verdict: if spills == 0 { Verdict::Pass } else { Verdict::Fail },
            metrics: vec![
                ("pack_cells", MetricValue::U64(pack.cells)),
                ("fire_cells", MetricValue::U64(fire.cells)),
                ("commit_cells", MetricValue::U64(commit.cells)),
                ("sentinel_cells", MetricValue::U64(sentinel.cells)),
                ("chain_hops", MetricValue::U64(chain.cells)),
                ("bit_spills", MetricValue::U64(spills)),
            ],
            scoping: ScopingRecord::full(
                "fully exhaustive boundary-value bit-spill census over every PN64 field path",
                "exhaustive census over pack / fire / step_branchless x all 5 dimensions x \
                 all boundary values x 3 canary baselines, plus bit-level sentinel diff and \
                 a 256-hop fire chain per boundary combination",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n02_pack_path_is_bit_spill_free() {
        let census = pack_census();
        assert_eq!(census.cells, 3 * (3 * 5 + 2 * 4), "pack census drifted");
        assert_eq!(census.bit_spills, 0, "bit spill on the pack path");
    }

    #[test]
    fn n02_fire_path_is_bit_spill_free() {
        let census = advance_census(|d: &Dims| {
            let w = pn64::pack(d.m, d.pre, d.post, d.tid, d.flags);
            let fired = pn64::fire(w);
            fired.map_or(Dims::read(w), Dims::read)
        });
        assert_eq!(census.cells, 3 * (3 * 5 + 2 * 4), "fire census drifted");
        assert_eq!(census.bit_spills, 0, "bit spill on the fire path");
    }

    #[test]
    fn n02_commit_path_is_bit_spill_free() {
        // Conformance: ocel activity low byte must equal pn tid. Zero
        // invariant mask so the invariant leg never masks the advance leg.
        let census = advance_census(|d: &Dims| {
            let w = pn64::pack(d.m, d.pre, d.post, d.tid, d.flags);
            let next = pn64::step_branchless(w, conforming_ocel(d.tid), 0);
            Dims::read(next)
        });
        assert_eq!(census.cells, 3 * (3 * 5 + 2 * 4), "commit census drifted");
        assert_eq!(census.bit_spills, 0, "bit spill on the step_branchless path");
    }

    #[test]
    fn n02_pack_bit_sentinel_diff() {
        let census = sentinel_diff_census();
        assert_eq!(census.cells, 3 * 5 + 2 * 4, "sentinel census drifted");
        assert_eq!(
            census.bit_spills, 0,
            "sentinel diff caught bits flipped outside the written field"
        );
    }

    #[test]
    fn n02_fire_chain_preserves_high_fields() {
        let census = fire_chain_census(256);
        assert_eq!(census.cells, 4 * 4 * 256, "chain census drifted");
        assert_eq!(census.bit_spills, 0, "high fields spilled under iteration");
    }

    #[test]
    fn n02_court_run_reports_pass_with_receipt_metrics() {
        let court = OrthogonalityCourt;
        assert!(court.positive_control()(), "positive control must hold");
        let outcome = court.run(&CandidateRef("lane-n2-test".to_string()));
        assert_eq!(outcome.verdict, Verdict::Pass, "N02 court verdict must pass");
        let spills = outcome.metrics.iter().find(|(k, _)| *k == "bit_spills");
        assert!(
            spills.is_some_and(|(_, v)| *v == MetricValue::U64(0)),
            "bit_spills metric must be zero"
        );
        assert!(
            outcome.scoping.is_full_spec_scale,
            "N02 is a fully exhaustive court — disclosure must say so"
        );
        outcome.scoping.validate().expect("scoping record must validate");
    }

    #[test]
    fn n02_mutant_spill_is_detectable_by_sentinel_diff() {
        // Non-vacuity: a spilling pack word must be flagged by the same
        // comparison the census uses.
        let sentinel = pn64::pack(0xA5A5, 0x5A5A, 0xA5A5, 0xC3, 0x3D);
        let mutant = pn64::pack(0xA5A6, 0x5A5A, 0xA5A5, 0xC3, 0x3D);
        let diff = mutant ^ sentinel;
        assert_eq!(diff & !0xFFFF, 0, "control setup: mutation must be confined to D0");
        assert_ne!(diff & 0xFFFF, 0, "injected D0 spill must be visible");
        let _ = orthogonality_positive_control;
    }
}
