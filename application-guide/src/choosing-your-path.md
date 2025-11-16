# Choosing Your Learning Path

> 🗺️ **NAVIGATION** | Find the right resources for your goals

This page helps you navigate the documentation based on what you want to accomplish.

---

## Quick Navigation by Goal

### "I'm just getting started"

**Duration**: 1-2 hours | **Difficulty**: Beginner

Start here and follow in order:

1. **[Introduction](introduction.md)** (10 min) - Understand Chicago TDD philosophy
2. **[Quick Start Tutorial](tutorials/getting-started.md)** (25 min) - Write your first test
3. **[Fixtures Deep Dive](tutorials/fixtures-tutorial.md)** (15 min) - Master test isolation
4. **[Core Testing Patterns](core/README.md)** (20 min) - Learn the essentials
5. **[Error Path Testing](core/error-paths.md)** (15 min) - Find where bugs hide

**Next step**: Pick one real-world tutorial based on what you want to build.

---

### "I want to build a CLI application"

**Duration**: 1-2 hours | **Difficulty**: Intermediate

Follow this path:

1. **[Quick Start](tutorials/getting-started.md)** (25 min) - Review basics
2. **[CLI Application Tutorial](tutorials/cli-app-tutorial.md)** (45 min) - Build complete CLI
3. **[CLI Testing](advanced/cli-testing.md)** (20 min) - Test CLI output with golden files
4. **[Best Practices](guides/best-practices.md)** (30 min) - Production patterns

**Key files to reference**:
- [TestFixture API](reference/fixtures-api.md) - For test isolation
- [Error Path Testing](core/error-paths.md) - Test error cases

---

### "I want to build a REST API / Web Service"

**Duration**: 1.5-2.5 hours | **Difficulty**: Intermediate

Follow this path:

1. **[Quick Start](tutorials/getting-started.md)** (25 min) - Review basics
2. **[REST Web Service Tutorial](tutorials/web-service-tutorial.md)** (50 min) - Build complete API
3. **[Integration Testing with Docker](guides/integration-docker.md)** (30 min) - Test with real database
4. **[Observability & Quality](guides/observability.md)** (20 min) - Add monitoring

**Key files to reference**:
- [Data Builders](core/data-builders.md) - For building test data
- [Error Path Testing](core/error-paths.md) - Test error responses

---

### "I need to test complex logic"

**Duration**: 2-3 hours | **Difficulty**: Intermediate-Advanced

Follow this path:

1. **[Error Path Testing](core/error-paths.md)** (15 min) - Comprehensive error testing
2. **[Property-Based Testing](advanced/property-testing.md)** (30 min) - Generate test cases
3. **[Mutation Testing](advanced/mutation-testing.md)** (20 min) - Validate test quality
4. **[Concurrency Testing](advanced/concurrency-testing.md)** (20 min) - Test thread safety

**Key files to reference**:
- [Core Testing Patterns](core/README.md) - For building blocks
- [Best Practices](guides/best-practices.md) - Quality assurance patterns

---

### "I want to ensure high-quality tests"

**Duration**: 1-2 hours | **Difficulty**: Intermediate

Follow this path:

1. **[Error Path Testing](core/error-paths.md)** (15 min) - Test error cases thoroughly
2. **[Mutation Testing](advanced/mutation-testing.md)** (20 min) - Validate test quality
3. **[Coverage & Performance](guides/coverage-performance.md)** (20 min) - Measure metrics
4. **[Best Practices](guides/best-practices.md)** (30 min) - Industry patterns

**Tools**:
- Run `cargo make coverage` - Generate coverage reports
- Run `cargo make test-mutation` - Validate test quality
- Use `TestFixture` snapshots - Track state changes

---

### "I'm migrating from another testing framework"

**Duration**: 1.5-2 hours | **Difficulty**: Beginner-Intermediate

Follow this path:

1. **[Introduction](introduction.md)** (10 min) - Understand differences
2. **[Quick Start](tutorials/getting-started.md)** (25 min) - Learn Chicago TDD style
3. **[Core Testing Patterns](core/README.md)** (20 min) - Relearn with new approach
4. **[Best Practices & Migration](guides/best-practices.md)** (30 min) - Strategies for migration

**Key concepts to learn**:
- Real dependencies vs mocks
- Type-level correctness
- AAA pattern enforcement
- Poka-yoke design

---

### "I need a specific feature"

Find your feature in this table:

| Feature | Where to Find It |
|---------|-----------------|
| Testing basic functions | [Core Patterns](core/README.md) |
| Fixtures & setup | [Fixtures Tutorial](tutorials/fixtures-tutorial.md) |
| Building test data | [Data Builders](core/data-builders.md) |
| Assertions | [Assertions & Verification](core/assertions.md) |
| Error testing | [Error Path Testing](core/error-paths.md) |
| Random test data | [Property Testing](advanced/property-testing.md) |
| Test output validation | [Snapshot Testing](advanced/snapshot-testing.md) |
| Testing CLI apps | [CLI Testing](advanced/cli-testing.md) |
| Testing threads | [Concurrency Testing](advanced/concurrency-testing.md) |
| Test quality validation | [Mutation Testing](advanced/mutation-testing.md) |
| Test metrics | [Coverage & Performance](guides/coverage-performance.md) |
| Observability | [OTEL Instrumentation](guides/otel.md) |
| Production patterns | [Best Practices](guides/best-practices.md) |

