---
id: FR-003
title: "Producer invocation contract"
type: FR
---

# FR-003 — Producer invocation contract

## Behavior

A producer SHALL be invoked as `<producer> --org <org> --repo <repo>
<input-dir>` and SHALL write canonical records as JSON on stdout.

WHEN a producer exits non-zero or writes output that is not JSON, the corpus
SHALL report the failure and SHALL NOT score the case as zero.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | The invocation template substitutes org, repo and input directory | Test (TC-016) |
| FR-003-AC-2 | A non-zero exit is reported with the producer's stderr, not scored | Test (TC-017) |
| FR-003-AC-3 | Output that is not JSON is reported, not scored | Test (TC-018) |
| FR-003-AC-4 | The contract carries a version, so a change to it invalidates prior observations | Test (TC-019) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-003-CON-1 | The corpus SHALL grade output only; no producer-specific knowledge belongs in the scorer | Interface | Inspection |
