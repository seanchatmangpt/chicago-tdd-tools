#![allow(warnings)]
#[cfg(feature = "weaver")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::trace::{Span, Tracer, TracerProvider as _};
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
    use opentelemetry_sdk::Resource;
    use std::time::Duration;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // Overridable so this standalone test producer can be pointed at an
        // isolated port (e.g. via scripts/otel-workflow.sh) instead of the
        // crate's DEFAULT_OTLP_GRPC_PORT (4317) -- keeps this smoke test
        // from ever colliding with a real, non-test OTLP collector (e.g.
        // Istio/Jaeger's own use of 4317 in a consuming project's cluster).
        let port = std::env::var("WEAVER_TEST_OTLP_PORT").unwrap_or_else(|_| "4317".to_string());
        let endpoint = format!("http://127.0.0.1:{port}");
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", &endpoint);

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&endpoint)
            .build()?;

        let resource = Resource::builder_empty()
            .with_service_name("chicago-tdd-tools-test")
            .with_attributes([
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("telemetry.sdk.language", "rust"),
                KeyValue::new("telemetry.sdk.name", "opentelemetry"),
                KeyValue::new("telemetry.sdk.version", "0.31.0"),
            ])
            .build();

        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_sampler(Sampler::TraceIdRatioBased(1.0))
            .with_id_generator(RandomIdGenerator::default())
            .with_resource(resource)
            .build();

        let tracer = provider.tracer("chicago-tdd-tools");

        // Positive case: Praxis Activity Executed event
        let mut span = tracer.span_builder("workflow").start(&tracer);
        span.add_event(
            "praxis.activity_executed",
            vec![
                KeyValue::new("process.workflow.id", "wf-123"),
                KeyValue::new("process.object.id", "obj-456"),
                KeyValue::new("process.object.type", "Invoice"),
                KeyValue::new("process.activity.iri", "http://chatman-equation.org/core/Execute"),
                KeyValue::new("process.outcome", "completed"),
            ],
        );
        span.end();

        provider.force_flush()?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        provider.shutdown()?;
        Ok(())
    })
}

#[cfg(not(feature = "weaver"))]
fn main() {}
