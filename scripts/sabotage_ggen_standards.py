#!/usr/bin/env python3
"""Falsifiers for ontology, template, generated-byte, and authority drift."""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERIFIER = ROOT / "scripts/verify_ggen_standards.py"


def run(root: Path) -> tuple[int, str]:
    completed = subprocess.run(
        [sys.executable, str(root / "scripts/verify_ggen_standards.py"), "--root", str(root)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stderr + completed.stdout


def expect_refusal(name: str, mutate, code: str) -> dict:
    with tempfile.TemporaryDirectory(prefix="ctdd-sabotage-") as tmp:
        sandbox = Path(tmp) / "repo"
        shutil.copytree(ROOT, sandbox)
        mutate(sandbox)
        returncode, output = run(sandbox)
        if returncode == 0 or code not in output:
            raise AssertionError(f"{name}: expected {code}, returncode={returncode}, output={output}")
        return {"name": name, "expected": code, "observed": True}


def main() -> int:
    baseline_code, baseline_output = run(ROOT)
    if baseline_code != 0:
        raise AssertionError(f"baseline verifier failed: {baseline_output}")

    results = []
    results.append(expect_refusal(
        "ontology-mutation",
        lambda root: (root / "standards/chicago-tdd/ontology.ttl").write_text(
            (root / "standards/chicago-tdd/ontology.ttl").read_text().replace(
                'ctdd:identifier "STD-003" ; dcterms:title "Zero unreceipted actuation" ; ctdd:phase "actuate" ; ctdd:standing "ADMITTED" ; ctdd:receiptRequirement true',
                'ctdd:identifier "STD-003" ; dcterms:title "Zero unreceipted actuation" ; ctdd:phase "actuate" ; ctdd:standing "ADMITTED" ; ctdd:receiptRequirement false',
                1,
            )
        ),
        "CTDD-ACT-002",
    ))
    results.append(expect_refusal(
        "template-mutation",
        lambda root: (root / "standards/chicago-tdd/templates/standards.md.tera").write_text(
            (root / "standards/chicago-tdd/templates/standards.md.tera").read_text() + "\nSABOTAGE\n"
        ),
        "CTDD-GEN-001",
    ))
    results.append(expect_refusal(
        "generated-output-mutation",
        lambda root: (root / "standards/chicago-tdd/generated/STANDARDS.md").write_text("tampered\n"),
        "CTDD-GEN-001",
    ))
    results.append(expect_refusal(
        "authority-mutation",
        lambda root: (root / "standards/chicago-tdd/O.star.toml").write_text(
            (root / "standards/chicago-tdd/O.star.toml").read_text().replace("broker_only = true", "broker_only = false", 1)
        ),
        "CTDD-ACT-001",
    ))
    print(json.dumps({"standing": "ALIVE", "falsifiers": results}, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
