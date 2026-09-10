---
id: StR-002
title: "Rust-native corpus tooling"
type: StR
---

# StR-002 — Rust-native corpus tooling

## Stakeholder Need

An engineer qualifying a graph producer needs one repository-owned tool whose
implementation, command surface, tests, and dependency evidence are Rust, while
the authored fixture repositories remain unchanged source-language evidence.

## Rationale

The corpus is intentionally multilingual data. Its bounds, revision identity,
producer execution, and scoring rules are executable assurance behavior rather
than data, and therefore need one reviewable implementation boundary. Keeping
those rules together also prevents two command implementations from disagreeing
about what corpus revision or score was observed.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-002-VC-1 | One Rust package owns the bounds, digest, producer-execution, scoring, coverage-validation, and producer-pin behavior | Inspection (TC-047) |
| StR-002-VC-2 | Removing the superseded Python and inline-Python implementations does not change the corpus truth, revision, or score semantics | Differential test (TC-048) |
| StR-002-VC-3 | Source files under `fixtures/**/input` remain authored fixture data and are neither translated nor executed by the corpus tool | Inspection (TC-049) |

## Stakeholders

Corpus maintainers, graph-producer authors, and downstream assurance programs
that compare observations at a named corpus revision.
