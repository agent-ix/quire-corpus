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

## Criterion coverage

The inventory says which cases exist. It cannot say whether they reach what the
producer promises, and "is the corpus exhaustive?" was an opinion until it did.

Each case names the criteria it asserts, in `criteria:`. The producer's own
criteria are pinned under `producers/`, and `bounds.py` derives three sets from
the two: **reached**, **unreachable with a recorded reason**, and **unreached**.
The last fails the gate — so a criterion added upstream cannot arrive quietly,
and neither can a case that claims one nobody states.

The pin is a committed snapshot rather than a live read of a sibling checkout,
because a number computed against whatever is on somebody's disk is not
reproducible and NFR-001 says a clean runner reproduces this corpus from its own
tree. Moving the pin is the reviewable event: it is where a new criterion
appears and where the corpus is obliged to notice it is unreached.

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-6 | A criterion the pin states that no case claims and no reason excuses fails the gate | Test (TC-043) |
| FR-002-AC-7 | A criterion a case claims that the pin does not state fails the gate | Test (TC-044) |
| FR-002-AC-8 | A criterion declared unreachable that the pin no longer states fails the gate | Test (TC-045) |
| FR-002-AC-9 | An unreachable declaration carries the reason a corpus of this shape cannot reach it | Test (TC-046) |
