//! GitHub Actions noun commands
//!
//! Commands for checking GitHub Actions status, workflows, and runs.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Debug)]
pub struct WorkflowStatus {
    pub name: String,
    pub path: String,
    pub jobs: usize,
    pub valid: bool,
}

#[derive(Serialize, Debug)]
pub struct GhStatus {
    pub workflows: Vec<WorkflowStatus>,
    pub total_workflows: usize,
    pub valid_workflows: usize,
    pub invalid_workflows: usize,
}

/// Show GitHub Actions status
#[verb]
fn stat() -> Result<GhStatus> {
    let workflows = discover_workflows();
    let valid_count = workflows.iter().filter(|w| w.valid).count();
    let invalid_count = workflows.len() - valid_count;

    println!("📊 GitHub Actions Status");
    println!("========================");
    println!();

    for workflow in &workflows {
        let status_icon = if workflow.valid { "✅" } else { "❌" };
        println!("{} {} (jobs: {})", status_icon, workflow.name, workflow.jobs);
        println!("   Path: {}", workflow.path);
    }
    println!();

    Ok(GhStatus {
        total_workflows: workflows.len(),
        valid_workflows: valid_count,
        invalid_workflows: invalid_count,
        workflows,
    })
}

/// List all GitHub Actions workflows (names format by default)
#[verb]
fn list() -> Result<Vec<String>> {
    let workflows = discover_workflows();
    let names: Vec<String> = workflows.iter().map(|w| w.name.clone()).collect();

    println!("📋 GitHub Actions Workflows:");
    for name in &names {
        println!("  • {}", name);
    }

    Ok(names)
}

/// Validate GitHub Actions workflows
#[verb]
fn check() -> Result<Vec<String>> {
    let workflows = discover_workflows();
    let mut issues = Vec::new();

    println!("🔍 Validating GitHub Actions workflows...");
    println!();

    for workflow in &workflows {
        println!("Checking: {}", workflow.name);

        if !workflow.valid {
            issues.push(format!("{}: Invalid YAML syntax", workflow.name));
        }

        let path = PathBuf::from(&workflow.path);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if !content.contains("permissions:") {
                issues.push(format!(
                    "{}: Missing explicit permissions (security best practice)",
                    workflow.name
                ));
            }

            let deprecated_actions = [
                ("actions/create-release@v1", "Use softprops/action-gh-release@v2"),
                ("actions/upload-release-asset@v1", "Use artifact upload pattern"),
                ("actions/cache@v1", "Use Swatinem/rust-cache@v2 for Rust"),
                ("actions/cache@v2", "Use Swatinem/rust-cache@v2 for Rust"),
            ];

            for (action, suggestion) in &deprecated_actions {
                if content.contains(action) {
                    issues.push(format!(
                        "{}: Uses deprecated action '{}'. {}",
                        workflow.name, action, suggestion
                    ));
                }
            }
        }
    }

    if issues.is_empty() {
        println!("✅ All workflows validated successfully!");
    } else {
        println!("⚠️  Found {} issue(s):", issues.len());
        for issue in &issues {
            println!("  • {}", issue);
        }
    }

    Ok(issues)
}

/// Show recent workflow runs (requires gh CLI)
#[verb]
fn runs() -> Result<String> {
    println!("🔄 Fetching recent workflow runs...");
    println!();

    if Command::new("gh").arg("--version").output().is_err() {
        println!("❌ GitHub CLI (gh) not found. Install from: https://cli.github.com");
        return Ok("gh CLI not available".to_string());
    }

    let limit = "10".to_string();
    let mut args = vec!["run", "list", "--limit", &limit];

    match Command::new("gh").args(&args).output() {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            println!("{}", stdout);
            Ok(format!("Fetched {} runs", stdout.lines().count().saturating_sub(1)))
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            println!("❌ Error: {}", stderr);
            Ok("gh command failed".to_string())
        }
        Err(e) => {
            println!("❌ Failed to execute: {}", e);
            Ok("execution failed".to_string())
        }
    }
}

/// Open GitHub Actions page in browser
#[verb]
fn open() -> Result<String> {
    println!("🌐 Opening GitHub Actions...");

    if let Ok(result) = Command::new("gh").args(["repo", "view", "--web"]).output() {
        if result.status.success() {
            return Ok("Opened in browser".to_string());
        }
    }

    if let Ok(output) = Command::new("git").args(["config", "--get", "remote.origin.url"]).output()
    {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout);
            let url = url.trim();

            let https_url = if url.starts_with("git@github.com:") {
                url.replace("git@github.com:", "https://github.com/").replace(".git", "")
            } else if url.starts_with("https://github.com") {
                url.replace(".git", "")
            } else {
                return Ok("Could not determine GitHub URL".to_string());
            };

            let actions_url = format!("{}/actions", https_url);
            println!("Actions URL: {}", actions_url);
            return Ok(actions_url);
        }
    }

    Ok("Could not open GitHub Actions".to_string())
}

fn discover_workflows() -> Vec<WorkflowStatus> {
    use std::fs;

    let workflows_dir = PathBuf::from(".github/workflows");
    if !workflows_dir.exists() {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(&workflows_dir) else {
        return Vec::new();
    };

    let mut workflows = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yml")
            || path.extension().and_then(|s| s.to_str()) == Some("yaml")
        {
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            let valid = serde_yaml::from_str::<serde_yaml::Value>(&content).is_ok();

            let jobs = if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                yaml.get("jobs").and_then(|j| j.as_mapping()).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            workflows.push(WorkflowStatus { name, path: path.display().to_string(), jobs, valid });
        }
    }

    workflows
}
