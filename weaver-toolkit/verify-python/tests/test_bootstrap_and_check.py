"""Verify weaver-wrapper.sh's non-live-check subcommands from outside Rust.

Real subprocess calls to the real bash script and the real downloaded
`weaver` binary; assertions are on real stdout/exit codes, not stubbed
return values.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

from .conftest import _run_wrapper


def test_bootstrap_produces_a_runnable_weaver_binary(weaver_home: Path) -> None:
    # Act: ask the real installed binary its own version (no wrapper indirection)
    result = subprocess.run(
        [str(weaver_home / "bin" / "weaver"), "--version"],
        capture_output=True,
        text=True,
        timeout=10,
        check=False,
    )

    # Assert: real process output, not a canned string
    assert result.returncode == 0
    assert "weaver" in result.stdout.lower()


def test_bootstrap_clones_a_real_semconv_registry(weaver_home: Path) -> None:
    registry = weaver_home / "registry"
    assert registry.is_dir()
    # A real git clone of open-telemetry/semantic-conventions carries a
    # model/ directory with real .yaml schema files -- not an empty stub.
    model_dir = registry / "model"
    assert model_dir.is_dir()
    yaml_files = list(model_dir.rglob("*.yaml"))
    assert len(yaml_files) > 10, f"expected a real registry checkout, found {len(yaml_files)} yaml files"


def test_check_runs_real_static_validation(weaver_home: Path) -> None:
    import os

    env = {**os.environ, "WEAVER_HOME": str(weaver_home)}
    result = _run_wrapper("check", env, timeout=30)

    assert result.returncode == 0, result.stderr
    # weaver itself writes its human-readable report to stderr, not stdout.
    assert "Weaver Registry Check" in result.stderr


def test_bootstrap_is_idempotent(weaver_home: Path) -> None:
    """Re-running bootstrap against an already-populated WEAVER_HOME must not
    re-download or fail -- this is the exact behavior consuming projects rely
    on when `just weaver-toolkit-bootstrap` runs on every CI job."""
    import os

    env = {**os.environ, "WEAVER_HOME": str(weaver_home)}
    result = _run_wrapper("bootstrap", env, timeout=30)

    assert result.returncode == 0, result.stderr
    assert "already present" in result.stdout
