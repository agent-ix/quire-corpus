---
id: SR-002
title: "Rust review of the corpus-tool port"
type: SpecReview
analysis: code-review
scope: "quire-corpus issue 2 implementation against StR-002, FR-003, FR-007, and NFR-003"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-corpus/spec/stakeholder/StR-002"
    type: reviews
  - target: "ix://agent-ix/quire-corpus/spec/functional/FR-003"
    type: reviews
  - target: "ix://agent-ix/quire-corpus/spec/functional/FR-007"
    type: reviews
  - target: "ix://agent-ix/quire-corpus/spec/non-functional/NFR-003"
    type: reviews
---

## Summary

The implementation review followed `agent-skills/rust-review/SKILL.md` and is
**PASS after remediation**. The port preserves the committed corpus oracle,
uses typed Rust library and CLI seams, invokes producers without a shell, keeps
fixture source trees opaque, and removes the superseded executable Python and
shell tooling. There are no open findings.

The scorer's per-case census and derived-query projections intentionally remain
schema-flexible JSON values because fixture families carry different authored
projections. The score observation itself is a typed `ScoreReport`; required
top-level fields cannot disappear through branch-local JSON construction.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-006 | high | **Closed.** Chained placeholder replacements reprocessed substituted values. An org literally equal to `{repo}` became the repository value, so the recorded invocation did not represent the requested producer input. One regex pass now substitutes only placeholders present in the original template; TC-058 proves placeholder-looking replacement values arrive literally. | `src/producer.rs:33`; TC-058 |
| FND-007 | high | **Closed.** The first draft computed revision identity only after executing producers. A producer that changed a fixture could be scored against one tree and reported under another tree's identity. Revision and per-case digests are now captured before execution and the revision is rechecked before report construction; TC-052 mutates an input through the real process boundary and proves refusal. | `src/score.rs:875`; TC-052 |
| FND-008 | medium | **Closed.** `Path::is_file` followed symlinks despite a non-following walk. A fixture or spec symlink could pull bytes or criteria from outside the selected repository root. Both walks now accept only the walk entry's own regular-file type; TC-049 and TC-053 prove external targets do not enter either result. | `src/digest.rs:23`; `src/refresh.rs:39`; TC-049; TC-053 |
| FND-009 | medium | **Closed.** `usize as u64` and `u64 as usize` narrowed evidence and diagnostic counts on non-64-bit targets, allowing authored bounds to compare against truncated values. Widening now uses checked conversion with fail-closed saturation and comparisons remain in `u64`. | `src/score.rs:535`; `src/score.rs:714` |
| FND-010 | medium | **Closed.** The score report was assembled and exposed as an untyped `serde_json::Value`, so a required top-level field could be omitted without compiler involvement. `ScoreReport` now owns the version, identities, count, confusion matrix, and cases as typed required fields. | `src/score.rs:836`; FR-007-AC-1; TC-047 |
| FND-011 | medium | **Closed.** The first license gate omitted the package's own AGPL-3.0-or-later license and therefore failed even though dependency licenses were compatible. The exact project license is now admitted, unused allowances are removed, and all four `cargo deny` lanes pass. | `deny.toml:8`; NFR-003-AC-6; TC-060 |
| FND-012 | low | **Closed.** Five nested conditional shapes failed the strict Clippy gate and obscured fail-closed predicates. The predicates were collapsed without allowances; strict all-target/all-feature Clippy passes. | `src/bounds.rs:271`; `src/score.rs:547` |

## Gate evidence

| Gate | Result |
|---|---|
| `cargo +1.98.1 fmt --all -- --check` | PASS |
| `cargo +1.98.1 clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS |
| `cargo +1.98.1 test --locked -- --test-threads=2` | PASS — 60 functions, all 61 Test Matrix identifiers bound, 0 failed |
| `RUSTDOCFLAGS='-D warnings' cargo +1.98.1 doc --locked --no-deps` | PASS |
| `cargo deny check` | PASS — advisories, bans, licenses, sources |
| `cargo audit` | PASS — 64 locked dependencies scanned, no vulnerabilities reported |
| `make verify` | PASS — 114 covered, 0 gaps, 0 unreached producer criteria |
| Fixture-input diff | PASS — no path under `fixtures/**/input` changed |
| Executable-language residue | PASS — no `.py` or `.sh` remains outside opaque fixture inputs |
| Hosted CI | Not run; no workflow was added or changed |

## Scope disposition

This review does not authorize source-language parsing, Quire profile grammar,
formal-clause semantics, extraction redesign, cross-repository CI, or a central
consumer registry. It covers only the issue-2 Rust port in `quire-corpus`.
