---
id: StR-001
title: "Comparable graph truth across producers"
type: StR
---

# StR-001 — Comparable graph truth across producers

## Stakeholder Need

An engineer deciding whether a code graph is trustworthy enough to drive
impact analysis, test selection or a release gate needs to know how good the
graph is, in numbers that mean the same thing next month and on somebody
else's producer.

## Rationale

Every producer in the ecosystem currently grades itself against its own
fixtures. Those fixtures are written by the same person who wrote the
resolver, in the same sitting, and they agree with it by construction. The
result is a set of green suites that cannot be compared to each other and
cannot say whether a graph is good enough for anything.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-001-VC-1 | Two producers scored at one corpus revision produce comparable confusion matrices | Test (TC-001) |
| StR-001-VC-2 | A change to expected truth changes the corpus revision, so scores at different revisions cannot be silently compared | Test (TC-002) |
| StR-001-VC-3 | A query a producer does not answer is reported as not-computed and stays out of every ratio | Test (TC-003) |
| StR-001-VC-4 | Every positive has a control whose only difference is the fact the producer needs | Test (TC-004) |

## Stakeholders

Producer authors (quire-code-rs, quire-rs), the Filament assurance programme
(agent-ix/filament-ide-rs#511), and anyone consuming a graph-derived decision.
