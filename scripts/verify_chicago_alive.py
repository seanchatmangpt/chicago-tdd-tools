#!/usr/bin/env python3
"""Independent CHICAGO_ALIVE standing court.

The court derives standing from evidence.  It never trusts a caller-supplied
standing and it has no actuation primitive.  The only lawful promotion chain is:

    PARTIAL_ALIVE -> CHICAGO_ALIVE -> ALIVE

CHICAGO_ALIVE means the exact software subject executed end-to-end in an
admitted executable world using real load-bearing collaborators, observable
consequences, adversarial falsifiers, receipts, deterministic replay, and a
bounded correspondence contract.  ALIVE additionally requires the exact real
target identity and an observed real-target consequence.
"""
from __future__ import annotations

import argparse
import copy
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

VALID_WORLD_KINDS = {"SYNTHETIC", "EMULATED", "TEST_TENANT", "REAL_TARGET"}
VALID_REPLAY = {"MATCH", "MISMATCH", "NOT_RUN"}
VALID_COLLABORATOR_KINDS = {"REAL", "TEST_DOUBLE"}
SHA40 = re.compile(r"^[0-9a-f]{40}$")


@dataclass(frozen=True)
class Finding:
    code: str
    detail: str


def _nonempty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _canonical_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")


def _blake3_hex(payload: bytes) -> str:
    executable = shutil.which("b3sum")
    if executable is None:
        raise RuntimeError("BLAKE3_UNAVAILABLE:b3sum")
    completed = subprocess.run(
        [executable],
        input=payload,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"BLAKE3_FAILED:{detail}")
    output = completed.stdout.decode("ascii", errors="strict").strip().split()
    if not output or len(output[0]) != 64:
        raise RuntimeError("BLAKE3_MALFORMED_OUTPUT")
    return output[0]


