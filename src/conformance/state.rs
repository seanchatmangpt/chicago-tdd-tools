//! Bitmask state-space analysis over a compiled 1-safe net.
//!
//! Markings are single `u64` words: bit `p` set means place `p` holds a token.
//! Transitions come from [`super::net::CompiledNet`], where `words[i]` packs
//! `((post & 0xFFFF) << 16) | (pre & 0xFFFF)` for transition `i` — the low
//! 16 bits are the preset mask, bits 16..32 the postset mask.
//!
//! NOTE: `super::net` currently defines its own placeholder `Marking64`
//! (to be deleted in favour of this module's). Until C7 removes it the two
//! definitions coexist in sibling modules; this module's `Marking64` is the
//! canonical one (it carries `Hash`/`Eq` for the visited set).

use std::collections::{HashSet, VecDeque};

use super::error::ConformanceError;
use super::net::CompiledNet;

/// A net marking packed into one word: bit `p` = place `p` is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Marking64(pub u64);

impl Marking64 {
    /// True if every place in `mask` is marked.
    #[must_use]
    pub const fn covers(self, mask: u64) -> bool {
        self.0 & mask == mask
    }
}

/// Breadth-first reachability result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reachability {
    /// Number of distinct reachable markings (including the initial one).
    pub count: usize,
    /// Reachable markings with no enabled transition (deadlocks).
    pub terminal: Vec<Marking64>,
}

/// State-space engine over a compiled net with a forbidden-place mask.
#[derive(Debug, Clone)]
pub struct StateSpace {
    net: CompiledNet,
    initial: Marking64,
    forbidden_mask: u64,
    max_markings: usize,
}

const DEFAULT_MAX_MARKINGS: usize = 200_000;

impl StateSpace {
    /// Builds a state space engine.
    ///
    /// # Errors
    /// [`ConformanceError::TooManyPlaces`] when the net declares more than 64
    /// places, or [`ConformanceError::ForbiddenBit`] (as the existing typed
    /// carrier for mask/marking intersection) when the initial marking
    /// already intersects the caller-supplied forbidden mask.
    pub fn new(
        net: CompiledNet,
        initial: Marking64,
        forbidden_mask: u64,
    ) -> Result<Self, ConformanceError> {
        if u64::from(net.places) > 64 {
            return Err(ConformanceError::TooManyPlaces { limit: 64 });
        }
        let place_bits = place_mask(net.places);
        if initial.0 & !place_bits != 0 {
            return Err(ConformanceError::BcinrValidation {
                message: format!(
                    "initial marking {:#x} sets bits outside the net's {} places",
                    initial.0, net.places
                ),
            });
        }
        if initial.0 & forbidden_mask != 0 {
            return Err(ConformanceError::ForbiddenBit {
                step: 0,
                // JUSTIFICATION: the ForbiddenBit error fields are u16 by
                // the shared error vocabulary; masks in this module's tests
                // and callers are 16-bit place masks, so the narrowing is
                // the documented diagnostic width, not silent data loss.
                #[allow(clippy::cast_possible_truncation)]
                marking: initial.0 as u16,
                #[allow(clippy::cast_possible_truncation)]
                mask: forbidden_mask as u16,
            });
        }
        Ok(Self { net, initial, forbidden_mask, max_markings: DEFAULT_MAX_MARKINGS })
    }

    /// Overrides the reachability cap (default `200_000` markings).
    #[must_use]
    pub const fn with_max_markings(mut self, max_markings: usize) -> Self {
        self.max_markings = max_markings;
        self
    }

    /// Bitmask of transitions enabled at `m`.
    ///
    /// Transition `i` is enabled iff `(m & W-) == W-`, i.e. every preset
    /// place is marked.
    #[must_use]
    pub fn enabled_mask(&self, m: Marking64) -> u64 {
        let mut enabled = 0u64;
        for (tid, &word) in self.net.words.iter().enumerate() {
            let pre = word & 0xFFFF;
            if m.0 & pre == pre {
                enabled |= 1u64 << tid;
            }
        }
        enabled
    }

