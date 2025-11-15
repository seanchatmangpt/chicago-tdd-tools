//! Output format utilities for multi-format serialization
//!
//! Provides helper functions to serialize data to JSON, YAML, TOML, CSV, and Table formats.

use serde::Serialize;
use std::fmt;

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format (default)
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// Table format (ASCII table)
    Table,
    /// TSV format (Tab-separated values)
    Tsv,
}

impl OutputFormat {
    /// Parse format from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" | "yml" => Ok(OutputFormat::Yaml),
            "toml" => Ok(OutputFormat::Toml),
            "table" | "txt" => Ok(OutputFormat::Table),
            "tsv" | "csv" => Ok(OutputFormat::Tsv),
            _ => Err(format!(
                "Unknown format: {}. Supported: json, yaml, toml, table, tsv",
                s
            )),
        }
    }

    /// Get default format
    pub fn default() -> Self {
        OutputFormat::Json
    }

    /// Serialize value to string in this format
    pub fn serialize<T: Serialize>(&self, value: &T) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            OutputFormat::Json => Ok(serde_json::to_string_pretty(value)?),
            OutputFormat::Yaml => Ok(serde_yaml::to_string(value)?),
            OutputFormat::Toml => {
                // TOML requires a table at root level
                let json_str = serde_json::to_value(value)?;
                if let serde_json::Value::Object(_) = json_str {
                    Ok(toml::to_string_pretty(value)?)
                } else {
                    Ok(format!("value = {}\n", serde_json::to_string(value)?))
                }
            }
            OutputFormat::Table => {
                // Simple table format - just pretty-print as JSON for now
                // In real implementation, would use comfy-table crate
                Ok(serde_json::to_string_pretty(value)?)
            }
            OutputFormat::Tsv => {
                // TSV format - serialize as JSON then convert
                // In real implementation, would use csv crate
                Ok(serde_json::to_string(value)?)
            }
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Toml => write!(f, "toml"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Tsv => write!(f, "tsv"),
        }
    }
}
