//! Poka-Yoke Types for OpenTelemetry
//!
//! Provides type-level error prevention for OTEL operations.
//! Uses Rust's type system to make invalid OTEL states unrepresentable.
//!
//! **Poka-Yoke Principles**:
//! - **Make invalid states unrepresentable**: Use types to prevent errors
//! - **Type-level prevention**: Invalid attribute types cannot be created
//! - **Feature gates**: Compile-time checks for required features
//! - **State machine**: Span lifecycle enforced by types
//!
//! # Error Modes Prevented
//!
//! 1. **Invalid attribute types**: Type system prevents wrong attribute types
//! 2. **Missing feature**: Compile-time error if feature not enabled
//! 3. **Span state errors**: Cannot complete already-completed span
//! 4. **Invalid attribute values**: Validated at creation time

//! Poka-Yoke Types for OpenTelemetry
//!
//! Provides type-level error prevention for OTEL operations.
//! Uses Rust's type system to make invalid OTEL states unrepresentable.
//!
//! **Poka-Yoke Principles**:
//! - **Make invalid states unrepresentable**: Use types to prevent errors
//! - **Type-level prevention**: Invalid attribute types cannot be created
//! - **Feature gates**: Compile-time checks for required features
//! - **State machine**: Span lifecycle enforced by types
//!
//! # Error Modes Prevented
//!
//! 1. **Invalid attribute types**: Type system prevents wrong attribute types
//! 2. **Missing feature**: Compile-time error if feature not enabled
//! 3. **Span state errors**: Cannot complete already-completed span
//! 4. **Invalid attribute values**: Validated at creation time

#[cfg(feature = "otel")]
use std::marker::PhantomData;

#[cfg(feature = "otel")]
/// Span state marker types
///
/// **Poka-yoke**: Phantom types prevent invalid operations at compile time.
/// A span is either `Active` or `Completed` - cannot be both.
pub mod state {
    /// Span is active (can add attributes, can complete)
    pub struct Active;

    /// Span is completed (read-only, cannot modify)
    pub struct Completed;
}

#[cfg(feature = "otel")]
/// Validated OTEL attribute value
///
/// **Poka-yoke**: Newtype prevents invalid attribute values.
/// The type system makes invalid attributes impossible.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidAttributeValue {
    /// String attribute (always non-empty)
    String(String),
    /// Integer attribute
    Int(i64),
    /// Float attribute
    Float(f64),
    /// Boolean attribute
    Bool(bool),
}

#[cfg(feature = "otel")]
impl ValidAttributeValue {
    /// Create a new validated string attribute
    ///
    /// **Poka-yoke**: Returns `Option` to prevent empty strings.
    ///
    /// # Errors
    ///
    /// Returns `None` if string is empty.
    #[must_use]
    pub fn string(value: &str) -> Option<Self> {
        if value.is_empty() {
            return None;
        }
        Some(Self::String(value.to_string()))
    }

    /// Create a new validated integer attribute
    #[must_use]
    pub const fn int(value: i64) -> Self {
        Self::Int(value)
    }

    /// Create a new validated float attribute
    #[must_use]
    pub const fn float(value: f64) -> Self {
        Self::Float(value)
    }

    /// Create a new validated boolean attribute
    #[must_use]
    pub const fn bool(value: bool) -> Self {
        Self::Bool(value)
    }
}

#[cfg(feature = "otel")]
/// Validated attribute name
///
/// **Poka-yoke**: Newtype prevents empty attribute names.
/// The type system makes invalid attribute names impossible.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidAttributeName {
    /// Name (always non-empty)
    name: String,
}

#[cfg(feature = "otel")]
impl ValidAttributeName {
    /// Create a new validated attribute name
    ///
    /// **Poka-yoke**: Returns `Option` to prevent empty names.
    ///
    /// # Errors
    ///
    /// Returns `None` if name is empty or whitespace-only.
    #[must_use]
    pub fn new(name: &str) -> Option<Self> {
        if name.trim().is_empty() {
            return None;
        }
        Some(Self { name: name.to_string() })
    }

    /// Get the name as `&str`
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[cfg(feature = "otel")]
/// Validated OTEL attribute
///
/// **Poka-yoke**: Type ensures name and value are both valid.
/// The type system makes invalid attributes impossible.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidAttribute {
    /// Validated name (always non-empty)
    name: ValidAttributeName,
    /// Validated value
    value: ValidAttributeValue,
}

#[cfg(feature = "otel")]
impl ValidAttribute {
    /// Create a new validated attribute
    ///
    /// **Poka-yoke**: Returns `Option` to prevent invalid attributes.
    ///
    /// # Errors
    ///
    /// Returns `None` if name or value is invalid.
    #[must_use]
    pub fn new(name: &str, value: ValidAttributeValue) -> Option<Self> {
        ValidAttributeName::new(name).map(|name| Self { name, value })
    }

