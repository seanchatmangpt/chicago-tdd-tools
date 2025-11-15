//! Analysis noun commands
//!
//! Demonstrates clap-noun-verb best practices through analysis methodology guidance.
//! Commands for analysis methodologies: TRIZ, QFD, Gap Analysis

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

/// Guidance information for analysis methodologies
#[derive(Serialize)]
pub struct AnalysisGuidanceInfo {
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

/// TRIZ (Theory of Inventive Problem Solving) guidance
///
/// Systematic method for solving problems using established patterns
/// and principles from thousands of patents.
#[verb]
fn triz() -> Result<AnalysisGuidanceInfo> {
    Ok(AnalysisGuidanceInfo {
        command: "analyze triz".to_string(),
        description: "TRIZ - Theory of Inventive Problem Solving".to_string(),
        steps: vec![
            "Step 1: Define Problem - Clearly state the technical problem".to_string(),
            "Step 2: Identify Contradictions - What are the competing objectives?".to_string(),
            "Step 3: Generalize Problem - Translate to generic problem".to_string(),
            "Step 4: Apply TRIZ Principles - Use contradiction matrix to find principles".to_string(),
            "Step 5: Generate Solutions - Apply principles to generate solutions".to_string(),
            "Step 6: Evaluate Solutions - Assess feasibility and potential".to_string(),
        ],
        key_principles: vec![
            "Patterns from patents - Solutions based on proven principles".to_string(),
            "Contradiction matrix - Maps problems to TRIZ principles".to_string(),
            "40 inventive principles - Well-documented problem-solving techniques".to_string(),
            "Innovation on demand - Systematic approach to innovation".to_string(),
            "Avoids creativity myth - Solutions follow patterns, not random ideas".to_string(),
        ],
    })
}

/// QFD (Quality Function Deployment) / Voice of Customer guidance
///
/// Translate customer needs into design requirements and ensure
/// customer voice drives design decisions.
#[verb]
fn qfd() -> Result<AnalysisGuidanceInfo> {
    Ok(AnalysisGuidanceInfo {
        command: "analyze qfd".to_string(),
        description: "QFD (Quality Function Deployment) - Voice of Customer".to_string(),
        steps: vec![
            "Step 1: Capture Voice of Customer - What do customers need?".to_string(),
            "Step 2: Organize Needs - Group into categories".to_string(),
            "Step 3: Define Requirements - How will we meet customer needs?".to_string(),
            "Step 4: Create House of Quality - Map needs to requirements".to_string(),
            "Step 5: Define Relationships - How strongly does requirement address need?".to_string(),
            "Step 6: Set Targets - What are target values for requirements?".to_string(),
        ],
        key_principles: vec![
            "Customer-focused - Start with customer needs, not assumptions".to_string(),
            "House of Quality - Matrix showing relationships between needs and requirements".to_string(),
            "Importance weighting - Prioritize based on customer importance".to_string(),
            "Multi-level - Can cascade to sub-requirements and components".to_string(),
            "Data-driven - Use customer data and competitive analysis".to_string(),
        ],
    })
}

/// Gap Analysis guidance
///
/// Identify gaps between current state and desired state,
/// prioritize by 80/20 rule.
#[verb]
fn gaps() -> Result<AnalysisGuidanceInfo> {
    Ok(AnalysisGuidanceInfo {
        command: "analyze gaps".to_string(),
        description: "80-20 Fill Gaps Analysis".to_string(),
        steps: vec![
            "Step 1: Define Current State - What capabilities exist now?".to_string(),
            "Step 2: Define Desired State - What capabilities are needed?".to_string(),
            "Step 3: Identify Gaps - What's the difference?".to_string(),
            "Step 4: Assess Impact - Which gaps matter most? (80/20)".to_string(),
            "Step 5: Prioritize - Focus on high-impact 20% of gaps".to_string(),
            "Step 6: Plan Fills - Design solutions for critical gaps".to_string(),
        ],
        key_principles: vec![
            "80/20 rule - 20% of gaps cause 80% of problems".to_string(),
            "Impact focus - Prioritize by business/customer impact".to_string(),
            "Current reality - Assess based on actual capabilities, not assumptions".to_string(),
            "Prioritization - Don't try to fill all gaps at once".to_string(),
            "Value-driven - Close gaps that deliver most value".to_string(),
        ],
    })
}
