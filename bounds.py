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
import re
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


def producer_criteria(manifest: dict) -> dict[str, dict]:
    """Every criterion each declared producer states, and its reason set.

    Read from the producer's own tree rather than restated here: a restated list
    goes stale the day somebody adds a criterion, and the whole point of the
    number is that nobody can add one without the corpus noticing it is
    unreached.
    """
    out: dict[str, dict] = {}
    for name, declared in (manifest.get("producers") or {}).items():
        pin = ROOT / declared["criteria_pin"]
        pinned = yaml.safe_load(pin.read_text()) if pin.exists() else None
        criteria: set[str] = set(pinned["criteria"]) if pinned else set()
        unreachable: dict[str, str] = {}
        for reason, text in (declared.get("unreachable") or {}).items():
            for identifier in re.findall(
                r"(?:FR|NFR|StR)-\d+-(?:AC|CON|VC)-\d+", text
            ):
                unreachable[identifier] = reason
        out[name] = {
            "root": str(pin),
            "readable": pinned is not None,
            "revision": (pinned or {}).get("revision"),
            "criteria": criteria,
            "unreachable": unreachable,
        }
    return out


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
            # Same language by default. A property that is language independent
            # may point at one control, but the case must name the language it
            # lives in — otherwise "the control is over there somewhere" is
            # indistinguishable from having none.
            where = meta.get("control_language", language)
            if (family, pair, where) not in on_disk:
                problems.append(
                    f"{family}/{case}/{language} names control {pair!r} in "
                    f"{where}, which does not exist"
                )

    # ── criterion coverage ───────────────────────────────────────────────
    claimed: set[str] = set()
    for path in on_disk.values():
        meta = yaml.safe_load((path / "case.yaml").read_text())
        claimed |= set(meta.get("criteria") or [])

    coverage: dict[str, dict] = {}
    for name, declared in producer_criteria(manifest).items():
        if not declared["readable"]:
            coverage[name] = {
                "state": "unavailable",
                "why": f"{declared['root']} is not readable from here",
            }
            continue
        reached = sorted(claimed & declared["criteria"])
        unreachable = {
            identifier: reason
            for identifier, reason in declared["unreachable"].items()
            if identifier in declared["criteria"]
        }
        unreached = sorted(declared["criteria"] - set(reached) - set(unreachable))
        stale = sorted(set(declared["unreachable"]) - declared["criteria"])
        phantom = sorted(claimed - declared["criteria"])
        coverage[name] = {
            "state": "measured",
            "total": len(declared["criteria"]),
            "reached": reached,
            "unreachable": unreachable,
            "unreached": unreached,
            # An id declared unreachable that the producer no longer states, or
            # claimed by a case and stated by nobody. Both are how a coverage
            # number drifts away from the thing it describes.
            "stale_unreachable": stale,
            "claimed_but_undeclared": phantom,
        }
        for identifier in unreached:
            problems.append(
                f"{name} {identifier} is reached by no case and declared "
                "unreachable by nothing"
            )
        for identifier in stale:
            problems.append(
                f"{name} {identifier} is declared unreachable and the producer "
                "no longer states it"
            )
        for identifier in phantom:
            problems.append(
                f"{name} {identifier} is claimed by a case and stated by no "
                "requirement"
            )

    return {
        "criterion_coverage": coverage,
        "covered": sorted(cells),
        "gaps": sorted(gaps),
        "gap_count": len(gaps),
        "scoped_out": sorted(scoped_out),
        "undeclared": sorted(undeclared),
        "problems": sorted(set(problems)),
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
        for name, coverage in sorted(report["criterion_coverage"].items()):
            if coverage["state"] != "measured":
                print(f"{name}: criteria unavailable — {coverage['why']}")
                continue
            print(
                f"{name}  {len(coverage['reached'])} reached, "
                f"{len(coverage['unreachable'])} unreachable, "
                f"{len(coverage['unreached'])} unreached  of "
                f"{coverage['total']}"
            )
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
