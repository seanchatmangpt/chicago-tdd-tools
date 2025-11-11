//! Tests for Weaver test macros

#![cfg(all(feature = "weaver", feature = "otel"))]
#![allow(missing_docs)]

use chicago_tdd_tools::{weaver_async_test, weaver_test};
use opentelemetry::trace::Tracer as _;
use opentelemetry::KeyValue;

weaver_test!(weaver_macro_happy_path, |fixture| {
    let tracer = fixture.tracer("macro-happy", "weaver-macro-tests")?;
    let mut span = tracer.tracer().start("macro-happy-span");
    span.set_attribute(KeyValue::new("service.name", "weaver-macro-tests"));
    span.end();
    Ok(())
});

#[should_panic(expected = "Weaver live-check validation failed")]
weaver_test!(weaver_macro_detects_violation, |fixture| {
    let tracer = fixture.tracer("macro-violation", "weaver-macro-tests")?;
    let mut span = tracer.tracer().start("macro-violation-span");
    span.set_attribute(KeyValue::new("unknown.attribute", "value"));
    span.end();
    Ok(())
});

weaver_async_test!(weaver_async_macro_happy_path, |fixture| {
    let tracer = fixture.tracer("macro-async", "weaver-macro-tests")?;
    let mut span = tracer.tracer().start("macro-async-span");
    span.set_attribute(KeyValue::new("service.name", "weaver-macro-tests"));
    span.end();
    Ok(())
});
