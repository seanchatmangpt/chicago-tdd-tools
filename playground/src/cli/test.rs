//! Test noun commands
//!
//! Demonstrates clap-noun-verb best practices through testing feature examples.
//! Commands for testing features: property, mutation, snapshot, concurrency, cli, generator, parameterized

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::testing;

// ============================================================================
// Output Types (all implement Serialize for JSON serialization)
// ============================================================================

#[derive(Serialize)]
pub struct TestFeatureStatus {
    /// Available testing features
    pub features: Vec<String>,
    /// Available example demonstrations
    pub examples: Vec<String>,
}

#[derive(Serialize)]
pub struct TestExecutionResult {
    /// Names of examples that executed successfully
    pub executed: Vec<String>,
    /// Whether all examples executed without errors
    pub success: bool,
    /// Summary message about execution
    pub message: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Collect available test features based on enabled crate features
fn collect_test_features() -> Vec<String> {
    let mut features = vec!["gen".to_string()];

    #[cfg(feature = "property-testing")]
    features.push("prop".to_string());
    #[cfg(feature = "mutation-testing")]
    features.push("mut".to_string());
    #[cfg(feature = "snapshot-testing")]
    features.push("snap".to_string());
    #[cfg(feature = "concurrency-testing")]
    features.push("conc".to_string());
    #[cfg(feature = "cli-testing")]
    features.push("cli".to_string());
    #[cfg(feature = "parameterized-testing")]
    features.push("param".to_string());

    features
}

// ============================================================================
// Verb Handlers (automatically registered by #[verb] macro)
// ============================================================================

/// Show testing features status
///
/// Displays information about all available testing features and examples.
/// Use -v for basic verbose output, -vv for detailed information.
///
/// # Examples
/// ```text
/// playg test stat           # Shows features in JSON format
/// playg test stat -v        # Shows features with verbose output
/// playg test stat --format yaml  # Shows features in YAML format
/// ```
#[verb]
fn stat(
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<TestFeatureStatus> {
    if verbose > 0 {
        eprintln!("📋 Testing Features Status");
    }

    let features = collect_test_features();
    Ok(TestFeatureStatus {
        examples: features.clone(),
        features,
    })
}

/// List available test examples
///
/// Shows all test example modules that can be executed with `playg test exec`.
#[verb]
fn list() -> Result<Vec<String>> {
    Ok(collect_test_features())
}

/// Execute multiple test examples
///
/// Run testing feature examples by name. You can execute multiple examples in one command.
///
/// # Arguments
/// * `names` - Space-separated example names (e.g., "gen prop snap")
///
/// # Options
/// * `-o, --output` - Optional output file for results
/// * `-v, --verbose` - Increase verbosity level
///
/// # Examples
/// ```text
/// playg test exec "gen"
/// playg test exec "gen prop snap"
/// playg test exec "gen prop" --output results.json
/// playg test exec "mut" -vv
/// ```
#[verb]
fn exec(
    #[arg(index = 0, value_name = "NAMES")]
    names: String,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<TestExecutionResult> {
    let mut executed = Vec::new();
    let mut errors = Vec::new();

    if verbose > 0 {
        eprintln!("🚀 Executing test examples...");
    }

    let name_list: Vec<String> = names.split_whitespace().map(|s| s.to_string()).collect();
    for name in name_list {
        if verbose > 1 {
            eprintln!("  Running: {}", name);
        }

        match execute_test_example(&name) {
            Ok(_) => {
                executed.push(name.clone());
                if verbose > 0 {
                    eprintln!("  ✅ {}", name);
                }
            }
            Err(e) => {
                errors.push(format!("{}: {}", name, e));
                if verbose > 0 {
                    eprintln!("  ❌ Error: {}", e);
                }
            }
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Executed {} example(s) successfully", executed.len())
    } else {
        format!("Executed {} example(s), {} error(s)", executed.len(), errors.len())
    };

    if verbose > 0 {
        eprintln!();
        eprintln!("📊 Summary: {}", message);
    }

    Ok(TestExecutionResult {
        executed,
        success,
        message,
    })
}

/// Information about testing patterns and best practices
#[derive(Serialize)]
pub struct TestingGuidanceInfo {
    /// The command to run this guidance
    pub command: String,
    /// Description of what this guidance covers
    pub description: String,
    /// Ordered steps for the guidance
    pub steps: Vec<String>,
    /// Key principles to follow
    pub key_principles: Vec<String>,
}

/// Expert testing patterns guidance
///
/// Advanced testing patterns and techniques for writing more effective tests.
/// Learn professional testing patterns that improve code quality and maintainability.
#[verb]
fn expert() -> Result<TestingGuidanceInfo> {
    Ok(TestingGuidanceInfo {
        command: "test expert".to_string(),
        description: "Expert Testing Patterns".to_string(),
        steps: vec![
            "Step 1: Understand Test Patterns - What patterns exist for different scenarios?".to_string(),
            "Step 2: Choose Appropriate Pattern - Select pattern that fits your test".to_string(),
            "Step 3: Apply Pattern Consistently - Use pattern across similar tests".to_string(),
            "Step 4: Verify Clarity - Ensure test intent is clear to readers".to_string(),
            "Step 5: Optimize for Maintenance - Make tests easy to update".to_string(),
        ],
        key_principles: vec![
            "Arrange-Act-Assert - Structure tests with clear sections".to_string(),
            "Test behavior, not implementation - Tests shouldn't change when implementation does".to_string(),
            "One assertion per test - Each test validates one behavior".to_string(),
            "Descriptive names - Test names explain what is being tested".to_string(),
            "DRY principle - Extract common test setup and utilities".to_string(),
        ],
    })
}

/// Verify tests workflow guidance
///
/// Systematic approach to verifying that tests are effective and working correctly.
/// Follow these steps to ensure test quality and effectiveness.
#[verb]
fn verify() -> Result<TestingGuidanceInfo> {
    Ok(TestingGuidanceInfo {
        command: "test verify".to_string(),
        description: "Verify Tests".to_string(),
        steps: vec![
            "Step 1: Run All Tests - Execute full test suite".to_string(),
            "Step 2: Check Coverage - Verify code coverage is adequate".to_string(),
            "Step 3: Test Mutation - Use mutation testing to verify test quality".to_string(),
            "Step 4: Test Failure Modes - Verify tests fail when code is wrong".to_string(),
            "Step 5: Performance Checks - Ensure tests run in acceptable time".to_string(),
            "Step 6: Flakiness Review - Identify and fix flaky tests".to_string(),
        ],
        key_principles: vec![
            "Tests must fail - If code breaks, tests should fail".to_string(),
            "Coverage metrics - Aim for high coverage (>80%)".to_string(),
            "Mutation testing - Good tests catch most mutations".to_string(),
            "Deterministic - Tests should have no random failures".to_string(),
            "Fast feedback - Tests should run quickly for rapid iteration".to_string(),
        ],
    })
}

fn execute_test_example(name: &str) -> std::result::Result<(), String> {
    match name {
        "gen" => {
            testing::generator::example_test_generator();
            testing::generator::example_compile_time_array();
            testing::generator::example_array_pattern();
            Ok(())
        }
        #[cfg(feature = "property-testing")]
        "prop" => {
            testing::property::example_property_generator();
            testing::property::example_property_validation();
            Ok(())
        }
        #[cfg(feature = "mutation-testing")]
        "mut" => {
            testing::mutation::example_mutation_test();
            Ok(())
        }
        #[cfg(feature = "snapshot-testing")]
        "snap" => {
            testing::snapshot::example_snapshot_test();
            Ok(())
        }
        #[cfg(feature = "concurrency-testing")]
        "conc" => {
            testing::concurrency::example_concurrency_test();
            Ok(())
        }
        #[cfg(feature = "cli-testing")]
        "cli" => {
            testing::cli::example_cli_test();
            Ok(())
        }
        #[cfg(feature = "parameterized-testing")]
        "param" => {
            testing::parameterized::example_parameterized_test();
            Ok(())
        }
        _ => {
            #[cfg(not(feature = "property-testing"))]
            if name == "prop" {
                return Err("Property testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "mutation-testing"))]
            if name == "mut" {
                return Err("Mutation testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "snapshot-testing"))]
            if name == "snap" {
                return Err("Snapshot testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "concurrency-testing"))]
            if name == "conc" {
                return Err("Concurrency testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "cli-testing"))]
            if name == "cli" {
                return Err("CLI testing feature not enabled".to_string());
            }
            #[cfg(not(feature = "parameterized-testing"))]
            if name == "param" {
                return Err("Parameterized testing feature not enabled".to_string());
            }
            Err(format!("Unknown example: {}", name))
        }
    }
}

