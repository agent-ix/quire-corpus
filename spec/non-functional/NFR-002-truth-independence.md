---
id: NFR-002
title: "Truth independence"
type: NFR
quality_attribute: reliability
---

# NFR-002 — Truth independence

## Description

Expected truth SHALL be authored from the fixture and the contract, never
captured from a producer's output. A truth set derived from the tool it
grades cannot disagree with that tool, which is the one thing it exists to be
able to do.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-002-AC-1 | Every case has a control, or a recorded reason why its positive already contains one | Test (TC-030) |
| NFR-002-AC-2 | A change to expected truth is accompanied by the contract clause deciding it | Analysis (TC-031) |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Positives with no control and no recorded reason | 0 | 0 | Test |
| Expectations changed without a quoted contract clause | 0 | 0 | Inspection |
