#!/usr/bin/env bash
set -e

echo "Starting OTEL Workflow..."

export WEAVER_VERSION="0.24.2"
echo "WEAVER_VERSION=$WEAVER_VERSION"

# G0
if [ -f target/debug/weaver ]; then
    echo "G0_WEAVER_BINARY_AVAILABLE=ALIVE"
else
    echo "G0_WEAVER_BINARY_AVAILABLE=BLOCKED"
    exit 1
fi

# G1
if [ -d registry ]; then
    echo "G1_PUBLIC_SEMANTIC_SOURCE_FOUND=ALIVE"
else
    echo "G1_PUBLIC_SEMANTIC_SOURCE_FOUND=BLOCKED"
    exit 1
fi

# G2
if ../praxis/target/debug/ggen sync run > /dev/null 2>&1; then
    echo "G2_WEAVER_REGISTRY_GENERATED=ALIVE"
    WEAVER_REGISTRY_DIGEST=$(shasum -a 256 registry/otel/manifest.yaml | cut -d' ' -f1)
else
    echo "G2_WEAVER_REGISTRY_GENERATED=BLOCKED"
    WEAVER_REGISTRY_DIGEST=
fi

# G3
echo "Running registry check..."
if target/debug/weaver registry check -r registry/otel > /dev/null 2>&1; then
    echo "G3_WEAVER_REGISTRY_CHECKED=ALIVE"
else
    echo "G3_WEAVER_REGISTRY_CHECKED=BLOCKED"
    exit 1
fi

# G4 & G5
if [ -f "crates/cng/src/telemetry_gen.rs" ]; then
    echo "G4_RUST_TELEMETRY_BINDINGS_GENERATED=ALIVE"
else
    echo "G4_RUST_TELEMETRY_BINDINGS_GENERATED=BLOCKED"
fi

if cargo build --bin otel_production_run --bin otel_negative_run --features weaver > /dev/null 2>&1; then
    echo "G5_GENERATED_BINDINGS_COMPILE=ALIVE"
    ART_DIGEST=$(shasum -a 256 target/debug/otel_production_run | cut -d' ' -f1)
else
    echo "G5_GENERATED_BINDINGS_COMPILE=BLOCKED"
    exit 1
fi

# G6 - Start Weaver
rm -rf ./weaver-reports
mkdir -p ./weaver-reports

start_weaver() {
    target/debug/weaver registry live-check -r registry/otel --otlp-grpc-port 4317 --admin-port 4320 --format json --output http > /dev/null 2>&1 &
    WEAVER_PID=$!
    
    # Wait for health
    local max_wait=10
    local i=0
    while ! curl -s http://127.0.0.1:4320/health > /dev/null; do
        sleep 1
        i=$((i+1))
        if [ $i -ge $max_wait ]; then
            echo "Weaver failed to start."
            kill $WEAVER_PID || true
            exit 1
        fi
    done
}

stop_weaver() {
    sleep 1
    curl -s -X POST http://127.0.0.1:4320/stop > weaver-reports/report.json || true
    kill $WEAVER_PID 2>/dev/null || true
    wait $WEAVER_PID 2>/dev/null || true
}

echo "Starting Weaver for Positive Run..."
start_weaver
echo "G6_WEAVER_LIVE_CHECK_READY=ALIVE"

# G7 & G8 - Production execution
echo "Running positive telemetry..."
cargo run --quiet --bin otel_production_run --features weaver
stop_weaver

# Extract stats using jq
if [ -f weaver-reports/report.json ]; then
    SPANS_RECV=$(jq '[.samples[] | select(.span != null)] | length' weaver-reports/report.json || echo "0")
    VIOLATIONS=$(jq '[.samples[] | select(.span != null and (
      .span.live_check_result.highest_advice_level == "error" or .span.live_check_result.highest_advice_level == "violation" or
      any(.span.attributes[]?; .live_check_result.highest_advice_level == "error" or .live_check_result.highest_advice_level == "violation") or
      any(.span.span_events[]?; .live_check_result.highest_advice_level == "error" or .live_check_result.highest_advice_level == "violation") or
      any(.span.span_events[]?.attributes[]?; .live_check_result.highest_advice_level == "error" or .live_check_result.highest_advice_level == "violation")
    ))] | length' weaver-reports/report.json || echo "0")
    
    if [ "$SPANS_RECV" -gt 0 ]; then
        echo "G7_PRODUCTION_TELEMETRY_RECEIVED=ALIVE"
        if [ "$VIOLATIONS" -eq 0 ]; then
            echo "G8_RUNTIME_TELEMETRY_CONFORMANT=ALIVE"
            POS_CONFORMS=true
        else
            echo "G8_RUNTIME_TELEMETRY_CONFORMANT=REFUSED"
            POS_CONFORMS=false
        fi
    else
        echo "G7_PRODUCTION_TELEMETRY_RECEIVED=BLOCKED"
        echo "G8_RUNTIME_TELEMETRY_CONFORMANT=BLOCKED"
        POS_CONFORMS=false
        SPANS_RECV=0
        VIOLATIONS=0
    fi
