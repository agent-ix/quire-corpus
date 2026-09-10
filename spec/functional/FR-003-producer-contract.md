---
id: FR-003
title: "Producer invocation contract"
type: FR
---

# FR-003 — Producer invocation contract

## Description

The scorer invokes an arbitrary producer through a producer-neutral process
contract and grades only the canonical JSON records written to standard output.

## Behavior

The corpus scorer SHALL invoke a producer as an executable plus an ordered
argument vector. The corpus scorer SHALL substitute `{org}`, `{repo}`, and
`{input}` placeholders within individual arguments. The corpus scorer SHALL
write no command through a shell. The producer SHALL write canonical records
as JSON on stdout.

The producer contract SHALL carry version 2. Each score observation SHALL
record the executable and ordered argument templates as separate JSON values.
An observation carrying the version-1 shell-string invocation is not compatible
with a version-2 observation.

WHEN a producer exits non-zero or writes output that is not JSON, the corpus
scorer SHALL report the failure without scoring the case.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | The ordered invocation arguments substitute org, repo and input directory without shell evaluation | Test (TC-016, TC-058) |
| FR-003-AC-2 | A non-zero exit is reported with the producer's stderr, not scored | Test (TC-017) |
| FR-003-AC-3 | Output that is not JSON is reported, not scored | Test (TC-018) |
| FR-003-AC-4 | The contract carries a version, so a change to it invalidates prior observations | Test (TC-019) |
| FR-003-AC-5 | Contract version 2 records a structured executable and ordered argument templates and is not presented as compatible with a version-1 shell-string observation | Test (TC-061) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-003-CON-1 | The corpus SHALL grade output only; no producer-specific knowledge belongs in the scorer | Interface | Inspection |
| FR-003-CON-2 | The corpus SHALL NOT invoke a shell to execute a producer | Security | Test (TC-057) |

## Dependencies

- **Upstream**: `corpus.yaml` declares producer contract version 2.
- **Downstream**: FR-004 grades only observations admitted by this contract.
