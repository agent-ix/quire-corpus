---
id: SR-001
title: "Base review of the Rust corpus-tool port specification"
type: SpecReview
analysis: base
scope: "StR-002; FR-003 amendment; FR-007; NFR-003; ADR-0001; TM-001 rows TC-047 through TC-061"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-corpus/spec/stakeholder/StR-002"
    type: reviews
  - target: "ix://agent-ix/quire-corpus/spec/functional/FR-003"
    type: reviews
  - target: "ix://agent-ix/quire-corpus/spec/functional/FR-007"
    type: reviews
  - target: "ix://agent-ix/quire-corpus/spec/non-functional/NFR-003"
    type: reviews
---

## Summary

The owner-selected base review examined the #63 `quire-corpus` slice for
correctness, completeness, consistency, testability, traceability, and scope.
The reviewed specification is **PASS after remediation**. It preserves corpus
truth, digest identity, scoring meaning, adverse cases, and source-language
fixture data while correcting the unsafe shell-string producer boundary through
an explicit version-2 contract.

The review applies the 2026-09-09 native-language ruling: this tool handles a
generic source-graph corpus and does not introduce a formal-clause profile,
native Quire grammar, or new source semantics. The fixture repositories remain
opaque producer inputs.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | **Closed.** The first draft removed shell interpretation without changing the producer contract version or defining the observation identity. FR-003 now requires version 2, a structured executable/argument identity, and explicit incompatibility with version-1 observations. | FR-003-AC-4/5; TC-019; TC-061 | wrong-requirement |
| FND-002 | medium | **Closed.** “Equivalent score observations” could hide unrelated port drift behind the intentional invocation change. NFR-003 now permits differences only in the contract-version and invocation-identity fields; bounds and digest observations remain exact. | NFR-003-AC-4; TC-048 | wrong-requirement |
| FND-003 | medium | **Closed.** The first draft required a coverage validator but did not allocate how the CLI obtains the Quire report or how partial output is handled. FR-007 now requires a direct, structured Quire invocation and refusal on process failure or malformed JSON before assertions run. | FR-007-AC-6; TC-054 | missing-requirement |
| FND-004 | medium | **Closed.** The historical Test Matrix cited a nonexistent FR-007 and mapped TC-039 through TC-042 to the wrong FR-006 criteria. The matrix now maps all four relation tests to the actual FR-006 acceptance criteria before assigning FR-007 to the port surface. | TM-001 functional coverage; FR-006-AC-1..4 | correct-requirement-no-evidence |
| FND-005 | low | **Closed.** The first draft did not distinguish authored foreign-language fixture data from executable test helpers strongly enough. StR-002, FR-007-CON-1, ADR-0001, and TC-049 now preserve the former and require replacement of the latter. | StR-002-VC-3; FR-007-CON-1; ADR-0001; TC-049 | missing-requirement |

## Base Checklist Result

| Check | Result | Evidence |
|---|---|---|
| Stakeholder intent | PASS | StR-002 allocates executable corpus behavior to Rust without rewriting the multilingual truth corpus. |
| Scope | PASS | Bounds, digest, scoring, producer execution, coverage validation, criteria refresh, and their tests are in scope; extraction, fixture translation, formal clauses, and profile grammar are out. |
| Correctness and consistency | PASS after FND-001/FND-002/FND-004 | FR-003 version 2 isolates the one intentional interface correction; the existing FR-001..006 oracle remains authoritative. |
| Completeness | PASS after FND-003/FND-005 | Human/JSON modes, failures, process boundaries, parity, source provenance, licenses, and fixture-data containment are specified. |
| Testability | PASS | TC-047..061 state observable compile, differential, mutation, integration, static, dependency, and process-trace oracles. |
| Traceability | PASS | Every new criterion maps to at least one planned Test Matrix row; rows remain planned until Rust evidence exists. |
| Native-language redesign containment | PASS | The slice consumes source trees opaquely and defines no Quire clause syntax, semantics, temporal model, or profile. |

## Validation Evidence

The current reviewed `quire-cli` 0.32.0 binary (exact Rust implementation at
`efae1b5d`) structurally validated each changed ISO artifact against the exact
installed `spec-artifacts-iso` module and validated ADR-0001, TM-001, and this
review against `spec-artifacts-process`. No document-level error or grammar
finding remained. The module manifests emit pre-existing registry diagnostics
(`part_of` inverse duplication in ISO and same-name archetype/edge-registry
diagnostics in process); those are LR07 catalog findings and not defects in
these documents.

Implementation may proceed against this reviewed specification. Any change to
the corpus oracle, digest identity, structured invocation contract, or fixture
classification reopens `/specify` and `/spec-review`.
