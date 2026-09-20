# quire-corpus

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

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
cargo +1.98.1 run --locked -- --root . bounds
cargo +1.98.1 run --locked -- --root . digest
cargo +1.98.1 run --locked -- --root . score \
  --producer /path/to/producer \
  --producer-arg=--org --producer-arg='{org}' \
  --producer-arg=--repo --producer-arg='{repo}' \
  --producer-arg='{input}'
make verify
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
src/                                        typed Rust library and CLI adapter
tests/                                      Rust qualification with ix-trace-rs
spec/                                        the contract
```

## License

AGPL-3.0-or-later.

## How much of the producer it reaches

`quire-corpus bounds` answers it from two trees rather than from a claim here:

```
quire-code-rs  84 reached, 40 unreachable, 0 unreached  of 124
```

Each case names the criteria it asserts. The producer's own criteria are pinned
under `producers/`. A criterion the pin states that no case claims and no reason
excuses **fails the gate**, so one added upstream cannot arrive quietly — and
neither can a case claiming one nobody states.

The 40 are not an exemption list. Each carries the reason a corpus of this shape
structurally cannot reach it: a property of the tool's own source rather than
its output, evidence that exists only in the tool's repository, a wall-clock
number its benchmark lane owns, an input that cannot exist as a committed
fixture. A criterion that is merely unwritten belongs in neither set.

## When the corpus goes green

A corpus every producer passes has stopped discriminating, and the number above
is not a score — it is the denominator. What keeps the data honest is that it
still separates two producers: at one revision the pre-fix extractor scores
0.982/0.880 with eleven failing cases where the current one scores 1.0/1.0, and
both runs are retained in `baselines/`.

Four expectations are authored and have never been answered by anything — the
`paths`, `impact`, `test_selection` and `derived_links` families ask about
traversal rather than extraction. The report names them and their blocking
issue on every run rather than letting them read as coverage.
