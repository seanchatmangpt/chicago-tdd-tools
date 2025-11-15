//! Feature metadata for searchability and discovery
//!
//! This module provides comprehensive metadata for all playground features,
//! enabling intelligent search, discovery, and adaptive help systems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill level for progressive disclosure
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SkillLevel {
    Beginner = 0,
    Intermediate = 1,
    Advanced = 2,
}

/// Feature category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Core,
    Testing,
    Validation,
    Observability,
    Integration,
    Analysis,
    Improvement,
    Process,
    Quality,
    Release,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Core => write!(f, "Core"),
            Category::Testing => write!(f, "Testing"),
            Category::Validation => write!(f, "Validation"),
            Category::Observability => write!(f, "Observability"),
            Category::Integration => write!(f, "Integration"),
            Category::Analysis => write!(f, "Analysis"),
            Category::Improvement => write!(f, "Improvement"),
            Category::Process => write!(f, "Process"),
            Category::Quality => write!(f, "Quality"),
            Category::Release => write!(f, "Release"),
        }
    }
}

/// Comprehensive metadata for a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMetadata {
    /// Unique identifier (e.g., "fixtures", "property-testing")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Brief one-line description
    pub brief: String,

    /// Detailed description
    pub detailed: String,

    /// Category for organization
    pub category: Category,

    /// Skill level required
    pub skill_level: SkillLevel,

    /// Keywords for search (lowercase for matching)
    pub keywords: Vec<String>,

    /// Related feature IDs for discovery
    pub related: Vec<String>,

    /// Command path (e.g., "playg core exec --names fixtures")
    pub command: String,

    /// Learning order hint (lower = should learn first)
    pub learning_order: u32,

    /// True if suitable for beginners
    pub beginner_friendly: bool,
}

/// Global feature registry
pub struct FeatureRegistry {
    features: HashMap<String, FeatureMetadata>,
}

