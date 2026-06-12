//! Core Governance Types
//!
//! Provides core diagnostic and severity types for the agent governance loop.
//! These types form the baseline for compile-time law enforcement macros and
//! diagnostic reporting.

#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display};

// Re-export channel contents
pub use channel::{
    close_channel, emit_diagnostic, get_domain, get_run_id, on_test_completed, on_test_started,
    register_domain, register_sink, set_channel_capacity, set_run_id, RunSummary,
};
pub mod channel;
pub mod laws;
pub mod sector;

pub use laws::*;
pub use sector::{MergeStrategy, ProcessIntelligenceSector, SectorStack};

/// Unique identifier for a governance execution run.
pub type RunId = String;

/// Unique identifier for an agent making an edit.
pub type AgentId = String;

/// Severity of a governance violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum Severity {
    /// STOP. Domain law violated. Agent must not declare done.
    Andon,
    /// Should stop. Law weakly violated or approaching violation.
    Warning,
    /// Informational. Law passed; noteworthy condition observed.
    Info,
}

impl Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Andon => write!(f, "ANDON"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// Categories of governance diagnostics as defined in PRD Section 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DiagnosticCategory {
    Admission,
    Lineage,
    Drift,
    Substrate,
    Ontology,
    Conformance,
    Intelligence,
    Bypass,
    Mutation,
    Snapshot,
    Coverage,
    Performance,
    Conflict,
}

impl DiagnosticCategory {
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Admission => "ADM",
            Self::Lineage => "LIN",
            Self::Drift => "DFT",
            Self::Substrate => "SUB",
            Self::Ontology => "ONT",
            Self::Conformance => "CON",
            Self::Intelligence => "INT",
            Self::Bypass => "BYP",
            Self::Mutation => "MUT",
            Self::Snapshot => "SNP",
            Self::Coverage => "COV",
            Self::Performance => "PER",
            Self::Conflict => "CFL",
        }
    }
}

impl Display for DiagnosticCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.prefix())
    }
}

/// Source location for a detected violation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceLocation {
    pub uri: String,
    pub line: u32,
    pub character: u32,
    pub file: String,
    pub column: u32,
}

/// Structured diagnostic code: {DOMAIN}-{CATEGORY}-{ORDINAL}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticCode {
    pub domain: String,
    pub category: DiagnosticCategory,
    pub ordinal: u16,
}

impl DiagnosticCode {
    pub fn new(domain: impl Into<String>, category: DiagnosticCategory, ordinal: u16) -> Self {
        Self { domain: domain.into(), category, ordinal }
    }
}

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{:03}", self.domain, self.category.prefix(), self.ordinal)
    }
}

/// The complete record emitted to the diagnostic channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub run_id: RunId,
    pub agent_id: Option<AgentId>,
    pub location: Option<SourceLocation>,
    pub message: String,
    pub severity: Severity,
    pub source_module: &'static str,
    pub context: HashMap<&'static str, serde_json::Value>,
    pub elapsed_ns: u64,
}

pub trait DiagnosticSink: Send + Sync {
    fn emit(&self, diagnostic: Diagnostic) -> Result<(), String>;
    fn close(&self, summary: RunSummary) -> Result<(), String>;
}

#[macro_export]
macro_rules! source_location {
    () => {
        $crate::core::governance::SourceLocation {
            uri: ::std::file!().to_string(),
            line: ::std::line!(),
            character: ::std::column!(),
            file: ::std::file!().to_string(),
            column: ::std::column!(),
        }
    };
}
