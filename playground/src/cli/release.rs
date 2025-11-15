//! Release noun commands
//!
//! Commands for release workflows: Release Preparation

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct GuidanceInfo {
    pub command: String,
    pub description: String,
    pub steps: Vec<String>,
    pub key_principles: Vec<String>,
}

/// Release preparation workflow guidance
///
/// Systematic checklist for preparing releases to production.
/// Ensures nothing is forgotten and quality is maintained.
#[verb]
fn prep() -> Result<GuidanceInfo> {
    Ok(GuidanceInfo {
        command: "release prep".to_string(),
        description: "Release Preparation Workflow".to_string(),
        steps: vec![
            "Step 1: Pre-Release Validation - Verify all requirements met".to_string(),
            "Step 2: Code Quality Checks - Run linters, tests, coverage".to_string(),
            "Step 3: Git State Verification - Ensure clean git state".to_string(),
            "Step 4: Version Management - Update version numbers consistently".to_string(),
            "Step 5: Release Notes - Create clear release notes for users".to_string(),
            "Step 6: Final Validation - Run release checklist".to_string(),
            "Step 7: Build Artifacts - Create release artifacts".to_string(),
            "Step 8: Deploy - Deploy to production".to_string(),
            "Step 9: Post-Release - Monitor and support".to_string(),
        ],
        key_principles: vec![
            "Pre-release checks - Don't release until all checks pass".to_string(),
            "Automated validation - Use CI/CD to catch issues early".to_string(),
            "Clear communication - Release notes help users understand changes".to_string(),
            "Rollback readiness - Have plan to rollback if needed".to_string(),
            "Post-release monitoring - Watch for issues after deployment".to_string(),
            "Clean releases - No uncommitted changes, no debug code".to_string(),
        ],
    })
}
