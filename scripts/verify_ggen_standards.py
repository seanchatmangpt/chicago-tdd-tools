#!/usr/bin/env python3
"""Hash-admitted loader for the independent v26.7.30 ggen verifier."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

PARTS = (
    ("part01.pyfrag", "09f4c052f49fac8fffa49bed8f76e1520c591225743d398746e83213a9fa6830"),
    ("part02.pyfrag", "5a8ed662c6810ee715df3352ecfd69fca2d74b3eb76d253081fea5bcf0c858e7"),
    ("part03.pyfrag", "b56ab7d22083bca4f74fe428dae16273fd3d9c84902ab68543e6de8ed4978afc"),
    ("part04.pyfrag", "a85eaf3c3cacdb6398c5bd422eb97a766610645d7e6f632ee6ed5e1a587a039a"),
    ("part05.pyfrag", "b2da218d41b541db06a01c635ab86b318f7a51f991312aea0108ddc222ad41cd"),
)


def _load() -> bytes:
    root = Path(__file__).resolve().parent / ".verify_ggen_standards"
    payload = bytearray()
    for name, expected in PARTS:
        data = (root / name).read_bytes()
        observed = hashlib.sha256(data).hexdigest()
        if observed != expected:
            print(json.dumps({
                "standing": "BLOCKED",
                "refusal": "CTDD-RCP-003",
                "message": f"verifier implementation shard drift: {name}",
            }, sort_keys=True), file=sys.stderr)
            raise SystemExit(2)
        payload.extend(data)
    return bytes(payload)


exec(compile(_load(), __file__, "exec"), globals(), globals())
