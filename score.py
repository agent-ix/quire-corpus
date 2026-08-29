#!/usr/bin/env python3
"""Score a producer's canonical records against the corpus's expected truth.

The corpus grades output, never a producer. Anything that reads a fixture's
`input/` tree and writes canonical records on stdout can be scored, and two
producers scored at the same `corpus_revision` are comparable.

    python3 score.py --producer 'quire-code-extract --org {org} --repo {repo} {input}'
    python3 score.py --producer '...' --json > observation.json
    python3 score.py --producer '...' --case relations/receiver-typed-call/rust

What it will not do:

* report an unsupported query as a measured zero. A plain extractor emits no
  `paths`, `impact` or `test_selection`, so those grade as `not-computed` and
  stay out of every ratio. A zero and an absence are different claims and the
  difference is the whole reason for grading a corpus at all.
* penalise a producer for edge types the case does not speak about. `edges`
  is an expected-present set scoped to the types it names, plus whatever
  `forbidden_edge_types` and `forbidden_edges` add. `nodes`, when present, is
  exhaustive — these trees are small and authored for that.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
from collections import defaultdict

import yaml

from bounds import CorpusError, discover
from digest import case_digests, corpus_revision

ROOT = pathlib.Path(__file__).resolve().parent


def _triple(edge: dict) -> tuple[str, str, str]:
    return (edge["source_ref"], edge["edge_type"], edge["target_ref"])


def _expected_triple(edge: dict) -> tuple[str, str, str]:
    return (edge["source"], edge["type"], edge["target"])


class Tally:
    """True/false positives and negatives, sliced every way the report needs."""

    def __init__(self) -> None:
        self.slices: dict[tuple[str, str], dict[str, int]] = defaultdict(
            lambda: {"tp": 0, "fp": 0, "fn": 0}
        )

    def add(self, outcome: str, **slices: str) -> None:
        self.slices[("total", "total")][outcome] += 1
        for axis, value in slices.items():
            if value:
                self.slices[(axis, value)][outcome] += 1

    def as_dict(self) -> dict:
        out: dict[str, dict[str, dict]] = defaultdict(dict)
        for (axis, value), counts in sorted(self.slices.items()):
            tp, fp, fn = counts["tp"], counts["fp"], counts["fn"]
            out[axis][value] = {
                "tp": tp,
                "fp": fp,
                "fn": fn,
                # Reported, never assumed: a partition with no population has
                # no precision, and 1.0 would read as a perfect score.
                "precision": None if tp + fp == 0 else round(tp / (tp + fp), 6),
                "recall": None if tp + fn == 0 else round(tp / (tp + fn), 6),
            }
        return dict(out)


def run_producer(template: str, case_dir: pathlib.Path, expected: dict) -> dict:
    command = template.format(
        org=expected["org"],
        repo=expected["repo"],
        input=str(case_dir / "input"),
    )
    proc = subprocess.run(
        command, shell=True, capture_output=True, text=True, cwd=ROOT
    )
    if proc.returncode != 0:
        raise CorpusError(
            f"producer exited {proc.returncode} on {case_dir.name}: "
            f"{proc.stderr.strip()[:400]}"
        )
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise CorpusError(
            f"producer output on {case_dir.name} is not JSON: {exc}"
        ) from exc


def score_case(meta: dict, expected: dict, produced: dict, tally: Tally) -> dict:
    language = meta["language"]
    findings: list[str] = []
    tier_disagreements: list[dict] = []
    nodes = produced.get("nodes", [])
    edges = produced.get("edges", [])

    by_name = {n["name"]: n for n in nodes}

    # ── nodes: exhaustive when declared ──────────────────────────────────
    #
    # Graded on `(object_type, name)`. `kind` is deliberately NOT graded: it is
    # the grammar's node kind, so it is language-specific by construction —
    # TypeScript reports a class member as `method`, Rust reports the same
    # declaration as `function`, and no cross-language vocabulary is declared
    # anywhere (agent-ix/quire-code-rs#16). Grading it would report a vocabulary
    # difference as a correctness defect and bury the real recall gaps beneath
    # it. Every observed kind is censused below, so the difference stays visible
    # rather than being dropped.
    kind_census: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for node in nodes:
        kind_census[node["object_type"]][str(node["data"].get("kind"))] += 1

    if "nodes" in expected:
        want = {(n["name"], n["object_type"]) for n in expected["nodes"]}
        got = {(n["name"], n["object_type"]) for n in nodes}
        for name, object_type in sorted(want - got):
            tally.add("fn", language=language, object_type=object_type, axis_kind="node")
            findings.append(f"missing node {object_type} {name}")
        for name, object_type in sorted(got - want):
            tally.add("fp", language=language, object_type=object_type, axis_kind="node")
            findings.append(f"unexpected node {object_type} {name}")
        for name, object_type in sorted(want & got):
            tally.add("tp", language=language, object_type=object_type, axis_kind="node")

    for kind in expected.get("forbidden_node_kinds", []):
        for node in nodes:
            if node["data"].get("kind") == kind:
                tally.add("fp", language=language, node_kind=kind, axis_kind="node")
                findings.append(
                    f"forbidden node kind {kind!r}: {node['name']} — this "
                    "language cannot declare one"
                )

    # ── edges: expected-present, scoped to the types the case names ──────
    want_edges = {_expected_triple(e): e for e in expected.get("edges", [])}
    got_edges = {_triple(e): e for e in edges}
    scoped_types = {e["type"] for e in expected.get("edges", [])}
    scoped_types |= set(expected.get("forbidden_edge_types", []))

    for triple, want in sorted(want_edges.items()):
        got = got_edges.get(triple)
        if got is None:
            tally.add("fn", language=language, relation=triple[1],
                      tier=want.get("reason"), axis_kind="edge")
            findings.append(f"missing edge {triple[0]} -{triple[1]}-> {triple[2]}")
            continue
        tally.add("tp", language=language, relation=triple[1],
                  tier=got.get("reason"), axis_kind="edge")
        # The edge is the truth; the tier is the producer's account of how it got
        # there. No contract states a rank order between `import-scoped` and
        # `receiver-typed` where both apply, so a disagreement is recorded and
        # reported, never scored as a wrong edge. A corpus that failed on it
        # would be grading a design choice.
        if want.get("reason") and got.get("reason") != want["reason"]:
            tier_disagreements.append({
                "edge": list(triple),
                "expected": want["reason"],
                "reported": got.get("reason"),
            })

    for triple, got in sorted(got_edges.items()):
        if triple in want_edges or triple[1] not in scoped_types:
            continue
        tally.add("fp", language=language, relation=triple[1],
                  tier=got.get("reason"), axis_kind="edge")
        findings.append(
            f"unexpected edge {triple[0]} -{triple[1]}-> {triple[2]} "
            f"(tier {got.get('reason')!r})"
        )

    for forbidden in expected.get("forbidden_edges", []):
        triple = _expected_triple(forbidden)
        if triple in got_edges:
            findings.append(
                f"forbidden edge emitted: {triple[0]} -{triple[1]}-> {triple[2]}"
            )

    # ── field fidelity ───────────────────────────────────────────────────
    for name, want_visibility in (expected.get("visibility") or {}).items():
        node = by_name.get(name)
        if node is None:
            tally.add("fn", language=language, axis_kind="visibility")
            findings.append(f"visibility unmeasurable: no node named {name}")
            continue
        got_visibility = node["data"].get("visibility")
        if got_visibility == want_visibility:
            tally.add("tp", language=language, axis_kind="visibility")
        else:
            tally.add("fp", language=language, axis_kind="visibility")
            findings.append(
                f"{name} classified {got_visibility!r}, expected {want_visibility!r}"
            )

    for name, want_signature in (expected.get("signatures") or {}).items():
        node = by_name.get(name)
        got_signature = None if node is None else node["data"].get("signature")
        if got_signature == want_signature:
            tally.add("tp", language=language, axis_kind="signature")
        else:
            tally.add("fp", language=language, axis_kind="signature")
            findings.append(
                f"{name} signature {got_signature!r}, expected {want_signature!r}"
            )

    # ── censuses ─────────────────────────────────────────────────────────
    census = {}
    stats = produced.get("stats") or {}
    if "ambiguous_call_sites" in expected:
        census["ambiguous_call_sites"] = {
            "expected": expected["ambiguous_call_sites"],
            "reported": stats.get("unresolved_calls"),
            "state": "measured" if "unresolved_calls" in stats else "not-computed",
        }
    if "expect_diagnostics" in expected:
        census["diagnostics"] = {
            "expected": expected["expect_diagnostics"],
            "reported": len(produced.get("diagnostics", [])),
            "state": "measured",
        }

    # ── derived queries: absent is not zero ──────────────────────────────
    derived = {}
    for key, payload_key in (("paths", "paths"), ("impact", "impact"),
                             ("test_selection", "test_selection"),
                             ("derived_links", "links")):
        if key not in expected:
            continue
        if payload_key not in produced:
            derived[key] = {
                "state": "not-computed",
                "why": "the producer emits no such answer; an extractor is "
                       "not a traversal engine, and a zero here would be a "
                       "measurement nobody took",
            }
        else:
            derived[key] = {"state": "measured",
                            "matches": produced[payload_key] == expected[key]}

    return {
        "findings": findings,
        "tier_disagreements": tier_disagreements,
        "kind_census": {k: dict(v) for k, v in sorted(kind_census.items())},
        "census": census,
        "derived": derived,
        "population": {
            "nodes_produced": len(nodes),
            "edges_produced": len(edges),
            "nodes_expected": len(expected.get("nodes", [])),
            "edges_expected": len(want_edges),
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--producer", required=True,
                        help="command template with {org} {repo} {input}")
    parser.add_argument("--case", action="append", default=[],
                        help="score only these <family>/<case>/<language> keys")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    selected = set(args.case)
    tally = Tally()
    cases: dict[str, dict] = {}
    failed = False

    for (family, case, language), case_dir in sorted(discover().items()):
        key = f"{family}/{case}/{language}"
        if selected and key not in selected:
            continue
        meta = yaml.safe_load((case_dir / "case.yaml").read_text())
        expected = yaml.safe_load((case_dir / "expected.yaml").read_text())
        produced = run_producer(args.producer, case_dir, expected)
        result = score_case(meta, expected, produced, tally)
        result["kind"] = meta.get("kind")
        cases[key] = result
        if result["findings"]:
            failed = True

    report = {
        "schema_version": 1,
        "corpus_revision": corpus_revision(),
        "case_digests": case_digests(),
        "producer_invocation": args.producer,
        "scored_cases": len(cases),
        "confusion": tally.as_dict(),
        "cases": cases,
    }

    if args.json:
        print(json.dumps(report, indent=1, sort_keys=True))
    else:
        totals = report["confusion"].get("total", {}).get("total", {})
        print(f"corpus_revision {report['corpus_revision'][:16]}  "
              f"cases {len(cases)}")
        print(f"tp {totals.get('tp')}  fp {totals.get('fp')}  "
              f"fn {totals.get('fn')}  "
              f"precision {totals.get('precision')}  "
              f"recall {totals.get('recall')}")
        for key, result in cases.items():
            mark = "FAIL" if result["findings"] else "ok  "
            print(f"  {mark} {key}")
            for finding in result["findings"]:
                print(f"       {finding}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
