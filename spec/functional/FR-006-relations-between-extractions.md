---
id: FR-006
title: "Relations between two extractions"
type: FR
---

# FR-006 — Relations between two extractions

## Behavior

A case MAY declare a `relation` that must hold between two extractions rather
than a fact about one. The second extraction reads the case's `variant/` tree
where one exists, and otherwise the same `input/` tree under the case's
`variant_org`.

WHEN a case declares a relation the scorer does not implement, the scorer SHALL
report it and SHALL NOT treat the relation as held.

## Rationale

A fixture is otherwise one expectation about one run, which leaves every
invariant stated as a *relation between two runs* structurally unreachable —
and those are the invariants a consumer depends on most.

A node that moves down a file must keep its identifier, or every reformat
becomes a delete plus an add and the consumer drops every edge that pointed at
it. Two organisations must share no names, or two repositories of the same name
collide in one graph. Adding a private helper must leave the existing records
alone, or the export-set optimisation that skips dependent re-resolution is
unsafe. None of these can be observed from a single extraction: each one looks
correct in isolation and is wrong only in comparison.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | `ids_preserved` fails when a node present in both runs changes identifier | Test (TC-039) |
| FR-006-AC-2 | `disjoint_names` fails when any name appears under both orgs | Test (TC-040) |
| FR-006-AC-3 | `identical_except` ignores only the records the case names, and fails on any other difference | Test (TC-041) |
| FR-006-AC-4 | A relation the scorer does not implement is reported, never treated as held | Test (TC-042) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-006-CON-1 | A relation SHALL be evaluated by running the producer a second time, never by transforming the first run's output | Interface | Inspection |
