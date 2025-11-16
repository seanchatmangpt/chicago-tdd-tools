//! RDF Ontology Loader and Query Interface
//!
//! Provides RDF ontology loading from TTL files and SPARQL querying for
//! workflow definitions, guards, and semantic constraints.

use std::collections::HashMap;
use std::path::Path;

/// Represents a workflow stage in the ontology
#[derive(Debug, Clone)]
pub struct WorkflowStage {
    /// Stage identifier (URI)
    pub id: String,
    /// Stage name
    pub name: String,
    /// Stage number in workflow
    pub stage_number: u32,
    /// Whether stage is deterministic
    pub is_deterministic: bool,
    /// Maximum latency in seconds
    pub max_latency_seconds: u32,
}

/// Represents a guard constraint
#[derive(Debug, Clone)]
pub struct GuardConstraint {
    /// Guard identifier
    pub id: String,
    /// Guard type (Legality, Budget, Chronology, Causality, Recursion)
    pub guard_type: String,
    /// List of constraints
    pub constraints: Vec<String>,
}

/// Represents a knowledge hook
#[derive(Debug, Clone)]
pub struct KnowledgeHook {
    /// Hook identifier
    pub id: String,
    /// Hook name
    pub name: String,
    /// Hook description
    pub description: String,
    /// Input type
    pub input_type: String,
    /// Output type
    pub output_type: String,
}

/// Sector-specific ontology loaded from RDF
#[derive(Debug, Clone)]
pub struct SectorOntology {
    /// Sector name (Academic, Claims, etc.)
    pub sector: String,
    /// Workflow stages
    pub stages: HashMap<String, WorkflowStage>,
    /// Guard constraints
    pub guards: HashMap<String, GuardConstraint>,
    /// Knowledge hooks
    pub hooks: HashMap<String, KnowledgeHook>,
    /// Raw RDF triples (for advanced querying)
    pub triples: Vec<(String, String, String)>,
}

impl SectorOntology {
    /// Create a new empty ontology
    pub fn new(sector: String) -> Self {
        Self {
            sector,
            stages: HashMap::new(),
            guards: HashMap::new(),
            hooks: HashMap::new(),
            triples: Vec::new(),
        }
    }

    /// Add a workflow stage
    pub fn add_stage(&mut self, stage: WorkflowStage) {
        self.stages.insert(stage.id.clone(), stage);
    }

    /// Add a guard constraint
    pub fn add_guard(&mut self, guard: GuardConstraint) {
        self.guards.insert(guard.id.clone(), guard);
    }

    /// Add a knowledge hook
    pub fn add_hook(&mut self, hook: KnowledgeHook) {
        self.hooks.insert(hook.id.clone(), hook);
    }

    /// Get stage by ID
    pub fn get_stage(&self, id: &str) -> Option<&WorkflowStage> {
        self.stages.get(id)
    }

    /// Get all deterministic stages
    pub fn deterministic_stages(&self) -> Vec<&WorkflowStage> {
        self.stages.values().filter(|s| s.is_deterministic).collect()
    }

    /// Count total stages
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Count total guards
    pub fn guard_count(&self) -> usize {
        self.guards.len()
    }

    /// Count total hooks
    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }
}

/// Loads and parses RDF ontologies
pub struct OntologyLoader {
    #[cfg(feature = "rdf")]
    store: Option<oxigraph::store::Store>,
}

