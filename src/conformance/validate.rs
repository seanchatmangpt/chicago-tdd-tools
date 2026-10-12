//! Layered fail-closed validation for POWL models and Petri nets.
//!
//! Validation is layered and ordered cheapest-first so a rejected model
//! fails at the least expensive layer that observes its defect:
//!
//! 1. **Size** — `node_count() <= MAX_MODEL_NODES` (`ModelTooLarge`).
//! 2. **Structural** — Sequence/Xor/Parallel arity `>= 1`
//!    (`ArityBelowMinimum`); loop termination bounds are a *type-level*
//!    guarantee of [`super::loader::PowlNode`] (`Loop.max` is `u8`); activity
//!    references are resolved here at validate time against the declared
//!    `activities` map, so the JSON path (`from_powl_json`) — which does not
//!    resolve them at load — is refused with
//!    [`ConformanceError::UnknownActivity`] (W27 work order, landed).
//!
//! The layering is fail-closed: any layer returning `Err` short-circuits.
//!
//! Block-structured constructors (Sequence/Xor/Parallel/Loop) cannot express
//! a cycle, so no runtime acyclicity layer exists: block structure is
//! acyclic by construction.
//!
//! See Also: `crate::conformance::error::ConformanceError`

use std::collections::BTreeMap;

use super::loader::{ActivityDecl, PowlModel, PowlNode, MAX_POWL_NODES};
use super::model::{PetriNet, MAX_PETRI_PLACES};
use crate::conformance::error::ConformanceError;

/// Maximum number of elements a [`PowlModel`] may contain.
///
/// Alias of [`super::loader::MAX_POWL_NODES`], kept so existing callers of
/// the validator surface stay stable.
pub const MAX_MODEL_NODES: usize = MAX_POWL_NODES;

/// Maximum number of places a [`PetriNet`] may declare (u16 marking encoding).
///
/// Alias of [`super::model::MAX_PETRI_PLACES`].
pub const MAX_NET_PLACES: usize = MAX_PETRI_PLACES as usize;

/// Validates a POWL model, cheapest layer first.
///
/// Returns `Ok(())` or the first [`ConformanceError`] observed.
///
/// # Errors
///
/// [`ConformanceError::ModelTooLarge`] when the node budget is exceeded,
/// [`ConformanceError::ArityBelowMinimum`] when a block constructor has no
/// children, and [`ConformanceError::UnknownActivity`] when an
/// [`PowlNode::Activity`] leaf names an activity absent from the declared
/// `activities` map (each layer short-circuits — see the module docs).
pub fn validate(model: &PowlModel) -> Result<(), ConformanceError> {
    // Layer 1 — size.
    let found = model.node_count();
    if found > MAX_MODEL_NODES {
        return Err(ConformanceError::ModelTooLarge { found, limit: MAX_MODEL_NODES });
    }

    // Layer 2 — structural (constructor arity; loop bounds are a type-level
    // guarantee of `PowlNode::Loop.max`) + activity-reference resolution
    // against the declared `activities` map (the JSON loader does not resolve
    // these — see module docs).
    walk(&model.root, &model.activities)?;

    Ok(())
}

/// Recursive depth-first structural walk over the [`PowlNode`] tree.
///
/// Enforces minimum arity 1 on every block constructor and resolves every
/// [`PowlNode::Activity`] name against `activities`; loop bounds are a
/// type-level guarantee (`Loop.max` is `u8`). Positional fields of the
/// [`ConformanceError::UnknownActivity`] diagnostic are `0, 0`: the serde
/// JSON path carries no source offsets.
fn walk(
    node: &PowlNode,
    activities: &BTreeMap<String, ActivityDecl>,
) -> Result<(), ConformanceError> {
    match node {
        PowlNode::Sequence(children) | PowlNode::Xor(children) | PowlNode::Parallel(children) => {
            if children.is_empty() {
                return Err(ConformanceError::ArityBelowMinimum {
                    constructor: constructor_name(node),
                    minimum: 1,
                    found: 0,
                });
            }
            for child in children {
                walk(child, activities)?;
            }
            Ok(())
        }
        PowlNode::Loop { body, redo, .. } => {
            walk(body, activities)?;
            walk(redo, activities)
        }
        PowlNode::Activity(name) => {
            if activities.contains_key(name) {
                Ok(())
            } else {
                Err(ConformanceError::UnknownActivity { name: name.clone(), line: 0, col: 0 })
            }
        }
        PowlNode::Silent => Ok(()),
    }
}