impl FeatureRegistry {
    /// Create a new registry with all playground features
    pub fn new() -> Self {
        let mut registry = FeatureRegistry {
            features: HashMap::new(),
        };

        // Core features
        registry.register(FeatureMetadata {
            id: "fixtures".to_string(),
            name: "Test Fixtures".to_string(),
            brief: "Set up test data and state for isolated testing".to_string(),
            detailed: "Fixtures provide a predictable test environment by handling setup and teardown. They enable test isolation and data management without side effects.".to_string(),
            category: Category::Core,
            skill_level: SkillLevel::Beginner,
            keywords: vec!["setup".to_string(), "teardown".to_string(), "test".to_string(), "data".to_string(), "isolation".to_string()],
            related: vec!["builders".to_string(), "assertions".to_string()],
            command: "playg core exec --names fixtures".to_string(),
            learning_order: 1,
            beginner_friendly: true,
        });

        registry.register(FeatureMetadata {
            id: "builders".to_string(),
            name: "Fluent Builders".to_string(),
            brief: "Create test data with readable, chainable syntax".to_string(),
            detailed: "Builder pattern provides a fluent API for constructing complex test objects. Improves readability and maintainability of test code.".to_string(),
            category: Category::Core,
            skill_level: SkillLevel::Beginner,
            keywords: vec!["builder".to_string(), "fluent".to_string(), "test data".to_string(), "construct".to_string(), "readable".to_string()],
            related: vec!["fixtures".to_string(), "assertions".to_string()],
            command: "playg core exec --names builders".to_string(),
            learning_order: 2,
            beginner_friendly: true,
        });

        registry.register(FeatureMetadata {
            id: "assertions".to_string(),
            name: "Custom Assertions".to_string(),
            brief: "Write clear, expressive assertions for test validation".to_string(),
            detailed: "Custom assertion helpers improve test readability and provide better error messages. Supports result assertions, predicates, ranges, and more.".to_string(),
            category: Category::Core,
            skill_level: SkillLevel::Beginner,
            keywords: vec!["assertion".to_string(), "verify".to_string(), "validate".to_string(), "assert".to_string(), "error".to_string()],
            related: vec!["fixtures".to_string(), "builders".to_string()],
            command: "playg core exec --names assert".to_string(),
            learning_order: 3,
            beginner_friendly: true,
        });

        registry.register(FeatureMetadata {
            id: "macros".to_string(),
            name: "Test Macros".to_string(),
            brief: "Compile-time test registration and helper macros".to_string(),
            detailed: "Macros like test!, async_test!, and fixture_test! provide ergonomic test declaration and enforcement of AAA pattern.".to_string(),
            category: Category::Core,
            skill_level: SkillLevel::Intermediate,
            keywords: vec!["macro".to_string(), "compile".to_string(), "test".to_string(), "aaa".to_string()],
            related: vec!["state".to_string(), "const_assert".to_string()],
            command: "playg core list".to_string(),
            learning_order: 5,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "state".to_string(),
            name: "Type-Level AAA State".to_string(),
            brief: "Enforce Arrange-Act-Assert pattern at compile time".to_string(),
            detailed: "Type-level state machines ensure correct AAA pattern usage by making invalid state transitions impossible to compile.".to_string(),
            category: Category::Core,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["state".to_string(), "aaa".to_string(), "type".to_string(), "compile".to_string(), "pattern".to_string()],
            related: vec!["macros".to_string(), "const_assert".to_string()],
            command: "playg core exec --names state".to_string(),
            learning_order: 9,
            beginner_friendly: false,
        });

        // Testing features
        registry.register(FeatureMetadata {
            id: "property-testing".to_string(),
            name: "Property-Based Testing".to_string(),
            brief: "Generate random test cases to find edge cases automatically".to_string(),
            detailed: "Property-based testing (proptest) generates hundreds of random inputs to verify properties hold across the entire input space, finding edge cases humans miss.".to_string(),
            category: Category::Testing,
            skill_level: SkillLevel::Intermediate,
            keywords: vec!["property".to_string(), "random".to_string(), "edge case".to_string(), "shrinking".to_string()],
            related: vec!["mutation".to_string(), "concurrency".to_string()],
            command: "playg test exec --names property".to_string(),
            learning_order: 10,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "mutation-testing".to_string(),
            name: "Mutation Testing".to_string(),
            brief: "Verify test quality by introducing deliberate code mutations".to_string(),
            detailed: "Mutation testing modifies code and checks if tests catch the mutations. Weak mutations = weak tests. Ensures comprehensive test coverage.".to_string(),
            category: Category::Testing,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["mutation".to_string(), "quality".to_string(), "coverage".to_string(), "test".to_string()],
            related: vec!["coverage".to_string(), "property-testing".to_string()],
            command: "playg test exec --names mutation".to_string(),
            learning_order: 14,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "snapshot-testing".to_string(),
            name: "Snapshot Testing".to_string(),
            brief: "Capture and verify complex output against saved snapshots".to_string(),
            detailed: "Snapshot testing stores expected output and compares against actual, useful for UI, JSON, and complex formatting validation.".to_string(),
            category: Category::Testing,
            skill_level: SkillLevel::Intermediate,
            keywords: vec!["snapshot".to_string(), "output".to_string(), "verify".to_string(), "insta".to_string()],
            related: vec!["property-testing".to_string()],
            command: "playg test exec --names snapshot".to_string(),
            learning_order: 11,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "concurrency-testing".to_string(),
            name: "Concurrency Testing".to_string(),
            brief: "Deterministically test concurrent code with loom".to_string(),
            detailed: "Loom explores all possible interleavings of concurrent operations, catching race conditions that might never occur in practice.".to_string(),
            category: Category::Testing,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["concurrency".to_string(), "race".to_string(), "loom".to_string(), "threading".to_string()],
            related: vec!["property-testing".to_string()],
            command: "playg test exec --names concurrency".to_string(),
            learning_order: 15,
            beginner_friendly: false,
        });

        // Validation features
        registry.register(FeatureMetadata {
            id: "coverage".to_string(),
            name: "Test Coverage Analysis".to_string(),
            brief: "Measure and report test code coverage".to_string(),
            detailed: "Coverage analysis identifies untested code paths. Tracks line coverage, branch coverage, and generates HTML reports.".to_string(),
            category: Category::Validation,
            skill_level: SkillLevel::Intermediate,
            keywords: vec!["coverage".to_string(), "report".to_string(), "percentage".to_string(), "validate".to_string()],
            related: vec!["mutation-testing".to_string()],
            command: "playg valid exec --names coverage".to_string(),
            learning_order: 12,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "guards".to_string(),
            name: "Guard Constraints".to_string(),
            brief: "Enforce invariant constraints through type-level guards".to_string(),
            detailed: "Guards ensure values stay within valid ranges. Type-safe, compile-time enforcement prevents invalid values at runtime.".to_string(),
            category: Category::Validation,
            skill_level: SkillLevel::Intermediate,
            keywords: vec!["guard".to_string(), "constraint".to_string(), "invariant".to_string(), "validation".to_string()],
            related: vec!["state".to_string()],
            command: "playg valid exec --names guards".to_string(),
            learning_order: 8,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "jtbd".to_string(),
            name: "Jobs To Be Done".to_string(),
            brief: "Validate features against customer jobs and outcomes".to_string(),
            detailed: "JTBD framework ensures features serve actual customer needs and measurable outcomes. Prevents building wrong features well.".to_string(),
            category: Category::Validation,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["jtbd".to_string(), "customer".to_string(), "outcome".to_string(), "job".to_string()],
            related: vec!["process".to_string()],
            command: "playg valid exec --names jtbd".to_string(),
            learning_order: 18,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "performance".to_string(),
            name: "Performance Measurement".to_string(),
            brief: "Measure code execution time with RDTSC ticks".to_string(),
            detailed: "Precise timing measurement for performance validation. Track execution time trends and catch performance regressions.".to_string(),
            category: Category::Validation,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["performance".to_string(), "timing".to_string(), "tick".to_string(), "benchmark".to_string()],
            related: vec!["coverage".to_string()],
            command: "playg valid exec --names performance".to_string(),
            learning_order: 19,
            beginner_friendly: false,
        });

        // Observability features
        registry.register(FeatureMetadata {
            id: "otel".to_string(),
            name: "OpenTelemetry Validation".to_string(),
            brief: "Validate OTEL spans and metrics against semantic conventions".to_string(),
            detailed: "OpenTelemetry instrumentation validation ensures compliance with semantic conventions for consistent observability across systems.".to_string(),
            category: Category::Observability,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["otel".to_string(), "opentelemetry".to_string(), "span".to_string(), "metric".to_string(), "observability".to_string()],
            related: vec!["weaver".to_string()],
            command: "playg obs otel".to_string(),
            learning_order: 16,
            beginner_friendly: false,
        });

        registry.register(FeatureMetadata {
            id: "weaver".to_string(),
            name: "Weaver Live Validation".to_string(),
            brief: "Live validation of OTEL spans using Weaver semantic registry".to_string(),
            detailed: "Real-time validation of OpenTelemetry instrumentation against the canonical semantic convention registry.".to_string(),
            category: Category::Observability,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["weaver".to_string(), "otel".to_string(), "validation".to_string(), "semantic".to_string()],
            related: vec!["otel".to_string()],
            command: "playg obs weav".to_string(),
            learning_order: 17,
            beginner_friendly: false,
        });

        // Integration features
        registry.register(FeatureMetadata {
            id: "testcontainers".to_string(),
            name: "Docker Container Integration".to_string(),
            brief: "Spin up Docker containers for integration testing".to_string(),
            detailed: "Testcontainers provides lightweight, disposable Docker containers for integration tests with real services.".to_string(),
            category: Category::Integration,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["docker".to_string(), "container".to_string(), "integration".to_string(), "test".to_string()],
            related: vec!["property-testing".to_string()],
            command: "playg integ contain".to_string(),
            learning_order: 20,
            beginner_friendly: false,
        });

        // Methodology features
        registry.register(FeatureMetadata {
            id: "triz".to_string(),
            name: "TRIZ Problem Solving".to_string(),
            brief: "Resolve contradictions to find innovative breakthrough solutions".to_string(),
            detailed: "TRIZ (Theory of Inventive Problem Solving) helps find solutions that eliminate contradictions rather than compromise.".to_string(),
            category: Category::Analysis,
            skill_level: SkillLevel::Advanced,
            keywords: vec!["triz".to_string(), "contradiction".to_string(), "innovation".to_string(), "solve".to_string()],
            related: vec!["process".to_string()],
            command: "playg analyze triz".to_string(),
            learning_order: 21,
            beginner_friendly: false,
        });

        registry
    }

