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
    /// Span completed successfully
    Ok,
    /// Span ended with an error
    Error,
    /// Span status is unset
    Unset,
}

/// Span
#[derive(Debug, Clone)]
pub struct Span {
    /// Span context (trace ID, span ID, etc.)
    pub context: SpanContext,
    /// Span name
    pub name: String,
    /// Start time in milliseconds since epoch
    pub start_time_ms: u64,
    /// End time in milliseconds since epoch (None if span is still active)
    pub end_time_ms: Option<u64>,
    /// Span attributes (key-value pairs)
    pub attributes: Attributes,
    /// Span events (annotations)
    pub events: Vec<SpanEvent>,
    /// Span status (Ok, Error, or Unset)
    pub status: SpanStatus,
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Counter metric (monotonically increasing)
    Counter(u64),
    /// Gauge metric (can increase or decrease)
    Gauge(f64),
    /// Histogram metric (distribution of values)
    Histogram(Vec<u64>),
}

/// Metric
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric value (Counter, Gauge, or Histogram)
    pub value: MetricValue,
    /// Timestamp in milliseconds since epoch
    pub timestamp_ms: u64,
    /// Metric attributes (key-value pairs)
    pub attributes: Attributes,
}
