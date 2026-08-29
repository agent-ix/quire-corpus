# Baselines

One file per producer, holding the full scorer payload plus the provenance that
makes it re-runnable: the producer's exact revision, the contract version it was
invoked under, and the `corpus_revision` it was scored against.

A baseline is only comparable to another baseline at the **same**
`corpus_revision`. Changing an expectation changes that revision on purpose, so
a re-score after a truth change is a new measurement rather than a movement in
an old one.

Retained, never replaced. A later run lands beside its predecessor; overwriting
one would delete the comparison it exists for.
