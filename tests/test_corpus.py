"""The corpus's own gates, and the guards that keep them able to fail.

Run with `make test`, which is `python3 tests/run.py` — a stdlib runner, no
pytest, no unittest. Two reasons, and the second is the load-bearing one:

* NFR-001 says a clean runner reproduces the corpus from the tree alone, and
  a test suite that needs an installed framework is not that;
* the ecosystem's Python scanner binds a trace tag to a module-level `def`,
  and a `unittest.TestCase` method is "a function — a kind that does not
  bind trace ids". Written as methods, all 31 rows of this matrix were
  reported unbacked with no test-side signal at all.

Every test carries its `TC-NNN` and the criterion it discharges on the comma
form, so `quire coverage` binds both. An em dash between them terminates the
id run and backs only the first (agent-ix/quire-code-rs#8).
"""
from __future__ import annotations

import contextlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

import yaml

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

import bounds  # noqa: E402
import digest  # noqa: E402
import score  # noqa: E402

STUB = ROOT / "tests" / "stub_producer.py"


@contextlib.contextmanager
def scratch():
    """A copy of the corpus a test may break on purpose."""
    tmp = pathlib.Path(tempfile.mkdtemp())
    try:
        corpus = tmp / "corpus"
        shutil.copytree(
            ROOT, corpus,
            ignore=shutil.ignore_patterns(".git", "__pycache__", "*.pyc"))
        yield corpus
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def run(corpus: pathlib.Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, *args], cwd=corpus, capture_output=True, text=True)


def revision(corpus: pathlib.Path) -> str:
    out = run(corpus, "digest.py", "--json")
    assert out.returncode == 0, out.stderr
    return json.loads(out.stdout)["corpus_revision"]


def scored(corpus: pathlib.Path, *extra: str, producer: str | None = None):
    command = producer or f"{sys.executable} {STUB} {{org}} {{repo}} {{input}}"
    return run(corpus, "score.py", "--producer", command, "--json", *extra)


# ── structure ────────────────────────────────────────────────────────────

# TC-005, FR-001-AC-1: every case directory holds its three parts.
def test_every_case_holds_case_input_and_expected():
    report = bounds.audit()
    assert report["problems"] == []
    assert report["covered"]


# TC-006, FR-001-AC-2: a case at the wrong depth is an error, not a silent
# omission.
def test_a_case_at_the_wrong_depth_is_an_error():
    with scratch() as corpus:
        (corpus / "fixtures" / "declarations" / "case.yaml").write_text("id: stray\n")
        result = run(corpus, "bounds.py")
        assert result.returncode != 0
        assert "four-segment" in result.stdout + result.stderr


# TC-007, FR-001-AC-3: a case with no issue_ref fails the gate.
def test_a_case_without_an_issue_ref_fails():
    with scratch() as corpus:
        case = corpus / "fixtures/ambiguity/same-name-two-files/mixed/case.yaml"
        case.write_text(case.read_text().replace(
            "issue_ref: agent-ix/quire-rs#385", "issue_ref: ''"))
        result = run(corpus, "bounds.py")
        assert result.returncode != 0
        assert "no issue_ref" in result.stdout


# TC-008, FR-001-AC-4: every expectation names the org and repo its fixture
# is extracted as.
def test_every_expectation_names_its_org_and_repo():
    for key, path in bounds.discover().items():
        expected = yaml.safe_load((path / "expected.yaml").read_text())
        assert "org" in expected, key
        assert "repo" in expected, key


# TC-009, FR-001-CON-2: a reserved family declares its owning issue and is
# not silently populated.
def test_reserved_families_declare_their_issue():
    manifest = bounds.load_manifest()
    reserved = manifest["reserved_families"]
    assert reserved
    for name, entry in reserved.items():
        assert entry.get("issue_ref"), name
        assert entry.get("reason", "").strip(), name
        assert name not in manifest["inventory"], name


# ── inventory ────────────────────────────────────────────────────────────

# TC-010, FR-002-AC-1: a declared cell with no fixture is a GAP and fails.
def test_a_declared_cell_without_a_fixture_is_a_gap():
    with scratch() as corpus:
        shutil.rmtree(corpus / "fixtures/relations/bare-import/python")
        result = run(corpus, "bounds.py")
        assert result.returncode != 0
        assert "GAP        relations/bare-import/python" in result.stdout