impl OntologyLoader {
    /// Create a new ontology loader
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rdf")]
            store: None,
        }
    }

    /// Load ontology from a TTL file
    #[cfg(feature = "rdf")]
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<SectorOntology, String> {
        use std::fs;
        use oxigraph::store::Store;
        use oxigraph::io::RdfFormat;

        // Read file
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Create store
        let store = Store::new().map_err(|e| format!("Failed to create RDF store: {}", e))?;

        // Load RDF
        store
            .load_read(content.as_bytes(), RdfFormat::Turtle, None)
            .map_err(|e| format!("Failed to load RDF: {}", e))?;

        self.store = Some(store);

        // Extract ontology information
        self.extract_ontology()
    }

    /// Extract ontology from store
    #[cfg(feature = "rdf")]
    fn extract_ontology(&self) -> Result<SectorOntology, String> {
        use oxigraph::sparql::QueryResults;

        let store = self.store.as_ref().ok_or("Store not initialized")?;

        // Determine sector from RDF
        let sector_query = r#"
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            SELECT ?sector WHERE {
                ?x rdf:type ?sector .
                FILTER(CONTAINS(STR(?sector), "Stack"))
            }
            LIMIT 1
        "#;

        let mut sector_name = "Unknown".to_string();

        if let Ok(QueryResults::Solutions(solutions)) =
            store.query(sector_query).map_err(|e| format!("Query failed: {}", e))
        {
            for solution in solutions {
                if let Ok(solution) = solution {
                    if let Some(sector_var) = solution.get("sector") {
                        sector_name = sector_var.to_string();
                    }
                }
            }
        }

        let mut ontology = SectorOntology::new(sector_name);

        // Extract stages
        let stages_query = r#"
            PREFIX ac: <http://chatman-equation.org/academic/>
            PREFIX cp: <http://chatman-equation.org/claims/>
            SELECT ?id ?name ?stage_num WHERE {
                {
                    ?id ac:stageName ?name .
                    ?id ac:stageNumber ?stage_num .
                } UNION {
                    ?id cp:stageName ?name .
                    ?id cp:stageNumber ?stage_num .
                }
            }
        "#;

        if let Ok(QueryResults::Solutions(solutions)) =
            store.query(stages_query).map_err(|e| format!("Query failed: {}", e))
        {
            for solution in solutions {
                if let Ok(solution) = solution {
                    if let (Some(id), Some(name)) = (solution.get("id"), solution.get("name")) {
                        let stage_num = solution.get("stage_num").and_then(|v| {
                            let s = v.to_string();
                            s.parse::<u32>().ok()
                        });

                        let stage = WorkflowStage {
                            id: id.to_string(),
                            name: name.to_string(),
                            stage_number: stage_num.unwrap_or(0),
                            is_deterministic: true,
                            max_latency_seconds: 60,
                        };

                        ontology.add_stage(stage);
                    }
                }
            }
        }

        Ok(ontology)
    }

    /// Load ontology (stub when rdf feature disabled)
    #[cfg(not(feature = "rdf"))]
    pub fn load_from_file<P: AsRef<Path>>(&mut self, _path: P) -> Result<SectorOntology, String> {
        Err("RDF feature not enabled. Use --features rdf to enable ontology loading".to_string())
    }

    /// Load ontology from TTL string
    #[cfg(feature = "rdf")]
    pub fn load_from_ttl(&mut self, ttl_content: &str) -> Result<SectorOntology, String> {
        use oxigraph::store::Store;
        use oxigraph::io::RdfFormat;

        // Create store
        let store = Store::new().map_err(|e| format!("Failed to create RDF store: {}", e))?;

        // Load RDF
        store
            .load_read(ttl_content.as_bytes(), RdfFormat::Turtle, None)
            .map_err(|e| format!("Failed to load RDF: {}", e))?;

        self.store = Some(store);

        // Extract ontology
        self.extract_ontology()
    }

    /// Load ontology from TTL string (stub)
    #[cfg(not(feature = "rdf"))]
    pub fn load_from_ttl(&mut self, _content: &str) -> Result<SectorOntology, String> {
        Err("RDF feature not enabled. Use --features rdf to enable ontology loading".to_string())
    }

    /// Query ontology using SPARQL
    #[cfg(feature = "rdf")]
    pub fn query(&self, sparql: &str) -> Result<Vec<Vec<(String, String)>>, String> {
        use oxigraph::sparql::QueryResults;

        let store = self.store.as_ref().ok_or("Store not initialized")?;

        let mut results = Vec::new();

        if let Ok(QueryResults::Solutions(solutions)) =
            store.query(sparql).map_err(|e| format!("Query failed: {}", e))
        {
            for solution in solutions {
                if let Ok(solution) = solution {
                    let mut row = Vec::new();
                    for (var, val) in solution.iter() {
                        row.push((var.to_string(), val.to_string()));
                    }
                    results.push(row);
                }
            }
        }

        Ok(results)
    }

    /// Query ontology (stub)
    #[cfg(not(feature = "rdf"))]
    pub fn query(&self, _sparql: &str) -> Result<Vec<Vec<(String, String)>>, String> {
        Err("RDF feature not enabled".to_string())
    }
}

impl Default for OntologyLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_creation() {
        let ontology = SectorOntology::new("Academic".to_string());
        assert_eq!(ontology.sector, "Academic");
        assert_eq!(ontology.stage_count(), 0);
    }

    #[test]
    fn test_add_stage() {
        let mut ontology = SectorOntology::new("Claims".to_string());

        let stage = WorkflowStage {
            id: "validation".to_string(),
            name: "Validation".to_string(),
            stage_number: 1,
            is_deterministic: true,
            max_latency_seconds: 30,
        };

        ontology.add_stage(stage.clone());
        assert_eq!(ontology.stage_count(), 1);
        assert_eq!(ontology.get_stage("validation").unwrap().name, "Validation");
    }

    #[test]
    fn test_deterministic_stages() {
        let mut ontology = SectorOntology::new("Academic".to_string());

        ontology.add_stage(WorkflowStage {
            id: "stage1".to_string(),
            name: "Stage 1".to_string(),
            stage_number: 1,
            is_deterministic: true,
            max_latency_seconds: 60,
        });

        ontology.add_stage(WorkflowStage {
            id: "stage2".to_string(),
            name: "Stage 2".to_string(),
            stage_number: 2,
            is_deterministic: false,
            max_latency_seconds: 120,
        });

        let deterministic = ontology.deterministic_stages();
        assert_eq!(deterministic.len(), 1);
    }

    #[test]
    fn test_add_guard() {
        let mut ontology = SectorOntology::new("Claims".to_string());

        let guard = GuardConstraint {
            id: "budget".to_string(),
            guard_type: "Budget".to_string(),
            constraints: vec!["settlement <= policy_limit".to_string()],
        };

        ontology.add_guard(guard);
        assert_eq!(ontology.guard_count(), 1);
    }

    #[test]
    fn test_loader_creation() {
        let loader = OntologyLoader::new();
        assert!(loader.query("SELECT * WHERE { ?s ?p ?o }").is_err());
    }
}