    /// Register a feature metadata
    pub fn register(&mut self, metadata: FeatureMetadata) {
        self.features.insert(metadata.id.clone(), metadata);
    }

    /// Get a feature by ID
    pub fn get(&self, id: &str) -> Option<&FeatureMetadata> {
        self.features.get(id)
    }

    /// Search features by keyword
    pub fn search(&self, query: &str) -> Vec<&FeatureMetadata> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<_> = self
            .features
            .values()
            .filter(|feature| {
                feature.id.contains(&query_lower)
                    || feature.name.to_lowercase().contains(&query_lower)
                    || feature.brief.to_lowercase().contains(&query_lower)
                    || feature.keywords.iter().any(|k| k.contains(&query_lower))
            })
            .collect();

        // Sort by relevance (exact ID match first, then name, then keywords)
        results.sort_by(|a, b| {
            let a_score = if a.id == query_lower {
                0
            } else if a.name.to_lowercase().contains(&query_lower) {
                1
            } else {
                2
            };

            let b_score = if b.id == query_lower {
                0
            } else if b.name.to_lowercase().contains(&query_lower) {
                1
            } else {
                2
            };

            a_score.cmp(&b_score)
        });

        results
    }

    /// Get all features by skill level
    pub fn by_skill_level(&self, level: SkillLevel) -> Vec<&FeatureMetadata> {
        let mut features: Vec<_> = self
            .features
            .values()
            .filter(|f| f.skill_level <= level)
            .collect();

        features.sort_by_key(|f| f.learning_order);
        features
    }

    /// Get all features by category
    pub fn by_category(&self, category: Category) -> Vec<&FeatureMetadata> {
        self.features.values().filter(|f| f.category == category).collect()
    }

    /// Get recommended learning path for a skill level
    pub fn learning_path(&self, level: SkillLevel) -> Vec<&FeatureMetadata> {
        let mut path = self.by_skill_level(level);
        path.sort_by_key(|f| f.learning_order);
        path
    }

    /// Get all features
    pub fn all(&self) -> Vec<&FeatureMetadata> {
        let mut all: Vec<_> = self.features.values().collect();
        all.sort_by_key(|f| f.learning_order);
        all
    }
}

impl Default for FeatureRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = FeatureRegistry::new();
        assert!(!registry.features.is_empty());
    }

    #[test]
    fn test_search_by_id() {
        let registry = FeatureRegistry::new();
        let results = registry.search("fixtures");
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "fixtures");
    }

    #[test]
    fn test_search_by_keyword() {
        let registry = FeatureRegistry::new();
        let results = registry.search("test");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_skill_level_filtering() {
        let registry = FeatureRegistry::new();
        let beginner = registry.by_skill_level(SkillLevel::Beginner);
        assert!(!beginner.is_empty());

        let all = registry.by_skill_level(SkillLevel::Advanced);
        assert!(all.len() >= beginner.len());
    }

    #[test]
    fn test_learning_path() {
        let registry = FeatureRegistry::new();
        let path = registry.learning_path(SkillLevel::Beginner);
        assert!(!path.is_empty());

        // Verify ordering
        for i in 1..path.len() {
            assert!(path[i - 1].learning_order <= path[i].learning_order);
        }
    }
}