# TC-011, FR-002-AC-2: an undeclared fixture fails rather than being counted.
def test_an_undeclared_fixture_fails():
    with scratch() as corpus:
        shutil.copytree(corpus / "fixtures/relations/bare-import/rust",
                        corpus / "fixtures/relations/bare-import/mixed")
        result = run(corpus, "bounds.py")
        assert result.returncode != 0
        assert "UNDECLARED relations/bare-import/mixed" in result.stdout


# TC-012, FR-002-AC-3: an exclusion with no reason fails the gate.
def test_an_exclusion_without_a_reason_fails():
    with scratch() as corpus:
        manifest = corpus / "corpus.yaml"
        manifest.write_text(manifest.read_text().replace(
            "      out_of_scope:\n", "      out_of_scope:\n        mixed: ''\n", 1))
        result = run(corpus, "bounds.py")
        assert result.returncode != 0
        assert "wearing a different word" in result.stdout + result.stderr


# TC-013, FR-002-AC-4: coverage is derived, so a fixture flips its own cell.
def test_coverage_is_derived_from_the_tree():
    before = len(bounds.audit()["covered"])
    with scratch() as corpus:
        shutil.rmtree(corpus / "fixtures/relations/bare-import/python")
        after = json.loads(run(corpus, "bounds.py", "--json").stdout)
        assert len(after["covered"]) == before - 1
        assert after["gap_count"] == 1


# TC-014, FR-002-AC-5: a positive naming a control that is gone fails.
def test_a_positive_naming_a_missing_control_fails():
    with scratch() as corpus:
        shutil.rmtree(corpus / "fixtures/relations/receiver-typed-call-ambiguous/rust")
        manifest = corpus / "corpus.yaml"
        manifest.write_text(manifest.read_text().replace(
            "    receiver-typed-call-ambiguous: [rust, typescript, python]",
            "    receiver-typed-call-ambiguous: [typescript, python]"))
        result = run(corpus, "bounds.py")
        assert result.returncode != 0
        assert "which does not exist" in result.stdout


# TC-015, FR-002-CON-1: gap_count is a count, never a ratio.
def test_gap_count_is_a_count():
    report = bounds.audit()
    assert isinstance(report["gap_count"], int)
    assert "gap_ratio" not in report


# ── reproducibility ──────────────────────────────────────────────────────

# TC-027, NFR-001-AC-1: two runs on an unchanged tree agree.
def test_repeated_runs_agree():
    with scratch() as corpus:
        assert revision(corpus) == revision(corpus)


# TC-028, NFR-001-AC-2: editing any fixture byte changes the revision.
def test_editing_a_fixture_changes_the_revision():
    with scratch() as corpus:
        before = revision(corpus)
        target = corpus / "fixtures/declarations/declaration-forms/rust/input/src/lib.rs"
        target.write_text(target.read_text() + "\npub fn added() {}\n")
        assert revision(corpus) != before


# TC-029, NFR-001-AC-3: moving a fixture changes the revision even though no
# byte changed.
def test_moving_a_fixture_changes_the_revision():
    with scratch() as corpus:
        before = revision(corpus)
        src = corpus / "fixtures/relations/bare-import/rust"
        shutil.move(str(src), str(src.parent / "moved"))
        assert revision(corpus) != before


# TC-002, StR-001-VC-2: a change to expected truth changes the revision, so
# scores at two revisions cannot be silently compared.
def test_a_truth_change_changes_the_revision():
    with scratch() as corpus:
        before = revision(corpus)
        target = corpus / "fixtures/relations/bare-import/rust/expected.yaml"
        target.write_text(target.read_text().replace(
            "forbidden_edge_types: [imports]", "forbidden_edge_types: []"))
        assert revision(corpus) != before


# ── scoring ──────────────────────────────────────────────────────────────

