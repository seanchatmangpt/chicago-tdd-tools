//! Data model for the conformance subsystem: the validated Petri-net surface
//! that POWL models lower into.
//!
//! This module is deliberately type-only (no I/O, no process state): it
//! defines the Petri-net vocabulary. The POWL vocabulary lives in
//! [`super::loader`] ([`PowlModel`](super::loader::PowlModel),
//! [`PowlNode`](super::loader::PowlNode)), whose constructor enforces the
//! 64-node bound ([`MAX_POWL_NODES`](super::loader::MAX_POWL_NODES)).
//! Validation lives in the constructors so an invalid value of these types
//! cannot be constructed through the public API.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::error::ConformanceError;

/// Maximum number of places a [`PetriNet`] may contain.
pub const MAX_PETRI_PLACES: u16 = 16;

/// Identifier of a place in a [`PetriNet`] (index into the place vector).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceId(pub u16);

/// Identifier of a transition in a [`PetriNet`] (index into the transition
/// vector).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransitionId(pub u16);

/// Either endpoint of an [`Arc`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NodeRef {
    /// A place endpoint.
    Place(PlaceId),
    /// A transition endpoint.
    Transition(TransitionId),
}

/// A directed, weighted flow relation edge between a place and a
/// transition (in either direction).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arc {
    /// Source endpoint of the arc.
    pub from: NodeRef,
    /// Target endpoint of the arc.
    pub to: NodeRef,
    /// Arc weight; only weight 1 is admitted by [`PetriNet::try_new`].
    pub weight: u16,
}

/// A labeled transition of a [`PetriNet`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    /// Structural name, unique within the net.
    pub name: String,
    /// Observable label; `None` marks a silent (tau) transition.
    pub label: Option<String>,
}

/// A place/transition net with an initial marking.
///
/// Constructed exclusively through [`PetriNet::try_new`], which enforces
/// ordinary arcs (weight 1) and the place bound ([`MAX_PETRI_PLACES`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetriNet {
    pub(crate) places: Vec<String>,
    pub(crate) transitions: Vec<Transition>,
    pub(crate) arcs: Vec<Arc>,
    pub(crate) initial_marking: BTreeSet<PlaceId>,
}

impl PetriNet {
    /// Builds a net, validating the constructor invariants.
    ///
    /// # Errors
    ///
    /// Returns a [`ConformanceError`] when any arc has a weight other
    /// than 1, or when more than [`MAX_PETRI_PLACES`] places are supplied.
    pub fn try_new(
        places: Vec<String>,
        transitions: Vec<Transition>,
        arcs: Vec<Arc>,
        initial_marking: BTreeSet<PlaceId>,
    ) -> Result<Self, ConformanceError> {
        if places.len() > usize::from(MAX_PETRI_PLACES) {
            return Err(ConformanceError::TooManyPlaces { limit: usize::from(MAX_PETRI_PLACES) });
        }
        if let Some(bad) = arcs.iter().find(|arc| arc.weight != 1) {
            return Err(ConformanceError::WeightedArc { weight: bad.weight });
        }
        Ok(Self { places, transitions, arcs, initial_marking })
    }

    /// Place names, indexed by [`PlaceId`].
    #[must_use]
    pub fn places(&self) -> &[String] {
        &self.places
    }

    /// Transitions, indexed by [`TransitionId`].
    #[must_use]
    pub fn transitions(&self) -> &[Transition] {
        &self.transitions
    }

    /// The flow relation.
    #[must_use]
    pub fn arcs(&self) -> &[Arc] {
        &self.arcs
    }

    /// The set of places holding a token in the initial marking.
    #[must_use]
    pub const fn initial_marking(&self) -> &BTreeSet<PlaceId> {
        &self.initial_marking
    }
}

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

