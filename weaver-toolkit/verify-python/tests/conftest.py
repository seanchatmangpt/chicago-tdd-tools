"""Chicago-style fixtures for weaver-toolkit black-box verification.

Every fixture here drives the real `weaver-wrapper.sh` as a real subprocess
against a real, freshly-downloaded weaver binary and a real cloned
semantic-conventions registry -- there is no fake/mock loader standing in
for any of it. This is deliberate: the whole point of this suite is to
prove the toolkit works when driven from *outside* Rust, by a client that
has never seen `WeaverValidator`/`WeaverLiveCheck` and only has the shell
CLI contract (`weaver-toolkit/README.md`) to go on.
"""

from __future__ import annotations

import json
import os
import socket
import subprocess
import time
from pathlib import Path
from typing import Iterator, NamedTuple

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
WRAPPER = REPO_ROOT / "weaver-toolkit" / "weaver-wrapper.sh"


def _free_tcp_port() -> int:
    """Ask the OS for a currently-unused TCP port (real socket, not a guess)."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def _run_wrapper(subcommand: str, env: dict[str, str], timeout: int) -> subprocess.CompletedProcess:
    assert WRAPPER.exists(), f"weaver-wrapper.sh not found at {WRAPPER}"
    return subprocess.run(
        ["bash", str(WRAPPER), subcommand],
        env=env,
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        timeout=timeout,
        check=False,
    )


@pytest.fixture(scope="session")
def weaver_home(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """A real weaver binary + real cloned semconv registry, downloaded once per session."""
    home = tmp_path_factory.mktemp("weaver-home")
    env = {**os.environ, "WEAVER_HOME": str(home)}
    result = _run_wrapper("bootstrap", env, timeout=120)
    assert result.returncode == 0, (
        f"bootstrap failed (rc={result.returncode})\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    )
    assert (home / "bin" / "weaver").is_file(), "bootstrap did not produce a weaver binary"
    assert os.access(home / "bin" / "weaver", os.X_OK), "weaver binary is not executable"
    return home


class LiveCheckSession(NamedTuple):
    grpc_endpoint: str
    admin_port: int
    reports_dir: Path
    env: dict[str, str]
    stopped_flag: list  # mutable sentinel (list-of-bool) so stop() is idempotent


def stop_and_load_report(session: LiveCheckSession) -> dict:
    """Explicitly stop the live-check listener (writing report.json) and load it.

    A test must call this itself rather than relying on fixture teardown --
    `live_check`'s own `live-stop` only runs *after* the test function
    returns, so report.json does not exist yet while the test body is still
    running.
    """
    if not session.stopped_flag:
        result = _run_wrapper("live-stop", session.env, timeout=15)
        assert result.returncode == 0, (
            f"live-stop failed (rc={result.returncode})\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
        session.stopped_flag.append(True)
    report_path = session.reports_dir / "report.json"
    assert report_path.is_file(), f"live-stop did not produce {report_path}"
    return json.loads(report_path.read_text())


@pytest.fixture
def live_check(weaver_home: Path, tmp_path: Path) -> Iterator[LiveCheckSession]:
    """Start a real weaver live-check listener via the real wrapper script, stop it on teardown.

    Each test gets its own OTLP/admin ports (via real OS port allocation) and
    its own reports directory, so parallel test runs never collide.
    """
    grpc_port = _free_tcp_port()
    admin_port = _free_tcp_port()
    reports_dir = tmp_path / "weaver-reports"
    env = {
        **os.environ,
        "WEAVER_HOME": str(weaver_home),
        "WEAVER_OTLP_GRPC_PORT": str(grpc_port),
        "WEAVER_ADMIN_PORT": str(admin_port),
        "WEAVER_REPORTS_DIR": str(reports_dir),
    }

    start = _run_wrapper("live-start", env, timeout=30)
    assert start.returncode == 0, (
        f"live-start failed (rc={start.returncode})\nstdout:\n{start.stdout}\nstderr:\n{start.stderr}"
    )

    # Poll the real admin HTTP port until weaver is actually listening, rather
    # than sleeping a fixed guessed duration.
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
            probe.settimeout(0.2)
            if probe.connect_ex(("127.0.0.1", admin_port)) == 0:
                break
        time.sleep(0.1)
    else:
        pytest.fail(f"weaver admin port {admin_port} never came up within 10s")

    stopped: list = []
    try:
        yield LiveCheckSession(
            grpc_endpoint=f"127.0.0.1:{grpc_port}",
            admin_port=admin_port,
            reports_dir=reports_dir,
            env=env,
            stopped_flag=stopped,
        )
    finally:
        # Idempotent: a test that already called stop_and_load_report() has
        # marked `stopped`, so this is a no-op cleanup for tests that don't
        # need the report (or that failed before reaching their own stop).
        if not stopped:
            _run_wrapper("live-stop", env, timeout=15)
