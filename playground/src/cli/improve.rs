//! Improve noun commands
//!
//! Commands for continuous improvement: Kaizen, Muda, Mura, Gemba, Poka-Yoke

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

/// Kaizen (continuous improvement) guidance
///
/// Make small, incremental improvements rather than big rewrites.
/// Continuous small improvements that compound over time.
///
/// Examples:
///   playg improve kaizen                  # Show Kaizen guidance
#[verb]
fn kaizen() -> Result<GuidanceInfo> {
    Ok(GuidanceInfo {
        command: "improve kaizen".to_string(),
        description: "Kaizen - Continuous Improvement".to_string(),
        steps: vec![
            "Step 1: Identify Opportunity - Find small, focused improvement".to_string(),
            "Step 2: Plan Change - Design minimal change that improves".to_string(),
            "Step 3: Do (Implement) - Implement the planned improvement".to_string(),
            "Step 4: Check (Verify) - Verify improvement achieved its goal".to_string(),
            "Step 5: Act (Standardize) - Apply pattern consistently, document".to_string(),
        ],
        key_principles: vec![
            "Small improvements - Can be done in minutes, not hours".to_string(),
            "Continuous - Make improvements regularly, not just once".to_string(),
            "Safe - Low risk of breaking things".to_string(),
            "Low risk - Small changes are safer than big rewrites".to_string(),
            "Compound effect - Small improvements add up over time".to_string(),
        ],
    })
}

/// Muda (waste) elimination guidance
///
/// Identify and eliminate waste (non-value-adding activities)
/// from processes.
#[verb]
fn muda() -> Result<GuidanceInfo> {
    Ok(GuidanceInfo {
        command: "improve muda".to_string(),
        description: "Eliminate Muda (Waste)".to_string(),
        steps: vec![
            "Step 1: Understand Value - What adds value to customer?".to_string(),
            "Step 2: Map Process - Document current process flow".to_string(),
            "Step 3: Identify Waste - Categorize non-value-adding activities".to_string(),
            "Step 4: Analyze Waste - Understand why waste exists".to_string(),
            "Step 5: Eliminate Waste - Remove waste, simplify process".to_string(),
            "Step 6: Verify - Confirm waste eliminated, value increased".to_string(),
        ],
        key_principles: vec![
            "Value focus - Remove what doesn't add customer value".to_string(),
            "Seven wastes - Transport, inventory, motion, waiting, overprocessing, overproduction, defects".to_string(),
            "Continuous - Keep looking for waste, never finished".to_string(),
            "Root cause - Eliminate root causes of waste, not symptoms".to_string(),
            "Lean mindset - Maximize value, minimize waste".to_string(),
        ],
    })
}

/// Mura (variation) elimination guidance
///
/// Identify and eliminate inconsistency and variation from processes
/// to improve reliability and predictability.
#[verb]
fn mura() -> Result<GuidanceInfo> {
    Ok(GuidanceInfo {
        command: "improve mura".to_string(),
        description: "Eliminate Mura (Variation)".to_string(),
        steps: vec![
            "Step 1: Identify Variation - Where is process inconsistent?".to_string(),
            "Step 2: Measure Variation - How much variation exists?".to_string(),
            "Step 3: Identify Causes - Why does variation exist?".to_string(),
            "Step 4: Standardize - Create standard methods to reduce variation".to_string(),
            "Step 5: Control - Implement controls to maintain standards".to_string(),
            "Step 6: Verify - Confirm variation reduced, process stabilized".to_string(),
        ],
        key_principles: vec![
            "Consistency - Standard methods reduce variation".to_string(),
            "Predictability - Lower variation = more predictable results".to_string(),
            "Root causes - Eliminate sources of variation, not symptoms".to_string(),
            "Standardization - Document standard methods, train team".to_string(),
            "Continuous - Keep reducing variation over time".to_string(),
        ],
    })
}

/// Gemba Walk guidance
///
/// Go to the actual place where work happens to understand reality,
/// observe firsthand, and identify improvement opportunities.
#[verb]
fn gemba() -> Result<GuidanceInfo> {
    Ok(GuidanceInfo {
        command: "improve gemba".to_string(),
        description: "Gemba Walk".to_string(),
        steps: vec![
            "Step 1: Prepare - Define purpose, identify area to visit".to_string(),
            "Step 2: Go - Visit the actual workplace (gemba)".to_string(),
            "Step 3: Observe - Watch work being done without interrupting".to_string(),
            "Step 4: Ask Why - Ask why work is done this way".to_string(),
            "Step 5: Listen - Listen to what workers say about problems".to_string(),
            "Step 6: Identify Opportunities - Note improvement ideas".to_string(),
        ],
        key_principles: vec![
            "Go see for yourself - Don't rely on reports, observe directly".to_string(),
            "Respect people - Listen to workers, they know the real situation".to_string(),
            "No judgment - Observe with curiosity, not criticism".to_string(),
            "Real conditions - See actual situation, not ideal situation".to_string(),
            "Hands-on leadership - Leaders spend time where work happens".to_string(),
        ],
    })
}

/// Poka-Yoke (mistake-proofing) design guidance
///
/// Design systems to prevent errors by making mistakes impossible or obvious.
///
/// Examples:
///   playg improve poka                   # Show Poka-Yoke guidance
#[verb]
fn poka() -> Result<GuidanceInfo> {
    Ok(GuidanceInfo {
        command: "improve poka".to_string(),
        description: "Poka-Yoke (Mistake-Proofing)".to_string(),
        steps: vec![
            "Step 1: Identify Error Risks - What mistakes are possible?".to_string(),
            "Step 2: Analyze Impacts - What happens if error occurs?".to_string(),
            "Step 3: Design Prevention - Can design make error impossible?".to_string(),
            "Step 4: Add Detection - If prevention fails, can we detect error?".to_string(),
            "Step 5: Implement - Add safeguards to design or process".to_string(),
            "Step 6: Verify - Test that poka-yoke works effectively".to_string(),
        ],
        key_principles: vec![
            "Prevent errors - Design so mistakes are impossible".to_string(),
            "Type safety - Use type system to prevent categories of errors".to_string(),
            "Fail-safe - When errors possible, fail safely or alert".to_string(),
            "User-centered - Design should be intuitive, not error-prone".to_string(),
            "No blame - Errors are design problems, not people problems".to_string(),
        ],
    })
}