# TC-001, StR-001-VC-1: a run yields a confusion matrix at a stated revision.
def test_a_run_yields_a_confusion_matrix_at_a_revision():
    with scratch() as corpus:
        # A case the stub cannot satisfy, so the matrix has a population.
        # Scoring one where nothing is expected produces no cells at all, and
        # a test that accepted an empty matrix would pass on a broken scorer.
        result = scored(corpus, "--case", "declarations/declaration-forms/rust")
        report = json.loads(result.stdout)
        assert report["corpus_revision"] == revision(corpus)
        assert report["confusion"]["total"]["total"]["fn"] > 0


# TC-016, FR-003-AC-1: the invocation template substitutes org, repo, input.
def test_the_invocation_template_substitutes():
    with scratch() as corpus:
        report = json.loads(
            scored(corpus, "--case", "relations/bare-import/rust").stdout)
        assert report["scored_cases"] == 1


# TC-017, FR-003-AC-2: a non-zero exit is reported, never scored as zero.
def test_a_failing_producer_is_reported_not_scored():
    with scratch() as corpus:
        result = scored(corpus, producer=f"{sys.executable} -c 'import sys; sys.exit(3)'")
        assert result.returncode != 0
        assert "producer exited 3" in result.stdout + result.stderr


# TC-018, FR-003-AC-3: output that is not JSON is reported, never scored.
def test_non_json_output_is_reported():
    with scratch() as corpus:
        result = scored(corpus, producer=f"{sys.executable} -c 'print(\"not json\")'")
        assert result.returncode != 0
        assert "is not JSON" in result.stdout + result.stderr


# TC-019, FR-003-AC-4: the producer contract carries a version.
def test_the_producer_contract_is_versioned():
    contract = bounds.load_manifest()["producer_contract"]
    assert contract["version"] == 1
    assert contract["stdout"] == "canonical-records-v1"


# TC-020, FR-004-AC-1: the grammar's kind is censused, never graded.
def test_kind_is_censused_not_graded():
    tally = score.Tally()
    expected = {"org": "o", "repo": "r", "nodes": [
        {"object_type": "code_function", "name": "o/r/a.rs::f", "kind": "method"}]}
    produced = {"nodes": [{"name": "o/r/a.rs::f", "object_type": "code_function",
                           "data": {"kind": "function"}}], "edges": []}
    result = score.score_case({"language": "rust"}, expected, produced, tally)
    assert result["findings"] == [], "a kind difference is not a correctness finding"
    assert result["kind_census"]["code_function"]["function"] == 1


# TC-021, FR-004-AC-2: edges are scoped to the types the case names.
def test_edges_are_scoped_to_the_declared_types():
    tally = score.Tally()
    expected = {"org": "o", "repo": "r", "edges": [
        {"source": "a", "type": "calls", "target": "b", "reason": "receiver-typed"}]}
    produced = {"nodes": [], "edges": [
        {"source_ref": "a", "edge_type": "calls", "target_ref": "b",
         "reason": "receiver-typed"},
        {"source_ref": "a", "edge_type": "contains", "target_ref": "c",
         "reason": "syntactic"}]}
    result = score.score_case({"language": "rust"}, expected, produced, tally)
    assert result["findings"] == [], (
        "a `contains` edge is not a false positive for a case that says "
        "nothing about containment")


# TC-022, FR-004-AC-3: a forbidden edge type emitted is a false positive.
def test_a_forbidden_edge_type_is_a_false_positive():
    tally = score.Tally()
    expected = {"org": "o", "repo": "r", "edges": [],
                "forbidden_edge_types": ["calls"]}
    produced = {"nodes": [], "edges": [
        {"source_ref": "a", "edge_type": "calls", "target_ref": "b",
         "reason": "name-match"}]}
    result = score.score_case({"language": "rust"}, expected, produced, tally)
    assert result["findings"]
    assert tally.as_dict()["total"]["total"]["fp"] == 1


# TC-023, FR-004-AC-4: an empty partition is unavailable, never 1.0.
def test_an_empty_partition_reports_unavailable():
    tally = score.Tally()
    tally.add("fn", language="rust")
    slices = tally.as_dict()
    assert slices["total"]["total"]["precision"] is None
    assert slices["total"]["total"]["recall"] == 0.0


