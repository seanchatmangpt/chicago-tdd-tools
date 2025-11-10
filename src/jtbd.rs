//! JTBD (Jobs To Be Done) Validation Framework
//!
//! Validates that code accomplishes its **intended purpose** in real-world scenarios,
//! not just returns technical success. This is the core principle: "Validate that code
//! does the job it's supposed to do, not just that it executes without errors."
//!
//! Based on DFLSS research document: RESEARCH_REFLEX_WORKFLOW_JTBD_INNOVATION.md
//!
//! JTBD validation ensures:
//! 1. Code executes in real contexts (not isolated unit tests)
//! 2. Code accomplishes its intended purpose (JTBD validation)
//! 3. Results are validated against expected behavior (state-based validation)
//! 4. OTEL telemetry reflects actual work (observability validation)
//!
//! ## Chicago TDD Principles Applied to JTBD
//!
//! 1. **State-based tests**: Verify outputs and state, not implementation details
//! 2. **Real collaborators**: Use actual dependencies, not mocks
//! 3. **End-to-end validation**: Complete workflows from execution to analysis
//! 4. **JTBD focus**: Validate actual use cases, not just technical integration
//!
//! ## Usage
//!
//! ```rust,no_run
//! use chicago_tdd_tools::jtbd::{JtbdValidator, JtbdScenario, JtbdValidationResult};
//!
//! // Create validator
//! let mut validator = JtbdValidator::new();
//!
//! // Register scenario
//! validator.register_scenario(JtbdScenario {
//!     name: "Order Processing".to_string(),
//!     setup_context: Box::new(|| create_test_context()),
//!     validate_result: Box::new(|ctx, result| {
//!         // Validate that order was actually processed
//!         result.success && result.variables.contains_key("order_id")
//!     }),
//!     expected_behavior: "Process order and update state".to_string(),
//! });
//!
//! // Validate
//! let results = validator.validate_all();
//! assert!(results.iter().all(|r| r.jtbd_success));
//! ```

use std::collections::HashMap;

/// JTBD validation result for a single scenario
#[derive(Debug, Clone)]
pub struct JtbdValidationResult {
    /// Scenario name
    pub scenario_name: String,
    /// Whether execution succeeded
    pub execution_success: bool,
    /// Whether JTBD validation passed (code accomplished intended purpose)
    pub jtbd_success: bool,
    /// Execution latency in milliseconds
    pub latency_ms: u64,
    /// Validation details
    pub details: Vec<String>,
    /// Expected behavior description
    pub expected_behavior: String,
    /// Actual behavior description
    pub actual_behavior: String,
}

impl JtbdValidationResult {
    /// Create a successful JTBD validation result
    pub fn success(scenario_name: String, latency_ms: u64, details: Vec<String>) -> Self {
        Self {
            scenario_name,
            execution_success: true,
            jtbd_success: true,
            latency_ms,
            details,
            expected_behavior: String::new(),
            actual_behavior: String::new(),
        }
    }

    /// Create a failed JTBD validation result
    pub fn failure(
        scenario_name: String,
        execution_success: bool,
        expected_behavior: String,
        actual_behavior: String,
        details: Vec<String>,
    ) -> Self {
        Self {
            scenario_name,
            execution_success,
            jtbd_success: false,
            latency_ms: 0,
            details,
            expected_behavior,
            actual_behavior,
        }
    }
}

/// Generic execution context (can be extended for specific use cases)
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Variables for execution
    pub variables: HashMap<String, String>,
    /// Additional context data
    pub metadata: HashMap<String, String>,
}

/// Generic execution result (can be extended for specific use cases)
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Output variables
    pub variables: HashMap<String, String>,
    /// Additional result data
    pub metadata: HashMap<String, String>,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn ok(variables: HashMap<String, String>) -> Self {
        Self {
            success: true,
            variables,
            metadata: HashMap::new(),
        }
    }

    /// Create a failed result
    pub fn err(message: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("error".to_string(), message);
        Self {
            success: false,
            variables: HashMap::new(),
            metadata,
        }
    }
}

/// JTBD validation scenario
pub struct JtbdScenario {
    /// Scenario name
    pub name: String,
    /// Setup function to create execution context
    pub setup_context: Box<dyn Fn() -> ExecutionContext + Send + Sync>,
    /// Execution function (returns execution result)
    pub execute: Box<dyn Fn(&ExecutionContext) -> ExecutionResult + Send + Sync>,
    /// Validation function to check if result accomplishes intended purpose
    pub validate_result: Box<dyn Fn(&ExecutionContext, &ExecutionResult) -> bool + Send + Sync>,
    /// Expected behavior description
    pub expected_behavior: String,
}

/// JTBD validator
pub struct JtbdValidator {
    /// JTBD scenarios
    scenarios: Vec<JtbdScenario>,
}

