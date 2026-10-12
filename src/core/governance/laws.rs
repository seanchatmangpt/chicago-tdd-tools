//! Backing data types for agent governance (W27 probe: no macros exist in this file — plain serde types; the module's only macro, `source_location!`, lives in mod.rs).

#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionMetadata {
    pub id: String,
    pub credentials: String,
    pub crown_receipt: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ContributionKind {
    SchemaConstraint = 1,
    Falsifier = 2,
    CognitiveStrategy = 3,
    ProcessPattern = 4,
    Receipt = 5,
    BoundaryRule = 6,
    ResidualClass = 7,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubstrateDelta {
    pub contributions: Vec<(ContributionKind, String)>,
}

impl SubstrateDelta {
    pub fn is_evolutionary(&self) -> bool {
        !self.contributions.is_empty()
    }
}
