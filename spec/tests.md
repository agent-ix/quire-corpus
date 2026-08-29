---
id: TM-001
title: "quire-corpus Test Matrix"
type: TestMatrix
---

# Test Matrix

## Overview

The corpus's own gates, and the guards that keep them able to fail. Every
test carries its `TC-NNN` and the criterion it discharges on the comma form,
so `quire coverage` binds both — an em dash between them terminates the id
run and backs only the first (agent-ix/quire-code-rs#8).

A corpus whose gates cannot fail measures nothing. Four of these tests exist
only to break the gate on purpose: TC-006 puts a case at the wrong depth,
TC-010 deletes a declared fixture, TC-011 adds an undeclared one, and TC-014
removes a control a positive names. Each asserts the specific message, not
just a non-zero exit, because a gate that fails for the wrong reason is a
gate that will pass for the wrong reason later.

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-CON-2 | TC-005, TC-006, TC-007, TC-008, TC-009 | ✅ |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5, FR-002-CON-1 | TC-010, TC-011, TC-012, TC-013, TC-014, TC-015, TC-040 | ✅ |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4 | TC-016, TC-017, TC-018, TC-019 | ✅ |
| FR-004 | FR-004-AC-5, FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4, FR-004-AC-6, FR-004-CON-1, FR-004-AC-7 | TC-003, TC-020, TC-021, TC-022, TC-023, TC-024, TC-025, TC-026, TC-042 | ✅ |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-CON-1, FR-005-AC-4, FR-005-AC-5, FR-005-AC-6, FR-005-AC-7 | TC-032, TC-033, TC-034, TC-035, TC-036, TC-037, TC-038 | ✅ |
| FR-006 | FR-006-AC-2 | TC-039 | ✅ |
| FR-007 | FR-007-AC-4 | TC-041 | ✅ |
| NFR-001 | NFR-001-AC-1, NFR-001-AC-2, NFR-001-AC-3 | TC-027, TC-028, TC-029 | ✅ |
| NFR-002 | NFR-002-AC-1, NFR-002-AC-2 | TC-004, TC-030, TC-031 | ✅ |
| StR-001 | StR-001-VC-1, StR-001-VC-2, StR-001-VC-3, StR-001-VC-4 | TC-001, TC-002, TC-003, TC-004, TC-024, TC-030 | ✅ |

---

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-001 | a run yields a confusion matrix at a stated revision | Unit | StR-001-VC-1 | ✅ |
| TC-002 | a change to expected truth changes the revision, so scores at two revisions cannot be silently compared | Unit | StR-001-VC-2 | ✅ |
| TC-003 | a query the producer does not answer is not-computed, and contributes no cell to any ratio | Unit | FR-004-AC-5, StR-001-VC-3 | ✅ |
| TC-004 | every positive has a control, or a recorded reason why its own fixture already contains one | Unit | StR-001-VC-4, NFR-002-AC-1 | ✅ |
| TC-005 | every case directory holds its three parts | Unit | FR-001-AC-1 | ✅ |
| TC-006 | a case at the wrong depth is an error, not a silent omission | Unit | FR-001-AC-2 | ✅ |
| TC-007 | a case with no issue_ref fails the gate | Unit | FR-001-AC-3 | ✅ |
| TC-008 | every expectation names the org and repo its fixture is extracted as | Unit | FR-001-AC-4 | ✅ |
| TC-009 | a reserved family declares its owning issue and is not silently populated | Unit | FR-001-CON-2 | ✅ |
| TC-010 | a declared cell with no fixture is a GAP and fails | Unit | FR-002-AC-1 | ✅ |
| TC-011 | an undeclared fixture fails rather than being counted | Unit | FR-002-AC-2 | ✅ |
| TC-012 | an exclusion with no reason fails the gate | Unit | FR-002-AC-3 | ✅ |
| TC-013 | coverage is derived, so a fixture flips its own cell | Unit | FR-002-AC-4 | ✅ |
| TC-014 | a positive naming a control that is gone fails | Unit | FR-002-AC-5 | ✅ |
| TC-015 | gap_count is a count, never a ratio | Unit | FR-002-CON-1 | ✅ |
| TC-016 | the invocation template substitutes org, repo, input | Unit | FR-003-AC-1 | ✅ |
| TC-017 | a non-zero exit is reported, never scored as zero | Unit | FR-003-AC-2 | ✅ |
| TC-018 | output that is not JSON is reported, never scored | Unit | FR-003-AC-3 | ✅ |
| TC-019 | the producer contract carries a version | Unit | FR-003-AC-4 | ✅ |
| TC-020 | the grammar's kind is censused, never graded | Unit | FR-004-AC-1 | ✅ |
| TC-021 | edges are scoped to the types the case names | Unit | FR-004-AC-2 | ✅ |
| TC-022 | a forbidden edge type emitted is a false positive | Unit | FR-004-AC-3 | ✅ |
| TC-023 | an empty partition is unavailable, never 1.0 | Unit | FR-004-AC-4 | ✅ |
| TC-024 | a query the producer does not answer is not-computed, and contributes no cell to any ratio | Unit | FR-004-AC-5, StR-001-VC-3 | ✅ |
| TC-025 | a tier disagreement is recorded and reported, never scored as a wrong edge | Unit | FR-004-AC-6, FR-004-CON-1 | ✅ |
| TC-026 | scoring pinned inputs is deterministic | Unit | FR-004-AC-7 | ✅ |
| TC-027 | two runs on an unchanged tree agree | Unit | NFR-001-AC-1 | ✅ |
| TC-028 | editing any fixture byte changes the revision | Unit | NFR-001-AC-2 | ✅ |
| TC-029 | moving a fixture changes the revision even though no byte changed | Unit | NFR-001-AC-3 | ✅ |
| TC-030 | every positive has a control, or a recorded reason why its own fixture already contains one | Unit | StR-001-VC-4, NFR-002-AC-1 | ✅ |
| TC-031 | the rule that a truth change carries the contract clause deciding it is written down where a reviewer will meet it | Unit | NFR-002-AC-2 | ✅ |
| TC-032 | a mention the case names and the producer omits is a false negative, not a silent pass | Unit | FR-005-AC-1 | ✅ |
| TC-033 | the kind is the claim, so the right identifier with the wrong kind is not the mention the case asked for | Unit | FR-005-AC-2 | ✅ |
| TC-034 | an unnamed mention is a false positive only where the case says its list is the whole list | Unit | FR-005-AC-3, FR-005-CON-1 | ✅ |
| TC-035 | diagnostic bounds are graded in both directions | Unit | FR-005-AC-4 | ✅ |
| TC-036 | a required code or path that nothing carries is a finding | Unit | FR-005-AC-5 | ✅ |
| TC-037 | two runs that disagree byte for byte are a finding | Unit | FR-005-AC-6 | ✅ |
| TC-038 | a forbidden substring in the raw payload is a finding | Unit | FR-005-AC-7 | ✅ |
| TC-039 | a declaration that moved keeps its identifier | Unit | FR-006-AC-2 | ✅ |
| TC-040 | two orgs share no names | Unit | FR-002-AC-4 | ✅ |
| TC-041 | an unrelated file leaves the others' records alone | Unit | FR-007-AC-4 | ✅ |
| TC-042 | an unknown relation is reported, never treated as held | Unit | FR-004-AC-7 | ✅ |

---

## Coverage Notes

- **Status legend**: ⬜ Planned (row authored, test not yet written),
  ✅ Complete (test written, tagged and green).
- `make test` runs the whole matrix. It needs no producer: the scorer's own
  behaviour is tested against `tests/stub_producer.py`, which finds nothing.
  A corpus whose tests need the tool it grades cannot test the grading.
- TC-031 is an `Analysis`-shaped assertion made testable: the rule that a
  truth change carries the contract clause deciding it lives in
  `CONTRIBUTING.md`, and the test asserts the rule is written down. Whether
  a *particular* change honoured it is a review judgement and stays one.
