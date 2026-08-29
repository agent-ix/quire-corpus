---
id: FR-005
title: "Grading beyond nodes and edges"
type: FR
---

# FR-005 — Grading beyond nodes and edges

## Behavior

The scorer SHALL grade mention kind, diagnostic bounds, run-to-run determinism,
and payload hygiene, in addition to nodes and edges.

WHEN a case declares its mention list exhaustive, the scorer SHALL score a
mention the case does not name as a false positive; otherwise it SHALL NOT.

WHEN a case declares itself deterministic, the scorer SHALL run the producer a
second time and compare the payloads byte for byte.

## Rationale

Each of these grades something a node-and-edge comparison structurally cannot
see. A mention's *kind* is the claim — the same identifier on a test and on
production code means "this discharges it" in one place and "this refers to it"
in the other, and a producer that types both the same way lets any file claim
coverage by writing a comment. A diagnostic is a claim about the input, and a
producer reporting none over a broken tree and one reporting one per package
import are both wrong in ways no edge set records. Determinism and payload
hygiene are invisible to every other check here, because every other check
compares against a *set*, and a set does not care what order it arrived in or
what absolute path was serialized beside it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | A mention the case names and the producer does not emit is a false negative | Test (TC-032) |
| FR-005-AC-2 | A mention emitted with the wrong kind is not counted as present | Test (TC-033) |
| FR-005-AC-3 | An unnamed mention is a false positive only where the case declares its list exhaustive | Test (TC-034) |
| FR-005-AC-4 | A diagnostic count below `min` or above `max` is a finding | Test (TC-035) |
| FR-005-AC-5 | A required diagnostic code or path that no diagnostic carries is a finding | Test (TC-036) |
| FR-005-AC-6 | A second producer run whose payload differs byte for byte is a finding | Test (TC-037) |
| FR-005-AC-7 | A forbidden payload substring appearing in the raw output is a finding | Test (TC-038) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-005-CON-1 | The scorer SHALL NOT infer that an unnamed mention is wrong; a case grades what it declares and no more | Interface | Test (TC-034) |
