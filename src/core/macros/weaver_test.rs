//! Macros for Weaver live-check integrated Chicago TDD tests.

#![cfg(all(feature = "weaver", feature = "otel"))]

/// Synchronous Weaver test helper.
///
/// ```
/// use chicago_tdd_tools::weaver_test;
///
/// weaver_test!(valid_span, |fixture| {
///     let tracer = fixture.tracer("example", "example-service")?;
///     let mut span = tracer.tracer().start("valid-span");
///     span.end();
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! weaver_test {
    ($name:ident, |$fixture:ident| $body:block) => {
        #[test]
        fn $name() {
            use $crate::observability::fixtures::WeaverTestFixture;

            let mut fixture = WeaverTestFixture::new()
                .unwrap_or_else(|err| panic!("Failed to initialise Weaver fixture: {err}"));

            // Wrap body in a closure that returns Result to allow `?` operator
            let result: $crate::observability::ObservabilityResult<()> = (|| {
                let $fixture = &mut fixture;
                $body
            })();

            if let Err(err) = result {
                panic!("Weaver test body returned error: {err}");
            }

            let validation = fixture
                .finish()
                .unwrap_or_else(|err| panic!("Failed to finish Weaver fixture: {err}"));

            $crate::assert_telemetry_valid!(&validation);
        }
    };
}

/// Asynchronous Weaver test helper (requires `tokio` runtime).
#[macro_export]
macro_rules! weaver_async_test {
    ($name:ident, |$fixture:ident| $body:block) => {
        #[tokio::test]
        async fn $name() {
            use $crate::observability::fixtures::WeaverTestFixture;

            let mut fixture = WeaverTestFixture::new()
                .unwrap_or_else(|err| panic!("Failed to initialise Weaver fixture: {err}"));

            let result: $crate::observability::ObservabilityResult<()> = {
                let $fixture = &mut fixture;
                async move $body
            }
            .await;

            if let Err(err) = result {
                panic!("Weaver async test body returned error: {err}");
            }

            let validation = fixture
                .finish()
                .unwrap_or_else(|err| panic!("Failed to finish Weaver fixture: {err}"));

            $crate::assert_telemetry_valid!(&validation);
        }
    };
}
