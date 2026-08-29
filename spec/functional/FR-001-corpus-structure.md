---
id: FR-001
title: "Corpus structure and fixture ownership"
type: FR
---

# FR-001 — Corpus structure and fixture ownership

## Behavior

The corpus SHALL store every case as static files under
`fixtures/<family>/<case>/<language>/`, holding `case.yaml`, an `input/`
tree, and `expected.yaml`.

WHEN a case is read, the corpus SHALL require no generation step, no harness
and no network.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | Every case directory holds `case.yaml`, `input/` and `expected.yaml` | Test (TC-005) |
| FR-001-AC-2 | A case directory that is not at the four-segment path is an error, not a silent omission | Test (TC-006) |
| FR-001-AC-3 | Every `case.yaml` names the issue the case exists for | Test (TC-007) |
| FR-001-AC-4 | A fixture's `input/` tree is a repository, and the expectation names the org and repo it is extracted as | Test (TC-008) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-001-CON-1 | A producer's own unit fixtures SHALL stay in the producer's repository; this corpus holds integrated graphs only | Interface | Inspection |
| FR-001-CON-2 | Reserved families SHALL be declared with their owning issue before any fixture is added to them | Maintainability | Test (TC-009) |
