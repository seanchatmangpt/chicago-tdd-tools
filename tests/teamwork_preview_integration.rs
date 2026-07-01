//! Integration tests for Multi-Agent Teamwork Orchestration `/teamwork-preview`.
//!
//! This test suite verifies requirements elicitation from `prompt_draft.md`,
//! folder-isolated agent coordination under `.agents/`, liveness progress heartbeats,
//! and compliance with the 5-component handoff protocol.

use std::fs;
use std::path::{Path, PathBuf};

/// Simulates the two-phase workflow for /teamwork-preview.
/// Phase 1: Requirements elicitation via prompt_draft.md.
/// Phase 2: Execution and workspace validation under .agents/.
#[test]
fn test_teamwork_preview_workflow_simulation() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let proj_root = temp_dir.path();

    // 1. Phase 1: Elicitation - Create a mock prompt_draft.md
    let prompt_draft_path = proj_root.join("prompt_draft.md");
    let prompt_draft_content = "\
# Task Draft: Doc Fixes
- Target Working Directory: .agents/worker_doc_fixer/
- Acceptance Criteria:
  1. Add Multi-Agent Teamwork Orchestration section to README.md
  2. Run cargo make and cargo test
";
    fs::write(&prompt_draft_path, prompt_draft_content).expect("failed to write prompt_draft.md");

    // Perform static elicitation validation
    let parsed_draft = parse_prompt_draft(&prompt_draft_path).expect("failed to parse draft");
    assert_eq!(parsed_draft.target_dir, PathBuf::from(".agents/worker_doc_fixer/"));
    assert!(parsed_draft
        .criteria
        .contains(&"Add Multi-Agent Teamwork Orchestration section to README.md".to_string()));

    // 2. Phase 2: Execution - Simulate subagent folder-isolated workspace under .agents/
    let agents_dir = proj_root.join(".agents");
    let worker_dir = agents_dir.join("worker_doc_fixer");
    fs::create_dir_all(&worker_dir).expect("failed to create worker directory");

    // Write a mock progress.md for liveness heartbeats
    let progress_path = worker_dir.join("progress.md");
    let progress_content = "\
# Progress Track
Last visited: 2026-06-30T11:34:00-07:00

- [x] Task initiated
";
    fs::write(&progress_path, progress_content).expect("failed to write progress.md");

    // Write a mock handoff.md following the 5-component handoff report
    let handoff_path = worker_dir.join("handoff.md");
    let handoff_content = "\
# Handoff Report

## 1. Observation
Observed README.md structure and verified tests.

## 2. Logic Chain
Traced insertion point to line 670.

## 3. Caveats
No caveats.

## 4. Conclusion
README.md updated successfully.

## 5. Verification Method
Run cargo make --no-workspace docs-check.
";
    fs::write(&handoff_path, handoff_content).expect("failed to write handoff.md");

    // Validate the coordination files (integrity checks)
    validate_liveness_heartbeat(&progress_path).expect("liveness heartbeat check failed");
    validate_handoff_report(&handoff_path).expect("handoff report check failed");
}

struct PromptDraft {
    target_dir: PathBuf,
    criteria: Vec<String>,
}

fn parse_prompt_draft(path: &Path) -> Result<PromptDraft, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut target_dir = PathBuf::new();
    let mut criteria = Vec::new();

    for line in content.lines() {
        if line.contains("Target Working Directory:") {
            let parts: Vec<&str> = line.split("Target Working Directory:").collect();
            if let Some(dir) = parts.get(1) {
                target_dir = PathBuf::from(dir.trim());
            }
        } else if line.trim().starts_with('-')
            || line.trim().starts_with("1.")
            || line.trim().starts_with("2.")
        {
            // Simple parsing of bullet points
            let text = line
                .trim()
                .trim_start_matches('-')
                .trim_start_matches("1.")
                .trim_start_matches("2.")
                .trim();
            if !text.is_empty() && !text.contains("Target Working Directory:") {
                criteria.push(text.to_string());
            }
        }
    }

    Ok(PromptDraft { target_dir, criteria })
}

fn validate_liveness_heartbeat(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if !content.contains("Last visited:") {
        return Err("progress.md missing 'Last visited' heartbeat timestamp".to_string());
    }
    Ok(())
}

fn validate_handoff_report(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let required_headers =
        ["Observation", "Logic Chain", "Caveats", "Conclusion", "Verification Method"];

    for header in &required_headers {
        if !content.contains(header) {
            return Err(format!("handoff.md missing required section: {}", header));
        }
    }
    Ok(())
}