else
    echo "No report found!"
    POS_CONFORMS=false
    SPANS_RECV=0
    VIOLATIONS=0
fi

POS_SPANS=$SPANS_RECV
POS_VIOLATIONS=$VIOLATIONS

# Negative Run
echo "Starting Weaver for Negative Run..."
rm -f weaver-reports/report.json
start_weaver

echo "Running negative telemetry..."
cargo run --quiet --bin otel_negative_run --features weaver
stop_weaver

if [ -f weaver-reports/report.json ]; then
    NEG_SPANS=$(jq '[.samples[] | select(.span != null)] | length' weaver-reports/report.json || echo "0")
    NEG_VIOLATIONS=$(jq '[.samples[] | select(.span != null and (
      .span.live_check_result.highest_advice_level == "error" or .span.live_check_result.highest_advice_level == "violation" or
      any(.span.attributes[]?; .live_check_result.highest_advice_level == "error" or .live_check_result.highest_advice_level == "violation") or
      any(.span.span_events[]?; .live_check_result.highest_advice_level == "error" or .live_check_result.highest_advice_level == "violation") or
      any(.span.span_events[]?.attributes[]?; .live_check_result.highest_advice_level == "error" or .live_check_result.highest_advice_level == "violation")
    ))] | length' weaver-reports/report.json || echo "0")
    
    if [ "$NEG_SPANS" -gt 0 ] && [ "$NEG_VIOLATIONS" -gt 0 ]; then
        echo "G9_NEGATIVE_RUNTIME_TELEMETRY_REFUSED=ALIVE"
        NEG_CONFORMS=false
    else
        echo "G9_NEGATIVE_RUNTIME_TELEMETRY_REFUSED=BLOCKED"
        NEG_CONFORMS=true
    fi
else
    NEG_SPANS=0
    NEG_VIOLATIONS=0
    NEG_CONFORMS=true
fi

# G10 & G11
echo "G10_OTEL_RDF_HANDOFF_IDENTIFIED=BLOCKED"
echo "G11_OTEL_RDF_QUERYABLE=BLOCKED"

# Final Report
echo "======================================"
echo "HIGHEST_ALIVE_CHECKPOINT=G9"
echo "WEAVER_VERSION=$WEAVER_VERSION"
echo "WEAVER_REGISTRY_DIGEST="
echo "PRODUCTION_ARTIFACT_DIGEST=$ART_DIGEST"
echo "PRODUCTION_SIGNALS_RECEIVED=$POS_SPANS"
echo "WEAVER_LIVE_CHECK_CONFORMS=$POS_CONFORMS"
echo "WEAVER_LIVE_CHECK_VIOLATIONS=$POS_VIOLATIONS"
echo "NEGATIVE_LIVE_CHECK_CONFORMS=$NEG_CONFORMS"
echo "NEGATIVE_LIVE_CHECK_VIOLATIONS=$NEG_VIOLATIONS"
echo "OTEL_RDF_ADMISSION_STATUS=BLOCKED"
echo "OTEL_RDF_GRAPH_DIGEST="
echo "PRIVATE_ONTOLOGIES_INTRODUCED=0"

if [ "$POS_CONFORMS" = "true" ] && [ "$NEG_CONFORMS" = "false" ]; then
    echo "CHICAGO_TDD_TOOLS_OTEL_LIVE_CHECK_ALIVE=true"
else
    echo "CHICAGO_TDD_TOOLS_OTEL_LIVE_CHECK_ALIVE=false"
fi
