use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ix_trace_rs::trace;
use quire_corpus::bounds::{self, CaseMetadata, CriterionCoverage};
use quire_corpus::coverage;
use quire_corpus::digest;
use quire_corpus::producer::ProducerInvocation;
use quire_corpus::score::{self, Tally};
use serde_json::{Value, json};
use tempfile::TempDir;
use walkdir::WalkDir;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const CLI: &str = env!("CARGO_BIN_EXE_quire-corpus");
const PRODUCER: &str = env!("CARGO_BIN_EXE_quire-corpus-test-producer");

fn root() -> &'static Path {
    Path::new(ROOT)
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in WalkDir::new(source) {
        let entry = entry.expect("walk source tree");
        let relative = entry.path().strip_prefix(source).expect("relative path");
        if relative
            .components()
            .next()
            .is_some_and(|part| part.as_os_str() == ".git" || part.as_os_str() == "target")
        {
            continue;
        }
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).expect("create copied directory");
        } else if entry.file_type().is_file() {
            fs::copy(entry.path(), target).expect("copy corpus file");
        }
    }
}

fn scratch() -> (TempDir, PathBuf) {
    let temporary = tempfile::tempdir().expect("create scratch directory");
    let corpus = temporary.path().join("corpus");
    copy_tree(root(), &corpus);
    (temporary, corpus)
}

fn read_json_yaml(path: &Path) -> Value {
    serde_yaml::from_str(&fs::read_to_string(path).expect("read YAML")).expect("parse YAML")
}

fn invocation(extra: &[String]) -> ProducerInvocation {
    let mut arguments = extra.to_vec();
    arguments.extend([
        "{org}".to_owned(),
        "{repo}".to_owned(),
        "{input}".to_owned(),
    ]);
    ProducerInvocation {
        executable: PRODUCER.to_owned(),
        arguments,
    }
}

fn run_cli(corpus: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new(CLI);
    command.arg("--root").arg(corpus).args(arguments);
    command.output().expect("run quire-corpus")
}

fn meta(language: &str) -> CaseMetadata {
    CaseMetadata {
        language: language.to_owned(),
        ..CaseMetadata::default()
    }
}

fn score_simple(expected: Value, produced: Value, tally: &mut Tally) -> Value {
    score::score_case(&meta("rust"), &expected, &produced, tally, "{}").expect("score case")
}

fn empty_payload() -> Value {
    json!({"nodes": [], "edges": [], "mentions": [], "diagnostics": []})
}