/// Validates a Petri net: declared endpoints, unit arc weights, place
/// budget, and initial-marking subset of declared places.
///
/// # Errors
///
/// [`ConformanceError::TooManyPlaces`] over the place budget,
/// [`ConformanceError::UnknownNodeRef`] for undeclared arc endpoints or
/// initial-marking places, and [`ConformanceError::WeightedArc`] for
/// non-unit arc weights.
pub fn validate_petri(net: &PetriNet) -> Result<(), ConformanceError> {
    // Place budget.
    if net.places.len() > MAX_NET_PLACES {
        return Err(ConformanceError::TooManyPlaces { limit: MAX_NET_PLACES });
    }

    for arc in &net.arcs {
        // Declared endpoints.
        for (side, id) in [("source", arc.from), ("target", arc.to)] {
            if !node_ref_declared(net, id) {
                return Err(ConformanceError::UnknownNodeRef { side, id: node_ref_id(id) });
            }
        }
        // Unit weights only (this validator admits weight 1 only).
        if arc.weight != 1 {
            return Err(ConformanceError::WeightedArc { weight: arc.weight });
        }
    }

    // Initial marking must be a subset of declared places.
    for place in &net.initial_marking {
        if usize::from(place.0) >= net.places.len() {
            return Err(ConformanceError::UnknownNodeRef {
                side: "initial-marking",
                id: format!("place {}", place.0),
            });
        }
    }

    Ok(())
}

/// Whether `id` names a declared place or transition of `net`.
fn node_ref_declared(net: &PetriNet, id: super::model::NodeRef) -> bool {
    match id {
        super::model::NodeRef::Place(place) => usize::from(place.0) < net.places.len(),
        super::model::NodeRef::Transition(transition) => {
            usize::from(transition.0) < net.transitions.len()
        }
    }
}

/// Diagnostic id string for an unresolvable [`super::model::NodeRef`].
fn node_ref_id(id: super::model::NodeRef) -> String {
    match id {
        super::model::NodeRef::Place(place) => format!("place {}", place.0),
        super::model::NodeRef::Transition(transition) => format!("transition {}", transition.0),
    }
}

/// Bridges to the `powl-bcinr` optional dependency.
///
/// Delegates to `bcinr_powl::validate_powl2` once the optional dependency
/// is wired into `Cargo.toml`; until then the bridge admits any model that
/// passed [`validate`], so callers behind the feature flag have a stable
/// admission surface today and a strictly stronger one after wiring.
#[cfg(feature = "powl-bcinr")]
// JUSTIFICATION: bcinr bridge stub; the Result-returning signature is locked
// to the future `bcinr_powl::validate_powl2` delegation, so clippy's
// const-fn and unnecessary-wraps suggestions do not apply yet.
#[allow(clippy::missing_const_for_fn, clippy::unnecessary_wraps)]
/// Validates a model through the (currently stubbed) bcinr bridge.
///
/// # Errors
///
/// Always returns `Ok(())` in the stub; returns the typed
/// [`ConformanceError`] once the bcinr-powl delegation lands and rejects.
pub fn validate_bcinr(model: &PowlModel) -> Result<(), ConformanceError> {
    let _ = model;
    Ok(())
}

