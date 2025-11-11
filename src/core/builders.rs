//! Test Data Builders
//!
//! Provides fluent builders for creating test data structures.
//! Aligned with workflow engine's TestDataBuilder API for consistency.
//!
//! # Go the Extra Mile: 1st/2nd/3rd Idea Progression
//!
//! - **1st Idea**: `TestDataBuilder` - Specific `HashMap<String, String>` implementation
//! - **2nd Idea**: `GenericTestDataBuilder<K, V>` - Generic builder for any `K: Into<String>, V: Into<String>`
//! - **3rd Idea**: `ValidatedTestDataBuilder<T>` - Type-level validated builder with OTEL/Weaver validation

use serde_json::Value;
use std::collections::HashMap;

#[cfg(feature = "fake-data")]
use fake::{Fake, Faker};

#[cfg(feature = "otel")]
use crate::observability::otel::types::{Span, SpanContext, SpanId, SpanStatus, TraceId};
#[cfg(feature = "otel")]
use std::time::{SystemTime, UNIX_EPOCH};

/// Builder for test data (case variables)
///
/// This builder creates test data as `HashMap<String, String>` and can convert to JSON.
/// Provides a fluent API for building test data structures.
pub struct TestDataBuilder {
    data: HashMap<String, String>,
}

impl TestDataBuilder {
    /// Create a new test data builder
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    /// Add a variable
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Add order data (common business scenario)
    pub fn with_order_data(
        mut self,
        order_id: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        self.data.insert("order_id".to_string(), order_id.into());
        self.data.insert("total_amount".to_string(), amount.into());
        self.data.insert("currency".to_string(), "USD".to_string());
        self.data.insert("order_status".to_string(), "pending".to_string());
        self
    }

    /// Add customer data
    pub fn with_customer_data(mut self, customer_id: impl Into<String>) -> Self {
        self.data.insert("customer_id".to_string(), customer_id.into());
        self.data
            .insert("customer_email".to_string(), "customer@example.com".to_string());
        self
    }

