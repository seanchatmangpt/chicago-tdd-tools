//! Tests for Weaver test macros

#![cfg(all(feature = "weaver", feature = "otel"))]
#![allow(missing_docs)]

use chicago_tdd_tools::{weaver_async_test, weaver_test};
use opentelemetry::trace::{Span as _, Tracer as _};
use opentelemetry::KeyValue;

weaver_test!(weaver_macro_happy_path, |fixture| {
    let tracer = fixture.tracer("macro-happy", "weaver-macro-tests")?;
    let mut span = tracer.tracer().start("macro-happy-span");
    span.set_attribute(KeyValue::new("service.name", "weaver-macro-tests"));
    span.end();
    Ok(())
});

// Note: #[should_panic] cannot be used on macro calls, so we test violation detection
// in a regular test function
#[test]
#[should_panic(expected = "Weaver live-check validation failed")]
fn weaver_macro_detects_violation() {
    use chicago_tdd_tools::observability::fixtures::WeaverTestFixture;
    use opentelemetry::trace::{Span as _, Tracer as _};
    use opentelemetry::KeyValue;

    let mut fixture = WeaverTestFixture::new()
        .unwrap_or_else(|err| panic!("Failed to initialise Weaver fixture: {err}"));

    let tracer = fixture
        .tracer("macro-violation", "weaver-macro-tests")
        .unwrap_or_else(|err| panic!("Failed to create tracer: {err}"));
    let mut span = tracer.tracer().start("macro-violation-span");
    span.set_attribute(KeyValue::new("unknown.attribute", "value"));
    span.end();

    let validation = fixture
        .finish()
        .unwrap_or_else(|err| panic!("Failed to finish Weaver fixture: {err}"));

    chicago_tdd_tools::assert_telemetry_valid!(&validation);
}

weaver_async_test!(weaver_async_macro_happy_path, |fixture| {
    let tracer = fixture.tracer("macro-async", "weaver-macro-tests")?;
    let mut span = tracer.tracer().start("macro-async-span");
    span.set_attribute(KeyValue::new("service.name", "weaver-macro-tests"));
    span.end();
    Ok(())
});
