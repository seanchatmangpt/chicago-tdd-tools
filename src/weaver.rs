//! Weaver Live Validation Integration
//!
//! Provides integration with Weaver live-check for runtime telemetry validation.
//! Ensures all OTEL spans and metrics conform to declared schema.

#[cfg(feature = "weaver")]
use crate::weaver_types::WeaverLiveCheck;
use std::path::PathBuf;
use std::process::Child;
use thiserror::Error;

/// Weaver validation error
#[derive(Error, Debug)]
pub enum WeaverValidationError {
    /// Weaver binary not found
    #[error("Weaver binary not found in PATH. Install with: ./scripts/install-weaver.sh")]
    BinaryNotFound,
    /// Weaver check failed
    #[error("Weaver validation failed: {0}")]
    ValidationFailed(String),
    /// Registry path does not exist
    #[error("Registry path does not exist: {0}")]
    RegistryNotFound(String),
    /// Failed to start Weaver process
    #[error("Failed to start Weaver process: {0}")]
    ProcessStartFailed(String),
    /// Failed to stop Weaver process
    #[error("Failed to stop Weaver process: {0}")]
    ProcessStopFailed(String),
    /// Weaver process not running
    #[error("Weaver process is not running")]
    ProcessNotRunning,
}

/// Result type for Weaver validation
pub type WeaverValidationResult<T> = Result<T, WeaverValidationError>;

/// Weaver live validation helper
#[cfg(feature = "weaver")]
pub struct WeaverValidator {
    live_check: Option<WeaverLiveCheck>,
    process: Option<Child>,
    registry_path: PathBuf,
    otlp_grpc_port: u16,
    admin_port: u16,
}

#[cfg(feature = "weaver")]
impl WeaverValidator {
    /// Create a new Weaver validator
    pub fn new(registry_path: PathBuf) -> Self {
        Self {
            live_check: None,
            process: None,
            registry_path,
            otlp_grpc_port: 4317,
            admin_port: 8080,
        }
    }

    /// Create a Weaver validator with custom configuration
    pub fn with_config(registry_path: PathBuf, otlp_grpc_port: u16, admin_port: u16) -> Self {
        Self {
            live_check: None,
            process: None,
            registry_path,
            otlp_grpc_port,
            admin_port,
        }
    }

    /// Check if Weaver binary is available
    pub fn check_weaver_available() -> WeaverValidationResult<()> {
        WeaverLiveCheck::check_weaver_available().map_err(|_| WeaverValidationError::BinaryNotFound)
    }

    /// Start Weaver live-check
    pub fn start(&mut self) -> WeaverValidationResult<()> {
        // Check Weaver binary availability
        Self::check_weaver_available()?;

        // Verify registry path exists
        if !self.registry_path.exists() {
            return Err(WeaverValidationError::RegistryNotFound(
                self.registry_path.display().to_string(),
            ));
        }

        // Create Weaver live-check instance
        let registry_str = self.registry_path.to_str().ok_or_else(|| {
            WeaverValidationError::ValidationFailed("Registry path is not valid UTF-8".to_string())
        })?;

        let live_check = WeaverLiveCheck::new()
            .with_registry(registry_str.to_string())
            .with_otlp_port(self.otlp_grpc_port)
            .with_admin_port(self.admin_port)
            .with_inactivity_timeout(300) // 5 minutes
            .with_format("json".to_string())
            .with_output("./weaver-reports".to_string());

        // Start Weaver live-check process
        let process = live_check
            .start()
            .map_err(|e| WeaverValidationError::ProcessStartFailed(e))?;

        self.live_check = Some(live_check);
        self.process = Some(process);

        Ok(())
    }

    /// Stop Weaver live-check
    pub fn stop(&mut self) -> WeaverValidationResult<()> {
        if let Some(ref live_check) = self.live_check {
            live_check
                .stop()
                .map_err(|e| WeaverValidationError::ProcessStopFailed(e))?;
        }

        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }

        self.live_check = None;
        Ok(())
    }

    /// Get OTLP endpoint for sending telemetry
    pub fn otlp_endpoint(&self) -> String {
        format!("http://{}:{}", "127.0.0.1", self.otlp_grpc_port)
    }

    /// Check if Weaver process is running
    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}

#[cfg(feature = "weaver")]
impl Drop for WeaverValidator {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Run Weaver static schema validation
///
/// Validates that schema files are valid without running live-check.
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::weaver::validate_schema_static;
/// use std::path::PathBuf;
///
/// let registry_path = PathBuf::from("registry/");
/// validate_schema_static(&registry_path)?;
/// ```
#[cfg(feature = "weaver")]
pub fn validate_schema_static(registry_path: &PathBuf) -> WeaverValidationResult<()> {
    use std::process::Command;

    // Check Weaver binary availability
    WeaverValidator::check_weaver_available()?;

    // Verify registry path exists
    if !registry_path.exists() {
        return Err(WeaverValidationError::RegistryNotFound(
            registry_path.display().to_string(),
        ));
    }

    // Run weaver registry check
    let output = Command::new("weaver")
        .args(["registry", "check", "-r", registry_path.to_str().unwrap()])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                WeaverValidationError::BinaryNotFound
            } else {
                WeaverValidationError::ValidationFailed(format!(
                    "Failed to execute weaver check: {}",
                    e
                ))
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WeaverValidationError::ValidationFailed(format!(
            "Weaver schema validation failed: {}",
            stderr
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "weaver")]
    #[test]
    fn test_weaver_validator_new() {
        let registry_path = PathBuf::from("registry/");
        let validator = WeaverValidator::new(registry_path);
        assert_eq!(validator.otlp_grpc_port, 4317);
        assert_eq!(validator.admin_port, 8080);
    }

    #[cfg(feature = "weaver")]
    #[test]
    fn test_weaver_validator_with_config() {
        let registry_path = PathBuf::from("registry/");
        let validator = WeaverValidator::with_config(registry_path, 4318, 8081);
        assert_eq!(validator.otlp_grpc_port, 4318);
        assert_eq!(validator.admin_port, 8081);
    }

    #[cfg(feature = "weaver")]
    #[test]
    fn test_weaver_validator_otlp_endpoint() {
        let registry_path = PathBuf::from("registry/");
        let validator = WeaverValidator::new(registry_path);
        assert_eq!(validator.otlp_endpoint(), "http://127.0.0.1:4317");
    }
}