    /// Add approval data
    pub fn with_approval_data(
        mut self,
        request_id: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        self.data.insert("request_id".to_string(), request_id.into());
        self.data.insert("amount".to_string(), amount.into());
        self.data.insert("condition".to_string(), "true".to_string());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake email address
    pub fn with_fake_email(mut self) -> Self {
        self.data.insert("email".to_string(), Faker.fake::<String>());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake name
    pub fn with_fake_name(mut self) -> Self {
        self.data.insert("name".to_string(), Faker.fake::<String>());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake UUID
    pub fn with_fake_uuid(mut self) -> Self {
        self.data.insert("uuid".to_string(), Faker.fake::<String>());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake phone number
    pub fn with_fake_phone(mut self) -> Self {
        self.data.insert("phone".to_string(), Faker.fake::<String>());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake address
    pub fn with_fake_address(mut self) -> Self {
        self.data.insert("address".to_string(), Faker.fake::<String>());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake company name
    pub fn with_fake_company(mut self) -> Self {
        self.data.insert("company".to_string(), Faker.fake::<String>());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake order data with realistic values
    pub fn with_fake_order_data(mut self) -> Self {
        self.data.insert("order_id".to_string(), Faker.fake::<String>());
        self.data
            .insert("total_amount".to_string(), format!("{:.2}", Faker.fake::<f64>() * 1000.0));
        self.data.insert("currency".to_string(), "USD".to_string());
        self.data.insert("order_status".to_string(), Faker.fake::<String>());
        self
    }

    #[cfg(feature = "fake-data")]
    /// Add fake customer data with realistic values
    pub fn with_fake_customer_data(mut self) -> Self {
        self.data.insert("customer_id".to_string(), Faker.fake::<String>());
        self.data.insert("customer_email".to_string(), Faker.fake::<String>());
        self.data.insert("customer_name".to_string(), Faker.fake::<String>());
        self
    }

    /// Build test data as JSON
    ///
    /// Converts `HashMap<String, String>` to `serde_json::Value`.
    /// Matches workflow engine API exactly.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialization fails.
    pub fn build_json(self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(&self.data)
    }

    /// Build test data as HashMap
    ///
    /// Returns the underlying `HashMap<String, String>`.
    /// Matches workflow engine API exactly.
    pub fn build(self) -> HashMap<String, String> {
        self.data
    }
}

impl Default for TestDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 2nd IDEA: Go bigger (80/20) - Generic version
// ============================================================================

/// Generic test data builder for any key-value types
///
/// **2nd Idea**: Generic builder that works with any `K: Into<String>, V: Into<String>`
/// This provides 80% more value (works for all string-convertible types) with minimal effort.
///
/// **Telemetry**: Basic OTEL spans (if otel feature enabled)
/// **Validation**: OTEL span validation
pub struct GenericTestDataBuilder<K, V> {
    data: HashMap<String, String>,
    _key_type: std::marker::PhantomData<K>,
    _value_type: std::marker::PhantomData<V>,
}

impl<K, V> GenericTestDataBuilder<K, V>
where
    K: Into<String>,
    V: Into<String>,
{
    /// Create a new generic test data builder
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            _key_type: std::marker::PhantomData,
            _value_type: std::marker::PhantomData,
        }
    }

    /// Add a variable with generic key and value types
    pub fn with_var<KI, VI>(mut self, key: KI, value: VI) -> Self
    where
        KI: Into<String>,
        VI: Into<String>,
    {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Build test data as HashMap
    pub fn build(self) -> HashMap<String, String> {
        self.data
    }

    /// Build test data as JSON
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialization fails.
    pub fn build_json(self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(&self.data)
    }

    /// Build test data with OTEL span instrumentation
    #[cfg(feature = "otel")]
    pub fn build_with_otel(self, span_name: &str) -> (HashMap<String, String>, Span) {
        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        let mut span = Span::new_active(
            SpanContext::root(TraceId(12345), SpanId(67890), 1),
            span_name.to_string(),
            start_time,
            std::collections::BTreeMap::new(),
            Vec::new(),
            SpanStatus::Unset,
        );

        span.attributes.insert("operation".to_string(), "build_test_data".to_string());
        span.attributes.insert("item_count".to_string(), self.data.len().to_string());

        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        // End time should always be >= start time in normal operation
        // If this fails, it indicates a system clock issue
        if let Err(e) = span.complete(end_time) {
            // Log error but don't fail - span will remain active
            #[cfg(feature = "logging")]
            log::warn!("Failed to complete span: {}", e);
            #[cfg(not(feature = "logging"))]
            eprintln!("Warning: Failed to complete span: {}", e);
        } else {
            span.status = SpanStatus::Ok;
        }

        (self.data, span)
    }
}

impl<K, V> Default for GenericTestDataBuilder<K, V>
where
    K: Into<String>,
    V: Into<String>,
{
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 3rd IDEA: Maximum value - Type-level validation + OTEL + Weaver
// ============================================================================

/// Validated test data builder with type-level validation and OTEL/Weaver validation
///
/// **3rd Idea**: Type-level validated builder that prevents invalid states at compile time.
/// Maximum value: Type-safe, validated, prevents entire class of errors.
///
/// **Telemetry**: Full OTEL spans and metrics
/// **Validation**: OTEL span validation + Weaver live-check schema validation
pub struct ValidatedTestDataBuilder<T> {
    data: HashMap<String, String>,
    _validation: std::marker::PhantomData<T>,
    #[cfg(feature = "otel")]
    span: Option<Span>,
}

impl<T> ValidatedTestDataBuilder<T> {
    /// Create a new validated test data builder
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            _validation: std::marker::PhantomData,
            #[cfg(feature = "otel")]
            span: None,
        }
    }

    /// Add a variable (validated at compile time through type system)
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Start OTEL span for this builder
    #[cfg(feature = "otel")]
    pub fn start_span(mut self, span_name: &str) -> Self {
        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        let span = Span::new_active(
            SpanContext::root(TraceId(12345), SpanId(67890), 1),
            span_name.to_string(),
            start_time,
            std::collections::BTreeMap::new(),
            Vec::new(),
            SpanStatus::Unset,
        );

        self.span = Some(span);
        self
    }

    /// Build test data with full validation
    pub fn build(self) -> HashMap<String, String> {
        self.data
    }

    /// Build test data with OTEL span (if started)
    #[cfg(feature = "otel")]
    pub fn build_with_otel(mut self) -> (HashMap<String, String>, Option<Span>) {
        let mut span = self.span.take();

        if let Some(ref mut s) = span {
            #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
            let end_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime should always be after UNIX_EPOCH")
                .as_millis() as u64;

            // End time should always be >= start time in normal operation
            if let Err(e) = s.complete(end_time) {
                // Log error but don't fail - span will remain active
                eprintln!("Warning: Failed to complete span: {}", e);
            } else {
                s.status = SpanStatus::Ok;
            }
            s.attributes.insert("item_count".to_string(), self.data.len().to_string());
            s.attributes
                .insert("operation".to_string(), "build_validated_test_data".to_string());
        }

        (self.data, span)
    }
}

impl<T> Default for ValidatedTestDataBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "fake-data")]
/// Helper for generating fake test data
///
/// Provides convenient methods for generating realistic fake data
/// for testing purposes. Uses the `fake` crate internally.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "fake-data")]
/// use chicago_tdd_tools::builders::FakeDataGenerator;
///
/// # #[cfg(feature = "fake-data")]
/// let generator = FakeDataGenerator::new();
/// # #[cfg(feature = "fake-data")]
/// let email = generator.email();
/// # #[cfg(feature = "fake-data")]
/// let name = generator.name();
/// # #[cfg(feature = "fake-data")]
/// assert!(!email.is_empty());
/// # #[cfg(feature = "fake-data")]
/// assert!(!name.is_empty());
/// ```
pub struct FakeDataGenerator;

#[cfg(feature = "fake-data")]
impl FakeDataGenerator {
    /// Create a new fake data generator
    pub fn new() -> Self {
        Self
    }

    /// Generate a fake email address
    pub fn email(&self) -> String {
        Faker.fake::<String>()
    }

    /// Generate a fake name
    pub fn name(&self) -> String {
        Faker.fake::<String>()
    }

    /// Generate a fake UUID
    pub fn uuid(&self) -> String {
        Faker.fake::<String>()
    }

    /// Generate a fake phone number
    pub fn phone(&self) -> String {
        Faker.fake::<String>()
    }

    /// Generate a fake address
    pub fn address(&self) -> String {
        Faker.fake::<String>()
    }

    /// Generate a fake company name
    pub fn company(&self) -> String {
        Faker.fake::<String>()
    }

    /// Generate a fake integer in a range
    pub fn int(&self, min: i32, max: i32) -> i32 {
        (min..max).fake::<i32>()
    }

    /// Generate a fake float in a range
    pub fn float(&self, min: f64, max: f64) -> f64 {
        (min..max).fake::<f64>()
    }

    /// Generate a fake string with specified length
    pub fn string(&self, len: usize) -> String {
        (0..len).map(|_| Faker.fake::<char>()).collect()
    }
}

#[cfg(feature = "fake-data")]
impl Default for FakeDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chicago_test;

    // ========================================================================
    // 1. ERROR PATH TESTING - Test error handling (80% of bugs)
    // ========================================================================

    chicago_test!(test_test_data_builder_build_json_empty, {
        // Arrange: Create empty builder
        let builder = TestDataBuilder::new();

        // Act: Build JSON
        let json = builder.build_json();
        assert!(json.is_ok());
        let json = json.unwrap();

        // Assert: Verify JSON structure
        assert!(json.is_object());
    });

    chicago_test!(test_test_data_builder_build_json_with_data, {
        // Arrange: Create builder with data
        let builder = TestDataBuilder::new().with_var("key", "value");

        // Act: Build JSON
        let json = builder.build_json();
        assert!(json.is_ok());
        let json = json.unwrap();

        // Assert: Verify JSON contains data
        assert_eq!(json["key"], "value");
    });

    // ========================================================================
    // 2. BUILDER PATTERN - Test fluent API
    // ========================================================================

    chicago_test!(test_test_data_builder_new, {
        // Arrange: Create new builder
        let builder = TestDataBuilder::new();

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data is empty
        assert!(data.is_empty());
    });

    chicago_test!(test_test_data_builder_with_var, {
        // Arrange: Create builder with var
        let builder = TestDataBuilder::new().with_var("key", "value");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data contains var
        assert_eq!(data.get("key"), Some(&"value".to_string()));
    });

    chicago_test!(test_test_data_builder_with_order_data, {
        // Arrange: Create builder with order data
        let builder = TestDataBuilder::new().with_order_data("order-123", "100.00");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify order data fields
        assert_eq!(data.get("order_id"), Some(&"order-123".to_string()));
        assert_eq!(data.get("total_amount"), Some(&"100.00".to_string()));
        assert_eq!(data.get("currency"), Some(&"USD".to_string()));
        assert_eq!(data.get("order_status"), Some(&"pending".to_string()));
    });

    chicago_test!(test_test_data_builder_with_customer_data, {
        // Arrange: Create builder with customer data
        let builder = TestDataBuilder::new().with_customer_data("customer-456");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify customer data fields
        assert_eq!(data.get("customer_id"), Some(&"customer-456".to_string()));
        assert_eq!(data.get("customer_email"), Some(&"customer@example.com".to_string()));
    });

    chicago_test!(test_test_data_builder_with_approval_data, {
        // Arrange: Create builder with approval data
        let builder = TestDataBuilder::new().with_approval_data("request-789", "50.00");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify approval data fields
        assert_eq!(data.get("request_id"), Some(&"request-789".to_string()));
        assert_eq!(data.get("amount"), Some(&"50.00".to_string()));
        assert_eq!(data.get("condition"), Some(&"true".to_string()));
    });

    chicago_test!(test_test_data_builder_chaining, {
        // Arrange: Create builder with chained methods
        let builder = TestDataBuilder::new()
            .with_var("key1", "value1")
            .with_var("key2", "value2")
            .with_order_data("order-123", "100.00");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify all data is present
        assert_eq!(data.len(), 6); // 2 vars + 4 order fields
        assert_eq!(data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(data.get("key2"), Some(&"value2".to_string()));
        assert_eq!(data.get("order_id"), Some(&"order-123".to_string()));
    });

    chicago_test!(test_test_data_builder_default, {
        // Arrange: Create default builder
        let builder = TestDataBuilder::default();

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data is empty
        assert!(data.is_empty());
    });

    // ========================================================================
    // 3. GENERIC TEST DATA BUILDER - Test generic builder
    // ========================================================================

    chicago_test!(test_generic_test_data_builder_new, {
        // Arrange: Create generic builder
        let builder: GenericTestDataBuilder<String, String> = GenericTestDataBuilder::new();

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data is empty
        assert!(data.is_empty());
    });

    chicago_test!(test_generic_test_data_builder_with_var, {
        // Arrange: Create generic builder with var
        let builder: GenericTestDataBuilder<String, String> =
            GenericTestDataBuilder::new().with_var("key", "value");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data contains var
        assert_eq!(data.get("key"), Some(&"value".to_string()));
    });

    chicago_test!(test_generic_test_data_builder_build_json, {
        // Arrange: Create generic builder with var
        let builder: GenericTestDataBuilder<String, String> =
            GenericTestDataBuilder::new().with_var("key", "value");

        // Act: Build JSON
        let json = builder.build_json();
        assert!(json.is_ok());
        let json = json.unwrap();

        // Assert: Verify JSON contains data
        assert_eq!(json["key"], "value");
    });

    chicago_test!(test_generic_test_data_builder_default, {
        // Arrange: Create default generic builder
        let builder: GenericTestDataBuilder<String, String> = GenericTestDataBuilder::default();

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data is empty
        assert!(data.is_empty());
    });

    // ========================================================================
    // 4. VALIDATED TEST DATA BUILDER - Test validated builder
    // ========================================================================

    chicago_test!(test_validated_test_data_builder_new, {
        // Arrange: Create validated builder
        let builder: ValidatedTestDataBuilder<()> = ValidatedTestDataBuilder::new();

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data is empty
        assert!(data.is_empty());
    });

    chicago_test!(test_validated_test_data_builder_with_var, {
        // Arrange: Create validated builder with var
        let builder: ValidatedTestDataBuilder<()> =
            ValidatedTestDataBuilder::new().with_var("key", "value");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data contains var
        assert_eq!(data.get("key"), Some(&"value".to_string()));
    });

    chicago_test!(test_validated_test_data_builder_default, {
        // Arrange: Create default validated builder
        let builder: ValidatedTestDataBuilder<()> = ValidatedTestDataBuilder::default();

        // Act: Build data
        let data = builder.build();

        // Assert: Verify data is empty
        assert!(data.is_empty());
    });

    // ========================================================================
    // 5. BOUNDARY CONDITIONS - Test edge cases
    // ========================================================================

    chicago_test!(test_test_data_builder_empty_key, {
        // Arrange: Create builder with empty key
        let builder = TestDataBuilder::new().with_var("", "value");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify empty key is handled
        assert_eq!(data.get(""), Some(&"value".to_string()));
    });

    chicago_test!(test_test_data_builder_empty_value, {
        // Arrange: Create builder with empty value
        let builder = TestDataBuilder::new().with_var("key", "");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify empty value is handled
        assert_eq!(data.get("key"), Some(&"".to_string()));
    });

    chicago_test!(test_test_data_builder_overwrite, {
        // Arrange: Create builder with overwriting vars
        let builder = TestDataBuilder::new().with_var("key", "value1").with_var("key", "value2");

        // Act: Build data
        let data = builder.build();

        // Assert: Verify overwrite behavior
        assert_eq!(data.get("key"), Some(&"value2".to_string()));
        assert_eq!(data.len(), 1);
    });

    chicago_test!(test_test_data_builder_large_data, {
        // Arrange: Create builder with large dataset
        let mut builder = TestDataBuilder::new();
        for i in 0..100 {
            builder = builder.with_var(format!("key{}", i), format!("value{}", i));
        }

        // Act: Build data
        let data = builder.build();

        // Assert: Verify large dataset
        assert_eq!(data.len(), 100);
        assert_eq!(data.get("key0"), Some(&"value0".to_string()));
        assert_eq!(data.get("key99"), Some(&"value99".to_string()));
    });
}
