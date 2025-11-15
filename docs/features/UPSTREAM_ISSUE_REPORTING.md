# Upstream Issue Reporting Guide

## When to Report Upstream Issues

Report upstream issues to OpenTelemetry semantic conventions repository when:
- Registry validation errors prevent Weaver from starting
- Schema validation errors are reproducible
- Errors affect multiple users/versions
- Errors are not environment-specific

## When to Work Around

Use workarounds (version pinning, skip validation) when:
- Errors are known upstream issues
- Upstream fix is in progress
- Errors don't block critical functionality
- Temporary workaround is acceptable

## How to Report Upstream Issues

### 1. Verify Issue is Reproducible

```bash
# Run Weaver registry check
weaver registry check -r registry

# Capture full error output
weaver registry check -r registry > registry-validation-error.txt 2>&1
```

### 2. File Issue on GitHub

**Repository**: https://github.com/open-telemetry/semantic-conventions

**Issue Template**:
```markdown
## Description
Weaver registry validation fails with [error type]

## Steps to Reproduce
1. Clone semantic conventions registry
2. Run `weaver registry check -r registry`
3. Observe error: [paste error]

## Expected Behavior
Registry should pass Weaver validation

## Actual Behavior
Registry validation fails with: [error details]

## Environment
- Weaver version: [version]
- Registry version: [version/tag]
- OS: [OS]

## Additional Context
[Any additional information]
```

### 3. Link to Issue in Documentation

Once issue is filed, update documentation:
- Add issue link to `docs/REGISTRY_VALIDATION_FIX.md`
- Document workaround until fix is available
- Update when upstream fix is released

## Current Known Issues

### XPath Validation Error
- **Issue**: `Invalid XPath `` detected while validating semantic convention spec`
- **Status**: Upstream issue
- **Workaround**: Use `WEAVER_SKIP_REGISTRY_VALIDATION=1` or pin to older version
- **GitHub Issue**: [Link when filed]

### Missing Note Fields
- **Issue**: Enum-type attributes missing required `note` fields
- **Status**: Fixed locally (should be upstreamed)
- **Workaround**: Use version pinning or local fixes
- **GitHub Issue**: [Link when filed]

## Best Practices

1. **Don't Fix Locally**: Report upstream instead of fixing locally
2. **Document Workarounds**: Document workarounds until upstream fix
3. **Update When Fixed**: Update documentation when upstream fixes are released
4. **Version Pinning**: Pin to known-good versions until fixes are available

## See Also

- **[Registry Version Pinning](REGISTRY_VERSION_PINNING.md)** - How to pin registry versions
- **[Weaver Live Check](WEAVER_LIVE_CHECK.md)** - Weaver integration testing guide
- **[Perfect Weaver Live Check](PERFECT_WEAVER_LIVE_CHECK.md)** - Perfect implementation guide
