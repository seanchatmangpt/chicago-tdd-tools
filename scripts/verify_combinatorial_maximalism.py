#!/usr/bin/env python3
"""Hash-admitted loader for the independent combinatorial-maximalism verifier."""
from __future__ import annotations
import hashlib
from pathlib import Path

_PARTS = (
    ("part01.pyfrag", "fe94bcada5345123ef9249088d7893529a41bd19484b4cc4d376e4c334f73287"),
    ("part02.pyfrag", "14941c15fad7a0cee2e93c0501ecdd25c08c2bc2c7a084f80c7e248019c9478b"),
    ("part03.pyfrag", "669055ad84d15b2ec834e859e1929a22f3f6ccebfd069feb9e24dff95264179d"),
 )
_ROOT = Path(__file__).resolve().parent
_DIR = _ROOT / ".verify_combinatorial_maximalism"
_chunks = []
for _name, _expected in _PARTS:
    _data = (_DIR / _name).read_bytes()
    if hashlib.sha256(_data).hexdigest() != _expected:
        raise SystemExit(f"CMD-RCP-003: verifier shard drift: {_name}")
    _chunks.append(_data)
exec(compile(b"".join(_chunks), str(_DIR / _PARTS[0][0]), "exec"), globals())