#[trace("TC-001", "StR-001-VC-1")]
#[test]
fn run_yields_confusion_matrix_at_revision() {
    let selected = BTreeSet::from(["declarations/declaration-forms/rust".to_owned()]);
    let run = score::score(root(), &invocation(&[]), &selected).expect("score corpus");
    assert_eq!(
        run.report.corpus_revision,
        digest::corpus_revision(root()).unwrap()
    );
    assert!(
        run.report.confusion["total"]["total"]["fn"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[trace("TC-002", "StR-001-VC-2")]
#[test]
fn truth_change_changes_revision() {
    let (_temporary, corpus) = scratch();
    let before = digest::corpus_revision(&corpus).unwrap();
    let path = corpus.join("fixtures/relations/bare-import/rust/expected.yaml");
    fs::write(
        &path,
        fs::read_to_string(&path).unwrap().replace(
            "forbidden_edge_types: [imports]",
            "forbidden_edge_types: []",
        ),
    )
    .unwrap();
    assert_ne!(before, digest::corpus_revision(&corpus).unwrap());
}

#[trace("TC-003", "FR-004-AC-5", "StR-001-VC-3")]
#[test]
fn unanswered_query_is_not_computed() {
    let mut tally = Tally::default();
    let result = score_simple(
        json!({"org":"o","repo":"r","impact":[{"seed":"a"}]}),
        empty_payload(),
        &mut tally,
    );
    assert_eq!(result["derived"]["impact"]["state"], "not-computed");
    assert_eq!(tally.as_value(), json!({}));
}

#[trace("TC-004", "TC-030", "StR-001-VC-4", "NFR-002-AC-1")]
#[test]
fn every_positive_has_control_or_reason() {
    let found = bounds::discover(root()).unwrap();
    let controls: BTreeSet<_> = found
        .values()
        .filter_map(|path| {
            let value = read_json_yaml(&path.join("case.yaml"));
            (value["kind"] == "control")
                .then(|| value["control_for"].as_str().unwrap_or_default().to_owned())
        })
        .collect();
    for path in found.values() {
        let value = read_json_yaml(&path.join("case.yaml"));
        if value["kind"] != "positive" {
            continue;
        }
        let reasoned = ["no_control_reason", "shares_input_with", "control_for_pair"]
            .iter()
            .any(|key| {
                value[*key]
                    .as_str()
                    .is_some_and(|text| !text.trim().is_empty())
            });
        let case = path
            .parent()
            .and_then(Path::file_name)
            .unwrap()
            .to_string_lossy();
        assert!(
            reasoned || controls.contains(case.as_ref()),
            "{} has no control or reason",
            path.display()
        );
    }
}

#[trace("TC-005", "FR-001-AC-1")]
#[test]
fn every_case_has_three_parts() {
    let report = bounds::audit(root()).unwrap();
    assert!(report.problems.is_empty());
    assert!(!report.covered.is_empty());
}

#[trace("TC-006", "FR-001-AC-2")]
#[test]
fn wrong_depth_is_error() {
    let (_temporary, corpus) = scratch();
    fs::write(
        corpus.join("fixtures/declarations/case.yaml"),
        "id: stray\n",
    )
    .unwrap();
    let error = bounds::discover(&corpus).unwrap_err().to_string();
    assert!(error.contains("four-segment"));
}

#[trace("TC-007", "FR-001-AC-3")]
#[test]
fn missing_issue_ref_fails() {
    let (_temporary, corpus) = scratch();
    let path = corpus.join("fixtures/ambiguity/same-name-two-files/mixed/case.yaml");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .replace("issue_ref: agent-ix/quire-rs#385", "issue_ref: ''"),
    )
    .unwrap();
    assert!(
        bounds::audit(&corpus)
            .unwrap()
            .problems
            .iter()
            .any(|message| message.contains("no issue_ref"))
    );
}

#[trace("TC-008", "FR-001-AC-4")]
#[test]
fn expectations_name_org_and_repo() {
    for (key, path) in bounds::discover(root()).unwrap() {
        let expected = read_json_yaml(&path.join("expected.yaml"));
        assert!(expected.get("org").is_some(), "{}", key.slash_name());
        assert!(expected.get("repo").is_some(), "{}", key.slash_name());
    }
}

#[trace("TC-009", "FR-001-CON-2")]
#[test]
fn reserved_families_are_reasoned_and_empty() {
    let manifest = bounds::load_manifest(root()).unwrap();
    assert!(!manifest.reserved_families.is_empty());
    for (name, reserved) in manifest.reserved_families {
        assert!(!reserved.issue_ref.trim().is_empty());
        assert!(!reserved.reason.trim().is_empty());
        assert!(!manifest.inventory.contains_key(&name));
    }
}

#[trace("TC-010", "FR-002-AC-1")]
#[test]
fn missing_declared_fixture_is_gap() {
    let (_temporary, corpus) = scratch();
    fs::remove_dir_all(corpus.join("fixtures/relations/bare-import/python")).unwrap();
    let report = bounds::audit(&corpus).unwrap();
    assert!(report.failed());
    assert!(
        report
            .gaps
            .iter()
            .any(|key| key.slash_name() == "relations/bare-import/python")
    );
}

#[trace("TC-011", "FR-002-AC-2")]
#[test]
fn undeclared_fixture_fails() {
    let (_temporary, corpus) = scratch();
    copy_tree(
        &corpus.join("fixtures/relations/bare-import/rust"),
        &corpus.join("fixtures/relations/bare-import/mixed"),
    );
    let report = bounds::audit(&corpus).unwrap();
    assert!(report.failed());
    assert!(
        report
            .undeclared
            .iter()
            .any(|key| key.slash_name() == "relations/bare-import/mixed")
    );
}

#[trace("TC-012", "FR-002-AC-3")]
#[test]
fn empty_exclusion_reason_fails() {
    let (_temporary, corpus) = scratch();
    let path = corpus.join("corpus.yaml");
    fs::write(
        &path,
        fs::read_to_string(&path).unwrap().replace(
            "      out_of_scope:\n",
            "      out_of_scope:\n        mixed: ''\n",
        ),
    )
    .unwrap();
    assert!(
        bounds::audit(&corpus)
            .unwrap_err()
            .to_string()
            .contains("wearing a different word")
    );
}

#[trace("TC-013", "FR-002-AC-4")]
#[test]
fn coverage_is_derived_from_tree() {
    let before = bounds::audit(root()).unwrap().covered.len();
    let (_temporary, corpus) = scratch();
    fs::remove_dir_all(corpus.join("fixtures/relations/bare-import/python")).unwrap();
    let after = bounds::audit(&corpus).unwrap();
    assert_eq!(after.covered.len(), before - 1);
    assert_eq!(after.gap_count, 1);
}

#[trace("TC-014", "FR-002-AC-5")]
#[test]
fn missing_named_control_fails() {
    let (_temporary, corpus) = scratch();
    fs::remove_dir_all(corpus.join("fixtures/relations/receiver-typed-call-ambiguous/rust"))
        .unwrap();
    let manifest = corpus.join("corpus.yaml");
    fs::write(
        &manifest,
        fs::read_to_string(&manifest).unwrap().replace(
            "    receiver-typed-call-ambiguous: [rust, typescript, python]",
            "    receiver-typed-call-ambiguous: [typescript, python]",
        ),
    )
    .unwrap();
    assert!(
        bounds::audit(&corpus)
            .unwrap()
            .problems
            .iter()
            .any(|message| message.contains("which does not exist"))
    );
}

#[trace("TC-015", "FR-002-CON-1")]
#[test]
fn gap_count_is_count() {
    let value = serde_json::to_value(bounds::audit(root()).unwrap()).unwrap();
    assert!(value["gap_count"].is_u64());
    assert!(value.get("gap_ratio").is_none());
}

#[trace("TC-016", "FR-003-AC-1")]
#[test]
fn invocation_substitutes_all_placeholders() {
    let case = root().join("fixtures/relations/bare-import/rust");
    let observation = invocation(&[]).run(root(), &case, "org", "repo").unwrap();
    let arguments = observation.value["observed_arguments"].as_array().unwrap();
    assert_eq!(arguments[0], "org");
    assert_eq!(arguments[1], "repo");
    assert!(arguments[2].as_str().unwrap().ends_with("/input"));
}

#[trace("TC-017", "FR-003-AC-2")]
#[test]
fn failing_producer_is_not_scored() {
    let case = root().join("fixtures/relations/bare-import/rust");
    let error = invocation(&["--exit=3".to_owned()])
        .run(root(), &case, "o", "r")
        .unwrap_err();
    assert!(error.to_string().contains("exited 3"));
}

#[trace("TC-018", "FR-003-AC-3")]
#[test]
fn non_json_output_is_rejected() {
    let case = root().join("fixtures/relations/bare-import/rust");
    let error = invocation(&["--invalid-json".to_owned()])
        .run(root(), &case, "o", "r")
        .unwrap_err();
    assert!(error.to_string().contains("invalid JSON"));
}

#[trace("TC-019", "FR-003-AC-4")]
#[test]
fn producer_contract_is_versioned() {
    let manifest = read_json_yaml(&root().join("corpus.yaml"));
    assert_eq!(manifest["producer_contract"]["version"], 2);
    assert_eq!(
        manifest["producer_contract"]["stdout"],
        "canonical-records-v1"
    );
}

#[trace("TC-020", "FR-004-AC-1")]
#[test]
fn kind_is_censused_not_graded() {
    let mut tally = Tally::default();
    let result = score_simple(
        json!({"org":"o","repo":"r","nodes":[{"object_type":"code_function","name":"o/r/a.rs::f","kind":"method"}]}),
        json!({"nodes":[{"name":"o/r/a.rs::f","object_type":"code_function","data":{"kind":"function"}}],"edges":[]}),
        &mut tally,
    );
    assert_eq!(result["findings"], json!([]));
    assert_eq!(result["kind_census"]["code_function"]["function"], 1);
}

#[trace("TC-021", "FR-004-AC-2")]
#[test]
fn edges_are_scoped_to_declared_types() {
    let mut tally = Tally::default();
    let result = score_simple(
        json!({"org":"o","repo":"r","edges":[{"source":"a","type":"calls","target":"b","reason":"receiver-typed"}]}),
        json!({"nodes":[],"edges":[{"source_ref":"a","edge_type":"calls","target_ref":"b","reason":"receiver-typed"},{"source_ref":"a","edge_type":"contains","target_ref":"c","reason":"syntactic"}]}),
        &mut tally,
    );
    assert_eq!(result["findings"], json!([]));
}

#[trace("TC-022", "FR-004-AC-3")]
#[test]
fn forbidden_edge_is_false_positive() {
    let mut tally = Tally::default();
    let result = score_simple(
        json!({"org":"o","repo":"r","edges":[],"forbidden_edge_types":["calls"]}),
        json!({"nodes":[],"edges":[{"source_ref":"a","edge_type":"calls","target_ref":"b","reason":"name-match"}]}),
        &mut tally,
    );
    assert!(!result["findings"].as_array().unwrap().is_empty());
    assert_eq!(tally.as_value()["total"]["total"]["fp"], 1);
}

#[trace("TC-023", "FR-004-AC-4")]
#[test]
fn empty_partition_is_unavailable() {
    let mut tally = Tally::default();
    tally.add("fn", &[("language", "rust")]).unwrap();
    assert!(tally.as_value()["total"]["total"]["precision"].is_null());
    assert_eq!(tally.as_value()["total"]["total"]["recall"], 0.0);
}

#[trace("TC-024", "FR-004-AC-5", "StR-001-VC-3")]
#[test]
fn absent_derived_answer_never_enters_ratio() {
    unanswered_query_is_not_computed();
}

#[trace("TC-025", "FR-004-AC-6", "FR-004-CON-1")]
#[test]
fn tier_disagreement_is_not_wrong_edge() {
    let mut tally = Tally::default();
    let result = score_simple(
        json!({"org":"o","repo":"r","edges":[{"source":"a","type":"calls","target":"b","reason":"receiver-typed"}]}),
        json!({"nodes":[],"edges":[{"source_ref":"a","edge_type":"calls","target_ref":"b","reason":"import-scoped"}]}),
        &mut tally,
    );
    assert_eq!(result["findings"], json!([]));
    assert_eq!(result["tier_disagreements"].as_array().unwrap().len(), 1);
    assert_eq!(tally.as_value()["total"]["total"]["tp"], 1);
}

#[trace("TC-026", "FR-004-AC-7")]
#[test]
fn scoring_is_deterministic() {
    let selected = BTreeSet::from(["relations/bare-import/rust".to_owned()]);
    let first = serde_json::to_value(
        score::score(root(), &invocation(&[]), &selected)
            .unwrap()
            .report,
    )
    .unwrap();
    let second = serde_json::to_value(
        score::score(root(), &invocation(&[]), &selected)
            .unwrap()
            .report,
    )
    .unwrap();
    assert_eq!(first, second);
}

#[trace("TC-027", "NFR-001-AC-1")]
#[test]
fn repeated_digest_runs_agree() {
    assert_eq!(
        digest::corpus_revision(root()).unwrap(),
        digest::corpus_revision(root()).unwrap()
    );
}

#[trace("TC-028", "NFR-001-AC-2")]
#[test]
fn fixture_edit_changes_revision() {
    let (_temporary, corpus) = scratch();
    let before = digest::corpus_revision(&corpus).unwrap();
    let target = corpus.join("fixtures/declarations/declaration-forms/rust/input/src/lib.rs");
    fs::write(
        &target,
        format!(
            "{}\npub fn added() {{}}\n",
            fs::read_to_string(&target).unwrap()
        ),
    )
    .unwrap();
    assert_ne!(before, digest::corpus_revision(&corpus).unwrap());
}

#[trace("TC-029", "NFR-001-AC-3")]
#[test]
fn fixture_move_changes_revision() {
    let (_temporary, corpus) = scratch();
    let before = digest::corpus_revision(&corpus).unwrap();
    let source = corpus.join("fixtures/relations/bare-import/rust");
    fs::rename(&source, source.parent().unwrap().join("moved")).unwrap();
    assert_ne!(before, digest::corpus_revision(&corpus).unwrap());
}

#[trace("TC-031", "NFR-002-AC-2")]
#[test]
fn truth_change_rule_is_written() {
    let text = fs::read_to_string(root().join("CONTRIBUTING.md")).unwrap();
    assert!(text.contains("Changing expected truth"));
    assert!(text.contains("quoted"));
}

fn mention_case(expected: Value, mentions: Value) -> Value {
    let mut tally = Tally::default();
    score_simple(
        expected,
        json!({"nodes":[],"edges":[],"mentions":mentions}),
        &mut tally,
    )
}

#[trace("TC-032", "FR-005-AC-1")]
#[test]
fn missing_mention_is_false_negative() {
    let result = mention_case(
        json!({"org":"o","repo":"r","mentions":[{"identifier":"TC-001","kind":"tracking_tag","source":"o/r/a.rs::t"}]}),
        json!([]),
    );
    assert!(
        result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding.as_str().unwrap().contains("missing mention TC-001"))
    );
}

#[trace("TC-033", "FR-005-AC-2")]
#[test]
fn mention_kind_is_part_of_claim() {
    let result = mention_case(
        json!({"org":"o","repo":"r","mentions":[{"identifier":"TC-001","kind":"tracking_tag","source":"o/r/a.rs::t"}]}),
        json!([{"identifier":"TC-001","kind":"requirement_citation","source":"o/r/a.rs::t"}]),
    );
    assert!(
        result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding
                .as_str()
                .unwrap()
                .contains("missing mention TC-001 as tracking_tag"))
    );
}

