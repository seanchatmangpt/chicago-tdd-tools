//! Error type for the conformance subsystem.
//!
//! Feature-agnostic by design: foreign error types (`serde_json`, `std::io`,
//! BCINR internals) are captured as plain messages so no `#[from]` binds this
//! enum to any optional dependency or upstream crate's error identity.

use thiserror::Error;

/// Errors raised while loading, validating, or replaying conformance
/// specifications and traces.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConformanceError {
    // ------------------------------------------------------------------
    // Loader errors — raised while parsing source specifications.
    // ------------------------------------------------------------------
    /// A source line could not be tokenized or parsed.
    #[error("malformed source at line {line}, column {col}: {reason}")]
    MalformedSource {
        /// 1-based line number of the offending text.
        line: u32,
        /// 1-based column offset of the offending text.
        col: u32,
        /// Why the text was rejected (static diagnostic string).
        reason: &'static str,
    },

    /// An activity name appeared in the source that is not declared.
    #[error("unknown activity '{name}' at line {line}, column {col}")]
    UnknownActivity {
        /// The undeclared activity name.
        name: String,
        /// 1-based line number where the name was referenced.
        line: u32,
        /// 1-based column offset of the name.
        col: u32,
    },

    /// The same activity name was declared more than once.
    #[error("duplicate activity declaration '{name}' at line {line}, column {col}")]
    DuplicateActivity {
        /// The redeclared activity name.
        name: String,
        /// 1-based line number of the duplicate declaration.
        line: u32,
        /// 1-based column offset of the duplicate declaration.
        col: u32,
    },

    /// The source was not valid JSON.
    #[error("invalid JSON source: {message}")]
    InvalidJson {
        /// Deserialization diagnostic (captured from the JSON parser).
        message: String,
    },

    /// The source could not be read from the filesystem.
    #[error("I/O error reading '{path}': {message}")]
    Io {
        /// Path of the file that could not be read.
        path: String,
        /// Underlying OS error message (captured as text).
        message: String,
    },

    // ------------------------------------------------------------------
    // Validation errors — raised against a loaded model.
    // ------------------------------------------------------------------
    /// A constructor received fewer inputs than its minimum arity.
    #[error("constructor '{constructor}' requires at least {minimum} inputs, found {found}")]
    ArityBelowMinimum {
        /// Name of the offending constructor.
        constructor: &'static str,
        /// Minimum permitted input count.
        minimum: usize,
        /// Actual input count.
        found: usize,
    },

    /// The dependency graph contains a cycle.
    #[error("dependency cycle detected among nodes {nodes:?}")]
    DependencyCycle {
        /// Node indices participating in the cycle.
        nodes: Vec<usize>,
    },

    /// The model exceeds the configured size limit.
    #[error("model too large: found {found} elements, limit is {limit}")]
    ModelTooLarge {
        /// Observed element count.
        found: usize,
        /// Maximum permitted element count.
        limit: usize,
    },

    /// A loop construct lacks a provable termination bound.
    #[error("unbounded loop construct")]
    UnboundedLoop,

    /// A reference names a node id that does not exist on the given side.
    #[error("unknown node reference ({side} side): id '{id}'")]
    UnknownNodeRef {
        /// Which side the reference came from (e.g. "source", "target").
        side: &'static str,
        /// The unresolvable node id.
        id: String,
    },

    /// An arc carries a weight outside the permitted range.
    #[error("arc weight {weight} outside permitted range")]
    WeightedArc {
        /// The rejected arc weight.
        weight: u16,
    },

    /// The model declares more places than the marking encoding supports.
    #[error("too many places: limit is {limit}")]
    TooManyPlaces {
        /// Maximum number of places supported by the marking encoding.
        limit: usize,
    },

    // ------------------------------------------------------------------
    // Session / trajectory errors — raised while replaying traces.
    // ------------------------------------------------------------------
    /// A recorded step was skipped relative to the expected sequence.
    #[error("skipped trajectory step {step}")]
    Skipped {
        /// The step number that was skipped.
        step: u32,
    },

    /// A step's tick regressed relative to the previous tick.
    #[error("out-of-order step {step}: tick {tick} is not after previous tick {prev_tick}")]
    OutOfOrder {
        /// The step at which the regression was observed.
        step: u32,
        /// The tick of the previous accepted step.
        prev_tick: u8,
        /// The offending tick.
        tick: u8,
    },

    /// A step references a transition id never declared in the model.
    #[error("undeclared transition id {tid} at step {step}")]
    Undeclared {
        /// The step at which the unknown transition fired.
        step: u32,
        /// The transition id not present in the model.
        tid: u8,
    },

    /// A step's marking enabled a bit that the policy forbids.
    #[error("forbidden bit at step {step}: marking {marking:#06b} intersects mask {mask:#06b}")]
    ForbiddenBit {
        /// The step at which the forbidden marking was produced.
        step: u32,
        /// The observed marking bits.
        marking: u16,
        /// The forbidden-bit mask it intersected.
        mask: u16,
    },

    /// A trace is structurally invalid (e.g. zero length or truncated).
    #[error("malformed trace: length {len}")]
    TraceMalformed {
        /// The rejected trace length.
        len: usize,
    },

    // ------------------------------------------------------------------
    // Cross-cutting — feature-agnostic pass-through.
    // ------------------------------------------------------------------
    /// A downstream validator rejected the specification; the foreign
    /// error's message is captured as text (no `#[from]` coupling).
    #[error("BCINR validation failed: {message}")]
    BcinrValidation {
        /// The validator's diagnostic, captured as text.
        message: String,
    },
}
