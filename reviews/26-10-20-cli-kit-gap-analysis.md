---
id: SR-004
title: "Gap analysis for shared CLI kit adoption"
type: SpecReview
analysis: gap-analysis
scope: "quire-corpus repository at CLI kit adoption PR 13; spec criteria, computed matrix, Rust tests, src/ and tests/"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-corpus/spec/spec"
    type: references
---

# SR-004: CLI kit adoption gap analysis

## Summary

The computed Quoin matrix tags all seven FR-007 acceptance criteria, including the new CLI JSON, exit-class and version-agreement obligations. The review found three FR-006 acceptance criteria with stale test tags, which were corrected in this PR. No source or test placeholder was found in the Rust tree.

## Verdict

**PASS after remediation** for repository-wide acceptance-criterion traceability. Every acceptance criterion in the computed matrix now has a test binder. Runtime evidence is absent from the Quoin store, so this verdict concerns the computed static matrix rather than discharge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-006-AC-1, FR-006-AC-3 and FR-006-AC-4 name relation tests, but those tests carry different criterion tags, so the computed matrix cannot bind the intended relation requirements. | spec/functional/FR-006-relations-between-extractions.md:37; tests/corpus.rs:673; tests/corpus.rs:722; tests/corpus.rs:744 |

## Coverage

Plan completion: not assessed

After remediation, `quoin matrix --repo . --json` reports 67 rows: 55 tagged, 6 untagged measurement rows, and 6 `method-without-symbol`. The latter rows declare inspection or other methods that need not bind a test symbol. Every acceptance criterion is tagged, including FR-007-AC-1 through FR-007-AC-7. The changed `src/main.rs` behavior is owned by FR-007-AC-3, FR-007-AC-4 and FR-007-AC-7; the dependency manifest and denied-source allowance support that behavior. No `todo!`, `unimplemented!`, `TODO`, `FIXME` or `dbg!` placeholder was found under `src/` or `tests/`. Optional semantic review was skipped.

## Dispositions

- FND-001: fixed. TC-039 through TC-042 now carry FR-006-AC-1 through FR-006-AC-4 respectively, matching the behavior each test asserts; the computed matrix tags all four.