    /// Fires transition `tid` at `m`, returning `None` if it is not enabled
    /// or the successor marking intersects the forbidden mask.
    #[must_use]
    pub fn fire(&self, m: Marking64, tid: u8) -> Option<Marking64> {
        self.try_fire(m, tid).ok()
    }

    /// Fires transition `tid` at `m` with a typed failure reason.
    ///
    /// # Errors
    /// [`ConformanceError::Undeclared`] when `tid` is not a transition of the
    /// net; [`ConformanceError::TraceMalformed`] (length 0 standing for
    /// "not enabled") when the preset is not fully marked;
    /// [`ConformanceError::ForbiddenBit`] when the successor marking
    /// intersects the forbidden mask.
    pub fn try_fire(&self, m: Marking64, tid: u8) -> Result<Marking64, ConformanceError> {
        let word = *self
            .net
            .words
            .get(usize::from(tid))
            .ok_or(ConformanceError::Undeclared { step: 0, tid })?;
        let pre = word & 0xFFFF;
        let post = word >> 16;
        if m.0 & pre != pre {
            return Err(ConformanceError::TraceMalformed { len: 0 });
        }
        let successor = Marking64((m.0 ^ pre) | post);
        if successor.0 & self.forbidden_mask != 0 {
            return Err(ConformanceError::ForbiddenBit {
                step: 0,
                // JUSTIFICATION: see StateSpace::new — the diagnostic fields
                // are u16 by the shared error vocabulary.
                #[allow(clippy::cast_possible_truncation)]
                marking: successor.0 as u16,
                #[allow(clippy::cast_possible_truncation)]
                mask: self.forbidden_mask as u16,
            });
        }
        Ok(successor)
    }

    /// Breadth-first reachable-marking enumeration from the initial marking.
    ///
    /// # Errors
    /// [`ConformanceError::ModelTooLarge`] when the reachable set exceeds the
    /// configured cap.
    pub fn reach(&self) -> Result<Reachability, ConformanceError> {
        let mut visited: HashSet<Marking64> = HashSet::new();
        visited.insert(self.initial);
        let mut queue: VecDeque<Marking64> = VecDeque::new();
        queue.push_back(self.initial);
        let mut terminal: Vec<Marking64> = Vec::new();

        while let Some(m) = queue.pop_front() {
            let enabled = self.enabled_mask(m);
            if enabled == 0 {
                terminal.push(m);
            }
            for tid in 0..self.net.words.len() {
                if enabled & (1u64 << tid) == 0 {
                    continue;
                }
                // Fire without the forbidden gate: forbidden successors are
                // still reachable states, they just must be *reported*.
                let word = self.net.words[tid];
                let successor = Marking64((m.0 ^ (word & 0xFFFF)) | (word >> 16));
                if visited.insert(successor) {
                    if visited.len() > self.max_markings {
                        return Err(ConformanceError::ModelTooLarge {
                            found: visited.len(),
                            limit: self.max_markings,
                        });
                    }
                    queue.push_back(successor);
                }
            }
        }

        Ok(Reachability { count: visited.len(), terminal })
    }

    /// True when no reachable marking intersects the forbidden mask.
    ///
    /// # Errors
    /// [`ConformanceError::ModelTooLarge`] when reachability is capped.
    pub fn is_safe(&self) -> Result<bool, ConformanceError> {
        Ok(self.forbidden_reachable()?.is_none())
    }

    /// Always true for bitmask-encoded nets: a `u64` marking holds at most
    /// one token per place by construction (a bit is either 0 or 1), so the
    /// 1-safe invariant cannot be violated. Kept as an explicit named check
    /// so callers express intent rather than rely on the encoding.
    #[must_use]
    pub const fn is_one_safe_by_encoding() -> bool {
        true
    }

    /// True when no reachable marking is a deadlock (zero enabled
    /// transitions).
    ///
    /// # Errors
    /// [`ConformanceError::ModelTooLarge`] when reachability is capped.
    pub fn deadlock_free(&self) -> Result<bool, ConformanceError> {
        Ok(self.reach()?.terminal.is_empty())
    }