    /// Get the attribute name
    #[must_use]
    pub const fn name(&self) -> &ValidAttributeName {
        &self.name
    }

    /// Get the attribute value
    #[must_use]
    pub const fn value(&self) -> &ValidAttributeValue {
        &self.value
    }
}

#[cfg(feature = "otel")]
/// Span with type-level state
///
/// **Poka-yoke**: Type parameter `S` prevents invalid operations.
/// - `Span<Active>`: Can add attributes, can complete
/// - `Span<Completed>`: Read-only, cannot modify
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::observability::otel::poka_yoke::*;
///
/// // Create active span
/// let span: Span<Active> = Span::new("operation")?;
///
/// // Add attribute (only works on active span)
/// span.add_attribute(ValidAttribute::new("key", ValidAttributeValue::string("value")?)?);
///
/// // Complete span (changes type to Completed)
/// let span: Span<Completed> = span.complete()?;
///
/// // Compile error: Cannot add attribute to completed span
/// // span.add_attribute(...); // ERROR!
/// ```
pub struct Span<S> {
    /// Span name (internal)
    name: String,
    /// State marker (compile-time guarantee)
    _state: PhantomData<S>,
}

#[cfg(feature = "otel")]
impl Span<state::Active> {
    /// Create a new active span
    ///
    /// **Poka-yoke**: Returns `Span<Active>` - can add attributes and complete.
    ///
    /// # Errors
    ///
    /// Returns error if span creation fails.
    pub fn new(name: &str) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Span name cannot be empty".to_string());
        }
        Ok(Self { name: name.to_string(), _state: PhantomData })
    }

    /// Add an attribute to the active span
    ///
    /// **Poka-yoke**: Only available on `Span<Active>`.
    /// Compiler prevents calling this on completed spans.
    ///
    /// # Note
    ///
    /// This is a placeholder for poka-yoke design demonstration.
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)] // Placeholder - will be implemented later
    pub fn add_attribute(&self, _attr: ValidAttribute) -> Result<(), String> {
        // Add attribute logic here - only works on active spans
        Ok(())
    }

    /// Complete the span
    ///
    /// **Poka-yoke**: Changes type from `Span<Active>` to `Span<Completed>`.
    /// After this call, span cannot be modified.
    ///
    /// # Note
    ///
    /// This is a placeholder for poka-yoke design demonstration.
    #[allow(clippy::unnecessary_wraps)] // Placeholder - will be implemented later
    pub fn complete(self) -> Result<Span<state::Completed>, String> {
        // Complete span logic here
        Ok(Span { name: self.name, _state: PhantomData })
    }
}

#[cfg(feature = "otel")]
impl Span<state::Completed> {
    /// Get span name (read-only)
    ///
    /// **Poka-yoke**: Only read operations available on completed spans.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(all(test, feature = "otel"))]
mod tests {
    use super::*;

    #[test]
    fn test_valid_attribute_name() {
        let name = ValidAttributeName::new("valid.name");
        assert!(name.is_some());
        if let Some(n) = name {
            assert_eq!(n.as_str(), "valid.name");
        }
    }

    #[test]
    fn test_invalid_attribute_name_empty() {
        let name = ValidAttributeName::new("");
        assert!(name.is_none()); // Type prevents empty name
    }

    #[test]
    fn test_invalid_attribute_name_whitespace() {
        let name = ValidAttributeName::new("   ");
        assert!(name.is_none()); // Type prevents whitespace-only name
    }

    #[test]
    fn test_valid_attribute_value_string() {
        let value = ValidAttributeValue::string("valid");
        assert!(value.is_some());
    }

    #[test]
    fn test_invalid_attribute_value_empty_string() {
        let value = ValidAttributeValue::string("");
        assert!(value.is_none()); // Type prevents empty string
    }

    #[test]
    fn test_valid_attribute() {
        let attr =
            ValidAttribute::new("key", ValidAttributeValue::string("value").expect("test data"));
        assert!(attr.is_some());
    }

    #[test]
    fn test_invalid_attribute_empty_name() {
        let attr =
            ValidAttribute::new("", ValidAttributeValue::string("value").expect("test data"));
        assert!(attr.is_none()); // Type prevents empty name
    }

    #[test]
    fn test_span_lifecycle() {
        // Create active span
        let span = Span::<state::Active>::new("test").expect("test span");

        // Add attribute (only works on active span)
        let attr =
            ValidAttribute::new("key", ValidAttributeValue::string("value").expect("test data"))
                .expect("test attribute");
        span.add_attribute(attr).expect("test attribute");

        // Complete span (changes type)
        let _completed: Span<state::Completed> = span.complete().expect("test completion");

        // Compile error: Cannot add attribute to completed span
        // completed.add_attribute(...); // ERROR!
    }
}