def evaluate(data: dict[str, Any], *, exact_head: str | None = None) -> dict[str, Any]:
    blockers: list[Finding] = []
    refusals: list[Finding] = []
    gaps: list[Finding] = []
    missing_for_alive: list[str] = []

    subject = data.get("subject", {})
    world = data.get("world", {})
    authority = data.get("authority", {})
    trial = data.get("trial", {})
    correspondence = data.get("correspondence", {})
    replay = data.get("replay", "NOT_RUN")
    collaborators = data.get("collaborators", [])
    falsifiers = data.get("falsifiers", [])
    receipt_refs = data.get("receipt_refs", [])

    if not isinstance(subject, dict):
        blockers.append(Finding("CTA-SUB-000", "subject must be an object"))
        subject = {}
    if not _nonempty(subject.get("ref")):
        blockers.append(Finding("CTA-SUB-001", "exact subject ref is missing"))
    if subject.get("exact") is not True:
        blockers.append(Finding("CTA-SUB-002", "subject identity is not exact"))
    commit_sha = subject.get("commit_sha")
    if not isinstance(commit_sha, str) or SHA40.fullmatch(commit_sha) is None:
        blockers.append(Finding("CTA-SUB-003", "subject commit_sha must be exact lowercase 40-hex"))
    elif exact_head is not None and commit_sha != exact_head:
        refusals.append(
            Finding(
                "CTA-SUB-004",
                f"subject commit {commit_sha} does not match required exact head {exact_head}",
            )
        )

    if not isinstance(world, dict):
        blockers.append(Finding("CTA-WLD-000", "world must be an object"))
        world = {}
    if not _nonempty(world.get("ref")):
        blockers.append(Finding("CTA-WLD-001", "world ref is missing"))
    if world.get("admitted") is not True:
        blockers.append(Finding("CTA-WLD-002", "world is not admitted"))
    kind = world.get("kind")
    if kind not in VALID_WORLD_KINDS:
        blockers.append(Finding("CTA-WLD-003", f"unsupported world kind: {kind!r}"))

    if not isinstance(collaborators, list):
        blockers.append(Finding("CTA-COL-000", "collaborators must be a list"))
        collaborators = []
    load_bearing: list[dict[str, Any]] = []
    for edge in collaborators:
        if not isinstance(edge, dict):
            blockers.append(Finding("CTA-COL-000", "collaborator edge must be an object"))
            continue
        if edge.get("load_bearing") is True:
            load_bearing.append(edge)
            edge_kind = edge.get("kind")
            edge_id = edge.get("id", "<unnamed>")
            if edge_kind not in VALID_COLLABORATOR_KINDS:
                blockers.append(
                    Finding(
                        "CTA-COL-002",
                        f"unsupported collaborator kind for {edge_id}: {edge_kind!r}",
                    )
                )
            elif edge_kind != "REAL":
                refusals.append(
                    Finding("CTA-COL-001", f"load-bearing collaborator is not real: {edge_id}")
                )
            if not _nonempty(edge.get("evidence_ref")):
                gaps.append(
                    Finding("CTA-COL-003", f"load-bearing collaborator lacks evidence: {edge_id}")
                )
    if not load_bearing:
        gaps.append(Finding("CTA-COL-004", "no load-bearing collaboration edge was evidenced"))

    if not isinstance(authority, dict):
        blockers.append(Finding("CTA-AUT-000", "authority must be an object"))
        authority = {}
    if authority.get("direct_actuation") is True:
        refusals.append(Finding("CTA-AUT-001", "direct actuation bypasses the broker boundary"))
    if authority.get("path_observed") is not True:
        gaps.append(Finding("CTA-AUT-002", "authority path was not observed"))
    if kind != "REAL_TARGET" and authority.get("production_authority_used") is True:
        refusals.append(
            Finding("CTA-AUT-003", "production authority was used outside the real target world")
        )

    if not isinstance(trial, dict):
        blockers.append(Finding("CTA-EXE-000", "trial must be an object"))
        trial = {}
    if trial.get("execution_observed") is not True:
        gaps.append(
            Finding("CTA-EXE-001", "exact subject execution was not observed in the admitted world")
        )
    if trial.get("consequence_observed") is not True:
        gaps.append(Finding("CTA-CON-001", "observable world consequence is missing"))

    if not isinstance(receipt_refs, list) or not any(_nonempty(item) for item in receipt_refs):
        gaps.append(Finding("CTA-RCP-001", "no consequence receipt was supplied"))

    if replay not in VALID_REPLAY:
        blockers.append(Finding("CTA-RPL-000", f"unsupported replay state: {replay!r}"))
    elif replay == "MISMATCH":
        refusals.append(Finding("CTA-RPL-001", "receipt replay diverged"))
    elif replay != "MATCH":
        gaps.append(Finding("CTA-RPL-002", "receipt replay has not matched"))

    if not isinstance(falsifiers, list):
        blockers.append(Finding("CTA-FAL-000", "falsifiers must be a list"))
        falsifiers = []
    if not falsifiers:
        gaps.append(Finding("CTA-FAL-001", "no adversarial falsifier was executed"))
    else:
        for item in falsifiers:
            if not isinstance(item, dict):
                blockers.append(Finding("CTA-FAL-000", "falsifier must be an object"))
                continue
            fid = item.get("id", "<unnamed>")
            if item.get("passed") is not True:
                refusals.append(Finding("CTA-FAL-002", f"falsifier did not pass: {fid}"))
            if not _nonempty(item.get("evidence_ref")):
                gaps.append(Finding("CTA-FAL-003", f"falsifier lacks evidence: {fid}"))

    if not isinstance(correspondence, dict):
        blockers.append(Finding("CTA-COR-000", "correspondence must be an object"))
        correspondence = {}
    if correspondence.get("bounded") is not True:
        blockers.append(Finding("CTA-COR-001", "target correspondence is not bounded"))
    if not _nonempty(correspondence.get("target_ref")):
        blockers.append(Finding("CTA-COR-002", "correspondence target ref is missing"))
    if not _nonempty(correspondence.get("contract_ref")):
        blockers.append(Finding("CTA-COR-003", "correspondence contract ref is missing"))

    real_consequence = trial.get("real_target_consequence_observed") is True
    real_identity = trial.get("real_target_identity_exact") is True
    if real_consequence and kind != "REAL_TARGET":
        refusals.append(
            Finding("CTA-ALV-001", "real-target consequence claimed outside REAL_TARGET world")
        )

    if blockers:
        standing = "BLOCKED"
    elif refusals:
        standing = "REFUSED"
    elif gaps:
        standing = "PARTIAL_ALIVE"
    elif kind == "REAL_TARGET" and real_identity and real_consequence:
        standing = "ALIVE"
    else:
        standing = "CHICAGO_ALIVE"
        if kind != "REAL_TARGET":
            missing_for_alive.append("exact real target world")
        if not real_identity:
            missing_for_alive.append("exact real target identity")
        if not real_consequence:
            missing_for_alive.append("observed real target consequence")

    def dump(findings: list[Finding]) -> list[dict[str, str]]:
        return [
            {"code": finding.code, "detail": finding.detail}
            for finding in sorted(findings, key=lambda finding: (finding.code, finding.detail))
        ]

    return {
        "standing": standing,
        "blockers": dump(blockers),
        "refusals": dump(refusals),
        "gaps": dump(gaps),
        "missing_for_alive": sorted(missing_for_alive),
        "direct_actuation": False,
    }


