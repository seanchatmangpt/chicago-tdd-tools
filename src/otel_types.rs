//! OpenTelemetry Type Definitions
//!
//! Minimal OTEL type definitions ported from knhk-otel for standalone use.
//! These types are used by the OTEL validation features.
//!
//! # Poka-Yoke: Type-Level Validation
//!
//! This module uses enums instead of `Option<T>` to prevent invalid states at compile time.
//! Use `SpanState` for active vs completed spans, and `SpanRelationship` for root vs child spans.

use std::collections::BTreeMap;

/// Trace ID (128-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceId(pub u128);

/// Span ID (64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanId(pub u64);

/// Span relationship type
///
/// **Poka-Yoke**: Use this enum instead of `Option<SpanId>` to prevent invalid states.
/// A span is either a root span (no parent) or a child span (has a parent).
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::otel_types::{SpanRelationship, SpanId};
///
/// // Root span - no parent
/// let root = SpanRelationship::Root;
///
/// // Child span - has parent
/// let child = SpanRelationship::Child { parent_span_id: SpanId(12345) };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanRelationship {
    /// Root span (no parent)
    Root,
    /// Child span (has a parent)
    Child {
        /// Parent span ID
        parent_span_id: SpanId,
    },
}

impl SpanRelationship {
    /// Get the parent span ID if this is a child span
    pub fn parent_span_id(&self) -> Option<SpanId> {
        match self {
            Self::Root => None,
            Self::Child { parent_span_id } => Some(*parent_span_id),
        }
    }

    /// Check if this is a root span
    pub fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    /// Check if this is a child span
    pub fn is_child(&self) -> bool {
        matches!(self, Self::Child { .. })
    }
}

/// Span context
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Trace ID
    pub trace_id: TraceId,
    /// Span ID
    pub span_id: SpanId,
    /// Span relationship (root or child)
    /// **Poka-Yoke**: Use enum instead of `Option<SpanId>` to prevent invalid states
    pub relationship: SpanRelationship,
    /// Context flags
    pub flags: u8,
}

impl SpanContext {
    /// Create a new root span context
    pub fn root(trace_id: TraceId, span_id: SpanId, flags: u8) -> Self {
        Self { trace_id, span_id, relationship: SpanRelationship::Root, flags }
    }

    /// Create a new child span context
    pub fn child(trace_id: TraceId, span_id: SpanId, parent_span_id: SpanId, flags: u8) -> Self {
        Self { trace_id, span_id, relationship: SpanRelationship::Child { parent_span_id }, flags }
    }

    /// Get the parent span ID (if this is a child span)
    pub fn parent_span_id(&self) -> Option<SpanId> {
        self.relationship.parent_span_id()
    }

    /// Check if this is a root span
    pub fn is_root(&self) -> bool {
        self.relationship.is_root()
    }

    /// Check if this is a child span
    pub fn is_child(&self) -> bool {
        self.relationship.is_child()
    }
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

/// Span state type
///
/// **Poka-Yoke**: Use this enum instead of `Option<u64>` to prevent invalid states.
/// A span is either active (not yet ended) or completed (has an end time).
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::otel_types::SpanState;
///
/// // Active span - not yet ended
/// let active = SpanState::Active { start_time_ms: 1000 };
///
/// // Completed span - has end time
/// let completed = SpanState::Completed { start_time_ms: 1000, end_time_ms: 2000 };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanState {
    /// Active span (not yet ended)
    Active {
        /// Start time in milliseconds since epoch
        start_time_ms: u64,
    },
    /// Completed span (has end time)
    Completed {
        /// Start time in milliseconds since epoch
        start_time_ms: u64,
        /// End time in milliseconds since epoch
        end_time_ms: u64,
    },
}

impl SpanState {
    /// Get the start time
    pub fn start_time_ms(&self) -> u64 {
        match self {
            Self::Active { start_time_ms } | Self::Completed { start_time_ms, .. } => {
                *start_time_ms
            }
        }
    }

    /// Get the end time (if completed)
    pub fn end_time_ms(&self) -> Option<u64> {
        match self {
            Self::Active { .. } => None,
            Self::Completed { end_time_ms, .. } => Some(*end_time_ms),
        }
    }

    /// Check if the span is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    /// Check if the span is completed
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// Complete the span (transition from active to completed)
    ///
    /// # Errors
    ///
    /// Returns an error if the span is already completed or if end_time < start_time.
    pub fn complete(self, end_time_ms: u64) -> Result<Self, String> {
        match self {
            Self::Active { start_time_ms } => {
                if end_time_ms < start_time_ms {
                    return Err(format!(
                        "End time {} must be >= start time {}",
                        end_time_ms, start_time_ms
                    ));
                }
                Ok(Self::Completed { start_time_ms, end_time_ms })
            }
            Self::Completed { .. } => Err("Span is already completed".to_string()),
        }
    }
}

/// Span
#[derive(Debug, Clone)]
pub struct Span {
    /// Span context (trace ID, span ID, etc.)
    pub context: SpanContext,
    /// Span name
    pub name: String,
    /// Span state (active or completed)
    /// **Poka-Yoke**: Use enum instead of `Option<u64>` to prevent invalid states
    pub state: SpanState,
    /// Span attributes (key-value pairs)
    pub attributes: Attributes,
    /// Span events (annotations)
    pub events: Vec<SpanEvent>,
    /// Span status (Ok, Error, or Unset)
    pub status: SpanStatus,
}

impl Span {
    /// Create a new active span
    pub fn new_active(
        context: SpanContext,
        name: String,
        start_time_ms: u64,
        attributes: Attributes,
        events: Vec<SpanEvent>,
        status: SpanStatus,
    ) -> Self {
        Self {
            context,
            name,
            state: SpanState::Active { start_time_ms },
            attributes,
            events,
            status,
        }
    }

    /// Create a new completed span
    pub fn new_completed(
        context: SpanContext,
        name: String,
        start_time_ms: u64,
        end_time_ms: u64,
        attributes: Attributes,
        events: Vec<SpanEvent>,
        status: SpanStatus,
    ) -> Result<Self, String> {
        if end_time_ms < start_time_ms {
            return Err(format!(
                "End time {} must be >= start time {}",
                end_time_ms, start_time_ms
            ));
        }
        Ok(Self {
            context,
            name,
            state: SpanState::Completed { start_time_ms, end_time_ms },
            attributes,
            events,
            status,
        })
    }

    /// Get the start time
    pub fn start_time_ms(&self) -> u64 {
        self.state.start_time_ms()
    }

    /// Get the end time (if completed)
    pub fn end_time_ms(&self) -> Option<u64> {
        self.state.end_time_ms()
    }

    /// Check if the span is active
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Check if the span is completed
    pub fn is_completed(&self) -> bool {
        self.state.is_completed()
    }

    /// Complete the span (transition from active to completed)
    ///
    /// # Errors
    ///
    /// Returns an error if the span is already completed or if end_time < start_time.
    pub fn complete(&mut self, end_time_ms: u64) -> Result<(), String> {
        self.state = self.state.complete(end_time_ms)?;
        Ok(())
    }
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
