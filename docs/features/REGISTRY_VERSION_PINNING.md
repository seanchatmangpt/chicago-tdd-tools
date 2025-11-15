# Registry Version Pinning Guide

## Quick Start

Pin to a known-good registry version to avoid upstream validation errors:

```bash
export WEAVER_REGISTRY_VERSION=v1.25.0
cargo make weaver-bootstrap
```

## Configuration Options

### Environment Variable (Recommended)

Set `WEAVER_REGISTRY_VERSION` before running bootstrap:

```bash
export WEAVER_REGISTRY_VERSION=v1.25.0
cargo make weaver-bootstrap
```

### Configuration File

Add to `chicago-tdd-tools.toml`:

```toml
[observability.weaver]
registry_version = "v1.25.0"
```

**Note**: Configuration file support is planned but not yet implemented. Use environment variable for now.

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

1. **Build-time** (`build.rs`): Reads `WEAVER_REGISTRY_VERSION` env var, clones specific version
2. **Runtime** (`weaver/mod.rs`): Reads `WEAVER_REGISTRY_VERSION` env var, clones specific version if registry missing
3. **Bootstrap script** (`scripts/weaver-bootstrap.sh`): Reads `WEAVER_REGISTRY_VERSION` env var, clones specific version

All three locations respect the environment variable for consistency.

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

## Related Documentation

- [Weaver Live Check](WEAVER_LIVE_CHECK.md) - Weaver integration testing guide

