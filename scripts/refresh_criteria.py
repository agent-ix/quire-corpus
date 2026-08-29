#!/usr/bin/env python3
"""Re-pin a producer's criteria from its own spec tree.

Pinned rather than read live: a coverage number computed against whatever is on
somebody's disk is not reproducible, and NFR-001 says a clean runner reproduces
this corpus from its own tree. Moving the pin is the reviewable event — it is
where a criterion appears and where the corpus is obliged to notice it is
unreached.

    python3 scripts/refresh_criteria.py ~/dev/quire-code-rs
"""
from __future__ import annotations

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: refresh_criteria.py <path to the producer's repository>")
        return 2
    repo = pathlib.Path(sys.argv[1]).expanduser().resolve()
    spec = repo / "spec"
    if not spec.is_dir():
        print(f"{spec} is not a directory")
        return 1

    revision = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True, text=True,
    ).stdout.strip()
    branch = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "--abbrev-ref", "HEAD"],
        capture_output=True, text=True,
    ).stdout.strip()

    criteria: set[str] = set()
    for document in sorted(spec.rglob("*.md")):
        criteria |= set(
            re.findall(
                r"^\| ((?:FR|NFR|StR)-\d+-(?:AC|CON|VC)-\d+)",
                document.read_text(),
                re.M,
            )
        )

    # Named by the producer's remote, not by the directory it happens to sit
    # in: a worktree is called `corpus-fixes`, and pinning under that name
    # writes a second pin nothing reads.
    remote = subprocess.run(
        ["git", "-C", str(repo), "remote", "get-url", "origin"],
        capture_output=True, text=True,
    ).stdout.strip()
    name = remote.rstrip("/").removesuffix(".git").rsplit("/", 1)[-1] or repo.name
    pin = ROOT / "producers" / f"{name}.criteria.yaml"
    header = pin.read_text().split("producer:")[0] if pin.exists() else ""
    pin.write_text(
        header
        + f"producer: agent-ix/{name}\n"
        + f"revision: {revision}\n"
        + f"branch: {branch}\n"
        + "criteria:\n"
        + "".join(f"- {c}\n" for c in sorted(criteria))
    )
    print(f"pinned {len(criteria)} criteria from {name} at {revision[:12]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