def build_receipt(
    data: dict[str, Any],
    decision: dict[str, Any],
    *,
    exact_head: str | None,
) -> dict[str, Any]:
    input_digest = _blake3_hex(_canonical_bytes(data))
    decision_digest = _blake3_hex(_canonical_bytes(decision))
    envelope = {
        "algorithm": "BLAKE3",
        "exact_head": exact_head,
        "input_digest": input_digest,
        "decision_digest": decision_digest,
        "standing": decision["standing"],
        "direct_actuation": False,
    }
    composite_digest = _blake3_hex(_canonical_bytes(envelope))
    return {**envelope, "composite_digest": composite_digest}


def replay_receipt(
    expected: dict[str, Any],
    data: dict[str, Any],
    decision: dict[str, Any],
    *,
    exact_head: str | None,
) -> str:
    actual = build_receipt(data, decision, exact_head=exact_head)
    return "MATCH" if expected == actual else "MISMATCH"


def _base() -> dict[str, Any]:
    return {
        "subject": {
            "ref": "seanchatmangpt/chicago-tdd-tools@0123456789abcdef0123456789abcdef01234567",
            "exact": True,
            "commit_sha": "0123456789abcdef0123456789abcdef01234567",
        },
        "world": {
            "ref": "urn:gymact:world:chicago-alive",
            "admitted": True,
            "kind": "SYNTHETIC",
        },
        "authority": {
            "path_observed": True,
            "direct_actuation": False,
            "production_authority_used": False,
        },
        "trial": {
            "execution_observed": True,
            "consequence_observed": True,
            "real_target_identity_exact": False,
            "real_target_consequence_observed": False,
        },
        "collaborators": [
            {
                "id": "router",
                "kind": "REAL",
                "load_bearing": True,
                "evidence_ref": "urn:evidence:router",
            },
            {
                "id": "authority",
                "kind": "REAL",
                "load_bearing": True,
                "evidence_ref": "urn:evidence:authority",
            },
        ],
        "falsifiers": [
            {"id": "authority-removed", "passed": True, "evidence_ref": "urn:evidence:f1"},
            {"id": "receipt-tamper", "passed": True, "evidence_ref": "urn:evidence:f2"},
        ],
        "correspondence": {
            "bounded": True,
            "target_ref": "urn:target:gcp",
            "contract_ref": "urn:contract:gcp:v1",
            "known_divergences": [],
        },
        "receipt_refs": ["urn:receipt:trial"],
        "replay": "MATCH",
    }