    /// First forbidden marking found in breadth-first order, if any.
    ///
    /// # Errors
    /// [`ConformanceError::ModelTooLarge`] when reachability is capped.
    pub fn forbidden_reachable(&self) -> Result<Option<Marking64>, ConformanceError> {
        if self.forbidden_mask == 0 {
            return Ok(None);
        }
        let mut visited: HashSet<Marking64> = HashSet::new();
        visited.insert(self.initial);
        let mut queue: VecDeque<Marking64> = VecDeque::new();
        queue.push_back(self.initial);

        while let Some(m) = queue.pop_front() {
            if m.0 & self.forbidden_mask != 0 {
                return Ok(Some(m));
            }
            let enabled = self.enabled_mask(m);
            for (tid, &word) in self.net.words.iter().enumerate() {
                if enabled & (1u64 << tid) == 0 {
                    continue;
                }
                let successor = Marking64((m.0 ^ (word & 0xFFFF)) | (word >> 16));
                if visited.insert(successor) {
                    if visited.len() > self.max_markings {
                        return Err(ConformanceError::ModelTooLarge {
                            found: visited.len(),
                            limit: self.max_markings,
                        });
                    }
                    queue.push_back(successor);
                }
            }
        }
        Ok(None)
    }
}

/// All-ones mask for `places` places (saturating at 64).
const fn place_mask(places: u8) -> u64 {
    if places >= 64 {
        u64::MAX
    } else {
        (1u64 << places) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::super::net::NetSpec;
    use super::*;

    /// Net under test (3 places, 3 transitions):
    ///
    /// ```text
    /// p0 --t0--> p1        (p0 -> p1)
    /// p1 --t1--> p2        (p1 -> p2)
    /// p2 --t2--> p0        (p2 -> p0)   cycle
    /// ```
    ///
    /// Built through the real `NetSpec` compiler (Chicago: real collaborator)
    /// so the packed-word layout is C7's, never a hand-rolled encoding.
    fn cycle_net() -> CompiledNet {
        NetSpec::builder()
            .place("p0")
            .place("p1")
            .place("p2")
            .transition("t0", "a0")
            .add_pre("p0")
            .add_post("p1")
            .transition("t1", "a1")
            .add_pre("p1")
            .add_post("p2")
            .transition("t2", "a2")
            .add_pre("p2")
            .add_post("p0")
            .build()
            .compile()
            .expect("cycle net is structurally valid")
    }

    /// Two parallel transitions from p0 into the same place p1 (conflict):
    /// t0: p0 -> p1, t1: p0 -> p1 — exercises dedup in BFS.
    fn conflict_net() -> CompiledNet {
        NetSpec::builder()
            .place("p0")
            .place("p1")
            .transition("t0", "a")
            .add_pre("p0")
            .add_post("p1")
            .transition("t1", "a")
            .add_pre("p0")
            .add_post("p1")
            .build()
            .compile()
            .expect("conflict net is structurally valid")
    }

    /// A one-place fan-out: 14 transitions each `p0 -> p_i` (i in 1..=14),
    /// producing 2^14 reachable states — used to exercise the cap. Kept at
    /// 15 declared places so every pre/post mask fits the compiled word's
    /// 16-bit pre/post windows (a place index >= 16 cannot be packed).
    fn fanout_net() -> CompiledNet {
        let mut builder = NetSpec::builder().place("p0");
        for i in 1..=14 {
            builder = builder
                .place(format!("p{i}"))
                .transition(format!("t{i}"), "fan")
                .add_pre("p0")
                .add_post(format!("p{i}"));
        }
        builder.build().compile().expect("fan-out net is valid")
    }

    #[test]
    fn small_net_reachability_counts_all_cycle_markings() {
        let ss = StateSpace::new(cycle_net(), Marking64(0b001), 0).expect("net is valid");
        let reach = ss.reach().expect("3-state cycle is under the cap");
        assert_eq!(reach.count, 3);
        // A 3-cycle has no deadlock: the token always has a move.
        assert!(reach.terminal.is_empty());
        assert!(ss.deadlock_free().expect("reach under cap"));
    }

    #[test]
    fn enabled_mask_refuses_unmarked_preset() {
        let ss = StateSpace::new(cycle_net(), Marking64(0b001), 0).expect("net is valid");
        // At p0 only t0 is enabled.
        assert_eq!(ss.enabled_mask(Marking64(0b001)), 0b001);
        // At the empty marking nothing is enabled.
        assert_eq!(ss.enabled_mask(Marking64(0)), 0);
    }

    #[test]
    fn fire_updates_marking_and_refuses_disabled() {
        let ss = StateSpace::new(cycle_net(), Marking64(0b001), 0).expect("net is valid");
        let next = ss.fire(Marking64(0b001), 0).expect("t0 enabled at p0");
        assert_eq!(next, Marking64(0b010));
        // t1 is not enabled at p0.
        assert!(ss.fire(Marking64(0b001), 1).is_none());
        // Out-of-range tid is a typed error, not a silent None.
        let err = ss.try_fire(Marking64(0b001), 9).expect_err("tid 9 is undeclared");
        assert!(matches!(err, ConformanceError::Undeclared { tid: 9, .. }));
    }

    #[test]
    fn deadlock_detection_on_empty_initial() {
        let ss = StateSpace::new(cycle_net(), Marking64(0), 0).expect("net is valid");
        let reach = ss.reach().expect("single state");
        assert_eq!(reach.count, 1);
        assert_eq!(reach.terminal, vec![Marking64(0)]);
        assert!(!ss.deadlock_free().expect("reach under cap"));
    }

    #[test]
    fn forbidden_bit_reachability_finds_and_guards() {
        // p1 is forbidden; the cycle reaches it in one step from p0.
        let ss = StateSpace::new(cycle_net(), Marking64(0b001), 0b010)
            .expect("initial p0 does not intersect the mask");
        let hit = ss
            .forbidden_reachable()
            .expect("3-state search under cap")
            .expect("p1 is reachable");
        assert_eq!(hit, Marking64(0b010));
        assert!(!ss.is_safe().expect("reach under cap"));

        // Firing into the forbidden place is a typed ForbiddenBit refusal.
        let err = ss.try_fire(Marking64(0b001), 0).expect_err("t0 lands on forbidden p1");
        assert!(matches!(err, ConformanceError::ForbiddenBit { marking: 0b010, mask: 0b010, .. }));
        // And the plain fire() collapses it to None.
        assert!(ss.fire(Marking64(0b001), 0).is_none());

        // With no forbidden places the same net is safe.
        let open = StateSpace::new(cycle_net(), Marking64(0b001), 0).expect("valid");
        assert!(open.is_safe().expect("reach under cap"));
    }

    /// Net: p0 --t0--> p1 (forbidden target via a DISABLED path) and
    /// p0 --t1--> p2 (enabled, benign). Only t1's preset is marked reachable.
    fn disabled_path_net() -> CompiledNet {
        NetSpec::builder()
            .place("p0")
            .place("p1")
            .place("p2")
            .place("p3")
            .transition("t0", "a")
            .add_pre("p3")
            .add_post("p1")
            .transition("t1", "b")
            .add_pre("p0")
            .add_post("p2")
            .build()
            .compile()
            .expect("disabled-path net is structurally valid")
    }

    #[test]
    fn forbidden_reachable_ignores_disabled_transition_paths() {
        // t0 (p3 -> p1) is never enabled from initial p0 (p3 unmarked), so the
        // forbidden place p1 is NOT reachable despite the transition existing.
        let ss = StateSpace::new(disabled_path_net(), Marking64(0b001), 0b010)
            .expect("initial p0 does not intersect the mask");
        assert!(
            ss.forbidden_reachable().expect("search under cap").is_none(),
            "disabled t0 must not manufacture a path to forbidden p1"
        );
        assert!(ss.is_safe().expect("reach under cap"));
    }

    #[test]
    fn forbidden_reachable_still_finds_enabled_transition_paths() {
        // Same net, but the initial marking marks p3, enabling t0 (p3 -> p1):
        // the forbidden place IS reachable through an enabled transition.
        let ss = StateSpace::new(disabled_path_net(), Marking64(0b1001), 0b010)
            .expect("initial {p0,p3} does not intersect the mask");
        let hit = ss
            .forbidden_reachable()
            .expect("search under cap")
            .expect("enabled t0 reaches forbidden p1");
        assert_eq!(hit, Marking64(0b011)); // {p0, p1}: p0 is not consumed by t0
        assert!(!ss.is_safe().expect("reach under cap"));
    }

    #[test]
    fn forbidden_mask_zero_short_circuits_search() {
        let ss = StateSpace::new(cycle_net(), Marking64(0b001), 0).expect("valid");
        assert!(ss.forbidden_reachable().expect("short-circuit").is_none());
    }

    #[test]
    fn cap_refusal_is_typed_model_too_large() {
        // A fan-out net: t_i moves p0 -> p_i for 20 distinct targets, so the
        // reachable set is 2^20 states — far past a tiny cap.
        let ss = StateSpace::new(fanout_net(), Marking64(0b1), 0)
            .expect("net is valid")
            .with_max_markings(8);
        let err = ss.reach().expect_err("2^20 states > cap of 8");
        assert!(
            matches!(err, ConformanceError::ModelTooLarge { limit: 8, .. }),
            "expected ModelTooLarge, got {err:?}"
        );
        assert!(ss.deadlock_free().is_err());
    }

    #[test]
    fn conflict_net_deduplicates_identical_successors() {
        let ss = StateSpace::new(conflict_net(), Marking64(0b01), 0).expect("valid");
        let reach = ss.reach().expect("2 states under cap");
        // p0 and p1 — both transitions produce the same successor.
        assert_eq!(reach.count, 2);
        assert_eq!(reach.terminal, vec![Marking64(0b10)]);
    }

    /// At-limit admission (T4): `places = 64` compiles Ok and the initial
    /// marking may use the full `u64` word — the entire justification for the
    /// `u64` marking encoding. `place_mask(64)` saturates to `u64::MAX`, so a
    /// marking with every bit set is inside the net.
    #[test]
    fn state_space_admits_64_place_net_with_full_width_marking() {
        let net = CompiledNet { places: 64, words: vec![] };
        let ss = StateSpace::new(net, Marking64(u64::MAX), 0).expect("64 places admit");
        let reach = ss.reach().expect("single-state search under cap");
        assert_eq!(reach.count, 1);
        assert_eq!(reach.terminal, vec![Marking64(u64::MAX)]);
        // With no transitions the lone state is terminal: deadlock observed,
        // not an error.
        assert!(!ss.deadlock_free().expect("reach under cap"));
    }

    #[test]
    fn constructor_rejects_oversized_nets_and_bad_initials() {
        let big = CompiledNet { places: 65, words: vec![] };
        let err = StateSpace::new(big, Marking64(0), 0)
            .expect_err("65 places exceed the u64 marking encoding");
        assert!(matches!(err, ConformanceError::TooManyPlaces { limit: 64 }));

        let net = cycle_net();
        let err = StateSpace::new(net.clone(), Marking64(1 << 5), 0)
            .expect_err("bit 5 is outside a 3-place net");
        assert!(matches!(err, ConformanceError::BcinrValidation { .. }));

        let err = StateSpace::new(net, Marking64(0b010), 0b010)
            .expect_err("initial marking intersects the forbidden mask");
        assert!(matches!(err, ConformanceError::ForbiddenBit { .. }));
    }

    #[test]
    fn one_safety_is_structural_for_bitmask_markings() {
        assert!(StateSpace::is_one_safe_by_encoding());
        // A set bit can never exceed 1 token: covers() semantics.
        let m = Marking64(0b101);
        assert!(m.covers(0b001));
        assert!(!m.covers(0b010));
    }
}
