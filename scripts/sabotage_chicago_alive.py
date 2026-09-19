#!/usr/bin/env python3
"""Black-box adversarial falsifiers for the CHICAGO_ALIVE receiver.

This script deliberately does not import the standing court.  It mutates complete
trial documents, executes the public verifier process, and checks externally
observable exit/status behavior.  That keeps the negative court separate from the
positive implementation it is trying to falsify.
"""
from __future__ import annotations

import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from typing import Any, Callable

ROOT = Path(__file__).resolve().parent.parent
VERIFIER = ROOT / "scripts" / "verify_chicago_alive.py"
FIXTURE = ROOT / "architecture" / "chicago-alive" / "example-trial.json"
DUMMY_SHA = "0123456789abcdef0123456789abcdef01234567"


def base_trial() -> dict[str, Any]:
    trial = json.loads(FIXTURE.read_text(encoding="utf-8"))
    if not isinstance(trial, dict):
        raise RuntimeError("FIXTURE_NOT_OBJECT")
    trial["subject"]["commit_sha"] = DUMMY_SHA
    trial["subject"]["ref"] = f"seanchatmangpt/chicago-tdd-tools@{DUMMY_SHA}"
    return trial


def execute(trial: dict[str, Any], *extra: str) -> tuple[int, dict[str, Any]]:
    with tempfile.TemporaryDirectory(prefix="chicago-alive-sabotage-") as directory:
        path = Path(directory) / "trial.json"
        path.write_text(json.dumps(trial, sort_keys=True, indent=2) + "\n", encoding="utf-8")
        completed = subprocess.run(
            [sys.executable, str(VERIFIER), str(path), *extra],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        try:
            payload = json.loads(completed.stdout)
        except json.JSONDecodeError as exc:
            raise AssertionError(
                f"verifier did not return JSON: rc={completed.returncode} "
                f"stdout={completed.stdout!r} stderr={completed.stderr!r}"
            ) from exc
        if not isinstance(payload, dict):
            raise AssertionError(f"verifier payload is not an object: {payload!r}")
        return completed.returncode, payload


def standing(payload: dict[str, Any]) -> str:
    decision = payload.get("decision")
    if not isinstance(decision, dict):
        raise AssertionError(f"missing decision object: {payload!r}")
    value = decision.get("standing")
    if not isinstance(value, str):
        raise AssertionError(f"missing standing: {payload!r}")
    return value


def mutate(path: tuple[str | int, ...], value: Any) -> Callable[[dict[str, Any]], None]:
    def apply(trial: dict[str, Any]) -> None:
        cursor: Any = trial
        for part in path[:-1]:
            cursor = cursor[part]
        cursor[path[-1]] = value

    return apply


def case(
    name: str,
    change: Callable[[dict[str, Any]], None],
    expected_standing: str,
    expected_rc: int,
    *extra: str,
) -> None:
    trial = copy.deepcopy(base_trial())
    change(trial)
    rc, payload = execute(trial, *extra)
    observed = standing(payload)
    if observed != expected_standing or rc != expected_rc:
        raise AssertionError(
            f"{name}: expected ({expected_standing}, rc={expected_rc}), "
            f"got ({observed}, rc={rc}): {payload}"
        )


def main() -> int:
    checks = 0

    case("positive synthetic", lambda _: None, "CHICAGO_ALIVE", 0)
    checks += 1
    case(
        "load-bearing test double",
        mutate(("collaborators", 0, "kind"), "TEST_DOUBLE"),
        "REFUSED",
        2,
    )
    checks += 1
    case(
        "direct actuation",
        mutate(("authority", "direct_actuation"), True),
        "REFUSED",
        2,
    )
    checks += 1
    case("unadmitted world", mutate(("world", "admitted"), False), "BLOCKED", 2)
    checks += 1
    case("replay absent", mutate(("replay",), "NOT_RUN"), "PARTIAL_ALIVE", 2)
    checks += 1
    case(
        "failed falsifier",
        mutate(("falsifiers", 0, "passed"), False),
        "REFUSED",
        2,
    )
    checks += 1
    case("no falsifiers", mutate(("falsifiers",), []), "PARTIAL_ALIVE", 2)
    checks += 1
    case("no receipt", mutate(("receipt_refs",), []), "PARTIAL_ALIVE", 2)
    checks += 1
    case(
        "unbounded correspondence",
        mutate(("correspondence", "bounded"), False),
        "BLOCKED",
        2,
    )
    checks += 1
    case(
        "missing consequence",
        mutate(("trial", "consequence_observed"), False),
        "PARTIAL_ALIVE",
        2,
    )
    checks += 1
    case(
        "production authority in synthetic world",
        mutate(("authority", "production_authority_used"), True),
        "REFUSED",
        2,
    )
    checks += 1
    case(
        "real consequence in synthetic world",
        mutate(("trial", "real_target_consequence_observed"), True),
        "REFUSED",
        2,
    )
    checks += 1

    def make_real(trial: dict[str, Any]) -> None:
        trial["world"]["kind"] = "REAL_TARGET"
        trial["trial"]["real_target_identity_exact"] = True
        trial["trial"]["real_target_consequence_observed"] = True

    case("exact real target", make_real, "ALIVE", 0)
    checks += 1
    case(
        "exact-head mismatch",
        lambda _: None,
        "REFUSED",
        2,
        "--require-exact-head",
        "ffffffffffffffffffffffffffffffffffffffff",
    )
    checks += 1

    print(f"CHICAGO_ALIVE_SABOTAGE={checks}/{checks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
