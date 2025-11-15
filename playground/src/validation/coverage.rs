//! Coverage Examples
//!
//! Demonstrates test coverage analysis and reporting, including comprehensive newtype usage.

use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::validation::coverage::*;

/// Example: Basic coverage report
pub fn example_coverage_basic() {
    // Arrange: Create coverage report
    let mut report = CoverageReport::new();

    // Act: Add coverage items
    report.add_item("function1".to_string(), true);
    report.add_item("function2".to_string(), true);
    report.add_item("function3".to_string(), false);

    // Assert: Verify coverage calculated
    assert_eq!(report.total.get(), 3);
    assert_eq!(report.covered.get(), 2);
    assert_eq!(report.percentage.get(), 66.67);
}

/// Example: Coverage with newtypes
pub fn example_coverage_newtypes() {
    // Arrange: Create counts with newtypes
    let total = TotalCount::new(100).unwrap();
    let covered = CoveredCount::new_for_total(80, total).unwrap();

    // Act-Assert: Verify type-safe counts
    assert_eq!(total.get(), 100);
    assert_eq!(covered.get(), 80);
    assert!(covered.get() <= total.get());
}

/// Example: CoveragePercentage newtype usage
pub fn example_coverage_percentage() {
    // Arrange: Create coverage percentage
    let total = TotalCount::new(100).unwrap();
    let covered = CoveredCount::new_for_total(75, total).unwrap();

    // Act: Calculate percentage using newtype
    let percentage = CoveragePercentage::from_counts(covered, total).unwrap();

    // Assert: Verify percentage calculated correctly
    assert_eq!(percentage.get(), 75.0);
    assert!(percentage.get() >= 0.0);
    assert!(percentage.get() <= 100.0);
}

/// Example: CoveragePercentage validation
pub fn example_coverage_percentage_validation() {
    // Arrange: Create valid percentage
    let percentage = CoveragePercentage::new(85.5).unwrap();

    // Act-Assert: Verify percentage is valid
    assert_eq!(percentage.get(), 85.5);
    assert!(percentage.get() >= 0.0);
    assert!(percentage.get() <= 100.0);

    // Invalid percentage returns None
    let invalid = CoveragePercentage::new(150.0);
    assert!(invalid.is_none());
}

/// Example: Coverage markdown report
pub fn example_coverage_markdown() {
    // Arrange: Create coverage report
    let mut report = CoverageReport::new();
    report.add_item("function1".to_string(), true);
    report.add_item("function2".to_string(), false);

    // Act: Generate markdown
    let markdown = report.generate_markdown();

    // Assert: Verify markdown generated
    assert!(markdown.contains("function1"));
    assert!(markdown.contains("function2"));
    assert!(markdown.contains("Coverage"));
}

/// Example: Coverage report with newtype conversions
pub fn example_coverage_conversions() {
    // Arrange: Create counts
    let total = TotalCount::new(50).unwrap();
    let covered = CoveredCount::new_for_total(40, total).unwrap();

    // Act: Convert to usize
    let total_usize: usize = total.into();
    let covered_usize: usize = covered.into();

    // Assert: Verify conversions work
    assert_eq!(total_usize, 50);
    assert_eq!(covered_usize, 40);
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_coverage_basic, {
        // Arrange-Act-Assert: Run example
        example_coverage_basic();
    });

    test!(test_coverage_newtypes, {
        // Arrange-Act-Assert: Run example
        example_coverage_newtypes();
    });

    test!(test_coverage_percentage, {
        // Arrange-Act-Assert: Run example
        example_coverage_percentage();
    });

    test!(test_coverage_percentage_validation, {
        // Arrange-Act-Assert: Run example
        example_coverage_percentage_validation();
    });

    test!(test_coverage_markdown, {
        // Arrange-Act-Assert: Run example
        example_coverage_markdown();
    });

    test!(test_coverage_conversions, {
        // Arrange-Act-Assert: Run example
        example_coverage_conversions();
    });
}