impl JtbdValidator {
    /// Create a new JTBD validator
    pub fn new() -> Self {
        Self {
            scenarios: Vec::new(),
        }
    }

    /// Register a JTBD scenario
    pub fn register_scenario(&mut self, scenario: JtbdScenario) {
        self.scenarios.push(scenario);
    }

    /// Validate a single scenario's JTBD
    pub fn validate_scenario(&self, index: usize) -> Option<JtbdValidationResult> {
        let scenario = self.scenarios.get(index)?;

        // Setup execution context
        let context = (scenario.setup_context)();

        // Execute
        let start_time = std::time::Instant::now();
        let execution_result = (scenario.execute)(&context);
        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Validate JTBD: Does the code accomplish its intended purpose?
        let jtbd_valid = (scenario.validate_result)(&context, &execution_result);

        if execution_result.success && jtbd_valid {
            Some(JtbdValidationResult::success(
                scenario.name.clone(),
                latency_ms,
                vec![format!(
                    "Scenario '{}' executed successfully and accomplished intended purpose",
                    scenario.name
                )],
            ))
        } else {
            let details = if !execution_result.success {
                vec!["Execution failed".to_string()]
            } else {
                vec!["Execution succeeded but did not accomplish intended purpose".to_string()]
            };

            Some(JtbdValidationResult::failure(
                scenario.name.clone(),
                execution_result.success,
                scenario.expected_behavior.clone(),
                format!(
                    "Execution: {}, JTBD: {}",
                    execution_result.success, jtbd_valid
                ),
                details,
            ))
        }
    }

    /// Validate all registered scenarios
    pub fn validate_all(&self) -> Vec<JtbdValidationResult> {
        let mut results = Vec::new();

        for i in 0..self.scenarios.len() {
            if let Some(result) = self.validate_scenario(i) {
                results.push(result);
            }
        }

        results
    }

    /// Get validation summary
    pub fn get_summary(&self, results: &[JtbdValidationResult]) -> JtbdValidationSummary {
        let total = results.len();
        let execution_passed = results.iter().filter(|r| r.execution_success).count();
        let jtbd_passed = results.iter().filter(|r| r.jtbd_success).count();
        let execution_failed = total - execution_passed;
        let jtbd_failed = execution_passed - jtbd_passed;

        JtbdValidationSummary {
            total_scenarios: total,
            execution_passed,
            execution_failed,
            jtbd_passed,
            jtbd_failed,
            avg_latency_ms: if !results.is_empty() {
                results.iter().map(|r| r.latency_ms).sum::<u64>() / total as u64
            } else {
                0
            },
        }
    }
}

impl Default for JtbdValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// JTBD validation summary
#[derive(Debug, Clone)]
pub struct JtbdValidationSummary {
    /// Total scenarios validated
    pub total_scenarios: usize,
    /// Scenarios that executed successfully
    pub execution_passed: usize,
    /// Scenarios that failed execution
    pub execution_failed: usize,
    /// Scenarios that accomplished their intended purpose (JTBD)
    pub jtbd_passed: usize,
    /// Scenarios that executed but didn't accomplish intended purpose
    pub jtbd_failed: usize,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
}

impl JtbdValidationSummary {
    /// Check if all scenarios passed JTBD validation
    pub fn all_passed(&self) -> bool {
        self.execution_passed == self.total_scenarios && self.jtbd_passed == self.total_scenarios
    }

    /// Get pass rate (0.0 to 1.0)
    pub fn pass_rate(&self) -> f64 {
        if self.total_scenarios == 0 {
            return 0.0;
        }
        self.jtbd_passed as f64 / self.total_scenarios as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jtbd_validator_creation() {
        let validator = JtbdValidator::new();
        assert_eq!(validator.scenarios.len(), 0);
    }

    #[test]
    fn test_jtbd_scenario_registration() {
        let mut validator = JtbdValidator::new();

        validator.register_scenario(JtbdScenario {
            name: "Test Scenario".to_string(),
            setup_context: Box::new(|| ExecutionContext::default()),
            execute: Box::new(|_ctx| ExecutionResult::ok(HashMap::new())),
            validate_result: Box::new(|_ctx, result| result.success),
            expected_behavior: "Should succeed".to_string(),
        });

        assert_eq!(validator.scenarios.len(), 1);
    }

    #[test]
    fn test_jtbd_validation_summary() {
        let summary = JtbdValidationSummary {
            total_scenarios: 10,
            execution_passed: 10,
            execution_failed: 0,
            jtbd_passed: 10,
            jtbd_failed: 0,
            avg_latency_ms: 5,
        };

        assert!(summary.all_passed());
        assert_eq!(summary.pass_rate(), 1.0);
    }
}
