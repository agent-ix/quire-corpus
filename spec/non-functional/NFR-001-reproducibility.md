---
id: NFR-001
title: "Reproducibility"
type: NFR
quality_attribute: reliability
---

# NFR-001 — Reproducibility

## Description

A clean runner SHALL reproduce the corpus revision and every per-case digest
from the tree alone, with no stored state and no network.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-001-AC-1 | Two runs on an unchanged tree produce identical revisions and digests | Test (TC-027) |
| NFR-001-AC-2 | Editing any fixture byte changes the corpus revision | Test (TC-028) |
| NFR-001-AC-3 | Moving a fixture between languages changes the corpus revision even though no byte changed | Test (TC-029) |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Revisions equal across repeated runs on an unchanged tree | all equal | all equal | Test |
| Stored state files required to compute a revision | 0 | 0 | architecture-conformance |