/// Static constructor label for the arity diagnostic.
const fn constructor_name(node: &PowlNode) -> &'static str {
    match node {
        PowlNode::Sequence(_) => "sequence",
        PowlNode::Xor(_) => "xor",
        PowlNode::Parallel(_) => "parallel",
        PowlNode::Loop { .. } => "loop",
        PowlNode::Activity(_) | PowlNode::Silent => "activity",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::model::test_fixtures;

    // -- Layer 1: size ---------------------------------------------------

    #[test]
    fn model_too_large_when_node_count_exceeds_limit() {
        let model = test_fixtures::model_with_nodes(MAX_MODEL_NODES + 1);
        assert_eq!(
            validate(&model),
            Err(ConformanceError::ModelTooLarge {
                found: MAX_MODEL_NODES + 1,
                limit: MAX_MODEL_NODES,
            })
        );
    }

    #[test]
    fn model_at_exact_limit_is_admitted() {
        let model = test_fixtures::model_with_nodes(MAX_MODEL_NODES);
        assert_eq!(validate(&model), Ok(()));
    }

    // -- Layer 2: structural arity ----------------------------------------

    #[test]
    fn empty_sequence_is_rejected() {
        let model = test_fixtures::empty_sequence_model();
        assert_eq!(
            validate(&model),
            Err(ConformanceError::ArityBelowMinimum {
                constructor: "sequence",
                minimum: 1,
                found: 0,
            })
        );
    }

    #[test]
    fn empty_xor_is_rejected() {
        let model = test_fixtures::empty_xor_model();
        assert_eq!(
            validate(&model),
            Err(ConformanceError::ArityBelowMinimum { constructor: "xor", minimum: 1, found: 0 })
        );
    }

    #[test]
    fn empty_parallel_is_rejected() {
        let model = test_fixtures::empty_parallel_model();
        assert_eq!(
            validate(&model),
            Err(ConformanceError::ArityBelowMinimum {
                constructor: "parallel",
                minimum: 1,
                found: 0,
            })
        );
    }

    // Loop termination is a type-level guarantee (`Loop.max` is `u8`);
    // activity references are resolved at validate time (module docs).
    // A bounded loop over declared activities admits.
    #[test]
    fn bounded_loop_and_activities_admit() {
        let model = test_fixtures::bounded_loop_model();
        assert_eq!(validate(&model), Ok(()));
    }

    // -- validate_petri ----------------------------------------------------

    #[test]
    fn net_with_undeclared_arc_endpoint_is_rejected() {
        let net = test_fixtures::net_arc_to_undeclared_target();
        let err = validate_petri(&net).expect_err("undeclared target must be rejected");
        assert!(
            matches!(err, ConformanceError::UnknownNodeRef { side: "target", .. }),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn net_with_non_unit_arc_weight_is_rejected() {
        let net = test_fixtures::net_with_arc_weight(2);
        assert_eq!(validate_petri(&net), Err(ConformanceError::WeightedArc { weight: 2 }));
    }

    #[test]
    fn net_exceeding_place_budget_is_rejected() {
        let net = test_fixtures::net_with_places((MAX_NET_PLACES + 1) as u16);
        assert_eq!(
            validate_petri(&net),
            Err(ConformanceError::TooManyPlaces { limit: MAX_NET_PLACES })
        );
    }

    #[test]
    fn net_at_exact_place_budget_is_admitted() {
        let net = test_fixtures::net_with_places(MAX_NET_PLACES as u16);
        assert_eq!(validate_petri(&net), Ok(()));
    }

    #[test]
    fn net_with_initial_marking_on_undeclared_place_is_rejected() {
        let net = test_fixtures::net_initial_marking_unknown_place();
        let err = validate_petri(&net).expect_err("unknown initial-marking place must be rejected");
        assert!(
            matches!(err, ConformanceError::UnknownNodeRef { side: "initial-marking", .. }),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn well_formed_net_admits() {
        let net = test_fixtures::well_formed_net();
        assert_eq!(validate_petri(&net), Ok(()));
    }

    // -- bcinr bridge (feature-gated) --------------------------------------

    #[cfg(feature = "powl-bcinr")]
    #[test]
    fn bcinr_bridge_admits_validated_model() {
        let model = test_fixtures::bounded_loop_model();
        assert_eq!(validate_bcinr(&model), Ok(()));
    }
}
