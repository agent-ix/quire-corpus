# Contributing cases

## Truth is authored, never harvested

Expected truth is written by hand from the fixture and the contract. It is
never captured from a producer's output — a truth set copied from the tool it
grades cannot disagree with that tool, which is the one thing it exists to be
able to do.

When a producer disagrees with a case, the question is which is wrong, and it
is answered from the contract, not from convenience:

* the **contract** says the producer is right → correct the expectation, and
  say in the file which criterion decided it;
* the **contract** says the producer is wrong → the case stays, the score
  stays red, and the defect gets a ticket.

"The tool does it the other way" is not an argument. Weakening a case to make
a score green destroys the only measurement the corpus was built to take.

## Adding a case

1. **File the issue first** and put its reference in `case.yaml`. `quire-corpus bounds`
   rejects a case without one: a fixture whose reason has been forgotten
   cannot be re-adjudicated later.
2. **Author the fixture as a repository**, not a snippet. If the behaviour
   reproduces in one file, it probably belongs in the producer's own unit
   tests instead — this corpus is for what only appears between files.
3. **Add the control.** A positive with no lookalike measures nothing about a
   producer that guesses. The control is byte-similar and differs only in
   whether the fact the producer needs is written down.
4. **Declare it in `corpus.yaml`.** A fixture the inventory does not declare
   is an error, not a bonus: it means the matrix under-reports its own
   denominator.
5. **Run `make verify`.** `quire-corpus bounds` must be green before the score means
   anything.

## Scoping a language out

`out_of_scope` needs a reason that says why the mode **cannot occur** in that
language. "Not implemented yet" and "expensive" are GAPs, and a GAP fails the
gate. A language whose positive already contains its own lookalike does not
need a separate control tree, and that is the reason to write down.

## Changing expected truth

Changing an expectation changes `corpus_revision`, which makes every recorded
observation at the old revision incomparable to every observation at the new
one. That is the point: a score is only meaningful against a stated revision.

A change to truth requires, in the pull request:

* the criterion or contract clause that decides it, quoted;
* the before and after score at both revisions;
* for every case whose verdict flips, whether it flipped because the truth
  was wrong or because the producer changed.

A routine producer upgrade is not authority to move a corpus expectation.
