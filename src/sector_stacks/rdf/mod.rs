//! RDF Ontology Integration for Sector Stacks
//!
//! This module provides RDF (Resource Description Framework) integration for loading,
//! querying, and validating sector-specific ontologies. It closes the loop between
//! RDF semantic definitions and Rust runtime implementations.
//!
//! ## Architecture
//!
//! The RDF module enables:
//! - **Ontology Loading**: Parse TTL files into an in-memory RDF store
//! - **SPARQL Querying**: Query workflow definitions, guards, and constraints
//! - **RDF-Driven Validation**: Validate operations against ontology definitions
//! - **Bidirectional Sync**: Map between RDF and Rust type systems

pub mod ontology;
pub mod validation;

#[cfg(feature = "rdf")]
pub use ontology::{GuardConstraint, KnowledgeHook, OntologyLoader, SectorOntology, WorkflowStage};
#[cfg(feature = "rdf")]
pub use validation::{RdfOperationValidator, RdfValidationError, RdfValidationResult};

/// RDF feature stub for builds without the rdf feature
#[cfg(not(feature = "rdf"))]
pub struct OntologyLoader;

#[cfg(not(feature = "rdf"))]
impl OntologyLoader {
    /// RDF module requires 'rdf' feature to be enabled
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rdf_module_available() {
        // Verify RDF module can be loaded
        #[cfg(feature = "rdf")]
        {
            use super::OntologyLoader;
            let _loader = OntologyLoader::new();
        }
    }
}
