#!/usr/bin/env python3
"""Negative witnesses for the combinatorial-maximalism architecture pack."""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CELL = Path("architecture/combinatorial-maximalism")


def run(root: Path, *extra: str) -> tuple[int, str]:
    completed = subprocess.run(
        [sys.executable, str(root / "scripts/verify_combinatorial_maximalism.py"), "--root", str(root), *extra],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stderr + completed.stdout


def replace(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise AssertionError(f"missing mutation anchor {old!r} in {path}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def expect(name: str, mutate, code: str, *extra: str) -> dict:
    with tempfile.TemporaryDirectory(prefix="cmd-sabotage-") as tmp:
        sandbox = Path(tmp) / "repo"
        shutil.copytree(ROOT, sandbox)
        mutate(sandbox)
        returncode, output = run(sandbox, *extra)
        if returncode == 0 or code not in output:
            raise AssertionError(f"{name}: expected {code}, code={returncode}, output={output}")
        return {"name": name, "expected": code, "observed": True}


def main() -> int:
    baseline_code, baseline_output = run(ROOT)
    if baseline_code != 0:
        raise AssertionError(f"baseline failed: {baseline_output}")

    results = [
        expect("observation-admission", lambda r: replace(r / CELL / "O.star.toml", "bounded = true", "bounded = false"), "CMD-ADM-001"),
        expect("source-closure", lambda r: replace(r / CELL / "O.star.toml", '"ontology.ttl",', '"missing.ttl",'), "CMD-SRC-001"),
        expect("facet-totality", lambda r: replace(r / CELL / "ontology.ttl", "<f8> a c:F", "<f8> a c:RemovedFacet"), "CMD-FAC-001"),
        expect("dependency-closure", lambda r: replace(r / CELL / "ontology.ttl", 'c:d "obs.contract"', 'c:d "missing.realization"'), "CMD-DEP-001"),
        expect("profile-totality", lambda r: replace(r / CELL / "ontology.ttl", ',replay.pipeline";c:s', '";c:s'), "CMD-PRF-001"),
        expect("external-contract", lambda r: replace(r / CELL / "ontology.ttl", "c:k false", "c:k true"), "CMD-EXT-001"),
        expect("authority-fence", lambda r: replace(r / CELL / "O.star.toml", "network_actuation = false", "network_actuation = true"), "CMD-ACT-001"),
        expect("projection-axis", lambda r: replace(r / CELL / "ontology.ttl", 'c:i "axis.consumer"', 'c:i "axis.duplicate"'), "CMD-PRJ-001"),
        expect("evidence-totality", lambda r: replace(r / CELL / "ontology.ttl", "c:n true", "c:n false"), "CMD-EVD-001"),
        expect("pin-closure", lambda r: replace(r / CELL / "O.star.toml", 'ggen = "00a924e73acf03be1dd18968f797b3bb61fb8650"', 'ggen = "main"'), "CMD-PIN-001"),
        expect("generated-drift", lambda r: (r / CELL / "generated/docs/GUIDE.md").write_text("tampered\n", encoding="utf-8"), "CMD-GEN-001"),
        expect("exact-head", lambda _r: None, "CMD-RCP-002", "--require-exact-head", "--exact-head", "LOCAL"),
        expect("standing-crown", lambda r: replace(r / CELL / "O.star.toml", "crown_complete = false", "crown_complete = true"), "CMD-STD-001"),
    ]
    print(json.dumps({"standing": "ALIVE", "falsifiers": results}, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
