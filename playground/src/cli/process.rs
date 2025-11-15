//! Process noun commands
//!
//! Demonstrates clap-noun-verb best practices through process methodologies.
//! Commands for process methodologies: DMEDI, DMAIC, ACP, Concept Selection

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

/// Guidance information for process methodologies
#[derive(Serialize)]
pub struct ProcessGuidanceInfo {
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

/// DMEDI design process guidance
///
/// DMEDI (Define, Measure, Explore, Develop, Implement) is a systematic design methodology
/// that ensures new designs meet customer needs, business goals, and quality requirements.
#[verb]
fn dmedi() -> Result<ProcessGuidanceInfo> {
    Ok(ProcessGuidanceInfo {
        command: "process dmedi".to_string(),
        description: "DMEDI Design Process - Multi-Step Workflow".to_string(),
        steps: vec![
            "Step 1: Define - Define project scope, goals, and success criteria".to_string(),
            "Step 2: Measure - Measure customer needs and current performance".to_string(),
            "Step 3: Explore - Explore design concepts and select best approach".to_string(),
            "Step 4: Develop - Develop detailed design and optimize performance".to_string(),
            "Step 5: Implement - Implement design, validate, and deploy".to_string(),
        ],
        key_principles: vec![
            "Design right the first time - Systematic design prevents costly redesigns".to_string(),
            "Customer-focused - Start with customer needs, end with satisfaction".to_string(),
            "Data-driven - Use data to make design decisions, not assumptions".to_string(),
            "Systematic - Follow phases systematically, don't skip steps".to_string(),
            "Iterative - Refine design through exploration and development".to_string(),
        ],
    })
}

/// DMAIC problem-solving methodology guidance
///
/// DMAIC (Define, Measure, Analyze, Improve, Control) is a systematic approach
/// for problem-solving and process improvement.
#[verb]
fn dmaic() -> Result<ProcessGuidanceInfo> {
    Ok(ProcessGuidanceInfo {
        command: "process dmaic".to_string(),
        description: "DMAIC Problem-Solving Methodology".to_string(),
        steps: vec![
            "Step 1: Define - Define the problem and project scope".to_string(),
            "Step 2: Measure - Measure current performance and establish baseline".to_string(),
            "Step 3: Analyze - Analyze root causes and identify improvement opportunities".to_string(),
            "Step 4: Improve - Improve by implementing solutions and optimizing".to_string(),
            "Step 5: Control - Control to maintain improvements over time".to_string(),
        ],
        key_principles: vec![
            "Data-driven - Use data to identify problems and measure improvements".to_string(),
            "Root cause focus - Address root causes, not symptoms".to_string(),
            "Incremental improvement - Make steady, measurable progress".to_string(),
            "Team collaboration - Leverage team expertise and insights".to_string(),
            "Sustainability - Implement controls to maintain gains".to_string(),
        ],
    })
}

/// ACP (Add, Commit, Push) workflow guidance
///
/// ACP is a structured git workflow for committing changes systematically.
#[verb]
fn acp() -> Result<ProcessGuidanceInfo> {
    Ok(ProcessGuidanceInfo {
        command: "process acp".to_string(),
        description: "Add, Commit, Push Workflow".to_string(),
        steps: vec![
            "Step 1: Add - Stage relevant files with git add".to_string(),
            "Step 2: Commit - Create meaningful commit with clear message".to_string(),
            "Step 3: Push - Push commits to remote repository".to_string(),
        ],
        key_principles: vec![
            "Atomic commits - Each commit should be a logical unit of work".to_string(),
            "Clear messages - Commit messages should explain why, not what".to_string(),
            "Frequent pushes - Push regularly to share changes with team".to_string(),
            "Small changes - Smaller changes are easier to review and revert if needed".to_string(),
            "CI integration - Ensure tests pass before pushing".to_string(),
        ],
    })
}

/// Concept selection methodology guidance
///
/// Systematic approach for selecting the best design concept from multiple alternatives.
#[verb]
fn concept() -> Result<ProcessGuidanceInfo> {
    Ok(ProcessGuidanceInfo {
        command: "process concept".to_string(),
        description: "Concept Selection Methodology".to_string(),
        steps: vec![
            "Step 1: Generate concepts - Brainstorm multiple design alternatives".to_string(),
            "Step 2: Define criteria - Establish evaluation criteria".to_string(),
            "Step 3: Evaluate - Score each concept against criteria".to_string(),
            "Step 4: Compare - Use Pugh Matrix or similar method to compare".to_string(),
            "Step 5: Select - Choose best concept or combination".to_string(),
        ],
        key_principles: vec![
            "Multiple alternatives - Generate diverse concepts before selecting".to_string(),
            "Clear criteria - Use explicit criteria for fair evaluation".to_string(),
            "Systematic comparison - Use structured methods (Pugh, AHP)".to_string(),
            "Rationale documentation - Document why concepts were selected".to_string(),
            "Feasibility check - Verify selected concept is feasible".to_string(),
        ],
    })
}
