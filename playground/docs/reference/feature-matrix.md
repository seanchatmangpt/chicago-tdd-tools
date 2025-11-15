# Feature Matrix

**Complete matrix** showing which examples use which features.

## Core Features Matrix

| Example | Fixtures | Builders | Assertions | Macros | State | Const | Alert |
|---------|----------|----------|-----------|--------|-------|-------|-------|
| fixtures | ✅ | - | ✅ | ✅ | - | - | - |
| builders | - | ✅ | ✅ | ✅ | - | - | - |
| assertions | ✅ | - | ✅ | ✅ | - | - | - |
| macros | - | - | - | ✅ | - | - | - |
| state | ✅ | - | ✅ | ✅ | ✅ | - | - |
| const | - | - | - | - | - | ✅ | - |
| alert | ✅ | - | - | ✅ | - | - | ✅ |

**✅ = Feature is used in example**
**- = Feature not highlighted in example**

## Testing Features Matrix

| Example | Property | Mutation | Snapshot | Concurrency | CLI | Generator |
|---------|----------|----------|----------|-------------|-----|-----------|
| prop | ✅ | - | - | - | - | - |
| mut | - | ✅ | - | - | - | - |
| snap | - | - | ✅ | - | - | - |
| conc | - | - | - | ✅ | - | - |
| cli | - | - | - | - | ✅ | - |
| gen | - | - | - | - | - | ✅ |

## Feature Requirements by Category

### Core Features (Always Available)

| Feature | Requirement | Status |
|---------|------------|--------|
| Fixtures | - | ✅ |
| Builders | - | ✅ |
| Assertions | - | ✅ |
| Macros | - | ✅ |
| State | - | ✅ |
| Const Assert | - | ✅ |
| Alert | - | ✅ |

### Testing Features (Optional)

| Feature | Flag | Requirement | Status |
|---------|------|------------|--------|
| Property-based | `property-testing` | proptest crate | ✅ |
| Mutation | `mutation-testing` | Internal | ✅ |
| Snapshot | `snapshot-testing` | insta crate | ✅ |
| Concurrency | `concurrency-testing` | loom crate | ✅ |
| CLI | `cli-testing` | assert_cmd | ✅ |
| Generator | (internal) | - | ✅ |

### Validation Features (Optional)

| Feature | Flag | Requirement | Status |
|---------|------|------------|--------|
| Coverage | `coverage` | - | ✅ |
| Guards | `guards` | - | ✅ |
| JTBD | `jtbd` | - | ✅ |
| Performance | `benchmarking` | - | ✅ |

### Observability Features (Optional)

| Feature | Flag | Requirement | Status |
|---------|------|------------|--------|
| OTEL | `otel` | opentelemetry | ✅ |
| Weaver | `weaver` | opentelemetry + Weaver CLI | ✅ |

### Integration Features (Optional)

| Feature | Flag | Requirement | Status |
|---------|------|------------|--------|
| Testcontainers | `testcontainers` | Docker | ✅ |

## Example Feature Coverage

### High-Level Feature Distribution

```
Core Features:     7 examples (100% coverage)
Testing Features:  6 examples (4 main techniques)
Validation:        4 examples (quality assurance)
Observability:     4 examples (OTEL + Weaver)
Integration:       1 example (Docker)
```

### Feature Complexity Levels

| Level | Examples | Complexity |
|-------|----------|-----------|
| Beginner | fixtures, builders | Simple patterns |
| Intermediate | assertions, macros, property, snapshot | Combined patterns |
| Advanced | state, mutation, concurrency, coverage | Type-level or complex testing |
| Expert | OTEL, Weaver, testcontainers | External systems integration |

## Feature Combinations

### Minimum Set (Learning Basics)

```
✅ fixtures
✅ builders
✅ assertions
✅ macros
```

Command:
```bash
cargo run -- core exec --names "fixtures builders assertions macros"
```

### Standard Set (80% Use Case)

```
Core:
✅ fixtures, builders, assertions, macros, state

Testing:
✅ property, snapshot, fake-data (testing-extras)
```

Command:
```bash
cargo run --features testing-extras -- core exec --names "fixtures builders assertions macros"
cargo run --features testing-extras -- test exec --names "prop snap"
```

### Complete Set (Advanced)

```
Core: All 7
Testing: All 6
Validation: All 4
Observability: OTEL + Weaver
Integration: Testcontainers
```

Command:
```bash
cargo run --all-features -- core stat
cargo run --all-features -- test stat
cargo run --all-features -- valid stat
cargo run --all-features -- obs stat
cargo run --all-features -- integ stat
```

## Feature Dependency Graph

```
Fixtures ───┬──→ Builders ──→ Assertions
            │
            └──→ Assertions ──→ Macros
                               ├──→ State
                               └──→ Property Testing
                                    ├──→ Mutation Testing
                                    └──→ Concurrency Testing
                                         ├──→ Coverage Analysis
                                         └──→ Snapshot Testing
                                              └──→ OTEL
                                                   └──→ Weaver
                                                        └──→ Testcontainers
```

## Real-World Example Combinations

### Web API Testing

```
Core: fixtures, builders, assertions, macros
Testing: property, snapshot
Observability: OTEL, Weaver
Integration: testcontainers (for database testing)
```

### CLI Tool Testing

```
Core: fixtures, builders, assertions, macros
Testing: CLI, snapshot
Validation: coverage, guards
```

### Library Testing

```
Core: all
Testing: property, mutation
Validation: all
```

### Data Processing

```
Core: fixtures, builders, assertions
Testing: property (for data properties), snapshot
Validation: coverage, performance
```

## Feature Maturity

| Feature | Status | Stability | Support |
|---------|--------|-----------|---------|
| Fixtures | Mature | Production | Full |
| Builders | Mature | Production | Full |
| Assertions | Mature | Production | Full |
| Macros | Mature | Production | Full |
| State | Mature | Production | Full |
| Property | Beta | Production | Full |
| Mutation | Alpha | Experimental | Community |
| Snapshot | Mature | Production | Full |
| Concurrency | Mature | Production | Full |
| OTEL | Beta | Production | Full |
| Weaver | Beta | Production | Full |
| Testcontainers | Mature | Production | Full |

## Quick Selection Guide

**Question → Answer → Use These Features**

- "I want basic testing?" → Use Core (fixtures, builders, assertions)
- "I want property-based testing?" → Add testing-extras
- "I want quality metrics?" → Add validation features
- "I want observability?" → Add otel, weaver
- "I want integration testing?" → Add testcontainers
- "I want everything?" → Use --all-features

## Next Steps

- **See all examples** → [Example Inventory](example-inventory.md)
- **See CLI commands** → [CLI Command Reference](cli-commands.md)
- **Understand organization** → [Directory Structure](directory-structure.md)

---

Use this matrix to understand which examples demonstrate which features.