def self_test() -> None:
    base = _base()

    def check(name: str, mutate: Any, expected: str) -> None:
        candidate = copy.deepcopy(base)
        mutate(candidate)
        actual = evaluate(candidate)["standing"]
        if actual != expected:
            raise AssertionError(
                f"{name}: expected {expected}, got {actual}: {evaluate(candidate)}"
            )

    check("synthetic crown", lambda _: None, "CHICAGO_ALIVE")
    check(
        "real crown",
        lambda data: (
            data["world"].update(kind="REAL_TARGET"),
            data["trial"].update(
                real_target_identity_exact=True,
                real_target_consequence_observed=True,
            ),
        ),
        "ALIVE",
    )
    check("replay not run", lambda data: data.update(replay="NOT_RUN"), "PARTIAL_ALIVE")
    check(
        "load-bearing mock",
        lambda data: data["collaborators"][0].update(kind="TEST_DOUBLE"),
        "REFUSED",
    )
    check(
        "direct actuation",
        lambda data: data["authority"].update(direct_actuation=True),
        "REFUSED",
    )
    check("unadmitted world", lambda data: data["world"].update(admitted=False), "BLOCKED")
    check(
        "failed falsifier",
        lambda data: data["falsifiers"][0].update(passed=False),
        "REFUSED",
    )
    check("no falsifiers", lambda data: data.update(falsifiers=[]), "PARTIAL_ALIVE")
    check("no receipt", lambda data: data.update(receipt_refs=[]), "PARTIAL_ALIVE")
    check(
        "unbounded correspondence",
        lambda data: data["correspondence"].update(bounded=False),
        "BLOCKED",
    )
    check(
        "real consequence wrong world",
        lambda data: data["trial"].update(real_target_consequence_observed=True),
        "REFUSED",
    )

    mismatch = evaluate(base, exact_head="f" * 40)
    if mismatch["standing"] != "REFUSED":
        raise AssertionError(f"exact-head mismatch should refuse: {mismatch}")

    reordered = copy.deepcopy(base)
    reordered["collaborators"].reverse()
    reordered["falsifiers"].reverse()
    if evaluate(base) != evaluate(reordered):
        raise AssertionError("input ordering changed semantic decision")

    print("CHICAGO_ALIVE_SELF_TESTS=13/13")


def _load_json(path: Path) -> dict[str, Any]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ValueError("INPUT_NOT_OBJECT")
    return raw


def main() -> int:
    parser = argparse.ArgumentParser(description="Independent CHICAGO_ALIVE standing court")
    parser.add_argument("trial", nargs="?", type=Path)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--require-exact-head")
    parser.add_argument("--require-blake3", action="store_true")
    parser.add_argument("--write-receipt", type=Path)
    parser.add_argument("--replay-receipt", type=Path)
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    if args.trial is None:
        parser.error("trial JSON path is required unless --self-test is used")

    if args.require_exact_head is not None and SHA40.fullmatch(args.require_exact_head) is None:
        print(json.dumps({"standing": "BLOCKED", "error": "INVALID_REQUIRED_HEAD"}, sort_keys=True))
        return 2

    try:
        data = _load_json(args.trial)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(json.dumps({"standing": "BLOCKED", "error": f"INPUT_ERROR:{exc}"}, sort_keys=True))
        return 2

    decision = evaluate(data, exact_head=args.require_exact_head)
    output: dict[str, Any] = {"decision": decision}

    needs_blake3 = args.require_blake3 or args.write_receipt is not None or args.replay_receipt is not None
    if needs_blake3:
        try:
            receipt = build_receipt(data, decision, exact_head=args.require_exact_head)
        except RuntimeError as exc:
            print(json.dumps({"standing": "BLOCKED", "error": str(exc)}, sort_keys=True))
            return 2
        output["receipt"] = receipt
        if args.write_receipt is not None:
            args.write_receipt.write_text(
                json.dumps(receipt, sort_keys=True, indent=2) + "\n",
                encoding="utf-8",
            )
        if args.replay_receipt is not None:
            try:
                expected = _load_json(args.replay_receipt)
            except (OSError, json.JSONDecodeError, ValueError) as exc:
                print(json.dumps({"standing": "BLOCKED", "error": f"REPLAY_INPUT_ERROR:{exc}"}, sort_keys=True))
                return 2
            court_replay = replay_receipt(
                expected,
                data,
                decision,
                exact_head=args.require_exact_head,
            )
            output["court_replay"] = court_replay
            if court_replay != "MATCH":
                print(json.dumps(output, sort_keys=True, indent=2))
                return 2

    print(json.dumps(output, sort_keys=True, indent=2))
    return 0 if decision["standing"] in {"CHICAGO_ALIVE", "ALIVE"} else 2


if __name__ == "__main__":
    sys.exit(main())
