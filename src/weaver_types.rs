//! Weaver Live-Check Types
//!
//! WeaverLiveCheck implementation ported from knhk-otel for standalone use.
//! Used for live validation of OpenTelemetry telemetry against semantic conventions.

use std::process::Child;
use thiserror::Error;

/// Weaver validation error
#[derive(Error, Debug)]
pub enum WeaverValidationError {
    /// Weaver binary not found in PATH
    #[error("Weaver binary not found: {0}")]
    BinaryNotFound(String),
    /// Weaver health check failed
    #[error("Weaver health check failed: {0}")]
    HealthCheckFailed(String),
    /// Failed to start Weaver process
    #[error("Failed to start Weaver: {0}")]
    StartFailed(String),
    /// Failed to stop Weaver process
    #[error("Failed to stop Weaver: {0}")]
    StopFailed(String),
}

/// Weaver live-check integration for telemetry validation
pub struct WeaverLiveCheck {
    registry_path: Option<String>,
    otlp_grpc_address: String,
    otlp_grpc_port: u16,
    admin_port: u16,
    inactivity_timeout: u64,
    format: String,
    output: Option<String>,
}

impl WeaverLiveCheck {
    /// Create a new Weaver live-check instance
    pub fn new() -> Self {
        Self {
            registry_path: None,
            otlp_grpc_address: "127.0.0.1".to_string(),
            otlp_grpc_port: 4317,
            admin_port: 8080,
            inactivity_timeout: 60,
            format: "json".to_string(),
            output: None,
        }
    }

    /// Set the semantic convention registry path
    pub fn with_registry(mut self, registry_path: String) -> Self {
        self.registry_path = Some(registry_path);
        self
    }

    /// Set the OTLP gRPC address
    pub fn with_otlp_address(mut self, address: String) -> Self {
        self.otlp_grpc_address = address;
        self
    }

    /// Set the OTLP gRPC port
    pub fn with_otlp_port(mut self, port: u16) -> Self {
        self.otlp_grpc_port = port;
        self
    }

    /// Set the admin HTTP port
    pub fn with_admin_port(mut self, port: u16) -> Self {
        self.admin_port = port;
        self
    }

    /// Set the inactivity timeout in seconds
    pub fn with_inactivity_timeout(mut self, timeout: u64) -> Self {
        self.inactivity_timeout = timeout;
        self
    }

    /// Set the output format (json, ansi)
    pub fn with_format(mut self, format: String) -> Self {
        self.format = format;
        self
    }

    /// Set the output directory (for JSON reports)
    pub fn with_output(mut self, output: String) -> Self {
        self.output = Some(output);
        self
    }

    /// Check if Weaver binary is available in PATH
    pub fn check_weaver_available() -> Result<(), String> {
        use std::process::Command;

        // Try to run weaver --version to check if it exists
        match Command::new("weaver").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err("Weaver binary found but --version failed".to_string())
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err("Weaver binary not found in PATH. Install with: ./scripts/install-weaver.sh or cargo install weaver".to_string())
                } else {
                    Err(format!("Failed to check Weaver binary: {}", e))
                }
            }
        }
    }

    /// Check Weaver health by querying the admin endpoint
    pub fn check_health(&self) -> Result<bool, String> {
        // Note: This requires reqwest, which may not be available
        // For now, return a basic connectivity check
        match std::net::TcpStream::connect(format!(
            "{}:{}",
            self.otlp_grpc_address, self.admin_port
        )) {
            Ok(_) => Ok(true), // Port is open, assume Weaver is running
            Err(e) => Err(format!(
                "Weaver admin endpoint not responding on {}:{}: {}",
                self.otlp_grpc_address, self.admin_port, e
            )),
        }
    }

    /// Run live-check and return the process handle
    /// The caller should send telemetry to the configured OTLP endpoint
    pub fn start(&self) -> Result<Child, String> {
        // Check Weaver binary availability first
        Self::check_weaver_available()?;
        use std::process::Command;

        let mut cmd = Command::new("weaver");

        cmd.args(["registry", "live-check"]);

        if let Some(ref registry) = self.registry_path {
            cmd.args(["--registry", registry]);
        }

        cmd.args(["--otlp-grpc-address", &self.otlp_grpc_address]);
        cmd.args(["--otlp-grpc-port", &self.otlp_grpc_port.to_string()]);
        cmd.args(["--admin-port", &self.admin_port.to_string()]);
        cmd.args(["--inactivity-timeout", &self.inactivity_timeout.to_string()]);
        cmd.args(["--format", &self.format]);

        if let Some(ref output) = self.output {
            cmd.args(["--output", output]);
        }

        cmd.spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "Weaver binary not found in PATH. Install with: ./scripts/install-weaver.sh or cargo install weaver".to_string()
                } else {
                    format!("Failed to start Weaver live-check: {}. Ensure Weaver is installed and in PATH.", e)
                }
            })
    }

    /// Stop the live-check process via HTTP admin endpoint
    pub fn stop(&self) -> Result<(), String> {
        // Note: This requires reqwest for HTTP requests
        // For now, return an error indicating this needs HTTP client
        Err(
            "Weaver stop requires HTTP client (reqwest). Not implemented in standalone version."
                .to_string(),
        )
    }

    /// Get the OTLP gRPC endpoint for sending telemetry
    /// Note: Weaver live-check listens on gRPC, but exporters typically use HTTP
    /// This returns the address:port format for configuration
    pub fn otlp_endpoint(&self) -> String {
        format!("{}:{}", self.otlp_grpc_address, self.otlp_grpc_port)
    }
}

