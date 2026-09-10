---
id: SPEC-001
title: "quire-corpus master requirements"
type: MasterRequirements
---

# quire-corpus

A versioned integration corpus spanning specifications, code, tests and
expected graph behaviour, shared by every producer in the ecosystem.

## Scope

In scope: fixture repositories, hand-authored expected truth, the inventory
and its GAP gate, the corpus revision and per-case digests, the producer
invocation contract, and the deterministic scorer.

Out of scope: extraction itself, traversal itself, and any producer's unit
fixtures. A producer's own fixtures stay in the producer's repository; this
corpus grades integrated graphs.

Deliberately not here: detection and minting over authored markdown, which is
`agent-ix/qa-corpus` and has its own contract (quire-rs FR-065).

## Requirements

| ID | Title |
|---|---|
| [StR-001](stakeholder/StR-001-comparable-graph-truth.md) | Comparable graph truth across producers |
| [StR-002](stakeholder/StR-002-rust-native-corpus-tooling.md) | Rust-native corpus tooling |
| [FR-001](functional/FR-001-corpus-structure.md) | Corpus structure and fixture ownership |
| [FR-002](functional/FR-002-inventory-and-bounds.md) | Inventory, controls, and the GAP gate |
| [FR-003](functional/FR-003-producer-contract.md) | Producer invocation contract |
| [FR-004](functional/FR-004-scoring-contract.md) | Deterministic scoring contract |
| [FR-005](functional/FR-005-grading-beyond-nodes-and-edges.md) | Grading beyond nodes and edges |
| [FR-006](functional/FR-006-relations-between-extractions.md) | Relations between two extractions |
| [FR-007](functional/FR-007-rust-command-and-library-surface.md) | Rust command and library surface |
| [NFR-001](non-functional/NFR-001-reproducibility.md) | Reproducibility |
| [NFR-002](non-functional/NFR-002-truth-independence.md) | Truth independence |
| [NFR-003](non-functional/NFR-003-rust-port-assurance.md) | Rust port assurance |

Decision record: [ADR-0001](assets/adr/0001-rust-corpus-tool-boundary.md).

Test matrix: [tests.md](tests.md).
