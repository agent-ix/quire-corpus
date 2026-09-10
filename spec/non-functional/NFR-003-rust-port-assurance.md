---
id: NFR-003
title: "Rust port assurance"
type: NFR
quality_attribute: security
---

# NFR-003 — Rust port assurance

## Statement

The corpus tool SHALL build with exact Rust 1.98.1. The producer invocation
layer SHALL use a structured executable-and-arguments interface. The producer
invocation layer SHALL pass shell metacharacters, whitespace, and
placeholder-looking values literally. The producer invocation layer SHALL NOT
evaluate producer arguments with a shell.

The port SHALL preserve the current corpus oracle and digest identity. The
differential baseline SHALL compare bounds and digest observations exactly. The
differential baseline SHALL compare score observations after excluding only the
intentionally changed producer-contract version and structured
invocation-identity fields. Every other difference is a port defect.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-003-AC-1 | `Cargo.toml` and `rust-toolchain.toml` select exact Rust 1.98.1 and local build commands use locked dependency resolution | Static mutation test (TC-056) |
| NFR-003-AC-2 | Producer execution invokes an executable with an argument vector and never invokes a shell | Unit + process trace (TC-057) |
| NFR-003-AC-3 | Spaces, shell metacharacters, and placeholder-looking text arrive at the producer as literal arguments | Integration test (TC-058) |
| NFR-003-AC-4 | The Rust candidate and frozen Python baseline produce exact bounds and digest observations and score observations differing only in the versioned invocation-identity fields; error and metamorphic-relation behavior also agrees over the committed corpus before baseline removal | Differential test (TC-048) |
| NFR-003-AC-5 | Rust tests use canonical `ix-trace-rs` markers and bind every implemented Test Matrix row | Static + traceability test (TC-059) |
| NFR-003-AC-6 | Dependency licenses are compatible with AGPL-3.0-or-later and advisories are checked from the locked graph | Dependency audit (TC-060) |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Shell processes used for producer execution | 0 | 0 | Process trace |
| Changed fixture input files in the port | 0 | 0 | Source diff inspection |
| Existing corpus test rows without a Rust trace binding | 0 | 0 | Traceability audit |

## Verification

The repository SHALL verify the port with `ix-trace-rs`-marked Rust unit,
integration, static-mutation, differential, and process-trace tests. Local
qualification SHALL use exact Rust 1.98.1, locked dependency resolution, and
`CARGO_BUILD_JOBS=2`; it SHALL NOT dispatch hosted CI.
