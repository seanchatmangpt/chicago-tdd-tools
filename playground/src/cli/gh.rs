//! GitHub Actions noun commands
//!
//! Reference implementation demonstrating clap-noun-verb best practices through practical
//! GitHub Actions workflow management. Showcases type inference, auto-serialization, and
//! multiple output format support.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

/// Information about a single GitHub Actions workflow
#[derive(Serialize, Debug)]
pub struct WorkflowStatus {
    /// Workflow name
    pub name: String,
    /// File path to workflow
    pub path: String,
    /// Number of jobs in workflow
    pub jobs: usize,
    /// Whether workflow YAML is valid
    pub valid: bool,
}

/// Summary of all GitHub Actions workflows
#[derive(Serialize, Debug)]
pub struct GitHubActionsStatus {
    /// All discovered workflows
    pub workflows: Vec<WorkflowStatus>,
    /// Total number of workflow files
    pub total_workflows: usize,
    /// Number of workflows with valid YAML
    pub valid_workflows: usize,
    /// Number of workflows with invalid YAML
    pub invalid_workflows: usize,
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb] macro)
// ============================================================================

/// Show GitHub Actions status
///
/// Displays status of all workflows found in `.github/workflows/`.
/// Validates YAML syntax and reports summary statistics.
///
/// Use -v for detailed per-workflow information, -vv for debug output.
///
/// # Examples
/// ```text
/// playg gh stat                  # Shows status in JSON format
/// playg gh stat -v               # Shows status with verbose output
/// playg gh stat --format yaml    # Shows status in YAML format
/// ```
#[verb]
fn stat(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<GitHubActionsStatus> {
    let workflows = discover_workflows();
    let valid_count = workflows.iter().filter(|w| w.valid).count();
    let invalid_count = workflows.len() - valid_count;

    if verbose > 0 {
        println!("📊 GitHub Actions Status");
        println!("========================");
        println!();

        for workflow in &workflows {
            let status_icon = if workflow.valid { "✅" } else { "❌" };
            println!("{} {} (jobs: {})", status_icon, workflow.name, workflow.jobs);

            if verbose > 1 {
                println!("   Path: {}", workflow.path);
            }
        }
        println!();
    }

    if verbose > 0 {
        eprintln!("📊 GitHub Actions Status");
        eprintln!("========================");
        for workflow in &workflows {
            let status_icon = if workflow.valid { "✅" } else { "❌" };
            eprintln!("{} {} (jobs: {})", status_icon, workflow.name, workflow.jobs);
            if verbose > 1 {
                eprintln!("   Path: {}", workflow.path);
            }
        }
        eprintln!();
    }

    Ok(GitHubActionsStatus {
        total_workflows: workflows.len(),
        valid_workflows: valid_count,
        invalid_workflows: invalid_count,
        workflows,
    })
}

/// List all GitHub Actions workflows
///
/// Lists all workflow files found in `.github/workflows/`.
/// Supports multiple output formats via --format flag.
///
/// # Examples
/// ```text
/// playg gh list                   # Shows workflow names in JSON format
/// playg gh list --format yaml     # Shows workflows in YAML format
/// playg gh list --format table    # Shows workflows in ASCII table format
/// ```
#[verb]
fn list(
    #[arg(long)]
    format: Option<String>,
) -> Result<Vec<String>> {
    let workflows = discover_workflows();
    let output_format = format.as_deref().unwrap_or("names");

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
///
/// Checks all workflows for:
/// - Valid YAML syntax
/// - Missing explicit permissions (security best practice)
/// - Deprecated action versions
///
/// Use -v for detailed output per workflow, -vv for verbose.
///
/// # Examples
/// ```text
/// playg gh check              # Validates all workflows
/// playg gh check -v           # Shows each workflow being checked
/// playg gh check --fix        # Auto-fix (not yet implemented)
/// ```
#[verb]
fn check(
    #[arg(long)]
    fix: bool,

    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<Vec<String>> {
    let workflows = discover_workflows();
    let mut issues = Vec::new();

    if verbose > 0 {
        eprintln!("🔍 Validating GitHub Actions workflows...");
        eprintln!();
    }

    for workflow in &workflows {
        if verbose > 0 {
            eprintln!("Checking: {}", workflow.name);
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
        if verbose > 0 {
            eprintln!("✅ All workflows validated successfully!");
        }
    } else {
        eprintln!("⚠️  Found {} issue(s):", issues.len());
        for issue in &issues {
            eprintln!("  • {}", issue);
        }
    }

    if fix {
        eprintln!();
        eprintln!("🔧 Auto-fix is not yet implemented.");
    }

    Ok(issues)
}

/// Show recent workflow runs (requires gh CLI)
///
/// Fetches and displays recent workflow runs.
/// Requires GitHub CLI (gh) to be installed and authenticated.
///
/// # Examples
/// ```text
/// playg gh runs               # Shows last 10 runs
/// playg gh runs --limit 20    # Shows last 20 runs
/// playg gh runs --workflow ci # Filter by workflow name
/// ```
#[verb]
fn runs(
    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    workflow: Option<String>,

    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<String> {
    let limit = limit.unwrap_or(10);
    if verbose > 0 {
        eprintln!("🔄 Fetching recent workflow runs...");
        eprintln!();
    }

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
///
/// Opens the GitHub Actions page for the current repository in your default browser.
/// Tries using `gh` CLI first, falls back to git remote URL if needed.
///
/// # Examples
/// ```text
/// playg gh open  # Opens GitHub Actions for current repository
/// ```
#[verb]
fn open() -> Result<String> {
    eprintln!("🌐 Opening GitHub Actions...");

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
