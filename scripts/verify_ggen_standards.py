#!/usr/bin/env python3
"""Independent verifier and bounded filesystem broker for the v26.7.30 ggen cell."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

CELL_REL = Path("standards/chicago-tdd")
REQUIRED_STANDARDS = {f"STD-{n:03d}" for n in range(1, 21)}
REQUIRED_REFUSALS = {
    "CTDD-ADM-001", "CTDD-GEN-001", "CTDD-GEN-002", "CTDD-GEN-003",
    "CTDD-RCP-001", "CTDD-RCP-002", "CTDD-ACT-001", "CTDD-ACT-002",
    "CTDD-HOOK-001", "CTDD-STD-001", "CTDD-VAL-001", "CTDD-VAL-002",
}
EXPECTED_OUTPUTS = {
    "STANDARDS.md", "REFUSALS.md", "src/standards.rs", "src/refusals.rs",
    "src/lib.rs", "Cargo.toml", "tests/contracts.rs",
}
RECEIPT_COMPONENTS = {
    "ttl", "query", "template", "observation", "output", "utility", "test", "previous_receipt"
}


class Refusal(RuntimeError):
    def __init__(self, code: str, message: str):
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message


@dataclass(frozen=True)
class Rule:
    name: str
    query: str
    template: Path
    output: Path


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def b3sum_bytes(data: bytes) -> str | None:
    exe = shutil.which("b3sum")
    if exe is None:
        return None
    completed = subprocess.run(
        [exe, "--no-names"], input=data, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False
    )
    if completed.returncode != 0:
        raise Refusal("CTDD-RCP-001", completed.stderr.decode("utf-8", "replace").strip())
    return completed.stdout.decode("ascii").strip().split()[0]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--write", action="store_true", help="atomically manufacture generated outputs")
    parser.add_argument("--write-receipt", action="store_true", help="write evidence/receipt.json")
    parser.add_argument("--require-blake3", action="store_true")
    parser.add_argument("--exact-head", default=os.environ.get("GITHUB_SHA", "LOCAL"))
    return parser.parse_args()


def load_toml(path: Path) -> dict:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise Refusal("CTDD-VAL-001", f"cannot parse {path}: {exc}") from exc


def parse_blocks(text: str, rdf_type: str) -> list[dict[str, str]]:
    block_pattern = re.compile(
        rf"<(?P<subject>[^>]+)>\s+a\s+{re.escape(rdf_type)}\s*;(?P<body>.*?)\.\s*(?=\n<|\Z)",
        re.DOTALL,
    )
    property_pattern = re.compile(
        r"(?P<key>(?:[A-Za-z][\w-]*):[A-Za-z][\w-]*)\s+"
        r"(?P<value>\"(?:[^\"\\]|\\.)*\"|true|false|-?\d+(?:\.\d+)?|<[^>]+>)\s*(?:;|$)",
        re.MULTILINE,
    )
    records: list[dict[str, str]] = []
    for match in block_pattern.finditer(text):
        row: dict[str, str] = {"subject": match.group("subject")}
        for prop in property_pattern.finditer(match.group("body")):
            value = prop.group("value")
            if value.startswith('"'):
                value = json.loads(value)
            row[prop.group("key")] = value
        records.append(row)
    return records


def require(row: dict[str, str], key: str, code: str = "CTDD-VAL-001") -> str:
    value = row.get(key)
    if value is None:
        raise Refusal(code, f"missing {key} on {row.get('subject', '<unknown>')}")
    return value


def normalize_standard(row: dict[str, str]) -> dict[str, str]:
    return {
        "order": require(row, "ctdd:order"),
        "id": require(row, "ctdd:identifier"),
        "title": require(row, "dcterms:title"),
        "phase": require(row, "ctdd:phase"),
        "standing": require(row, "ctdd:standing"),
        "receipt": require(row, "ctdd:receiptRequirement"),
        "description": require(row, "dcterms:description"),
    }


def normalize_refusal(row: dict[str, str]) -> dict[str, str]:
    return {
        "order": require(row, "ctdd:order"),
        "code": require(row, "ctdd:code"),
        "name": require(row, "ctdd:name"),
        "boundary": require(row, "ctdd:boundary"),
        "description": require(row, "dcterms:description"),
    }


def parse_rules(config: dict, cell: Path) -> list[Rule]:
    rules = []
    output_dir = Path(config.get("generation", {}).get("output_dir", "generated"))
    for raw in config.get("generation", {}).get("rules", []):
        try:
            rules.append(Rule(
                name=raw["name"],
                query=raw["query"]["inline"],
                template=cell / raw["template"]["file"],
                output=cell / output_dir / raw["output_file"],
            ))
        except KeyError as exc:
            raise Refusal("CTDD-VAL-001", f"malformed generation rule: missing {exc}") from exc
    if not rules:
        raise Refusal("CTDD-VAL-001", "no generation rules")
    return rules


def validate_query(rule: Rule, expected_fields: Iterable[str]) -> None:
    if "SELECT" not in rule.query or "WHERE" not in rule.query:
        raise Refusal("CTDD-VAL-001", f"{rule.name} is not a bounded SELECT query")
    for field in expected_fields:
        if f"?{field}" not in rule.query:
            raise Refusal("CTDD-VAL-001", f"{rule.name} does not select ?{field}")


def render(template: str, rows: list[dict[str, str]]) -> bytes:
    loop = re.compile(
        r"(?P<prefix>.*?)\{%\s*for\s+row\s+in\s+results\s*%\}(?P<body>.*?)\{%\s*endfor\s*%\}(?P<suffix>.*)",
        re.DOTALL,
    )
    match = loop.fullmatch(template)
    if match is None:
        if "{{" in template or "{%" in template:
            raise Refusal("CTDD-VAL-001", "unsupported template construct")
        return template.encode("utf-8")
    rendered_rows = []
    variable = re.compile(r"\{\{\s*row\.([A-Za-z_][A-Za-z0-9_]*)\s*\}\}")
    for row in rows:
        body = match.group("body")
        referenced = set(variable.findall(body))
        missing = referenced - row.keys()
        if missing:
            raise Refusal("CTDD-VAL-001", f"template references missing fields: {sorted(missing)}")
        rendered_rows.append(variable.sub(lambda m: row[m.group(1)], body))
    output = match.group("prefix") + "".join(rendered_rows) + match.group("suffix")
    return output.encode("utf-8")


def atomic_write(path: Path, data: bytes, generated_root: Path) -> None:
    resolved_root = generated_root.resolve()
    resolved_parent = path.parent.resolve()
    if resolved_root != resolved_parent and resolved_root not in resolved_parent.parents:
        raise Refusal("CTDD-ACT-001", f"output escapes broker root: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(tmp_name, path)
    finally:
        if os.path.exists(tmp_name):
            os.unlink(tmp_name)


def manufacture(root: Path) -> tuple[dict[Path, bytes], dict, list[dict[str, str]], list[dict[str, str]], list[Rule]]:
    cell = root / CELL_REL
    observation = load_toml(cell / "O.star.toml")
    config = load_toml(cell / "ggen.toml")
    ontology = (cell / "ontology.ttl").read_text(encoding="utf-8")
    standards = sorted((normalize_standard(row) for row in parse_blocks(ontology, "ctdd:Standard")), key=lambda r: int(r["order"]))
    refusals = sorted((normalize_refusal(row) for row in parse_blocks(ontology, "ctdd:Refusal")), key=lambda r: int(r["order"]))
    rules = parse_rules(config, cell)

    outputs: dict[Path, bytes] = {}
    standard_fields = ("order", "id", "title", "phase", "standing", "receipt", "description")
    refusal_fields = ("order", "code", "name", "boundary", "description")
    for rule in rules:
        template = rule.template.read_text(encoding="utf-8")
        if rule.name.startswith("standards-"):
            validate_query(rule, standard_fields)
            rows = standards
        elif rule.name.startswith("refusals-"):
            validate_query(rule, refusal_fields)
            rows = refusals
        else:
            rows = []
        outputs[rule.output] = render(template, rows)
    return outputs, observation, standards, refusals, rules


def validate_gates(root: Path, outputs: dict[Path, bytes], observation: dict, standards: list[dict[str, str]], refusals: list[dict[str, str]], rules: list[Rule]) -> list[str]:
    gates: list[str] = []
    obs = observation.get("observation", {})
    if not all(obs.get(key) is True for key in ("admitted", "aligned", "complete", "grounded", "bounded")):
        raise Refusal("CTDD-ADM-001", "O* admission fields must all be true")
    gates.append("010-admitted-observation")

    authority = observation.get("authority", {})
    if authority.get("broker_only") is not True:
        raise Refusal("CTDD-ACT-001", "broker_only must be true")
    if authority.get("hook_direct_actuation") is not False:
        raise Refusal("CTDD-HOOK-001", "hooks must be intent-only")
    if authority.get("network_actuation") is not False or authority.get("arbitrary_shell_actuation") is not False:
        raise Refusal("CTDD-ACT-001", "unbounded actuation authority admitted")
    gates.append("020-broker-authority")

    ids = {row["id"] for row in standards}
    if ids != REQUIRED_STANDARDS:
        raise Refusal("CTDD-VAL-001", f"standards closure mismatch: {sorted(ids ^ REQUIRED_STANDARDS)}")
    if len({row["order"] for row in standards}) != len(standards):
        raise Refusal("CTDD-VAL-001", "duplicate standard order")
    gates.append("030-standard-totality")

    codes = {row["code"] for row in refusals}
    if codes != REQUIRED_REFUSALS:
        raise Refusal("CTDD-VAL-001", f"refusal closure mismatch: {sorted(codes ^ REQUIRED_REFUSALS)}")
    if len({row["order"] for row in refusals}) != len(refusals):
        raise Refusal("CTDD-VAL-001", "duplicate refusal order")
    gates.append("040-refusal-totality")

    if any(row["standing"] != "ADMITTED" for row in standards):
        raise Refusal("CTDD-STD-001", "ontology overclaims standing")
    gates.append("050-standing-separation")

    if any(row["receipt"] != "true" for row in standards):
        raise Refusal("CTDD-ACT-002", "a standard permits unreceipted consequence")
    gates.append("060-receipt-totality")

    lifecycle = observation.get("manufacturing", {}).get("lifecycle")
    if lifecycle != ["Resolve", "Enrich", "Extract", "Render", "Write", "Receipt"]:
        raise Refusal("CTDD-VAL-001", "ggen lifecycle drift")
    gates.append("070-lifecycle-exactness")

    components = set(observation.get("receipt", {}).get("components", []))
    if components != RECEIPT_COMPONENTS:
        raise Refusal("CTDD-RCP-001", f"receipt component mismatch: {sorted(components ^ RECEIPT_COMPONENTS)}")
    if not observation.get("receipt", {}).get("previous_receipt"):
        raise Refusal("CTDD-RCP-002", "previous receipt is empty")
    gates.append("080-receipt-v2-closure")

    actual_outputs = {path.relative_to(root / CELL_REL / "generated").as_posix() for path in outputs}
    if actual_outputs != EXPECTED_OUTPUTS:
        raise Refusal("CTDD-GEN-001", f"generated ownership mismatch: {sorted(actual_outputs ^ EXPECTED_OUTPUTS)}")
    gates.append("090-output-ownership")

    first = {path: data for path, data in outputs.items()}
    second, *_ = manufacture(root)
    if first != second:
        raise Refusal("CTDD-GEN-003", "two projection passes differ")
    gates.append("100-second-sync-identity")

    for rule in rules:
        if rule.output not in outputs:
            raise Refusal("CTDD-GEN-001", f"unmaterialized rule {rule.name}")
        if not rule.template.is_file():
            raise Refusal("CTDD-VAL-001", f"missing template {rule.template}")
    gates.append("110-query-template-closure")

    if observation.get("manufacturing", {}).get("permanent_validation_read_only") is not True or observation.get("manufacturing", {}).get("fail_closed") is not True:
        raise Refusal("CTDD-VAL-001", "permanent validation must be read-only and fail-closed")
    gates.append("120-read-only-fail-closed")
    return gates


def compare_or_write(outputs: dict[Path, bytes], write: bool, generated_root: Path) -> None:
    for path, expected in outputs.items():
        if write:
            atomic_write(path, expected, generated_root)
            continue
        if not path.exists():
            raise Refusal("CTDD-GEN-001", f"missing generated output {path}")
        actual = path.read_bytes()
        if actual != expected:
            raise Refusal("CTDD-GEN-001", f"generated output drift at {path}")


def composite(parts: Iterable[tuple[str, bytes]]) -> bytes:
    payload = bytearray()
    for name, data in sorted(parts, key=lambda item: item[0]):
        payload.extend(name.encode("utf-8"))
        payload.extend(b"\0")
        payload.extend(len(data).to_bytes(8, "big"))
        payload.extend(data)
    return bytes(payload)


def receipt(root: Path, outputs: dict[Path, bytes], exact_head: str, require_blake3: bool) -> dict:
    cell = root / CELL_REL
    template_paths = sorted((cell / "templates").glob("*.tera"))
    components = {
        "ttl": (cell / "ontology.ttl").read_bytes(),
        "query": (cell / "ggen.toml").read_bytes(),
        "template": composite((path.relative_to(cell).as_posix(), path.read_bytes()) for path in template_paths),
        "observation": (cell / "O.star.toml").read_bytes(),
        "output": composite((path.relative_to(cell).as_posix(), data) for path, data in outputs.items()),
        "utility": (root / "scripts/verify_ggen_standards.py").read_bytes(),
        "test": (root / "scripts/sabotage_ggen_standards.py").read_bytes(),
        "previous_receipt": load_toml(cell / "O.star.toml")["receipt"]["previous_receipt"].encode("utf-8"),
    }
    if set(components) != RECEIPT_COMPONENTS:
        raise Refusal("CTDD-RCP-001", "receipt components incomplete")
    component_sha256 = {name: sha256_bytes(data) for name, data in sorted(components.items())}
    envelope = composite((name, data) for name, data in components.items())
    blake3 = b3sum_bytes(envelope)
    if require_blake3 and blake3 is None:
        raise Refusal("UNSUPPORTED", "BLAKE3 executor unavailable")
    return {
        "schema": "ctdd.receipt.v2",
        "standing": "ALIVE" if blake3 else "PARTIAL_ALIVE",
        "exact_head": exact_head,
        "algorithm": "blake3" if blake3 else "UNSUPPORTED",
        "digest": blake3,
        "diagnostic_sha256": sha256_bytes(envelope),
        "components_sha256": component_sha256,
        "previous_receipt": components["previous_receipt"].decode("utf-8"),
    }


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    cell = root / CELL_REL
    try:
        outputs, observation, standards, refusals, rules = manufacture(root)
        gates = validate_gates(root, outputs, observation, standards, refusals, rules)
        compare_or_write(outputs, args.write, cell / "generated")
        # Read-only validation after any bounded write proves committed bytes.
        compare_or_write(outputs, False, cell / "generated")
        evidence = receipt(root, outputs, args.exact_head, args.require_blake3)
        evidence["gates"] = gates
        evidence["counts"] = {"standards": len(standards), "refusals": len(refusals), "outputs": len(outputs)}
        if args.write_receipt:
            target = cell / "evidence/receipt.json"
            atomic_write(target, (json.dumps(evidence, indent=2, sort_keys=True) + "\n").encode(), cell / "evidence")
        print(json.dumps(evidence, indent=2, sort_keys=True))
        return 0
    except Refusal as exc:
        print(json.dumps({"standing": "BLOCKED", "refusal": exc.code, "message": exc.message}, sort_keys=True), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
