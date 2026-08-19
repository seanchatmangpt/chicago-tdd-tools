# weaver-toolkit/verify-python

Chicago-style black-box tests of `weaver-toolkit` **from outside Rust**.

This is a self-contained [uv](https://docs.astral.sh/uv/) project. It has no
dependency on the chicago-tdd-tools Rust crate, `cargo`, or anything in
`src/` — it only exercises the public contract other projects actually
consume: `weaver-wrapper.sh`'s CLI, driven as a real subprocess, and the
real OTLP gRPC endpoint it stands up, driven by a real Python
OpenTelemetry SDK. This is deliberate: it is the same shape of consumer a
non-Rust project vendoring `weaver-toolkit/` would be.

No mocks anywhere — every test downloads and runs the real `weaver` binary,
clones the real `open-telemetry/semantic-conventions` registry, sends real
spans over a real gRPC connection, and asserts on the real `report.json`
`weaver` itself writes.

## Run it

```bash
cd weaver-toolkit/verify-python
uv sync
uv run pytest -v
```

or, from the repo root:

```bash
just weaver-toolkit-verify-python
```

First run downloads a real `weaver` binary and clones the real
`semantic-conventions` registry (session-scoped fixture, ~30-60s); every
test after that reuses it.

## What's covered

- `test_bootstrap_and_check.py` — `bootstrap` produces a runnable binary and
  a real registry checkout; `check` runs real static validation; `bootstrap`
  is idempotent on a second run (the exact path CI hits every job).
- `test_live_check_otlp.py` — `live-start` brings up a real OTLP gRPC
  listener; a real span sent through it shows up, verbatim, in the real
  `report.json` after `live-stop`; a span carrying a made-up attribute gets
  a real `missing_attribute` advisory from weaver's own registry-backed
  advisory engine (proving the registry is actually consulted, not just
  that spans are accepted unconditionally).

## Why this exists

`weaver-toolkit`'s whole premise is "other projects can vendor this without
depending on chicago-tdd-tools' Rust workspace." The only way to actually
prove that promise is to test it from a client that has never seen
`WeaverValidator`/`WeaverLiveCheck` and only has the shell CLI contract
(`../README.md`) to go on — this project is that client.
