#!/usr/bin/env python3
"""Idempotence law for emit-dossier (audit W6).

sealed_at is the only wall-clock input to the dossier wire: receipts,
merkle root, and the ML-DSA-65 signature are all deterministic. When a
re-emit differs from HEAD solely in sealed_at, restore the committed
bytes so a no-material-change re-seal never dirties the tree.
"""
import json
import subprocess
import sys

FILE = "docs/nato/NATO_STANAG_4107_DOSSIER.json"


def main() -> int:
    dirty = subprocess.run(
        ["git", "diff", "--quiet", "--", FILE], capture_output=True
    )
    if dirty.returncode == 0:
        return 0
    head = subprocess.run(
        ["git", "show", f"HEAD:{FILE}"], capture_output=True, text=True
    )
    if head.returncode != 0:
        return 0
    with open(FILE) as f:
        new = json.load(f)
    old = json.loads(head.stdout)
    if new.get("merkle_root") != old.get("merkle_root"):
        print("[emit-dossier] root moved: keeping the fresh dossier")
        return 0
    if new.get("court_receipts") != old.get("court_receipts"):
        print("[emit-dossier] receipts moved: keeping the fresh dossier")
        return 0
    if new.get("signature") != old.get("signature"):
        print("[emit-dossier] signature moved (key rotation): keeping fresh")
        return 0
    with open(FILE, "w") as f:
        f.write(head.stdout)
    print("[emit-dossier] sealed_at-only drift: committed bytes restored")
    return 0


if __name__ == "__main__":
    sys.exit(main())
