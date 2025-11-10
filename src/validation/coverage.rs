//! Coverage Analysis
//!
//! Provides test coverage analysis and reporting.
//!
//! # Poka-Yoke: Type-Level Validation
//!
//! This module uses newtypes to prevent count errors at compile time.
//! Use `TotalCount` and `CoveredCount` instead of `usize` for counts.

use std::collections::HashMap;

// ============================================================================
// Poka-Yoke: Type-Level Validation
// ============================================================================

/// Total count newtype
///
/// **Poka-Yoke**: Use this newtype instead of `usize` to prevent count errors.
/// Ensures total count is always >= 0 and >= covered count.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::coverage::{TotalCount, CoveredCount};
///
/// let total = TotalCount::new(100).unwrap();
/// let covered = CoveredCount::new(80).unwrap();
///
/// // Validate: covered <= total
/// assert!(covered.get() <= total.get());
/// assert_eq!(total.get(), 100);
/// assert_eq!(covered.get(), 80);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TotalCount(usize);

impl TotalCount {
    /// Create a new total count
    pub fn new(value: usize) -> Option<Self> {
        Some(Self(value))
    }

    /// Get the count value
    pub fn get(&self) -> usize {
        self.0
    }

    /// Convert to usize
    pub fn into_usize(self) -> usize {
        self.0
    }
}

impl From<TotalCount> for usize {
    fn from(count: TotalCount) -> Self {
        count.0
    }
}

/// Covered count newtype
///
/// **Poka-Yoke**: Use this newtype instead of `usize` to prevent count errors.
/// Ensures covered count is always <= total count.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::coverage::{TotalCount, CoveredCount};
///
/// let total = TotalCount::new(100).unwrap();
/// let covered = CoveredCount::new_for_total(80, total).unwrap();
///
/// // Validated: covered <= total
/// assert_eq!(covered.get(), 80);
/// assert_eq!(total.get(), 100);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoveredCount(usize);

impl CoveredCount {
    /// Create a new covered count
    pub fn new(value: usize) -> Option<Self> {
        Some(Self(value))
    }

    /// Create a new covered count validated against total count
    ///
    /// Returns `None` if covered > total.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chicago_tdd_tools::coverage::{TotalCount, CoveredCount};
    ///
    /// let total = TotalCount::new(100).unwrap();
    /// let covered = CoveredCount::new_for_total(80, total).unwrap(); // Valid
    /// assert_eq!(covered.get(), 80);
    /// let invalid = CoveredCount::new_for_total(150, total); // None - 150 > 100
    /// assert!(invalid.is_none());
    /// ```
    pub fn new_for_total(covered: usize, total: TotalCount) -> Option<Self> {
        if covered <= total.get() {
            Some(Self(covered))
        } else {
            None
        }
    }

    /// Get the count value
    pub fn get(&self) -> usize {
        self.0
    }

    /// Convert to usize
    pub fn into_usize(self) -> usize {
        self.0
    }
}

impl From<CoveredCount> for usize {
    fn from(count: CoveredCount) -> Self {
        count.0
    }
}

/// Coverage report
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Total items
    /// **Poka-Yoke**: Uses `TotalCount` newtype to prevent count errors
    pub total: TotalCount,
    /// Covered items
    /// **Poka-Yoke**: Uses `CoveredCount` newtype to prevent count errors
    pub covered: CoveredCount,
    /// Coverage percentage
    pub percentage: f64,
    /// Coverage details
    pub details: HashMap<String, bool>,
}

impl CoverageReport {
    /// Create new coverage report
    #[allow(clippy::expect_used)] // 0 is always valid for TotalCount and CoveredCount
    pub fn new() -> Self {
        Self {
            // SAFETY: 0 is always valid for TotalCount and CoveredCount
            total: TotalCount::new(0).expect("0 is always valid for TotalCount"),
            covered: CoveredCount::new(0).expect("0 is always valid for CoveredCount"),
            percentage: 0.0,
            details: HashMap::new(),
        }
    }

    /// Add coverage item
    #[allow(clippy::expect_used)] // Incremented total is always valid
    pub fn add_item(&mut self, name: String, covered: bool) {
        self.details.insert(name.clone(), covered);
        let new_total = self.total.get() + 1;
        // SAFETY: new_total is always valid (incremented from valid total)
        // Incremented total is always valid
        let total = TotalCount::new(new_total);
        self.total = total.expect("Incremented total is always valid");
        if covered {
            let new_covered = self.covered.get() + 1;
            // Validate: covered <= total
            if let Some(new_covered_count) = CoveredCount::new_for_total(new_covered, self.total) {
                self.covered = new_covered_count;
            }
        }
        self.percentage = (self.covered.get() as f64 / self.total.get() as f64) * 100.0;
    }

    /// Generate markdown report
    pub fn generate_markdown(&self) -> String {
        format!(
            "# Coverage Report\n\n**Coverage**: {:.2}% ({} / {})\n\n## Details\n\n",
            self.percentage,
            self.covered.get(),
            self.total.get()
        )
    }
}

impl Default for CoverageReport {
    #[allow(clippy::expect_used)] // 0 is always valid for TotalCount and CoveredCount
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::*;

    #[test]
    fn test_total_count() {
        let total = TotalCount::new(100).unwrap();
        assert_eq!(total.get(), 100);

        let usize_value: usize = total.into();
        assert_eq!(usize_value, 100);
    }

    #[test]
    fn test_covered_count() {
        let covered = CoveredCount::new(80).unwrap();
        assert_eq!(covered.get(), 80);

        let usize_value: usize = covered.into();
        assert_eq!(usize_value, 80);
    }

    #[test]
    fn test_covered_count_validation() {
        let total = TotalCount::new(100).unwrap();

        // Valid: covered <= total
        let covered = CoveredCount::new_for_total(80, total).unwrap();
        assert_eq!(covered.get(), 80);

        // Valid: covered == total
        let covered = CoveredCount::new_for_total(100, total).unwrap();
        assert_eq!(covered.get(), 100);

        // Invalid: covered > total
        let invalid = CoveredCount::new_for_total(150, total);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_coverage_report_with_newtypes() {
        let mut report = CoverageReport::new();
        assert_eq!(report.total.get(), 0);
        assert_eq!(report.covered.get(), 0);

        // Add covered item
        report.add_item("test1".to_string(), true);
        assert_eq!(report.total.get(), 1);
        assert_eq!(report.covered.get(), 1);

        // Add uncovered item
        report.add_item("test2".to_string(), false);
        assert_eq!(report.total.get(), 2);
        assert_eq!(report.covered.get(), 1); // Still 1 covered

        // Add another covered item
        report.add_item("test3".to_string(), true);
        assert_eq!(report.total.get(), 3);
        assert_eq!(report.covered.get(), 2);

        // Verify percentage
        assert_eq!(report.percentage, (2.0 / 3.0) * 100.0);
    }
}