# TC-024, FR-004-AC-5, TC-003, StR-001-VC-3: a query the producer does not
# answer is not-computed, and contributes no cell to any ratio.
def test_an_unanswered_query_is_not_computed():
    tally = score.Tally()
    expected = {"org": "o", "repo": "r", "impact": [{"seed": "a"}]}
    produced = {"nodes": [], "edges": []}
    result = score.score_case({"language": "mixed"}, expected, produced, tally)
    assert result["derived"]["impact"]["state"] == "not-computed"
    assert tally.as_dict() == {}


# TC-025, FR-004-AC-6, FR-004-CON-1: a tier disagreement is recorded and
# reported, never scored as a wrong edge.
def test_a_tier_disagreement_is_recorded_not_scored():
    tally = score.Tally()
    expected = {"org": "o", "repo": "r", "edges": [
        {"source": "a", "type": "calls", "target": "b", "reason": "receiver-typed"}]}
    produced = {"nodes": [], "edges": [
        {"source_ref": "a", "edge_type": "calls", "target_ref": "b",
         "reason": "import-scoped"}]}
    result = score.score_case({"language": "rust"}, expected, produced, tally)
    assert result["findings"] == []
    assert len(result["tier_disagreements"]) == 1
    assert tally.as_dict()["total"]["total"]["tp"] == 1


# TC-026, FR-004-AC-7: scoring pinned inputs is deterministic.
def test_scoring_is_deterministic():
    with scratch() as corpus:
        case = ("--case", "relations/bare-import/rust")
        assert scored(corpus, *case).stdout == scored(corpus, *case).stdout


# ── truth independence ───────────────────────────────────────────────────

# TC-004, StR-001-VC-4, TC-030, NFR-002-AC-1: every positive has a control,
# or a recorded reason why its own fixture already contains one.
def test_every_positive_has_a_control_or_a_reason():
    manifest = bounds.load_manifest()
    found = bounds.discover()
    controls = {
        yaml.safe_load((p / "case.yaml").read_text()).get("control_for")
        for p in found.values()
        if yaml.safe_load((p / "case.yaml").read_text()).get("kind") == "control"
    }
    for family, cases in manifest["inventory"].items():
        for case, entry in cases.items():
            languages, out_of_scope = bounds._declared(entry)
            if not languages:
                continue
            meta = yaml.safe_load(
                (found[(family, case, languages[0])] / "case.yaml").read_text())
            if meta.get("kind") != "positive":
                continue
            has_control = case in controls or meta.get("control_for_pair")
            reason = meta.get("no_control_reason") or meta.get("shares_input_with")
            assert has_control or out_of_scope or reason, (
                f"{family}/{case} is a positive with no control and no reason")


# TC-031, NFR-002-AC-2: the rule that a truth change carries the contract
# clause deciding it is written down where a reviewer will meet it.
def test_the_truth_change_rule_is_written_down():
    text = (ROOT / "CONTRIBUTING.md").read_text()
    assert "Changing expected truth" in text
    assert "quoted" in text


# ── grading beyond nodes and edges ───────────────────────────────────────────

def _mention_case(expected_extra: dict, produced_mentions: list) -> dict:
    tally = score.Tally()
    expected = {"org": "o", "repo": "r", **expected_extra}
    produced = {"nodes": [], "edges": [], "mentions": produced_mentions}
    return score.score_case({"language": "rust"}, expected, produced, tally)


# TC-032, FR-005-AC-1: a mention the case names and the producer omits is a
# false negative, not a silent pass.
def test_a_missing_mention_is_a_false_negative():
    result = _mention_case(
        {"mentions": [{"identifier": "TC-001", "kind": "tracking_tag",
                       "source": "o/r/a.rs::t"}]},
        [],
    )
    assert any("missing mention TC-001" in f for f in result["findings"])


# TC-033, FR-005-AC-2: the kind is the claim, so the right identifier with the
# wrong kind is not the mention the case asked for.
def test_a_mention_with_the_wrong_kind_is_not_present():
    result = _mention_case(
        {"mentions": [{"identifier": "TC-001", "kind": "tracking_tag",
                       "source": "o/r/a.rs::t"}]},
        [{"identifier": "TC-001", "kind": "requirement_citation",
          "source": "o/r/a.rs::t"}],
    )
    assert any("missing mention TC-001 as tracking_tag" in f
               for f in result["findings"]), (
        "a citation is not a verification claim, and grading them as equal "
        "would let any file claim coverage by writing a comment")


