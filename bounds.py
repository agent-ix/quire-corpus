#!/usr/bin/env python3
"""Derive the coverage matrix from the inventory and the filesystem.

`corpus.yaml` declares **intent** — for each case, the languages it must
exist in, and any cell deliberately scoped out with a reason. This computes
the **state**: which cells have a fixture, which do not, and the counts.

Nothing is stored. A stored count is a number that can go stale; a derived
one cannot disagree with the tree it describes, and adding a fixture flips
its own cell with no edit to any central file.

    python3 bounds.py            # human summary
    python3 bounds.py --json     # the matrix, for a runner

Exit status is 1 when a declared cell has no fixture, when a fixture exists
that the inventory does not declare, when a case has no `issue_ref`, or when
a positive that names a control has none. Each of those is a way for a
corpus to look complete while measuring less than it claims.
"""
from __future__ import annotations

import json
import pathlib
import sys

import yaml

ROOT = pathlib.Path(__file__).resolve().parent
FIXTURES = ROOT / "fixtures"


class CorpusError(RuntimeError):
    """A declaration and the tree disagree."""


def load_manifest() -> dict:
    return yaml.safe_load((ROOT / "corpus.yaml").read_text())


def _declared(entry) -> tuple[list[str], dict[str, str]]:
    """A cell list is either a bare language list or a scoped-out mapping."""
    if isinstance(entry, list):
        return entry, {}
    languages = entry.get("languages", [])
    out_of_scope = entry.get("out_of_scope", {}) or {}
    for language, reason in out_of_scope.items():
        if not str(reason).strip():
            raise CorpusError(
                f"out_of_scope[{language}] has no reason; an exclusion "
                "without one is a gap wearing a different word"
            )
    return languages, out_of_scope


def discover() -> dict[tuple[str, str, str], pathlib.Path]:
    """Every fixture on disk, keyed by (family, case, language)."""
    found: dict[tuple[str, str, str], pathlib.Path] = {}
    for case_file in sorted(FIXTURES.rglob("case.yaml")):
        rel = case_file.relative_to(FIXTURES).parts
        if len(rel) != 4:
            raise CorpusError(
                f"{case_file} is not at the four-segment path "
                "fixtures/<family>/<case>/<language>/case.yaml"
            )
        family, case, language, _ = rel
        found[(family, case, language)] = case_file.parent
    return found


def audit() -> dict:
    manifest = load_manifest()
    inventory = manifest["inventory"]
    on_disk = discover()

    cells, gaps, scoped_out, undeclared, problems = [], [], [], [], []

    for family, cases in inventory.items():
        for case, entry in cases.items():
            languages, out_of_scope = _declared(entry)
            for language in languages:
                key = (family, case, language)
                if key in on_disk:
                    cells.append(key)
                else:
                    gaps.append(key)
            for language, reason in out_of_scope.items():
                scoped_out.append((family, case, language, reason))
                if (family, case, language) in on_disk:
                    problems.append(
                        f"{family}/{case}/{language} is scoped out and also "
                        "has a fixture; one of the two is wrong"
                    )

    declared_keys = set(cells) | set(gaps)
    for key in on_disk:
        if key not in declared_keys:
            undeclared.append(key)

    controls = {
        (f, yaml.safe_load((p / "case.yaml").read_text()).get("control_for"), lang)
        for (f, _c, lang), p in on_disk.items()
        if yaml.safe_load((p / "case.yaml").read_text()).get("kind") == "control"
    }
    for (family, case, language), path in sorted(on_disk.items()):
        meta = yaml.safe_load((path / "case.yaml").read_text())
        if not str(meta.get("issue_ref", "")).strip():
            problems.append(f"{family}/{case}/{language} has no issue_ref")
        if "pending" in meta and not str(meta.get("pending", "")).strip():
            problems.append(
                f"{family}/{case}/{language} is pending on nothing; a marker "
                "with no issue is a case nobody comes back to"
            )
        if not (path / "expected.yaml").exists():
            problems.append(f"{family}/{case}/{language} has no expected.yaml")
        if not (path / "input").is_dir():
            problems.append(f"{family}/{case}/{language} has no input tree")
        if meta.get("kind") == "positive" and meta.get("control_for_pair"):
            pair = meta["control_for_pair"]
            if (family, pair, language) not in on_disk:
                problems.append(
                    f"{family}/{case}/{language} names control {pair!r}, "
                    "which does not exist"
                )

    return {
        "covered": sorted(cells),
        "gaps": sorted(gaps),
        "gap_count": len(gaps),
        "scoped_out": sorted(scoped_out),
        "undeclared": sorted(undeclared),
        "problems": sorted(problems),
        "controls": sorted(c for c in controls if c[1]),
    }


def main() -> int:
    try:
        report = audit()
    except CorpusError as err:
        # A structural error is a finding, not a crash: a traceback reads as a
        # broken tool and gets the tool blamed instead of the tree.
        print(f"  PROBLEM    {err}")
        return 1
    if "--json" in sys.argv:
        print(json.dumps(report, indent=1, sort_keys=True))
    else:
        print(f"covered      {len(report['covered'])}")
        print(f"gap_count    {report['gap_count']}")
        print(f"scoped out   {len(report['scoped_out'])}")
        for family, case, language in report["gaps"]:
            print(f"  GAP        {family}/{case}/{language}")
        for family, case, language in report["undeclared"]:
            print(f"  UNDECLARED {family}/{case}/{language}")
        for problem in report["problems"]:
            print(f"  PROBLEM    {problem}")
    failed = report["gap_count"] or report["undeclared"] or report["problems"]
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