/// Shared test fixtures for the conformance subsystem.
///
/// The `net_*` / `well_formed_net` fixtures build [`PetriNet`] values
/// directly through the `pub(crate)` fields — not [`PetriNet::try_new`] —
/// because several of them represent *invalid* nets (`try_new` refuses
/// those, so exercising downstream consumers requires constructing the
/// struct literally).
#[cfg(test)]
pub mod test_fixtures {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{Arc, NodeRef, PetriNet, PlaceId, Transition, TransitionId};
    use crate::conformance::loader::{ActivityDecl, PowlModel, PowlNode};

    /// A [`PowlModel`] containing exactly `n` nodes: one sequence root plus
    /// `n - 1` activity leaves (`n >= 1`; `n = 0` yields the empty sequence).
    pub fn model_with_nodes(n: usize) -> PowlModel {
        let leaves = n.saturating_sub(1);
        PowlModel {
            activities: (0..leaves)
                .map(|i| {
                    (
                        format!("a{i}"),
                        ActivityDecl { source: "s".to_string(), event: format!("e{i}") },
                    )
                })
                .collect(),
            declaration_order: (0..leaves).map(|i| format!("a{i}")).collect(),
            root: PowlNode::Sequence(
                (0..leaves).map(|i| PowlNode::Activity(format!("a{i}"))).collect(),
            ),
        }
    }

    /// A model whose root is an empty sequence.
    pub fn empty_sequence_model() -> PowlModel {
        PowlModel {
            activities: BTreeMap::new(),
            declaration_order: Vec::new(),
            root: PowlNode::Sequence(Vec::new()),
        }
    }

    /// A model whose root is an empty xor.
    pub fn empty_xor_model() -> PowlModel {
        PowlModel {
            activities: BTreeMap::new(),
            declaration_order: Vec::new(),
            root: PowlNode::Xor(Vec::new()),
        }
    }

    /// A model whose root is an empty parallel block.
    pub fn empty_parallel_model() -> PowlModel {
        PowlModel {
            activities: BTreeMap::new(),
            declaration_order: Vec::new(),
            root: PowlNode::Parallel(Vec::new()),
        }
    }

    /// A model whose root is a bounded loop (`max = 3`) over one activity.
    pub fn bounded_loop_model() -> PowlModel {
        PowlModel {
            activities: BTreeMap::from([(
                "a0".to_string(),
                ActivityDecl { source: "s".to_string(), event: "e0".to_string() },
            )]),
            declaration_order: vec!["a0".to_string()],
            root: PowlNode::Loop {
                body: Box::new(PowlNode::Activity("a0".to_string())),
                redo: Box::new(PowlNode::Silent),
                max: 3,
            },
        }
    }

    /// A net whose arc targets a transition that was never declared.
    pub fn net_arc_to_undeclared_target() -> PetriNet {
        PetriNet {
            places: vec!["p0".to_string()],
            transitions: vec![Transition {
                name: "t0".to_string(),
                label: Some("act".to_string()),
            }],
            arcs: vec![Arc {
                from: NodeRef::Place(PlaceId(0)),
                to: NodeRef::Transition(TransitionId(7)),
                weight: 1,
            }],
            initial_marking: BTreeSet::from([PlaceId(0)]),
        }
    }

    /// A net carrying a non-unit arc weight (invalid per `try_new`).
    pub fn net_with_arc_weight(weight: u16) -> PetriNet {
        PetriNet {
            places: vec!["p0".to_string(), "p1".to_string()],
            transitions: vec![Transition {
                name: "t0".to_string(),
                label: Some("act".to_string()),
            }],
            arcs: vec![Arc {
                from: NodeRef::Place(PlaceId(0)),
                to: NodeRef::Transition(TransitionId(0)),
                weight,
            }],
            initial_marking: BTreeSet::from([PlaceId(0)]),
        }
    }

    /// A net with `n` declared places (caller chooses, including over
    /// [`MAX_PETRI_PLACES`] to exercise the budget rejection).
    pub fn net_with_places(n: u16) -> PetriNet {
        let places: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        PetriNet {
            places,
            transitions: Vec::new(),
            arcs: Vec::new(),
            initial_marking: BTreeSet::new(),
        }
    }