# TC-034, FR-005-AC-3, FR-005-CON-1: an unnamed mention is a false positive only
# where the case says its list is the whole list.
def test_an_unnamed_mention_is_a_false_positive_only_when_exhaustive():
    extra = [{"identifier": "FR-002", "kind": "requirement_citation",
              "source": "o/r/a.rs"}]
    lenient = _mention_case({"mentions": []}, extra)
    assert lenient["findings"] == [], (
        "a case that says nothing about FR-002 is not entitled to an opinion "
        "about it")
    strict = _mention_case({"mentions": [], "exhaustive_mentions": True}, extra)
    assert any("unexpected mention FR-002" in f for f in strict["findings"])


# TC-035, FR-005-AC-4: diagnostic bounds are graded in both directions.
def test_diagnostic_bounds_are_graded_both_ways():
    tally = score.Tally()
    too_few = score.score_case(
        {"language": "rust"},
        {"org": "o", "repo": "r", "diagnostics": {"min": 1}},
        {"nodes": [], "edges": [], "diagnostics": []}, tally)
    assert any("at least 1 diagnostic" in f for f in too_few["findings"])
    too_many = score.score_case(
        {"language": "rust"},
        {"org": "o", "repo": "r", "diagnostics": {"max": 0}},
        {"nodes": [], "edges": [],
         "diagnostics": [{"code": "unresolved_import", "path": "src/a.rs"}]},
        tally)
    assert any("at most 0 diagnostic" in f for f in too_many["findings"]), (
        "reporting a diagnostic per package import is as wrong as reporting "
        "none over a broken tree, and only a bounded expectation separates them")


# TC-036, FR-005-AC-5: a required code or path that nothing carries is a finding.
def test_a_required_diagnostic_code_or_path_is_graded():
    tally = score.Tally()
    result = score.score_case(
        {"language": "rust"},
        {"org": "o", "repo": "r",
         "diagnostics": {"codes": ["parse_error"], "paths": ["src/broken.rs"]}},
        {"nodes": [], "edges": [],
         "diagnostics": [{"code": "unresolved_import", "path": "src/other.rs"}]},
        tally)
    assert any("code 'parse_error'" in f for f in result["findings"])
    assert any("naming 'src/broken.rs'" in f for f in result["findings"])


# TC-037, FR-005-AC-6: two runs that disagree byte for byte are a finding.
def test_a_nondeterministic_producer_is_a_finding():
    with scratch() as corpus:
        flaky = corpus / "flaky.py"
        # Writes a different payload each run, and is otherwise a valid
        # producer: the difference is in the producer, not the input.
        flaky.write_text(
            "import json, sys, itertools, pathlib\n"
            "counter = pathlib.Path(sys.argv[3]) / '.runs'\n"
            "n = int(counter.read_text()) if counter.exists() else 0\n"
            "counter.write_text(str(n + 1))\n"
            "json.dump({'nodes': [], 'edges': [], 'mentions': [],\n"
            "           'diagnostics': [], 'run': n}, sys.stdout)\n"
        )
        result = scored(
            corpus, "--case", "determinism/repeated-extraction/mixed",
            producer=f"{sys.executable} {flaky} {{org}} {{repo}} {{input}}")
        report = json.loads(result.stdout)
        case = report["cases"]["determinism/repeated-extraction/mixed"]
        assert case["deterministic"] is False
        assert any("different bytes" in f for f in case["findings"])


# TC-038, FR-005-AC-7: a forbidden substring in the raw payload is a finding.
def test_a_forbidden_payload_substring_is_a_finding():
    tally = score.Tally()
    result = score.score_case(
        {"language": "rust"},
        {"org": "o", "repo": "r", "forbidden_payload_substrings": ["/Users/"]},
        {"nodes": [], "edges": []}, tally,
        raw='{"path": "/Users/someone/dev/repo/src/a.rs"}')
    assert any("/Users/" in f for f in result["findings"]), (
        "an absolute path is not a wrong edge, which is worse: it fails "
        "silently as drift between two machines that both look green")
