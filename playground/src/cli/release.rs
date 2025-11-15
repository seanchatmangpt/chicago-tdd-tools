//! Release noun commands
//!
//! Demonstrates clap-noun-verb best practices through release workflows.
//! Commands for release workflows: Release Preparation

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

/// Guidance information for release workflows
#[derive(Serialize)]
pub struct ReleaseGuidanceInfo {
    /// The command to run this guidance
    pub command: String,
    /// Description of what this methodology covers
    pub description: String,
    /// Ordered steps for the methodology
    pub steps: Vec<String>,
    /// Key principles to follow
    pub key_principles: Vec<String>,
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb] macro)
// ============================================================================

/// Release preparation workflow guidance
///
/// Systematic checklist for preparing releases to production.
/// Ensures nothing is forgotten and quality is maintained.
#[verb]
fn prep() -> Result<ReleaseGuidanceInfo> {
    Ok(ReleaseGuidanceInfo {
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
