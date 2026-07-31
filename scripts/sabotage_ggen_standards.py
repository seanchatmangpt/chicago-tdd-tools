#!/usr/bin/env python3
"""Falsifiers for authored law, projection, authority, checkpoints, pins, OCEL, and replay."""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXACT_HEAD = "0" * 40


def run(root: Path, *extra: str) -> tuple[int, str]:
    completed = subprocess.run(
        [
            sys.executable,
            str(root / "scripts/verify_ggen_standards.py"),
            "--root", str(root),
            "--exact-head", EXACT_HEAD,
            *extra,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stderr + completed.stdout


def expect_refusal(name: str, mutate, code: str, *extra: str) -> dict:
    with tempfile.TemporaryDirectory(prefix="ctdd-sabotage-") as tmp:
        sandbox = Path(tmp) / "repo"
        shutil.copytree(ROOT, sandbox)
        mutate(sandbox)
        returncode, output = run(sandbox, *extra)
        if returncode == 0 or code not in output:
            raise AssertionError(
                f"{name}: expected {code}, returncode={returncode}, output={output}",
            )
        return {"name": name, "expected": code, "observed": True}


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise AssertionError(f"mutation anchor missing in {path}: {old}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def main() -> int:
    baseline_code, baseline_output = run(ROOT)
    if baseline_code != 0:
        raise AssertionError(f"baseline verifier failed: {baseline_output}")

    results = []
    results.append(expect_refusal(
        "ontology-mutation",
        lambda root: replace_once(
            root / "standards/chicago-tdd/ontology.ttl",
            'ctdd:identifier "STD-003" ; dcterms:title "Zero unreceipted actuation" ; ctdd:phase "actuate" ; ctdd:standing "ADMITTED" ; ctdd:receiptRequirement true',
            'ctdd:identifier "STD-003" ; dcterms:title "Zero unreceipted actuation" ; ctdd:phase "actuate" ; ctdd:standing "ADMITTED" ; ctdd:receiptRequirement false',
        ),
        "CTDD-ACT-002",
    ))
    results.append(expect_refusal(
        "template-mutation",
        lambda root: (
            root / "standards/chicago-tdd/templates/standards.md.tera"
        ).write_text(
            (
                root / "standards/chicago-tdd/templates/standards.md.tera"
            ).read_text(encoding="utf-8") + "\nSABOTAGE\n",
            encoding="utf-8",
        ),
        "CTDD-GEN-001",
    ))
    results.append(expect_refusal(
        "generated-output-mutation",
        lambda root: (
            root / "standards/chicago-tdd/generated/STANDARDS.md"
        ).write_text("tampered\n", encoding="utf-8"),
        "CTDD-GEN-001",
    ))
    results.append(expect_refusal(
        "authority-mutation",
        lambda root: replace_once(
            root / "standards/chicago-tdd/O.star.toml",
            "broker_only = true",
            "broker_only = false",
        ),
        "CTDD-ACT-001",
    ))
    results.append(expect_refusal(
        "checkpoint-totality-mutation",
        lambda root: replace_once(
            root / "standards/chicago-tdd/ontology.ttl",
            next(
                line + "\n"
                for line in (
                    root / "standards/chicago-tdd/ontology.ttl"
                ).read_text(encoding="utf-8").splitlines()
                if 'ctdd:identifier "GALL-018"' in line
            ),
            "",
        ),
        "CTDD-CHK-001",
    ))
    results.append(expect_refusal(
        "checkpoint-dependency-mutation",
        lambda root: replace_once(
            root / "standards/chicago-tdd/ontology.ttl",
            'ctdd:identifier "GALL-015" ; dcterms:title "Checkpoint dependency closure" ; ctdd:phase "route" ; ctdd:dependsOn "GALL-014"',
            'ctdd:identifier "GALL-015" ; dcterms:title "Checkpoint dependency closure" ; ctdd:phase "route" ; ctdd:dependsOn "GALL-013"',
        ),
        "CTDD-CHK-002",
    ))
    results.append(expect_refusal(
        "checkpoint-standing-mutation",
        lambda root: replace_once(
            root / "standards/chicago-tdd/ontology.ttl",
            'ctdd:identifier "GALL-016" ; dcterms:title "State and crown separation" ; ctdd:phase "diagnose" ; ctdd:dependsOn "GALL-015" ; ctdd:standing "ADMITTED"',
            'ctdd:identifier "GALL-016" ; dcterms:title "State and crown separation" ; ctdd:phase "diagnose" ; ctdd:dependsOn "GALL-015" ; ctdd:standing "ALIVE"',
        ),
        "CTDD-CHK-003",
    ))
    results.append(expect_refusal(
        "toolchain-pin-mutation",
        lambda root: replace_once(
            root / "standards/chicago-tdd/O.star.toml",
            'ggen = "00a924e73acf03be1dd18968f797b3bb61fb8650"',
            'ggen = "1111111111111111111111111111111111111111"',
        ),
        "CTDD-PIN-001",
    ))
    results.append(expect_refusal(
        "ocel-schema-mutation",
        lambda root: replace_once(
            root / "standards/chicago-tdd/O.star.toml",
            'ocel_schema = "2.0"',
            'ocel_schema = "1.0"',
        ),
        "CTDD-OCL-001",
    ))

    with tempfile.TemporaryDirectory(prefix="ctdd-sabotage-") as tmp:
        sandbox = Path(tmp) / "repo"
        shutil.copytree(ROOT, sandbox)
        completed = subprocess.run(
            [
                sys.executable,
                str(sandbox / "scripts/verify_ggen_standards.py"),
                "--root", str(sandbox),
                "--exact-head", "LOCAL",
                "--require-exact-head",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        output = completed.stderr + completed.stdout
        if completed.returncode == 0 or "CTDD-VAL-002" not in output:
            raise AssertionError(
                "exact-head-mutation: expected CTDD-VAL-002, "
                f"returncode={completed.returncode}, output={output}",
            )
        results.append({
            "name": "exact-head-mutation",
            "expected": "CTDD-VAL-002",
            "observed": True,
        })

    results.append(expect_refusal(
        "receipt-replay-mutation",
        lambda root: (
            root / "scripts/.verify_ggen_standards/part01.pyfrag"
        ).write_bytes(
            (root / "scripts/.verify_ggen_standards/part01.pyfrag").read_bytes()
            + b"\n# receipt replay drift\n"
        ),
        "CTDD-RCP-003",
    ))

    print(json.dumps({
        "standing": "ALIVE",
        "falsifiers": results,
        "count": len(results),
    }, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
