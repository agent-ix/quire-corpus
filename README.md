# quire-corpus

The shared **complete-graph** truth set for the Filament/Quire ecosystem.
Static files, read in place, producer-agnostic. Contract: [`spec/`](spec/).

## Why this repository exists

[`agent-ix/qa-corpus`](https://github.com/agent-ix/qa-corpus) grades *detection
and minting* over authored markdown: one defect per case, one file at a time.
Nothing graded the other half — whether the **graph** a producer builds from
source is the graph the source describes.

That half fails differently. Its defects live *between* files: a method that
resolves in one file and vanishes in a batch, a name declared twice, an
import that resolves by luck, a trait method that never becomes a node at
all. A snippet cannot exhibit any of them, so a case here is a small
repository, not a fragment.

It is also where a producer is most tempted to guess. A resolver that binds
every call by method name scores perfectly on a corpus of positives and is
wrong about half of a real repository. **Every positive here has a
byte-similar control** whose only difference is whether the fact the producer
needs is actually written down.

## Using it

```bash
python3 bounds.py                  # inventory vs the tree; the GAP gate
python3 digest.py                  # corpus revision + per-case digests
python3 score.py --producer '<cmd> --org {org} --repo {repo} {input}'
make verify                        # all three
```

A producer reads one fixture's `input/` tree and writes canonical records on
stdout. Nothing else is read and no network is available. The corpus grades
the output, never the producer, so two producers scored at the same
`corpus_revision` are comparable.

## What it grades, and what it refuses to

| Graded | How |
|---|---|
| Nodes | `(object_type, name)`, exhaustive where a case declares them |
| Edges | expected-present, scoped to the edge types the case names |
| Absences | `forbidden_edge_types`, `forbidden_edges`, `forbidden_node_kinds` |
| Visibility and signature | per named declaration |
| Censuses | ambiguous call sites, diagnostics |

It refuses two things on purpose.

**`kind` is not graded.** It is the grammar's node kind and therefore
language-specific: TypeScript reports a class member as `method`, Rust
reports the same declaration as `function`, and no cross-language vocabulary
is declared anywhere. Grading it would report a vocabulary difference as a
correctness defect and bury the real recall gaps underneath. Every observed
kind is censused instead.

**An unsupported query is `not-computed`, never zero.** A plain extractor
emits no paths, no impact closure, no test selection and no resolved
cross-layer links, so those grade as absent and stay out of every ratio. A
zero and an absence are different claims, and collapsing them is how a
measurement programme ends up trusting a number nobody took.

## Layout

```
corpus.yaml                                  intent: languages, families, inventory
fixtures/<family>/<case>/<language>/
    case.yaml                                what this case is for, and its issue
    input/                                   the fixture repository
    expected.yaml                            hand-authored truth
bounds.py  digest.py  score.py               derived state, never stored
spec/                                        the contract
```

## License

AGPL-3.0-or-later.

## When the corpus goes green

A corpus every producer passes has stopped discriminating. The first producer
scored 0.974/0.864 here; after the six defects it found were fixed it scores
1.0/1.0, which is the moment to add harder cases rather than the moment to
celebrate. The controls still fail on a regression — that is what they are for —
but nothing currently open is telling anyone something they did not know.

Tracked as [#1](https://github.com/agent-ix/quire-corpus/issues/1).
