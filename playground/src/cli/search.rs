//! Search command for discovering features
//!
//! Provides intelligent feature discovery through keyword search, skill level filtering,
//! and category browsing. Enables world-class discoverability.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

use crate::cli::metadata::{Category, FeatureRegistry, SkillLevel};

#[derive(Serialize)]
pub struct SearchResult {
    pub query: String,
    pub matches: usize,
    pub features: Vec<SearchResultFeature>,
}

#[derive(Serialize)]
pub struct SearchResultFeature {
    pub id: String,
    pub name: String,
    pub brief: String,
    pub category: String,
    pub skill_level: String,
    pub command: String,
}

#[derive(Serialize)]
pub struct LearningPath {
    pub skill_level: String,
    pub features: Vec<SearchResultFeature>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct CategoryResults {
    pub category: String,
    pub features: Vec<SearchResultFeature>,
    pub total: usize,
}

/// Search for features by keyword
///
/// Find playground features by keyword, returns matching features with commands to run them.
///
/// Examples:
///   playg search test
///   playg search "property based"
///   playg search fixtures
#[verb]
fn find(query: String) -> Result<SearchResult> {
    let registry = FeatureRegistry::new();
    let results = registry.search(&query);

    let features = results
        .iter()
        .map(|feature| SearchResultFeature {
            id: feature.id.clone(),
            name: feature.name.clone(),
            brief: feature.brief.clone(),
            category: feature.category.to_string(),
            skill_level: skill_level_str(feature.skill_level),
            command: feature.command.clone(),
        })
        .collect();

    Ok(SearchResult {
        query,
        matches: results.len(),
        features,
    })
}

/// Show learning path for a skill level
///
/// Display features recommended for a specific skill level, in learning order.
///
/// Examples:
///   playg search learn beginner
///   playg search learn intermediate
///   playg search learn advanced
#[verb]
fn learn(level: String) -> Result<LearningPath> {
    let registry = FeatureRegistry::new();

    let skill_level = match level.to_lowercase().as_str() {
        "beginner" => SkillLevel::Beginner,
        "intermediate" => SkillLevel::Intermediate,
        "advanced" => SkillLevel::Advanced,
        _ => SkillLevel::Beginner,
    };

    let path = registry.learning_path(skill_level);

    let features = path
        .iter()
        .map(|feature| SearchResultFeature {
            id: feature.id.clone(),
            name: feature.name.clone(),
            brief: feature.brief.clone(),
            category: feature.category.to_string(),
            skill_level: skill_level_str(feature.skill_level),
            command: feature.command.clone(),
        })
        .collect();

    Ok(LearningPath {
        skill_level: skill_level_str(skill_level),
        features,
        total: features.len(),
    })
}

/// Browse features by category
///
/// Show all features in a specific category.
///
/// Examples:
///   playg search category core
///   playg search category testing
///   playg search category observability
#[verb]
fn category(name: String) -> Result<CategoryResults> {
    let registry = FeatureRegistry::new();

    let category = match name.to_lowercase().as_str() {
        "core" => Category::Core,
        "testing" => Category::Testing,
        "validation" => Category::Validation,
        "observability" => Category::Observability,
        "integration" => Category::Integration,
        "analysis" => Category::Analysis,
        "improvement" => Category::Improvement,
        "process" => Category::Process,
        "quality" => Category::Quality,
        "release" => Category::Release,
        _ => Category::Core,
    };

    let features_list = registry.by_category(category);

    let features = features_list
        .iter()
        .map(|feature| SearchResultFeature {
            id: feature.id.clone(),
            name: feature.name.clone(),
            brief: feature.brief.clone(),
            category: feature.category.to_string(),
            skill_level: skill_level_str(feature.skill_level),
            command: feature.command.clone(),
        })
        .collect();

    Ok(CategoryResults {
        category: category.to_string(),
        total: features.len(),
        features,
    })
}

fn skill_level_str(level: SkillLevel) -> String {
    match level {
        SkillLevel::Beginner => "Beginner".to_string(),
        SkillLevel::Intermediate => "Intermediate".to_string(),
        SkillLevel::Advanced => "Advanced".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_fixtures() -> Result<()> {
        let result = find("fixtures".to_string())?;
        assert!(result.matches > 0);
        assert!(result.features.iter().any(|f| f.id == "fixtures"));
        Ok(())
    }

    #[test]
    fn test_search_test() -> Result<()> {
        let result = find("test".to_string())?;
        assert!(result.matches > 0);
        Ok(())
    }

    #[test]
    fn test_learning_path_beginner() -> Result<()> {
        let path = learn("beginner".to_string())?;
        assert!(path.total > 0);
        assert_eq!(path.skill_level, "Beginner");
        Ok(())
    }

    #[test]
    fn test_learning_path_advanced() -> Result<()> {
        let path = learn("advanced".to_string())?;
        assert!(path.total > 0);
        assert!(path.total >= 5); // At least some advanced features
        Ok(())
    }

    #[test]
    fn test_category_core() -> Result<()> {
        let results = category("core".to_string())?;
        assert!(results.total > 0);
        assert_eq!(results.category, "Core");
        Ok(())
    }
}
