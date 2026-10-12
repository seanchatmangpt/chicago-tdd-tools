# Registry Version Pinning Guide

## Quick Start

Pin to a known-good registry version to avoid upstream validation errors:

```bash
export WEAVER_REGISTRY_VERSION=v1.25.0
just weaver-bootstrap
```

## Configuration Options

### Environment Variable (Recommended)

Set `WEAVER_REGISTRY_VERSION` before running bootstrap:

```bash
export WEAVER_REGISTRY_VERSION=v1.25.0
just weaver-bootstrap
```

### Configuration File

Add to `chicago-tdd-tools.toml`:

```toml
[observability.weaver]
registry_version = "v1.25.0"
```

**Note**: The version can be pinned via `chicago-tdd-tools.toml` or overridden with the `WEAVER_REGISTRY_VERSION` environment variable (read in `src/core/config/loading.rs`).

## Available Versions

Check available registry versions:

```bash
git ls-remote --tags https://github.com/open-telemetry/semantic-conventions.git
```

**Known-good versions** (tested):
- `v1.25.0` - Older stable version (has different validation errors)
- `v1.30.0` - More recent (has XPath validation error)

**Latest version**: May have validation errors - use pinning for stability.

**Note**: Even pinned versions may have some validation errors. Different versions may have different errors. Use `WEAVER_SKIP_REGISTRY_VALIDATION=1` as workaround if needed.

## Why Pin?

**Problem**: Upstream registry may have validation errors:
- Missing `note` fields in enum-type attributes
- Invalid XPath expressions
- Schema validation failures

**Solution**: Pin to a known-good version that has been tested and validated.

## How It Works

> **AUDIT NOTE (W15):** `WEAVER_REGISTRY_VERSION` is read only in
> `src/core/config/loading.rs`; `build.rs` and the bootstrap script do
> not read it (the script reads `WEAVER_VERSION` — the tool release).




1. **Config loading** (`src/core/config/loading.rs`): The only reader of
   `WEAVER_REGISTRY_VERSION` — resolves the pinned version into the
   `observability.weaver.registry_version` config value (the TOML key above).
2. **Bootstrap script** (`scripts/weaver-bootstrap.sh`): Reads `WEAVER_VERSION`
   (the Weaver *tool* release — a different variable) to download the binary.
3. `build.rs` and the runtime registry fallback do not read the pin; they clone
   whatever default the clone target provides.

The pin takes effect by being loaded into config before bootstrap; set the env
var (or the TOML key) and re-run `just weaver-bootstrap`.

## Troubleshooting

**Registry still has errors after pinning**:
- Some validation errors may persist across versions (e.g., XPath errors)
- Use `WEAVER_SKIP_REGISTRY_VALIDATION=1` as workaround
- Report upstream issues to OpenTelemetry semantic conventions repository

**Version not found**:
- Verify version exists: `git ls-remote --tags https://github.com/open-telemetry/semantic-conventions.git`
- Use tag format: `v1.25.0` (not `1.25.0`)
- Can also use branch names: `main`, `v1.25.x`
- Can also use commit hashes

**Tests still fail**:
- Weaver validates registry during startup (can't be skipped)
- Even with `WEAVER_SKIP_REGISTRY_VALIDATION=1`, Weaver itself validates
- Pin to older version or wait for upstream fixes

## See Also

- **[Weaver Live Check](WEAVER_LIVE_CHECK.md)** - Weaver integration testing guide
- **[Perfect Weaver Live Check](PERFECT_WEAVER_LIVE_CHECK.md)** - Perfect implementation guide
- **[Upstream Issue Reporting](UPSTREAM_ISSUE_REPORTING.md)** - Reporting upstream issues
- **[Observability Testing Guide](../observability/observability-testing-guide.md)** - Comprehensive observability testing
- **[OTEL & Weaver Guide](../observability/otel-weaver-guide.md)** - OTEL and Weaver integration
