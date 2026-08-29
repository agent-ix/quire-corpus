---
id: FR-002
title: "Inventory, controls, and the GAP gate"
type: FR
---

# FR-002 — Inventory, controls, and the GAP gate

## Behavior

`corpus.yaml` SHALL declare, for each case, the languages it must exist in.
The coverage state SHALL be derived from the filesystem on every run and
never stored.

WHEN a declared cell has no fixture, the corpus SHALL report a GAP and fail.

WHEN a fixture exists that the inventory does not declare, the corpus SHALL
fail rather than count it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | A declared cell with no fixture is a GAP and fails the gate | Test (TC-010) |
| FR-002-AC-2 | An undeclared fixture fails the gate | Test (TC-011) |
| FR-002-AC-3 | An `out_of_scope` entry without a reason fails the gate | Test (TC-012) |
| FR-002-AC-4 | Adding a fixture flips its own cell with no edit to `corpus.yaml` beyond its declaration | Test (TC-013) |
| FR-002-AC-5 | A positive naming a control that does not exist fails the gate | Test (TC-014) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-002-CON-1 | `gap_count` SHALL be a count, never a ratio: a ratio falls as easy cases are added while the hard missing case stays missing | Maintainability | Test (TC-015) |
