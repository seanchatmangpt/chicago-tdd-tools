//! Manufactured combinatorial-maximalism architecture kernel.
mod evidence;
mod external_contracts;
mod facets;
mod kernel;
mod profiles;
mod projection_axes;
mod realizations;
mod refusals;
mod types;

pub use evidence::EVIDENCE_OBLIGATIONS;
pub use external_contracts::EXTERNAL_CONTRACTS;
pub use facets::FACETS;
pub use kernel::{admit_consequence, authorize, broker_intent, compose, replay_matches, required_evidence_bits, verify};
pub use profiles::PROFILES;
pub use projection_axes::PROJECTION_AXES;
pub use realizations::REALIZATIONS;
pub use refusals::REFUSALS;
pub use types::*;
