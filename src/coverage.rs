//! Coverage Analysis
//!
//! Provides test coverage analysis and reporting.

use std::collections::HashMap;

/// Coverage report
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Total items
    pub total: usize,
    /// Covered items
    pub covered: usize,
    /// Coverage percentage
    pub percentage: f64,
    /// Coverage details
    pub details: HashMap<String, bool>,
}

impl CoverageReport {
    /// Create new coverage report
    pub fn new() -> Self {
        Self {
            total: 0,
            covered: 0,
            percentage: 0.0,
            details: HashMap::new(),
        }
    }

    /// Add coverage item
    pub fn add_item(&mut self, name: String, covered: bool) {
        self.details.insert(name.clone(), covered);
        self.total += 1;
        if covered {
            self.covered += 1;
        }
        self.percentage = (self.covered as f64 / self.total as f64) * 100.0;
    }

    /// Generate markdown report
    pub fn generate_markdown(&self) -> String {
        format!(
            "# Coverage Report\n\n**Coverage**: {:.2}% ({} / {})\n\n## Details\n\n",
            self.percentage, self.covered, self.total
        )
    }
}

impl Default for CoverageReport {
    fn default() -> Self {
        Self::new()
    }
}