impl Default for WeaverLiveCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test error types (critical - 80% of bugs)
    #[test]
    fn test_weaver_validation_error_display() {
        // Test all error variants display correctly
        let errors = vec![
            WeaverValidationError::BinaryNotFound("test".to_string()),
            WeaverValidationError::HealthCheckFailed("test".to_string()),
            WeaverValidationError::StartFailed("test".to_string()),
            WeaverValidationError::StopFailed("test".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty(), "Error should have display message");
            assert!(display.contains("test"), "Error should contain message");
        }
    }

    #[test]
    fn test_weaver_validation_error_debug() {
        // Test error is debuggable
        let error = WeaverValidationError::BinaryNotFound("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("BinaryNotFound"));
        assert!(debug.contains("test"));
    }

    // Test builder pattern (important - used frequently)
    #[test]
    fn test_weaver_live_check_new() {
        // Test default values
        let check = WeaverLiveCheck::new();
        assert_eq!(check.otlp_grpc_address, "127.0.0.1");
        assert_eq!(check.otlp_grpc_port, 4317);
        assert_eq!(check.admin_port, 8080);
        assert_eq!(check.inactivity_timeout, 60);
        assert_eq!(check.format, "json");
        assert!(check.registry_path.is_none());
        assert!(check.output.is_none());
    }

    #[test]
    fn test_weaver_live_check_default() {
        // Test Default trait implementation
        let check = WeaverLiveCheck::default();
        assert_eq!(check.otlp_grpc_address, "127.0.0.1");
        assert_eq!(check.otlp_grpc_port, 4317);
    }

    #[test]
    fn test_weaver_live_check_builder_pattern() {
        // Test builder pattern (chaining)
        let check = WeaverLiveCheck::new()
            .with_registry("/path/to/registry".to_string())
            .with_otlp_address("0.0.0.0".to_string())
            .with_otlp_port(4318)
            .with_admin_port(8081)
            .with_inactivity_timeout(120)
            .with_format("ansi".to_string())
            .with_output("/tmp/output".to_string());

        assert_eq!(check.registry_path, Some("/path/to/registry".to_string()));
        assert_eq!(check.otlp_grpc_address, "0.0.0.0");
        assert_eq!(check.otlp_grpc_port, 4318);
        assert_eq!(check.admin_port, 8081);
        assert_eq!(check.inactivity_timeout, 120);
        assert_eq!(check.format, "ansi");
        assert_eq!(check.output, Some("/tmp/output".to_string()));
    }

    #[test]
    fn test_weaver_live_check_otlp_endpoint() {
        // Test OTLP endpoint generation (important - used frequently)
        let check = WeaverLiveCheck::new();
        assert_eq!(check.otlp_endpoint(), "127.0.0.1:4317");

        let check = WeaverLiveCheck::new()
            .with_otlp_address("0.0.0.0".to_string())
            .with_otlp_port(4318);
        assert_eq!(check.otlp_endpoint(), "0.0.0.0:4318");
    }

    // Test boundary conditions (important - 80% of bugs)
    #[test]
    fn test_weaver_live_check_port_boundaries() {
        // Test port boundaries (u16: 0-65535)
        let check = WeaverLiveCheck::new()
            .with_otlp_port(0)
            .with_admin_port(65535);
        assert_eq!(check.otlp_grpc_port, 0);
        assert_eq!(check.admin_port, 65535);
    }

    #[test]
    fn test_weaver_live_check_timeout_boundaries() {
        // Test timeout boundaries (u64)
        let check = WeaverLiveCheck::new()
            .with_inactivity_timeout(0)
            .with_inactivity_timeout(u64::MAX);
        assert_eq!(check.inactivity_timeout, u64::MAX);
    }
}
