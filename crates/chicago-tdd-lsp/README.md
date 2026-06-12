# Chicago TDD LSP Server

Language Server Protocol (LSP) server for Chicago TDD Tools.

## Purpose

Enforces Chicago TDD Tools best practices, such as ensuring that `chicago-tdd-tools` is always a dev-dependency and never a production dependency.

## Features

- **Dependency Validation**: Checks `Cargo.toml` and warns if `chicago-tdd-tools` is in the `[dependencies]` section.
- **TDD Pattern Hints**: (Planned) Provides real-time feedback on AAA (Arrange-Act-Assert) patterns.

## Installation

```bash
cargo install chicago-tdd-lsp
```

## License

MIT
