//! OpenTelemetry Type Definitions
//!
//! Minimal OTEL type definitions ported from knhk-otel for standalone use.
//! These types are used by the OTEL validation features.

use std::collections::BTreeMap;

/// Trace ID (128-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceId(pub u128);

/// Span ID (64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanId(pub u64);

/// Span context
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Trace ID
    pub trace_id: TraceId,
    /// Span ID
    pub span_id: SpanId,
    /// Parent span ID (if any)
    pub parent_span_id: Option<SpanId>,
    /// Context flags
    pub flags: u8,
}

/// Span attributes
pub type Attributes = BTreeMap<String, String>;

/// Span event
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Timestamp in milliseconds
    pub timestamp_ms: u64,
    /// Event attributes
    pub attributes: Attributes,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

/// Span
#[derive(Debug, Clone)]
pub struct Span {
    pub context: SpanContext,
    pub name: String,
    pub start_time_ms: u64,
    pub end_time_ms: Option<u64>,
    pub attributes: Attributes,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<u64>),
}

/// Metric
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub timestamp_ms: u64,
    pub attributes: Attributes,
}
