import os

for f in ["src/bin/otel_production_run.rs", "src/bin/otel_negative_run.rs"]:
    with open(f, "r") as file:
        content = file.read()
    content = content.replace('let mut span = tracer.span_builder(', 'let mut span = tracer.span_builder(')
    content = content.replace('.start(&tracer);', '.with_kind(opentelemetry::trace::SpanKind::Client).start(&tracer);')
    with open(f, "w") as file:
        file.write(content)
