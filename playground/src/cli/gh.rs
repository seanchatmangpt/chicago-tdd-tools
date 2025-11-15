//! GitHub Actions noun commands
//!
//! Commands for checking GitHub Actions status, workflows, and runs.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::format_utils::OutputFormat;

#[derive(Serialize, Debug)]
struct WorkflowStatus {
    name: String,
    path: String,
    jobs: usize,
    valid: bool,
}

#[derive(Serialize, Debug)]
struct GhStatus {
    workflows: Vec<WorkflowStatus>,
    total_workflows: usize,
    valid_workflows: usize,
    invalid_workflows: usize,
}

/// Show GitHub Actions status
#[verb]
fn stat(
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level")]
    verbose: usize,
    #[arg(short = 'f', long, default_value = "json", help = "Output format: json, yaml, toml, table, tsv")]
    format: String,
) -> Result<GhStatus> {
    let workflows = discover_workflows();
    let valid_count = workflows.iter().filter(|w| w.valid).count();
    let invalid_count = workflows.len() - valid_count;

    let status = GhStatus {
        total_workflows: workflows.len(),
        valid_workflows: valid_count,
        invalid_workflows: invalid_count,
        workflows,
    };

    // Format and print output
    if let Ok(fmt) = OutputFormat::from_str(&format) {
        if let Ok(formatted) = fmt.serialize(&status) {
            println!("{}", formatted);
        }
    } else if verbose > 0 {
        // Fallback to verbose output if format parsing fails
        println!("📊 GitHub Actions Status");
        println!("========================");
        println!();

        for workflow in &status.workflows {
            let status_icon = if workflow.valid { "✅" } else { "❌" };
            println!("{} {} (jobs: {})", status_icon, workflow.name, workflow.jobs);

            if verbose > 1 {
                println!("   Path: {}", workflow.path);
            }
        }
        println!();
    }

    Ok(status)
}

/// List all GitHub Actions workflows
#[verb]
fn list(
    #[arg(short = 'f', long, default_value = "json", help = "Output format: json, yaml, toml, table, tsv")]
    format: String,
) -> Result<Vec<String>> {
    let workflows = discover_workflows();
    let names: Vec<String> = workflows.iter().map(|w| w.name.clone()).collect();

    // Format and print output
    if let Ok(fmt) = OutputFormat::from_str(&format) {
        if let Ok(formatted) = fmt.serialize(&names) {
            println!("{}", formatted);
        }
    }

    Ok(names)
}

// Legacy handler for backward compatibility
fn _list_workflows_by_format(output_format: &str, workflows: &[WorkflowStatus]) -> Result<Vec<String>> {
    match output_format {
        "names" => {
            let names: Vec<String> = workflows.iter().map(|w| w.name.clone()).collect();
            println!("📋 GitHub Actions Workflows:");
            for name in &names {
                println!("  • {}", name);
            }
            Ok(names)
        }
        "paths" => {
            let paths: Vec<String> = workflows.iter().map(|w| w.path.clone()).collect();
            println!("📁 Workflow Files:");
            for path in &paths {
                println!("  • {}", path);
            }
            Ok(paths)
        }
        "json" => {
            let json = serde_json::to_string_pretty(&workflows).unwrap_or_else(|_| "[]".to_string());
            println!("{}", json);
            Ok(workflows.iter().map(|w| w.name.clone()).collect())
        }
        _ => Ok(vec![format!("Unknown format: {}. Use 'names', 'paths', or 'json'", output_format)]),
    }
}

/// Validate GitHub Actions workflows
#[verb]
fn check(
    #[arg(long, help = "Attempt to auto-fix issues (not yet implemented)")]
    fix: bool,
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level")]
    verbose: usize,
) -> Result<Vec<String>> {
    let workflows = discover_workflows();
    let mut issues = Vec::new();

    println!("🔍 Validating GitHub Actions workflows...");
    println!();

    for workflow in &workflows {
        if verbose > 0 {
            println!("Checking: {}", workflow.name);
        }

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

    if fix {
        println!();
        println!("🔧 Auto-fix is not yet implemented.");
    }

    Ok(issues)
}

/// Show recent workflow runs (requires gh CLI)
#[verb]
fn runs(
    #[arg(short = 'l', long, value_name = "NUM", help = "Limit number of runs to display")]
    limit: Option<usize>,
    #[arg(short = 'w', long, value_name = "WORKFLOW", help = "Filter by workflow name")]
    workflow: Option<String>,
    #[arg(short = 'v', long, action = "count", help = "Increase verbosity level")]
    _verbose: usize,
) -> Result<String> {
    let limit = limit.unwrap_or(10);
    println!("🔄 Fetching recent workflow runs...");
    println!();

    if Command::new("gh").arg("--version").output().is_err() {
        println!("❌ GitHub CLI (gh) not found. Install from: https://cli.github.com");
        return Ok("gh CLI not available".to_string());
    }

    let limit_str = limit.to_string();
    let mut args = vec!["run", "list", "--limit", &limit_str];
    let wf_str;
    if let Some(ref wf) = workflow {
        wf_str = wf.clone();
        args.extend(["--workflow", &wf_str]);
    }

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

    if let Ok(output) = Command::new("git").args(["config", "--get", "remote.origin.url"]).output() {
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
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            let valid = serde_yaml::from_str::<serde_yaml::Value>(&content).is_ok();

            let jobs = if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                yaml.get("jobs")
                    .and_then(|j| j.as_mapping())
                    .map(|m| m.len())
                    .unwrap_or(0)
            } else {
                0
            };

            workflows.push(WorkflowStatus {
                name,
                path: path.display().to_string(),
                jobs,
                valid,
            });
        }
    }

    workflows
}
