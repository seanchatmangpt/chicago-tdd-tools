#!/usr/bin/env python3
"""SPARQL gate set over the NATO court receipt RDF lift.

Usage: python3 spec-harness/run_gates.py <turtle-file-or-dir>...

Loads the lifted receipt graph(s), runs the gate queries, and exits 0 iff
every gate passes. Report is printed as JSON on stdout.

Gates:
  all-courts-pass   every lifted receipt's verdict is "Pass"
  refused-courts    (advisory) names receipts whose verdict is not "Pass"
  seal-presence     every receipt carries a 64-hex sealed digest
"""

import json
import sys

from rdflib import Graph

NS = "http://chicago-tdd-tools.dev/ontology/nato-court#"
HEX64 = "^[0-9a-fA-F]{64}$"

ALL_COURTS_PASS = f"""
PREFIX ct: <{NS}>
SELECT ?court ?v WHERE {{
  ?court a ct:CourtReceipt ;
         ct:verdict ?v .
  FILTER(?v != "Pass")
}}
"""

REFUSED_COURTS = f"""
PREFIX ct: <{NS}>
SELECT ?court ?v WHERE {{
  ?court a ct:CourtReceipt ;
         ct:verdict ?v .
}}
"""

SEAL_PRESENCE = f"""
PREFIX ct: <{NS}>
SELECT ?court WHERE {{
  ?court a ct:CourtReceipt ;
         ct:sealedDigest ?d .
  FILTER(!REGEX(?d, "{HEX64}"))
}}
"""

COUNT_RECEIPTS = f"""
PREFIX ct: <{NS}>
SELECT (COUNT(?c) AS ?n) WHERE {{ ?c a ct:CourtReceipt . }}
"""


def load(paths):
    g = Graph()
    for p in paths:
        fmt = "turtle" if p.endswith(".ttl") else "turtle"
        g.parse(p, format=fmt)
    return g


def run_gates(g):
    total = int(g.query(COUNT_RECEIPTS).bindings[0]["n"].toPython() or 0)
    non_pass = sorted(
        (str(row.court).split("#")[-1], str(row.v)) for row in g.query(ALL_COURTS_PASS)
    )
    seals = sorted(str(row.court).split("#")[-1] for row in g.query(SEAL_PRESENCE))
    total_query = g.query(REFUSED_COURTS)
    verdicts = {
        str(row.court).split("#")[-1]: str(row.v) for row in total_query
    }
    gates = {
        "all-courts-pass": {"pass": total > 0 and not non_pass, "violations": non_pass},
        "refused-courts": {"pass": True, "verdicts": verdicts},
        "seal-presence": {"pass": total > 0 and not seals, "violations": seals},
    }
    return {
        "total_receipts": total,
        "gates": gates,
        "ok": all(g["pass"] for g in gates.values()),
    }


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    g = load(argv[1:])
    report = run_gates(g)
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
