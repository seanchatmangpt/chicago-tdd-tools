//! Bridge from the local [`PowlModel`](super::model::PowlModel) surface to the
//! BCINR POWL2 conformance validator.
//!
//! **Bridge target**: `bcinr-powl` 26.10.11 — `validate_powl2` /
//! `bcinr_powl::powl2::Powl2Model`, with `POWL2_MAX_NODES = 64` (matching this
//! crate's `MAX_POWL_NODES`).
//!
//! **Dependency wiring** (owned by the coordinator, not this lane): when the
//! `powl-bcinr` feature lands in `Cargo.toml`, it must read
//!
//! ```toml
//! bcinr-powl = { version = "26.10.11", optional = true, default-features = false }
//! [features]
//! powl-bcinr = ["dep:bcinr-powl"]
//! ```
//!
//! Until then this entire module is compiled out under default features; no
//! stub bodies are compiled and no dependency is required.
//!
//! **Node mapping table** (`PowlNode` -> `bcinr_powl::powl2`):
//!
//! | `conformance::model::PowlNode`     | `bcinr_powl::powl2` target        | Notes                                   |
//! |------------------------------------|-----------------------------------|-----------------------------------------|
//! | `Step(StepCoord)`                  | `Activity`                        | Leaf activity occurrence.                |
//! | `Silent`                           | `Silent`                          | Tau transition, both directions.         |
//! | `Sequence(children)`               | `Sequence`                        | Strict total order preserved.            |
//! | `Xor(children)`                    | `ChoiceGraph`                     | Exclusive choice; arity <= 64 by `MAX_POWL_NODES`. |
//! | `Parallel(children)`               | `PartialOrder`                    | Interleaving becomes a partial order.    |
//! | `Loop { body, redo, max_iters }`   | `DoRedo`                          | `max_iters` carried as the iteration bound; `max_iters == 0` is rejected (`UnboundedLoop`). |
//!
//! Both directions of the bound check are enforced: a [`PowlModel`] can never
//! exceed `MAX_POWL_NODES = 64` nodes (its constructor rejects it), and the
//! converted `Powl2Model` is validated against `bcinr_powl::powl2::POWL2_MAX_NODES`
//! (also 64) before being handed to `validate_powl2`.

// Everything below is compiled only when the `bcinr-powl` optional
// dependency is actually wired into Cargo.toml. Declaring the `powl-bcinr`
// feature alone is not sufficient: the module body references
// `bcinr_powl::powl2::*`, which does not exist until the dependency lands
// (coordinator work). `cfg(any())` keeps this file compiling empty — the
// documented pre-wiring state — instead of breaking `--all-features` builds.
#[cfg(any())]
#![allow(clippy::unwrap_used)] // JUSTIFICATION: cfg(any()) bridge; unwrap is inside cfg-gated code
pub use powl_bcinr_impl::*;

#[cfg(any())]
mod powl_bcinr_impl {
    use super::super::error::ConformanceError;
    use super::super::loader::{PowlModel, PowlNode};

    /// Converts a validated local POWL model into the BCINR POWL2
    /// representation expected by `bcinr_powl::powl2::validate_powl2`.
    ///
    /// See the [module mapping table](super) for the per-node translation.
    ///
    /// # Errors
    ///
    /// Returns [`ConformanceError::ModelTooLarge`] if the converted tree would
    /// exceed `bcinr_powl::powl2::POWL2_MAX_NODES`, and
    /// [`ConformanceError::UnboundedLoop`] if any `Loop` node carries a zero
    /// iteration bound (BCINR's `DoRedo` requires a positive bound).
    pub fn to_bcinr_powl2(
        model: &PowlModel,
    ) -> Result<bcinr_powl::powl2::Powl2Model, ConformanceError> {
        let converted = convert_node(model.root(), model)?;
        Ok(bcinr_powl::powl2::Powl2Model::new(converted))
    }

    /// Validates `model` through the BCINR POWL2 validator (`validate_powl2`).
    ///
    /// This is the admission hop: a model conformant here satisfies the
    /// external BCINR court, not merely the local constructor invariants.
    ///
    /// # Errors
    ///
    /// Returns [`ConformanceError::BcinrValidation`] with the validator's
    /// diagnostic text when `validate_powl2` rejects the model, plus the same
    /// conversion errors as [`to_bcinr_powl2`].
    pub fn validate_bcinr(model: &PowlModel) -> Result<(), ConformanceError> {
        let powl2 = to_bcinr_powl2(model)?;
        bcinr_powl::powl2::validate_powl2(&powl2)
            .map_err(|err| ConformanceError::BcinrValidation { message: err.to_string() })
    }

    /// Recursive per-node translation, bounded by the model's own node count
    /// (`<= MAX_POWL_NODES = 64`), so recursion depth is fixed.
    fn convert_node(
        node: &PowlNode,
        model: &PowlModel,
    ) -> Result<bcinr_powl::powl2::Powl2Node, ConformanceError> {
        if model.node_count() > bcinr_powl::powl2::POWL2_MAX_NODES {
            return Err(ConformanceError::ModelTooLarge {
                found: model.node_count(),
                limit: bcinr_powl::powl2::POWL2_MAX_NODES,
            });
        }
        match node {
            PowlNode::Step(coord) => {
                let name = activity_name_for(coord, model);
                Ok(bcinr_powl::powl2::Powl2Node::Activity(name))
            }
            PowlNode::Silent => Ok(bcinr_powl::powl2::Powl2Node::Silent),
            PowlNode::Sequence(children) => {
                let mapped = map_children(children, model)?;
                Ok(bcinr_powl::powl2::Powl2Node::Sequence(mapped))
            }
            PowlNode::Xor(children) => {
                let mapped = map_children(children, model)?;
                Ok(bcinr_powl::powl2::Powl2Node::ChoiceGraph(mapped))
            }
            PowlNode::Parallel(children) => {
                let mapped = map_children(children, model)?;
                Ok(bcinr_powl::powl2::Powl2Node::PartialOrder(mapped))
            }
            PowlNode::Loop { body, redo, max_iters } => {
                if *max_iters == 0 {
                    return Err(ConformanceError::UnboundedLoop);
                }
                let body = convert_node(body, model)?;
                let redo = convert_node(redo, model)?;
                Ok(bcinr_powl::powl2::Powl2Node::DoRedo {
                    body: Box::new(body),
                    redo: Box::new(redo),
                    max_iters: *max_iters,
                })
            }
        }
    }

    fn map_children(
        children: &[PowlNode],
        model: &PowlModel,
    ) -> Result<Vec<bcinr_powl::powl2::Powl2Node>, ConformanceError> {
        children.iter().map(|child| convert_node(child, model)).collect()
    }

    /// Resolves a [`StepCoord`] to its declared activity name via the model's
    /// activity table; falls back to the coordinate itself for undeclared
    /// leaves so the bridge is total over the local type.
    fn activity_name_for(coord: &super::super::model::StepCoord, model: &PowlModel) -> String {
        model
            .activities()
            .iter()
            .find(|(_, first)| *first == coord)
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| format!("step@{coord:?}"))
    }
}
