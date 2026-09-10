---
id: ADR-0001
title: "Rust corpus tool boundary"
type: ADR
status: accepted
---

# ADR-0001 — Rust corpus tool boundary

## Context

Before this decision, three Python commands jointly defined corpus completeness,
revision identity, and producer grading. Separate Python runners and inline
shell-embedded Python qualified that behavior. The source-language repositories
under `fixtures/**/input`, by contrast, are deliberately multilingual test data.

The historical score interface interpolates placeholders into one string and
passes it to a shell. That interface confuses producer arguments with shell
syntax and makes literal paths or values depend on quoting rules.

## Decision

One AGPL-3.0-or-later Rust package in this repository owns the corpus library and
`quire-corpus` CLI. It uses exact Rust 1.98.1. Bounds, digest, score,
run-relation, coverage validation, and criteria refresh are library operations;
the CLI is a thin adapter.

Producer invocation is an executable plus an ordered argument vector. The
placeholders `{org}`, `{repo}`, and `{input}` are substituted within individual
arguments, and the result is passed directly to the operating system without a
shell. The fixture input languages remain data and are not port targets.

The Python implementation is retained only long enough to run differential
tests against the Rust candidate, then removed in the same governed slice so
there is one behavioral owner.

## Consequences

- Existing shell-string examples change to an explicit executable and argument
  list; this is an intentional interface correction, not backward compatibility
  work.
- Rust tests own the corpus assertions and use `ix-trace-rs`.
- Make remains command orchestration only.
- Foreign-language fixture files and YAML/JSON/Markdown corpus data remain in
  their native data formats.
- No Quire profile grammar or formal-clause semantics are introduced here.
