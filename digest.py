#!/usr/bin/env python3
"""Derive the corpus revision and the expected-result digests.

Both are computed from the tree, never stored in it. A recorded digest is a
claim that goes stale the moment somebody edits a fixture and forgets the
file that records it; a derived one cannot disagree with what it describes.

    python3 digest.py            # the corpus revision and per-case digests
    python3 digest.py --json

A consumer records the revision it scored against. Changing expected truth
changes the revision, which is exactly the reviewable event CONTRIBUTING
describes: a producer's result is only comparable to another result at the
same revision.
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent
FIXTURES = ROOT / "fixtures"
MANIFEST = ROOT / "corpus.yaml"


def _digest_paths(paths: list[pathlib.Path]) -> str:
    """Content digest over a sorted path/content stream.

    Paths are included, not just contents: moving a fixture from `rust/` to
    `python/` changes what the corpus measures without changing a byte of
    any file, and a content-only digest would call that the same corpus.
    """
    h = hashlib.blake2b(digest_size=32)
    for path in sorted(paths):
        h.update(str(path.relative_to(ROOT)).encode())
        h.update(b"\0")
        h.update(path.read_bytes())
        h.update(b"\0")
    return h.hexdigest()


def case_digests() -> dict[str, str]:
    out = {}
    for case_file in sorted(FIXTURES.rglob("case.yaml")):
        case_dir = case_file.parent
        key = str(case_dir.relative_to(FIXTURES))
        out[key] = _digest_paths([p for p in sorted(case_dir.rglob("*")) if p.is_file()])
    return out


def corpus_revision() -> str:
    files = [MANIFEST] + [p for p in sorted(FIXTURES.rglob("*")) if p.is_file()]
    return _digest_paths(files)


def main() -> int:
    report = {
        "schema_version": 1,
        "corpus_revision": corpus_revision(),
        "cases": case_digests(),
    }
    if "--json" in sys.argv:
        print(json.dumps(report, indent=1, sort_keys=True))
    else:
        print(f"corpus_revision  {report['corpus_revision']}")
        print(f"cases            {len(report['cases'])}")
        for key, digest in report["cases"].items():
            print(f"  {digest[:16]}  {key}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