---

## Learning Path Summary

### Beginner (0-1 day)
```
Quick Start (25 min)
  ↓
Fixtures Deep Dive (15 min)
  ↓
Core Patterns (20 min)
  ↓
Pick a real-world tutorial (45-50 min)
```

### Intermediate (1-3 days)
```
Complete Beginner path
  ↓
Advanced Techniques (property, mutation, snapshot)
  ↓
Real-world applications (CLI, Web Service, Docker)
  ↓
Best Practices
```

### Advanced (3+ days)
```
Complete Intermediate path
  ↓
Observability & OTEL
  ↓
Performance & Coverage
  ↓
Mutation testing & quality validation
  ↓
Write your own applications
```

---

## Decision Matrix

Choose based on your situation:

### If you're ...

| Situation | Time Available | Difficulty | Start Here |
|-----------|----------------|-----------|-----------|
| Completely new to testing | 2 hours | Easy | [Quick Start](tutorials/getting-started.md) |
| Have testing experience | 1.5 hours | Medium | [Introduction](introduction.md) |
| Experienced developer | 1 hour | Medium | [CLI](tutorials/cli-app-tutorial.md) or [Web Service](tutorials/web-service-tutorial.md) |
| Need specific feature | Varies | Varies | [Feature table above](#need-a-specific-feature) |
| Want certification-level knowledge | 1 week | Hard | Complete all tutorials + real-world projects |

---

## Common Starting Points

### "Show me code examples"
→ Jump to [Real-World Applications](guides/real-world.md)

### "I need to solve a problem right now"
→ Search [Best Practices](guides/best-practices.md) or use feature table above

### "I want to understand the philosophy"
→ Read [Introduction](introduction.md) and ["Go the Extra Mile"](guides/extra-mile.md)

### "I'm evaluating this for my team"
→ Read [Introduction](introduction.md) and skim [Best Practices](guides/best-practices.md)

### "I want to learn everything"
→ Follow the progressive learning paths in "Learning Path Summary" above

---

## Next Steps After Learning

### After completing Quick Start & Fixtures
→ Choose one real-world tutorial to build something real

### After completing a real-world tutorial
→ Explore Advanced Techniques that match your needs

### After exploring multiple areas
→ [Best Practices](guides/best-practices.md) - See industry patterns

### After mastering the basics
→ Build something with Chicago TDD:
- Open-source contribution
- Side project
- Production application
- Team migration

---

## FAQ

**Q: What's the best starting point?**
A: [Quick Start](tutorials/getting-started.md) if brand new, otherwise [Introduction](introduction.md).

**Q: How long does it take to learn?**
A: 2-4 hours for basics, 1-2 weeks to master all techniques.

**Q: Should I read everything?**
A: No. Focus on tutorials matching your needs, use reference as needed.

**Q: Can I skip sections?**
A: Yes. Each section is independent except tutorials (follow in order).

**Q: Where's the API reference?**
A: [API Reference](reference/fixtures-api.md) - More coming soon!

**Q: Do I need prior testing experience?**
A: No, but experience helps. Chicago TDD teaches a specific approach.

---

## Document Index

### Tutorials (Learning-Oriented)
- [Quick Start](tutorials/getting-started.md) - 25 min, beginner
- [Fixtures Deep Dive](tutorials/fixtures-tutorial.md) - 15 min, beginner
- [CLI Application](tutorials/cli-app-tutorial.md) - 45 min, intermediate
- [REST Web Service](tutorials/web-service-tutorial.md) - 50 min, intermediate

### Core (How-To Guides)
- [Fixtures](core/fixtures.md) - How to use fixtures
- [Data Builders](core/data-builders.md) - Build complex test data
- [Assertions](core/assertions.md) - Make clear assertions
- [Error Paths](core/error-paths.md) - Test failure cases

### Advanced (How-To Guides)
- [Property Testing](advanced/property-testing.md) - Generate test cases
- [Mutation Testing](advanced/mutation-testing.md) - Validate tests
- [Snapshot Testing](advanced/snapshot-testing.md) - Compare golden files
- [CLI Testing](advanced/cli-testing.md) - Test CLI apps
- [Concurrency Testing](advanced/concurrency-testing.md) - Thread safety

### Guides (How-To + Explanation)
- [The Extra Mile Pattern](guides/extra-mile.md) - Design decisions
- [Best Practices](guides/best-practices.md) - Production patterns
- [Coverage & Performance](guides/coverage-performance.md) - Metrics
- [Observability](guides/observability.md) - Monitoring & telemetry
- [OTEL Instrumentation](guides/otel.md) - Add OpenTelemetry
- [Weaver Validation](guides/weaver.md) - Validate telemetry
- [Real-World Applications](guides/real-world.md) - Example projects

### Reference (Lookup-Oriented)
- [TestFixture API](reference/fixtures-api.md) - Complete API docs

---

## Quick Links

| Need | Link |
|------|------|
| Quick start | [Getting Started](tutorials/getting-started.md) |
| Code examples | [Real-World Applications](guides/real-world.md) |
| Specific feature | Search this page or use feature table |
| API documentation | [API Reference](reference/fixtures-api.md) |
| Best practices | [Best Practices](guides/best-practices.md) |
| Understanding concepts | [Introduction](introduction.md) |

---

**Tip**: Bookmark this page and return when you need guidance on what to learn next.
