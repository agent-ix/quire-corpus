# Rust port differential evidence

Observed locally on 2026-09-10 before removal of the Python baseline. Both
implementations read the same working-tree `corpus.yaml`, fixtures, and producer
pin. The candidate used exact Rust 1.98.1 with locked dependency resolution and
two Cargo build jobs.

- Bounds: byte-exact JSON. Both SHA-256 digests were
  `670b79de1518ac33c1628dc1e21fc3177731d4a45af3460c62346e5229923137`.
- Digest: byte-exact JSON. Both SHA-256 digests were
  `ef4a0fa5f2722c67d4291c00c2656b4cf1e121ebca7974d9c19e01932bfaa844`.
- Score: exact after excluding only `schema_version` and
  `producer_invocation`, the two fields intentionally changed by producer
  contract version 2. Both normalized SHA-256 digests were
  `2684d9154032764a6e0e22419a94c1b49b2171047da38e6257d6eeee81bfb4ce`.

The full committed corpus was scored. Finding text, finding order, tallies,
case populations, case digests, corpus revision, pending state, and the stale
pending result all agreed. The Python scorer's pre-JSON stale notice was
removed from stdout before normalization; the Rust CLI emits that notice on
stderr so JSON stdout remains parseable.

The baseline producer was the pre-port empty producer. It emitted the canonical
records-v1 empty graph, mentions, diagnostics, and unresolved-call census. The
Rust qualification suite now owns that helper and the continuing behavioral
oracles.
