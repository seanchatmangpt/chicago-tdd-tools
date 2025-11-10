//! Test Data Builders
//!
//! Provides fluent builders for creating test data structures.
//! Aligned with workflow engine's TestDataBuilder API for consistency.

use serde_json::Value;
use std::collections::HashMap;

/// Builder for test data (case variables)
///
/// This builder creates test data as `HashMap<String, String>` and can convert to JSON.
/// Provides a fluent API for building test data structures.
pub struct TestDataBuilder {
    data: HashMap<String, String>,
}

impl TestDataBuilder {
    /// Create a new test data builder
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Add a variable
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Add order data (common business scenario)
    pub fn with_order_data(
        mut self,
        order_id: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        self.data.insert("order_id".to_string(), order_id.into());
        self.data.insert("total_amount".to_string(), amount.into());
        self.data.insert("currency".to_string(), "USD".to_string());
        self.data
            .insert("order_status".to_string(), "pending".to_string());
        self
    }

    /// Add customer data
    pub fn with_customer_data(mut self, customer_id: impl Into<String>) -> Self {
        self.data
            .insert("customer_id".to_string(), customer_id.into());
        self.data.insert(
            "customer_email".to_string(),
            "customer@example.com".to_string(),
        );
        self
    }

    /// Add approval data
    pub fn with_approval_data(
        mut self,
        request_id: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        self.data
            .insert("request_id".to_string(), request_id.into());
        self.data.insert("amount".to_string(), amount.into());
        self.data
            .insert("condition".to_string(), "true".to_string());
        self
    }

    /// Build test data as JSON
    ///
    /// Converts `HashMap<String, String>` to `serde_json::Value`.
    /// Matches workflow engine API exactly.
    pub fn build_json(self) -> Value {
        serde_json::to_value(&self.data).unwrap_or(serde_json::json!({}))
    }

    /// Build test data as HashMap
    ///
    /// Returns the underlying `HashMap<String, String>`.
    /// Matches workflow engine API exactly.
    pub fn build(self) -> HashMap<String, String> {
        self.data
    }
}

impl Default for TestDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
