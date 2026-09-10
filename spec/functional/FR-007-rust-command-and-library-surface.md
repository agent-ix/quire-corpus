---
id: FR-007
title: "Rust command and library surface"
type: FR
---

# FR-007 — Rust command and library surface

## Description

The corpus exposes one typed Rust implementation through a library and a thin
command-line adapter, so every command derives its result from the same rules.

## Behavior

The repository SHALL provide a Rust library and a `quire-corpus` CLI. The
`quire-corpus` CLI SHALL expose `bounds`, `digest`, `score`, `refresh-criteria`, and
`check-coverage` subcommands over typed library operations.

The repository root SHALL be an explicit input to the library and CLI. Each
command SHALL derive `corpus.yaml`, `fixtures/`, `producers/`, and `spec/` from
that root. Each command SHALL operate independently of the caller's current
directory.

`bounds`, `digest`, and `score` SHALL retain both human-readable and JSON output
modes. A failure SHALL return a non-zero status and identify the rejected
artifact or producer observation without reporting a successful score.

WHEN `check-coverage` runs, the CLI SHALL invoke the selected `quire` executable
directly with `coverage --scope <root> --json` and the selected module path.
WHEN Quire fails or writes malformed JSON, the CLI SHALL report that failure
without applying coverage assertions to a partial observation.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | The library exposes typed bounds, digest, producer-execution, scoring, run-relation, coverage-validation, and producer-pin operations | Compile + inspection (TC-047) |
| FR-007-AC-2 | The CLI exposes all five required subcommands and derives repository loci from an explicit root | Integration test (TC-050) |
| FR-007-AC-3 | Bounds, digest, and score emit deterministic JSON output in addition to human-readable output | Integration test (TC-051) |
| FR-007-AC-4 | Errors are non-zero, name their failing context, and never emit a success-shaped observation | Integration test (TC-052) |
| FR-007-AC-5 | Criteria refresh writes only `producers/<remote-repository-name>.criteria.yaml`, using the producer's exact revision and deterministically sorted criteria | Integration test (TC-053) |
| FR-007-AC-6 | Coverage validation rejects empty reconciliation, status lies, unexplained unbacked rows, and the five named hollow-declaration diagnostics; the CLI obtains the report by directly invoking the selected Quire executable | Unit + integration test (TC-054) |

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-007-CON-1 | The Rust package SHALL NOT parse, translate, execute, or rewrite source files under `fixtures/**/input` except to pass the selected input path to the producer and include bytes in the revision digest | Boundary | Inspection (TC-049) |
| FR-007-CON-2 | Repository Make targets SHALL contain only orchestration and no inline implementation of corpus assertions, scoring, digesting, or report interpretation | Boundary | Static test (TC-055) |

## Dependencies

- **Upstream**: FR-001 through FR-006 define the behavior exposed by the Rust
  boundary; ADR-0001 allocates ownership.
- **Downstream**: producer repositories may invoke the CLI but do not own or
  restate its scoring rules.
