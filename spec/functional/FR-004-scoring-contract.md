---
id: FR-004
title: "Deterministic scoring contract"
type: FR
---

# FR-004 — Deterministic scoring contract

## Behavior

The scorer SHALL compute true positives, false positives and false negatives
for nodes, edges, visibility and signatures, sliced by language, object type,
relation kind and resolution tier.

WHEN a partition has no population, the scorer SHALL report its precision and
recall as unavailable rather than as 1.0 or 0.0.

WHEN a producer emits no answer for a derived query the case declares, the
scorer SHALL report `not-computed` and SHALL exclude it from every ratio.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-004-AC-1 | Nodes are graded on `(object_type, name)` and the grammar's `kind` is censused, not graded | Test (TC-020) |
| FR-004-AC-2 | Edges are graded as an expected-present set scoped to the edge types the case names | Test (TC-021) |
| FR-004-AC-3 | A forbidden edge type, edge or node kind emitted is a false positive | Test (TC-022) |
| FR-004-AC-4 | An empty partition reports unavailable precision and recall, never 1.0 | Test (TC-023) |
| FR-004-AC-5 | A derived query the producer does not answer is `not-computed` and out of every ratio | Test (TC-024) |
| FR-004-AC-6 | A resolution-tier disagreement is recorded and reported, never scored as a wrong edge | Test (TC-025) |
| FR-004-AC-7 | Repeated scoring of pinned inputs yields byte-identical output | Test (TC-026) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-004-CON-1 | The scorer SHALL NOT infer a rank order between resolution tiers that no contract states | Interface | Test (TC-025) |
