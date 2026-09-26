#!/usr/bin/env python3
"""Verify the repository Just shell starts safely and retains nounset.

This is a behavioral court over the real ``just`` executable.  The temporary
probe Justfile reuses the exact ``set shell := ...`` line admitted by the
repository Justfile so the verifier cannot silently test a different shell
contract.
"""

from __future__ import annotations

import pathlib
import re
import shutil
import subprocess
import sys
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
JUSTFILE = ROOT / "Justfile"
SHELL_RE = re.compile(r"^set shell := .+$", re.MULTILINE)
STARTUP_MARKER = "CTDD_JUST_STARTUP_OK"
UNSET_NAME = "CTDD_SHELL_CONTRACT_INTENTIONALLY_UNSET"


def refuse(code: str, detail: str) -> "NoReturn":
    print(f"REFUSED:{code}:{detail}", file=sys.stderr)
    raise SystemExit(1)


def run(*args: str, cwd: pathlib.Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        env={key: value for key, value in __import__("os").environ.items() if key != UNSET_NAME},
    )


def main() -> int:
    if shutil.which("just") is None:
        refuse("JUST_UNAVAILABLE", "real just executable is required")

    source = JUSTFILE.read_text(encoding="utf-8")
    matches = SHELL_RE.findall(source)
    if len(matches) != 1:
        refuse("SHELL_CONFIG_CARDINALITY", f"expected exactly one shell config, observed {len(matches)}")
    shell_config = matches[0]

    root_probe = run("just", "default", cwd=ROOT)
    if root_probe.returncode != 0:
        refuse(
            "ROOT_STARTUP_FAILED",
            f"exit={root_probe.returncode} stderr={root_probe.stderr.strip()!r}",
        )

    with tempfile.TemporaryDirectory(prefix="ctdd-just-shell-") as tmp:
        tmp_path = pathlib.Path(tmp)
        probe = tmp_path / "Justfile"
        probe.write_text(
            "\n".join(
                (
                    shell_config,
                    "",
                    "startup:",
                    f"    @printf '{STARTUP_MARKER}\\n'",
                    "",
                    "nounset:",
                    f"    @printf '%s\\n' \"${{{UNSET_NAME}}}\"",
                    "",
                )
            ),
            encoding="utf-8",
        )

        startup = run("just", "--justfile", str(probe), "startup", cwd=tmp_path)
        if startup.returncode != 0 or STARTUP_MARKER not in startup.stdout:
            refuse(
                "STARTUP_PROBE_FAILED",
                f"exit={startup.returncode} stdout={startup.stdout.strip()!r} stderr={startup.stderr.strip()!r}",
            )

        nounset = run("just", "--justfile", str(probe), "nounset", cwd=tmp_path)
        combined = f"{nounset.stdout}\n{nounset.stderr}".lower()
        if nounset.returncode == 0:
            refuse("NOUNSET_NOT_ENFORCED", "intentionally unset repository variable was accepted")
        if "unbound variable" not in combined:
            refuse(
                "NOUNSET_WRONG_FAILURE",
                f"exit={nounset.returncode} stderr={nounset.stderr.strip()!r}",
            )

    print(f"JUST_SHELL_CONTRACT=ALIVE shell={shell_config}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
