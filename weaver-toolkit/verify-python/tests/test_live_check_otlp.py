"""Verify the weaver live-check OTLP pipeline end to end from a real Python
OpenTelemetry SDK -- the actual scenario weaver-toolkit exists to support
(a non-Rust project emitting real spans at a standing weaver listener).

No fake exporter, no canned report.json: a real `opentelemetry-sdk`
TracerProvider exports over a real gRPC connection to the real weaver
process started by weaver-wrapper.sh, and assertions read the real
report.json weaver itself wrote after `live-stop`.
"""

from __future__ import annotations

from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

from .conftest import LiveCheckSession, stop_and_load_report


def _send_one_span(session: LiveCheckSession, span_name: str, resource_attrs: dict) -> None:
    provider = TracerProvider(resource=Resource.create(resource_attrs))
    exporter = OTLPSpanExporter(endpoint=session.grpc_endpoint, insecure=True)
    processor = BatchSpanProcessor(exporter, schedule_delay_millis=100)
    provider.add_span_processor(processor)

    tracer = provider.get_tracer("weaver-toolkit-verify-python")
    with tracer.start_as_current_span(span_name) as span:
        span.set_attribute("http.request.method", "GET")

    # Force the batch out now rather than waiting on the export interval,
    # then give weaver's live-check receiver a moment to process it before
    # we ask it to stop and write the report.
    processor.force_flush(timeout_millis=5000)
    provider.shutdown()


def test_live_check_ingests_a_real_span_over_grpc(live_check: LiveCheckSession) -> None:
    # Arrange & Act: a real OTel SDK sends a real span to the real weaver process
    _send_one_span(
        live_check,
        span_name="weaver_toolkit_verify_python_span",
        resource_attrs={"service.name": "weaver-toolkit-verify-python"},
    )

    # Assert: the report weaver itself wrote actually recorded our span,
    # proving the OTLP receiver -> live-check -> JSON report pipeline works
    # when driven from outside Rust.
    report = stop_and_load_report(live_check)
    assert report["samples"], "expected at least one sample in the live-check report"

    span_samples = [s for s in report["samples"] if "span" in s]
    assert any(
        s["span"]["name"] == "weaver_toolkit_verify_python_span" for s in span_samples
    ), f"our span was not found in the report: {[s['span']['name'] for s in span_samples]}"


def test_live_check_flags_an_unregistered_attribute(live_check: LiveCheckSession) -> None:
    # Arrange & Act: send a resource attribute that is not part of any
    # semantic-conventions registry entry -- a deliberately-invalid input,
    # the negative-path counterpart to the positive test above.
    made_up_attribute = "chicago.tdd.tools.verify_python.not_a_real_semconv_key"
    _send_one_span(
        live_check,
        span_name="weaver_toolkit_verify_python_negative_span",
        resource_attrs={
            "service.name": "weaver-toolkit-verify-python",
            made_up_attribute: "marker-value",
        },
    )

    # Assert: weaver's real advisory engine actually flagged the unknown
    # attribute -- this proves the registry is genuinely being consulted,
    # not just that spans are accepted unconditionally.
    report = stop_and_load_report(live_check)
    resource_samples = [s for s in report["samples"] if "resource" in s]
    assert resource_samples, "expected at least one resource sample in the report"

    flagged = [
        attr
        for sample in resource_samples
        for attr in sample["resource"]["attributes"]
        if attr["name"] == made_up_attribute
    ]
    assert flagged, f"made-up attribute {made_up_attribute!r} was not recorded in the report at all"

    advice = flagged[0]["live_check_result"]["all_advice"]
    assert any(item["id"] == "missing_attribute" for item in advice), (
        f"expected a missing_attribute advisory for {made_up_attribute!r}, got: {advice}"
    )