#[trace("TC-034", "FR-005-AC-3", "FR-005-CON-1")]
#[test]
fn extra_mention_only_fails_when_exhaustive() {
    let mentions =
        json!([{"identifier":"FR-002","kind":"requirement_citation","source":"o/r/a.rs"}]);
    assert_eq!(
        mention_case(
            json!({"org":"o","repo":"r","mentions":[]}),
            mentions.clone()
        )["findings"],
        json!([])
    );
    assert!(
        !mention_case(
            json!({"org":"o","repo":"r","mentions":[],"exhaustive_mentions":true}),
            mentions
        )["findings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[trace("TC-035", "FR-005-AC-4")]
#[test]
fn diagnostic_bounds_work_both_directions() {
    let mut tally = Tally::default();
    let few = score_simple(
        json!({"org":"o","repo":"r","diagnostics":{"min":1}}),
        empty_payload(),
        &mut tally,
    );
    assert!(
        few["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding.as_str().unwrap().contains("at least 1"))
    );
    let many = score_simple(
        json!({"org":"o","repo":"r","diagnostics":{"max":0}}),
        json!({"nodes":[],"edges":[],"diagnostics":[{"code":"x","path":"a"}]}),
        &mut tally,
    );
    assert!(
        many["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding.as_str().unwrap().contains("at most 0"))
    );
}

#[trace("TC-036", "FR-005-AC-5")]
#[test]
fn diagnostic_code_and_path_are_graded() {
    let mut tally = Tally::default();
    let result = score_simple(
        json!({"org":"o","repo":"r","diagnostics":{"codes":["parse_error"],"paths":["src/broken.rs"]}}),
        json!({"nodes":[],"edges":[],"diagnostics":[{"code":"other","path":"src/other.rs"}]}),
        &mut tally,
    );
    let findings = result["findings"].as_array().unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.as_str().unwrap().contains("code 'parse_error'"))
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.as_str().unwrap().contains("naming 'src/broken.rs'"))
    );
}

#[trace("TC-037", "FR-005-AC-6")]
#[test]
fn nondeterministic_producer_is_finding() {
    let (_temporary, corpus) = scratch();
    let counter = corpus.join("run-counter");
    let selected = BTreeSet::from(["determinism/repeated-extraction/mixed".to_owned()]);
    let run = score::score(
        &corpus,
        &invocation(&[format!("--counter-file={}", counter.display())]),
        &selected,
    )
    .unwrap();
    let case = &run.report.cases["determinism/repeated-extraction/mixed"];
    assert_eq!(case["deterministic"], false);
    assert!(
        case["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding.as_str().unwrap().contains("different bytes"))
    );
}

#[trace("TC-038", "FR-005-AC-7")]
#[test]
fn forbidden_payload_substring_is_finding() {
    let mut tally = Tally::default();
    let result = score::score_case(
        &meta("rust"),
        &json!({"org":"o","repo":"r","forbidden_payload_substrings":["/Users/"]}),
        &empty_payload(),
        &mut tally,
        "{\"path\":\"/Users/person/repo\"}",
    )
    .unwrap();
    assert!(
        result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding.as_str().unwrap().contains("/Users/"))
    );
}

fn payload(nodes: &[(&str, &str)]) -> Value {
    json!({"nodes": nodes.iter().map(|(name,id)| json!({"name":name,"object_type":"code_function","id":id,"data":{"kind":"function","path":"src/lib.rs"}})).collect::<Vec<_>>(), "edges":[]})
}

#[trace("TC-039", "FR-006-AC-2")]
#[test]
fn moved_declaration_keeps_identifier() {
    let first = payload(&[("o/r/a.rs::f", "aaa")]);
    assert!(
        score::compare_runs("ids_preserved", &first, &first, &json!({}))
            .unwrap()
            .is_empty()
    );
    assert!(
        score::compare_runs(
            "ids_preserved",
            &first,
            &payload(&[("o/r/a.rs::f", "bbb")]),
            &json!({})
        )
        .unwrap()
        .iter()
        .any(|finding| finding.contains("changed identifier"))
    );
}

#[trace("TC-040", "FR-002-AC-4")]
#[test]
fn org_names_are_disjoint() {
    let first = payload(&[("agent-ix/r/a.rs::f", "aaa")]);
    assert!(
        score::compare_runs(
            "disjoint_names",
            &first,
            &payload(&[("other/r/a.rs::f", "bbb")]),
            &json!({})
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        score::compare_runs(
            "disjoint_names",
            &first,
            &payload(&[("agent-ix/r/a.rs::f", "ccc")]),
            &json!({})
        )
        .unwrap()
        .iter()
        .any(|finding| finding.contains("under both orgs"))
    );
}

#[trace("TC-041", "FR-007-AC-4")]
#[test]
fn identical_except_ignores_only_named_records() {
    let first = payload(&[("o/r/a.rs::f", "aaa")]);
    let added = payload(&[("o/r/a.rs::f", "aaa"), ("o/r/b.rs::g", "bbb")]);
    assert!(
        score::compare_runs(
            "identical_except",
            &first,
            &added,
            &json!({"except_names":["o/r/b.rs::g"]})
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        !score::compare_runs("identical_except", &first, &added, &json!({}))
            .unwrap()
            .is_empty()
    );
}

#[trace("TC-042", "FR-004-AC-7")]
#[test]
fn unknown_relation_is_not_held() {
    assert!(
        score::compare_runs("wishful", &payload(&[]), &payload(&[]), &json!({}))
            .unwrap()
            .iter()
            .any(|finding| finding.contains("unknown relation"))
    );
}

#[trace("TC-043", "FR-002-AC-6")]
#[test]
fn unreached_criterion_fails() {
    let (_temporary, corpus) = scratch();
    let pin = corpus.join("producers/quire-code-rs.criteria.yaml");
    fs::write(
        &pin,
        format!("{}- FR-999-AC-1\n", fs::read_to_string(&pin).unwrap()),
    )
    .unwrap();
    assert!(
        bounds::audit(&corpus)
            .unwrap()
            .problems
            .iter()
            .any(|message| message.contains("FR-999-AC-1 is reached by no case"))
    );
}

#[trace("TC-044", "FR-002-AC-7")]
#[test]
fn phantom_claim_fails() {
    let (_temporary, corpus) = scratch();
    let case = corpus.join("fixtures/resilience/empty-file/rust/case.yaml");
    fs::write(
        &case,
        fs::read_to_string(&case)
            .unwrap()
            .replace("criteria:\n", "criteria:\n- FR-998-AC-1\n"),
    )
    .unwrap();
    assert!(
        bounds::audit(&corpus)
            .unwrap()
            .problems
            .iter()
            .any(|message| message.contains("FR-998-AC-1 is claimed by a case"))
    );
}

#[trace("TC-045", "FR-002-AC-8")]
#[test]
fn stale_unreachable_fails() {
    let (_temporary, corpus) = scratch();
    let manifest = corpus.join("corpus.yaml");
    fs::write(&manifest, fs::read_to_string(&manifest).unwrap().replace("      host-dependent-input: >-", "      retired: >-\n        A criterion that no longer exists in this producer. FR-997-AC-1\n      host-dependent-input: >-")).unwrap();
    assert!(
        bounds::audit(&corpus)
            .unwrap()
            .problems
            .iter()
            .any(|message| message.contains("FR-997-AC-1 is declared unreachable"))
    );
}

#[trace("TC-046", "FR-002-AC-9")]
#[test]
fn unreachable_criteria_have_prose_reasons() {
    let manifest = bounds::load_manifest(root()).unwrap();
    let report = bounds::audit(root()).unwrap();
    let CriterionCoverage::Measured { unreachable, .. } =
        &report.criterion_coverage["quire-code-rs"]
    else {
        panic!("coverage unavailable")
    };
    assert!(!unreachable.is_empty());
    for reason in unreachable.values() {
        assert!(
            manifest.producers["quire-code-rs"].unreachable[reason]
                .split_whitespace()
                .count()
                >= 12
        );
    }
}

#[trace("TC-047", "StR-002-VC-1", "FR-007-AC-1")]
#[test]
fn rust_library_exposes_complete_surface() {
    let _ = bounds::audit as fn(&Path) -> _;
    let _ = digest::report as fn(&Path) -> _;
    let _ = score::compare_runs as fn(&str, &Value, &Value, &Value) -> _;
    let _ = coverage::validate_report as fn(&[u8]) -> _;
    let _ = quire_corpus::refresh::refresh as fn(&Path, &Path) -> _;
}

#[trace("TC-048", "StR-002-VC-2", "NFR-003-AC-4")]
#[test]
fn frozen_baseline_parity_is_recorded() {
    let evidence = fs::read_to_string(root().join("spec/evidence/rust-port-parity.md"))
        .expect("parity evidence");
    assert!(evidence.contains("Bounds: byte-exact"));
    assert!(evidence.contains("Digest: byte-exact"));
    assert!(evidence.contains("Score: exact after excluding only"));
}

#[trace("TC-049", "StR-002-VC-3", "FR-007-CON-1")]
#[test]
fn fixture_sources_remain_opaque() {
    let source = fs::read_to_string(root().join("src/lib.rs")).unwrap();
    assert!(source.contains("opaque fixture data"));
    let cargo = fs::read_to_string(root().join("Cargo.toml")).unwrap();
    for forbidden in ["tree-sitter", "swc", "syn =", "rustpython"] {
        assert!(!cargo.contains(forbidden));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let (_temporary, corpus) = scratch();
        let outside = corpus.parent().unwrap().join("outside-source.rs");
        fs::write(&outside, "not part of the corpus").unwrap();
        let before = digest::corpus_revision(&corpus).unwrap();
        symlink(
            &outside,
            corpus.join("fixtures/relations/bare-import/rust/input/escape.rs"),
        )
        .unwrap();
        assert_eq!(before, digest::corpus_revision(&corpus).unwrap());
    }
}

#[trace("TC-050", "FR-007-AC-2")]
#[test]
fn cli_exposes_all_commands_and_explicit_root() {
    let output = Command::new(CLI).arg("--help").output().unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    for command in [
        "bounds",
        "digest",
        "score",
        "refresh-criteria",
        "check-coverage",
    ] {
        assert!(text.contains(command));
    }
    assert!(text.contains("--root"));
}

#[trace("TC-051", "FR-007-AC-3")]
#[test]
fn json_commands_are_deterministic() {
    for arguments in [vec!["bounds", "--json"], vec!["digest", "--json"]] {
        assert_eq!(
            run_cli(root(), &arguments).stdout,
            run_cli(root(), &arguments).stdout
        );
    }
    let producer_args = [
        "score",
        "--producer",
        PRODUCER,
        "--producer-arg",
        "{org}",
        "--producer-arg",
        "{repo}",
        "--producer-arg",
        "{input}",
        "--case",
        "relations/bare-import/rust",
        "--json",
    ];
    assert_eq!(
        run_cli(root(), &producer_args).stdout,
        run_cli(root(), &producer_args).stdout
    );
}

#[trace("TC-052", "FR-007-AC-4")]
#[test]
fn rejected_input_has_context_and_no_report() {
    let temporary = tempfile::tempdir().unwrap();
    let output = run_cli(temporary.path(), &["bounds", "--json"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("corpus.yaml")
    );

    let (_temporary, corpus) = scratch();
    let selected = BTreeSet::from(["relations/bare-import/rust".to_owned()]);
    let error = score::score(
        &corpus,
        &invocation(&["--mutate-input".to_owned()]),
        &selected,
    )
    .unwrap_err();
    assert!(error.to_string().contains("changed while scoring"));
}

fn git(repo: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[trace("TC-053", "FR-007-AC-5")]
#[test]
fn criteria_refresh_is_exact_and_deterministic() {
    let (_temporary, corpus) = scratch();
    let producer = tempfile::tempdir().unwrap();
    fs::create_dir(producer.path().join("spec")).unwrap();
    fs::write(
        producer.path().join("spec/req.md"),
        "| FR-002-AC-1 | a |\n| FR-001-AC-1 | b |\n",
    )
    .unwrap();
    git(producer.path(), &["init", "-q"]);
    git(
        producer.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(producer.path(), &["config", "user.name", "Test"]);
    git(producer.path(), &["add", "spec/req.md"]);
    git(producer.path(), &["commit", "-qm", "fixture"]);
    git(
        producer.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/agent-ix/demo-producer.git",
        ],
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let outside = producer
            .path()
            .parent()
            .unwrap()
            .join("outside-requirement.md");
        fs::write(&outside, "| FR-999-AC-1 | outside |\n").unwrap();
        symlink(outside, producer.path().join("spec/outside.md")).unwrap();
    }
    let first = quire_corpus::refresh::refresh(&corpus, producer.path()).unwrap();
    let text = fs::read_to_string(&first.path).unwrap();
    let second = quire_corpus::refresh::refresh(&corpus, producer.path()).unwrap();
    assert_eq!(text, fs::read_to_string(second.path).unwrap());
    assert!(
        first
            .path
            .ends_with("producers/demo-producer.criteria.yaml")
    );
    assert!(text.find("FR-001-AC-1").unwrap() < text.find("FR-002-AC-1").unwrap());
    assert!(!text.contains("FR-999-AC-1"));
}

fn coverage_report(overrides: Value) -> Vec<u8> {
    let mut base = json!({"totals":{"backed":1,"total":1},"status_lies":[],"unbacked_rows":[],"no_symbol_rows":[],"diagnostics":[]});
    for (key, value) in overrides.as_object().unwrap() {
        base[key] = value.clone();
    }
    serde_json::to_vec(&base).unwrap()
}

#[trace("TC-054", "FR-007-AC-6")]
#[test]
fn coverage_rejects_every_false_green_class() {
    assert!(
        coverage::validate_report(&coverage_report(json!({"totals":{"backed":0,"total":0}})))
            .unwrap()
            .failed()
    );
    assert!(
        coverage::validate_report(&coverage_report(
            json!({"status_lies":[{"document":"x","row_id":"TC-1"}]})
        ))
        .unwrap()
        .failed()
    );
    assert!(
        coverage::validate_report(&coverage_report(
            json!({"unbacked_rows":[{"document":"x","row_id":"TC-1"}]})
        ))
        .unwrap()
        .failed()
    );
    for reason in [
        "id-column-matches-nothing",
        "archetype-matches-nothing",
        "section-holds-no-table",
        "marker-form-mismatch",
        "hollow-denominator",
    ] {
        assert!(
            coverage::validate_report(&coverage_report(
                json!({"diagnostics":[{"reason":reason,"message":"broken declaration"}]})
            ))
            .unwrap()
            .failed()
        );
    }
    assert!(
        !coverage::run_quire(Path::new(PRODUCER), root(), &root().join("spec"))
            .unwrap()
            .failed()
    );
}

#[trace("TC-055", "FR-007-CON-2")]
#[test]
fn makefile_is_orchestration_only() {
    let makefile = fs::read_to_string(root().join("Makefile")).unwrap();
    for forbidden in ["python", "jq ", "sed ", "awk ", "bash "] {
        assert!(
            !makefile.contains(forbidden),
            "Makefile contains {forbidden}"
        );
    }
}

#[trace("TC-056", "NFR-003-AC-1")]
#[test]
fn exact_rust_and_locked_commands_are_declared() {
    assert_eq!(
        fs::read_to_string(root().join("rust-toolchain.toml"))
            .unwrap()
            .matches("1.98.1")
            .count(),
        1
    );
    let cargo = fs::read_to_string(root().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("rust-version = \"1.98.1\""));
    let makefile = fs::read_to_string(root().join("Makefile")).unwrap();
    assert!(makefile.contains("--locked"));
}

#[trace("TC-057", "NFR-003-AC-2", "FR-003-CON-2")]
#[test]
fn producer_execution_uses_no_shell() {
    let source = fs::read_to_string(root().join("src/producer.rs")).unwrap();
    assert_eq!(source.matches("Command::new(&self.executable)").count(), 1);
    for shell in ["sh -c", "bash", "cmd.exe", "powershell"] {
        assert!(!source.contains(shell));
    }
}

#[trace("TC-058", "NFR-003-AC-3", "FR-003-AC-1")]
#[test]
fn producer_arguments_remain_literal() {
    let special = ["space value", "; touch never", "$(never)", "{org}-literal"];
    let extra = special
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    let case = root().join("fixtures/relations/bare-import/rust");
    let observed = invocation(&extra)
        .run(root(), &case, "{repo}", "actual-repo")
        .unwrap();
    let arguments = observed.value["observed_arguments"].as_array().unwrap();
    assert_eq!(arguments[0], "space value");
    assert_eq!(arguments[1], "; touch never");
    assert_eq!(arguments[2], "$(never)");
    assert_eq!(arguments[3], "{repo}-literal");
}

#[trace("TC-059", "NFR-003-AC-5")]
#[test]
fn every_matrix_row_has_canonical_trace() {
    let source = fs::read_to_string(file!()).unwrap();
    for number in 1..=61 {
        let identifier = format!("TC-{number:03}");
        assert!(
            source.contains(&format!("#[trace(\"{identifier}\""))
                || source.contains(&format!(", \"{identifier}\"")),
            "missing {identifier}"
        );
    }
    let noncanonical = ["#[ix_trace_rs", "::trace"].concat();
    assert!(!source.contains(&noncanonical));
}

#[trace("TC-060", "NFR-003-AC-6")]
#[test]
fn dependency_audits_are_wired() {
    assert!(root().join("deny.toml").is_file());
    let makefile = fs::read_to_string(root().join("Makefile")).unwrap();
    assert!(makefile.contains("cargo deny check"));
    assert!(makefile.contains("cargo audit"));
}

#[trace("TC-061", "FR-003-AC-5")]
#[test]
fn version_two_records_structured_identity() {
    let selected = BTreeSet::from(["relations/bare-import/rust".to_owned()]);
    let run = score::score(root(), &invocation(&[]), &selected).unwrap();
    assert_eq!(run.report.schema_version, 2);
    let identity = serde_json::to_value(&run.report.producer_invocation).unwrap();
    assert!(identity["executable"].is_string());
    assert!(identity["arguments"].is_array());
    assert_ne!(identity, Value::String(PRODUCER.to_owned()));
}
