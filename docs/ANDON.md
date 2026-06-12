# ANDON Operating Principle

The ANDON system in `chicago-tdd-tools` is a critical governance mechanism designed to provide mandatory interrupts for AI agents and human developers alike.

## The Principle

The core mandate of the ANDON principle is:

> **An agent treating `Severity::Andon` as a mandatory stop is the intended consumer.**

When a domain law assertion fails with `Severity::Andon`, it is not a suggestion or a warning; it is a **veto**. The artifact under evaluation has violated a fundamental law of the domain and must not be admitted into the system.

## Diagnostic Severities

| Severity | Meaning | Action Required |
|---|---|---|
| **Andon** | **STOP.** A domain law has been violated. | The agent MUST NOT declare the task as "done". Immediate repair is required. |
| **Warning** | **Caution.** A law is weakly violated or a threshold is approached. | The agent should investigate and address the issue, but it is not a hard block. |
| **Info** | **Note.** A noteworthy condition or a successful pass of a law. | No immediate action required. Used for audit trails and evidence accumulation. |

## Implementation in chicago-tdd-tools

In the `chicago-tdd-tools` framework, ANDON signals are emitted as structured `Diagnostic` records through the governance channel.

### Structured Diagnostic Codes

Every ANDON signal carries a stable diagnostic code in the format:
`{DOMAIN}-{CATEGORY}-{ORDINAL}`

For example: `MYAPP-ADM-001` (Admission violation in MYAPP domain).

### Integration with LSP

When integrated with a Language Server, `Severity::Andon` diagnostics are mapped to `DiagnosticSeverity::Error`. This ensures that the IDE provides a clear, visual indicator of the violation, and agents receiving push diagnostics are interrupted in the same turn as the edit.

## Governance Arc

The use of ANDON signals is the foundation of the Governance Arc. By making governance structural rather than instructional, we ensure that:
1. Domain laws are enforceable at the source level.
2. AI agents cannot self-certify artifacts that violate these laws.
3. Institutional intelligence compounds over time as admission boundaries are refined.

---
*chicago-tdd-tools Governance Documentation*
