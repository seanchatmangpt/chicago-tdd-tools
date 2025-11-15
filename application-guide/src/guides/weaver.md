# Weaver Live-Check Validation

Weaver validates telemetry against semantic conventions.

## What is Weaver?

Weaver ensures your OTEL telemetry complies with OpenTelemetry semantic conventions - the industry standard for attribute names and structure.

## Checking Weaver Availability

```rust
use chicago_tdd_tools::observability::weaver::WeaverValidator;

test!(test_weaver_check, {
    match WeaverValidator::check_weaver_available() {
        Ok(()) => {
            alert_success!("Weaver is available");
            // Can validate telemetry
        }
        Err(e) => {
            alert_info!("Weaver not available: {}", e);
            // Install with: cargo make weaver-bootstrap
        }
    }
});
```

## Installing Weaver

```bash
# Bootstrap Weaver
cargo make weaver-bootstrap

# Run smoke test
cargo make weaver-smoke
```

## Semantic Conventions

Weaver checks that your attributes follow conventions:

```rust
// ✅ Correct - follows semantic conventions
let mut span = create_span("http.request");
span.attributes.insert("http.method".to_string(), "GET".to_string());
span.attributes.insert("http.target".to_string(), "/api/users".to_string());
span.attributes.insert("http.status_code".to_string(), "200".to_string());

// ❌ Wrong - custom attributes
span.attributes.insert("method".to_string(), "GET".to_string());
span.attributes.insert("endpoint".to_string(), "/api/users".to_string());
```

## Common Conventions

| Attribute | Format | Example |
|-----------|--------|---------|
| `http.method` | HTTP method | GET, POST, PUT |
| `http.status_code` | Integer | 200, 404, 500 |
| `http.target` | Path | /api/users |
| `db.system` | Database type | mysql, postgresql |
| `db.operation` | SQL operation | SELECT, INSERT |

## Best Practices

✅ **Do:**
- Follow semantic conventions
- Validate with Weaver
- Document telemetry schema
- Keep conventions up-to-date

❌ **Don't:**
- Use custom attribute names
- Ignore Weaver validation
- Duplicate information in attributes

## Troubleshooting

### Weaver Binary Not Found

Install Weaver:

```bash
cargo make weaver-bootstrap
```

### Validation Fails

Check attribute names against conventions:

```bash
# See Weaver registry
cargo make weaver-smoke
```

## Next Steps

Combine observability with testing: [Observability & Quality](observability.md)