    /// A net whose initial marking references a place id outside the
    /// declared place vector.
    pub fn net_initial_marking_unknown_place() -> PetriNet {
        PetriNet {
            places: vec!["p0".to_string()],
            transitions: Vec::new(),
            arcs: Vec::new(),
            initial_marking: BTreeSet::from([PlaceId(7)]),
        }
    }

    /// A minimal valid net: `p0 -> t0 -> p1`, `t0` labeled, one initial
    /// token on `p0`.
    pub fn well_formed_net() -> PetriNet {
        PetriNet {
            places: vec!["p0".to_string(), "p1".to_string()],
            transitions: vec![Transition {
                name: "t0".to_string(),
                label: Some("act".to_string()),
            }],
            arcs: vec![
                Arc {
                    from: NodeRef::Place(PlaceId(0)),
                    to: NodeRef::Transition(TransitionId(0)),
                    weight: 1,
                },
                Arc {
                    from: NodeRef::Transition(TransitionId(0)),
                    to: NodeRef::Place(PlaceId(1)),
                    weight: 1,
                },
            ],
            initial_marking: BTreeSet::from([PlaceId(0)]),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::test_fixtures;
    use super::*;

    #[test]
    fn net_rejects_non_unit_arc_weight() {
        let net = PetriNet::try_new(
            vec!["p0".to_string(), "p1".to_string()],
            vec![Transition { name: "t0".to_string(), label: Some("act".to_string()) }],
            vec![Arc {
                from: NodeRef::Place(PlaceId(0)),
                to: NodeRef::Transition(TransitionId(0)),
                weight: 2,
            }],
            BTreeSet::from([PlaceId(0)]),
        );
        assert!(matches!(
            net.expect_err("non-unit weight must be rejected"),
            ConformanceError::WeightedArc { .. }
        ));
    }

    #[test]
    fn net_rejects_more_than_max_places() {
        let places: Vec<String> = (0..=MAX_PETRI_PLACES).map(|i| format!("p{i}")).collect();
        let net = PetriNet::try_new(places, Vec::new(), Vec::new(), BTreeSet::new());
        assert!(matches!(
            net.expect_err("place overflow must be rejected"),
            ConformanceError::TooManyPlaces { .. }
        ));
    }

    /// At-limit admission (T4): exactly [`MAX_PETRI_PLACES`] (16) places
    /// construct Ok — the cap is inclusive.
    #[test]
    fn net_admits_exactly_max_places() {
        let places: Vec<String> = (0..MAX_PETRI_PLACES).map(|i| format!("p{i}")).collect();
        let net = PetriNet::try_new(places, Vec::new(), Vec::new(), BTreeSet::new())
            .expect("16 places == MAX_PETRI_PLACES must be admitted");
        assert_eq!(net.places().len(), usize::from(MAX_PETRI_PLACES));
    }

    #[test]
    fn net_admits_a_valid_minimal_net() {
        let net = PetriNet::try_new(
            vec!["source".to_string(), "sink".to_string()],
            vec![Transition { name: "t0".to_string(), label: None }],
            vec![
                Arc {
                    from: NodeRef::Place(PlaceId(0)),
                    to: NodeRef::Transition(TransitionId(0)),
                    weight: 1,
                },
                Arc {
                    from: NodeRef::Transition(TransitionId(0)),
                    to: NodeRef::Place(PlaceId(1)),
                    weight: 1,
                },
            ],
            BTreeSet::from([PlaceId(0)]),
        )
        .unwrap_or_else(|e| panic!("valid net must build: {e}"));
        assert_eq!(net.places().len(), 2);
        assert_eq!(net.transitions().len(), 1);
        assert_eq!(net.arcs().len(), 2);
        assert_eq!(net.initial_marking().len(), 1);
    }

    #[test]
    fn fixture_well_formed_net_matches_constructor_output() {
        let net = test_fixtures::well_formed_net();
        assert_eq!(net.places().len(), 2);
        assert_eq!(net.transitions().len(), 1);
        assert_eq!(net.arcs().len(), 2);
        assert_eq!(net.initial_marking().len(), 1);
    }
}
