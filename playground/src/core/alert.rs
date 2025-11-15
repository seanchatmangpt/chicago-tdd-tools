//! Alert Examples
//!
//! Demonstrates alert helpers for visual problem indicators.

use chicago_tdd_tools::prelude::*;

/// Example: Alert macros (macros are available via prelude)
pub fn example_alert_macros() {
    // Arrange: Alert macros for visual indicators

    // Act: Emit alerts
    alert_critical!("Critical issue detected", "Fix immediately");
    alert_warning!("Warning condition", "Review and fix");
    alert_info!("Informational message");
    alert_success!("Operation completed successfully");
    alert_debug!("Debug information");
    alert!("custom", "Custom alert", false, "");
}

#[cfg(feature = "logging")]
/// Example: Alert logger integration
pub fn example_alert_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Initialize alert logger
    AlertLogger::init_default()?;

    // Act: Use log macros with alert format
    log::error!("Error log message");
    log::warn!("Warning log message");
    log::info!("Info log message");

    // Assert: Logger initialized
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_alert_macros, {
        // Arrange-Act-Assert: Run example
        example_alert_macros();
    });

    #[cfg(feature = "logging")]
    test!(test_alert_logger, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_alert_logger());
    });
}
