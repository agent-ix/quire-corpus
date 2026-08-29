#!/usr/bin/env python3
"""Run every module-level `test_*` in `tests/`, with the stdlib alone.

Not unittest and not pytest. Two reasons: NFR-001 says a clean runner
reproduces this corpus from the tree, and a suite that needs an installed
framework is not that; and the ecosystem's Python scanner binds a trace tag
to a module-level `def`, so a `TestCase` method carries its tag and backs
nothing.
"""
from __future__ import annotations

import importlib.util
import pathlib
import sys
import traceback

HERE = pathlib.Path(__file__).resolve().parent


def load(path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(path.stem, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    selected = sys.argv[1:]
    passed, failed = 0, []
    for path in sorted(HERE.glob("test_*.py")):
        module = load(path)
        for name in sorted(vars(module)):
            if not name.startswith("test_"):
                continue
            if selected and not any(s in name for s in selected):
                continue
            try:
                getattr(module, name)()
            except Exception:  # noqa: BLE001 - a failure is the result here
                failed.append((f"{path.stem}.{name}", traceback.format_exc()))
                print(f"FAIL {name}")
            else:
                passed += 1
                print(f"ok   {name}")
    for name, tb in failed:
        print(f"\n=== {name}\n{tb}")
    print(f"\n{passed} passed, {len(failed)} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
