//! Producer-neutral graph scoring and metamorphic comparison.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::bounds::{CaseMetadata, discover};
use crate::digest::{case_digests, corpus_revision};
use crate::error::read_text;
use crate::producer::ProducerInvocation;
use crate::{CorpusError, Result};

type Triple = (String, String, String);

fn object(value: &Value) -> Result<&Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| CorpusError::Invariant("observation root is not an object".to_owned()))
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn required_string(value: &Value, key: &str, context: &str) -> Result<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| CorpusError::Invariant(format!("{context} has no string {key:?}")))
}

fn triple(edge: &Value, produced: bool) -> Result<Triple> {
    let (source, relation, target) = if produced {
        ("source_ref", "edge_type", "target_ref")
    } else {
        ("source", "type", "target")
    };
    Ok((
        required_string(edge, source, "edge")?,
        required_string(edge, relation, "edge")?,
        required_string(edge, target, "edge")?,
    ))
}

/// One confusion-matrix population.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Counts {
    /// True positives: correctly reported.
    pub tp: u64,
    /// False positives: reported but not expected.
    pub fp: u64,
    /// False negatives: expected but not reported.
    pub fn_: u64,
}

/// Confusion tally sliced by stable `(axis, value)` keys.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    slices: BTreeMap<(String, String), Counts>,
}

impl Tally {
    /// Add one outcome to the total and every non-empty named slice.
    pub fn add(&mut self, outcome: &str, slices: &[(&str, &str)]) -> Result<()> {
        self.increment(outcome, "total", "total")?;
        for (axis, value) in slices {
            if !value.is_empty() {
                self.increment(outcome, axis, value)?;
            }
        }
        Ok(())
    }

    fn increment(&mut self, outcome: &str, axis: &str, value: &str) -> Result<()> {
        let counts = self
            .slices
            .entry((axis.to_owned(), value.to_owned()))
            .or_default();
        match outcome {
            "tp" => counts.tp += 1,
            "fp" => counts.fp += 1,
            "fn" => counts.fn_ += 1,
            other => {
                return Err(CorpusError::Invariant(format!(
                    "unknown tally outcome {other:?}"
                )));
            }
        }
        Ok(())
    }

    /// Serialize slices with unavailable empty-population ratios.
    #[must_use]
    pub fn as_value(&self) -> Value {
        let mut axes: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
        for ((axis, value), counts) in &self.slices {
            let ratio = |top: u64, bottom: u64| {
                if bottom == 0 {
                    Value::Null
                } else {
                    let rounded =
                        ((top as f64 / bottom as f64) * 1_000_000.0).round() / 1_000_000.0;
                    json!(rounded)
                }
            };
            axes.entry(axis.clone()).or_default().insert(
                value.clone(),
                json!({
                    "tp": counts.tp,
                    "fp": counts.fp,
                    "fn": counts.fn_,
                    "precision": ratio(counts.tp, counts.tp + counts.fp),
                    "recall": ratio(counts.tp, counts.tp + counts.fn_),
                }),
            );
        }
        serde_json::to_value(axes).expect("BTreeMap JSON serialization cannot fail")
    }
}

fn node_records(payload: &Value) -> Result<Vec<(String, String)>> {
    let mut records = Vec::new();
    for node in array(payload, "nodes") {
        records.push((
            required_string(node, "object_type", "node")?,
            required_string(node, "name", "node")?,
        ));
    }
    records.sort();
    Ok(records)
}

fn edge_records(payload: &Value) -> Result<Vec<Triple>> {
    let mut records = array(payload, "edges")
        .iter()
        .map(|edge| triple(edge, true))
        .collect::<Result<Vec<_>>>()?;
    records.sort();
    Ok(records)
}

/// Evaluate a declared relation between two producer observations.
pub fn compare_runs(
    relation: &str,
    first: &Value,
    second: &Value,
    specification: &Value,
) -> Result<Vec<String>> {
    let mut findings = Vec::new();
    match relation {
        "identical" => {
            if (node_records(first)?, edge_records(first)?)
                != (node_records(second)?, edge_records(second)?)
            {
                findings.push(
                    "the two extractions disagree, and the input difference between them is not one the records may depend on"
                        .to_owned(),
                );
            }
        }
        "identical_except" => {
            let exempt: BTreeSet<_> = array(specification, "except_names")
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            let keep_nodes = |records: Vec<(String, String)>| {
                records
                    .into_iter()
                    .filter(|(_, name)| !exempt.contains(name))
                    .collect::<Vec<_>>()
            };
            let keep_edges = |records: Vec<Triple>| {
                records
                    .into_iter()
                    .filter(|(source, _, target)| {
                        !exempt.contains(source) && !exempt.contains(target)
                    })
                    .collect::<Vec<_>>()
            };
            let first_nodes = keep_nodes(node_records(first)?);
            let second_nodes = keep_nodes(node_records(second)?);
            if first_nodes != second_nodes {
                let first_set: BTreeSet<_> = first_nodes.into_iter().collect();
                let second_set: BTreeSet<_> = second_nodes.into_iter().collect();
                let missing: Vec<_> = first_set
                    .symmetric_difference(&second_set)
                    .take(4)
                    .cloned()
                    .collect();
                findings.push(format!(
                    "records outside the changed file differ between the two runs: {missing:?}"
                ));
            }
            if keep_edges(edge_records(first)?) != keep_edges(edge_records(second)?) {
                findings
                    .push("edges outside the changed file differ between the two runs".to_owned());
            }
        }
        "ids_preserved" => {
            let ids = |payload: &Value| -> Result<BTreeMap<String, Option<String>>> {
                array(payload, "nodes")
                    .iter()
                    .map(|node| {
                        Ok((
                            required_string(node, "name", "node")?,
                            node.get("id").and_then(Value::as_str).map(str::to_owned),
                        ))
                    })
                    .collect()
            };
            let before = ids(first)?;
            let after = ids(second)?;
            for (name, before_id) in &before {
                if let Some(after_id) = after.get(name) {
                    if before_id != after_id {
                        findings.push(format!(
                            "{name} changed identifier between the two runs; a node that moves is a modification, never a delete plus an add"
                        ));
                    }
                } else {
                    findings.push(format!("{name} is absent from the second run"));
                }
            }
        }
        "disjoint_names" => {
            let names = |payload: &Value| -> Result<BTreeSet<String>> {
                array(payload, "nodes")
                    .iter()
                    .map(|node| required_string(node, "name", "node"))
                    .collect()
            };
            let first_names = names(first)?;
            let second_names = names(second)?;
            let shared: Vec<_> = first_names.intersection(&second_names).cloned().collect();
            if !shared.is_empty() {
                findings.push(format!(
                    "{} name(s) appear under both orgs, so two repositories collide in one graph: {:?}",
                    shared.len(),
                    shared.iter().take(3).collect::<Vec<_>>()
                ));
            }
        }
        other => findings.push(format!("unknown relation {other:?}")),
    }
    Ok(findings)
}

fn value_repr(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".to_owned(),
        Some(Value::String(value)) => string_repr(value),
        Some(value) => value.to_string(),
    }
}

fn string_repr(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn optional_string_list(values: impl Iterator<Item = Option<String>>) -> String {
    let members = values
        .map(|value| value.map_or_else(|| "None".to_owned(), |value| string_repr(&value)))
        .collect::<BTreeSet<_>>();
    format!("[{}]", members.into_iter().collect::<Vec<_>>().join(", "))
}

fn add_finding(result: &mut Map<String, Value>, message: String) {
    result
        .entry("findings")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .expect("findings is initialized as an array")
        .push(Value::String(message));
}

/// Score one producer observation against one expected fixture.
pub fn score_case(
    metadata: &CaseMetadata,
    expected: &Value,
    produced: &Value,
    tally: &mut Tally,
    raw: &str,
) -> Result<Value> {
    object(expected)?;
    object(produced)?;
    let language = metadata.language.as_str();
    let nodes = array(produced, "nodes");
    let edges = array(produced, "edges");
    let mut findings = Vec::new();
    let mut tier_disagreements = Vec::new();
    let mut by_name = BTreeMap::new();
    let mut kind_census: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for node in nodes {
        let name = required_string(node, "name", "node")?;
        let object_type = required_string(node, "object_type", "node")?;
        by_name.insert(name, node);
        let kind = node
            .get("data")
            .and_then(|data| data.get("kind"))
            .map_or_else(
                || "None".to_owned(),
                |value| {
                    value
                        .as_str()
                        .map_or_else(|| value.to_string(), str::to_owned)
                },
            );
        *kind_census
            .entry(object_type)
            .or_default()
            .entry(kind)
            .or_default() += 1;
    }

    if expected.get("nodes").is_some() {
        let want: BTreeSet<_> = array(expected, "nodes")
            .iter()
            .map(|node| {
                Ok((
                    required_string(node, "name", "expected node")?,
                    required_string(node, "object_type", "expected node")?,
                ))
            })
            .collect::<Result<_>>()?;
        let got: BTreeSet<_> = nodes
            .iter()
            .map(|node| {
                Ok((
                    required_string(node, "name", "node")?,
                    required_string(node, "object_type", "node")?,
                ))
            })
            .collect::<Result<_>>()?;
        for (name, object_type) in want.difference(&got) {
            tally.add(
                "fn",
                &[
                    ("language", language),
                    ("object_type", object_type),
                    ("axis_kind", "node"),
                ],
            )?;
            findings.push(format!("missing node {object_type} {name}"));
        }
        if expected
            .get("exhaustive_nodes")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            for (name, object_type) in got.difference(&want) {
                tally.add(
                    "fp",
                    &[
                        ("language", language),
                        ("object_type", object_type),
                        ("axis_kind", "node"),
                    ],
                )?;
                findings.push(format!("unexpected node {object_type} {name}"));
            }
        }
        for (_, object_type) in want.intersection(&got) {
            tally.add(
                "tp",
                &[
                    ("language", language),
                    ("object_type", object_type),
                    ("axis_kind", "node"),
                ],
            )?;
        }
    }

    if expected
        .get("unique_node_names")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let mut seen: BTreeMap<(String, String), u64> = BTreeMap::new();
        for node in nodes {
            let key = (
                required_string(node, "object_type", "node")?,
                required_string(node, "name", "node")?,
            );
            *seen.entry(key).or_default() += 1;
        }
        for ((object_type, name), count) in seen {
            if count > 1 {
                tally.add(
                    "fp",
                    &[
                        ("language", language),
                        ("object_type", &object_type),
                        ("axis_kind", "node"),
                    ],
                )?;
                findings.push(format!(
                    "{count} nodes share the name {name} ({object_type}); a consumer keying on identity keeps one of them"
                ));
            }
        }
    }

    for name in array(expected, "forbidden_node_names")
        .iter()
        .filter_map(Value::as_str)
    {
        if by_name.contains_key(name) {
            tally.add("fp", &[("language", language), ("axis_kind", "node")])?;
            findings.push(format!(
                "forbidden node minted: {name} is not a declaration"
            ));
        } else {
            tally.add("tp", &[("language", language), ("axis_kind", "node")])?;
        }
    }
    for kind in array(expected, "forbidden_node_kinds")
        .iter()
        .filter_map(Value::as_str)
    {
        for node in nodes {
            if node
                .get("data")
                .and_then(|data| data.get("kind"))
                .and_then(Value::as_str)
                == Some(kind)
            {
                tally.add(
                    "fp",
                    &[
                        ("language", language),
                        ("node_kind", kind),
                        ("axis_kind", "node"),
                    ],
                )?;
                findings.push(format!(
                    "forbidden node kind {kind:?}: {} — this language cannot declare one",
                    required_string(node, "name", "node")?
                ));
            }
        }
    }

    let mut want_edges = BTreeMap::new();
    let mut scoped_types = BTreeSet::new();
    for edge in array(expected, "edges") {
        let edge_triple = triple(edge, false)?;
        scoped_types.insert(edge_triple.1.clone());
        want_edges.insert(edge_triple, edge);
    }
    scoped_types.extend(
        array(expected, "forbidden_edge_types")
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned),
    );
    let mut got_edges = BTreeMap::new();
    for edge in edges {
        got_edges.insert(triple(edge, true)?, edge);
    }

    for (edge_triple, want) in &want_edges {
        let Some(got) = got_edges.get(edge_triple) else {
            let tier = want.get("reason").and_then(Value::as_str).unwrap_or("");
            tally.add(
                "fn",
                &[
                    ("language", language),
                    ("relation", &edge_triple.1),
                    ("tier", tier),
                    ("axis_kind", "edge"),
                ],
            )?;
            findings.push(format!(
                "missing edge {} -{}-> {}",
                edge_triple.0, edge_triple.1, edge_triple.2
            ));
            continue;
        };
        let tier = got.get("reason").and_then(Value::as_str).unwrap_or("");
        tally.add(
            "tp",
            &[
                ("language", language),
                ("relation", &edge_triple.1),
                ("tier", tier),
                ("axis_kind", "edge"),
            ],
        )?;
        for field in ["confidence", "count"] {
            if let Some(wanted) = want.get(field) {
                if got.get(field) == Some(wanted) {
                    tally.add("tp", &[("language", language), ("axis_kind", "provenance")])?;
                } else {
                    tally.add("fp", &[("language", language), ("axis_kind", "provenance")])?;
                    findings.push(format!(
                        "edge {} -{}-> {} has {field} {}, expected {}",
                        edge_triple.0,
                        edge_triple.1,
                        edge_triple.2,
                        value_repr(got.get(field)),
                        value_repr(Some(wanted))
                    ));
                }
            }
        }
        if let Some(wanted) = want.get("evidence").and_then(Value::as_array) {
            let project = |entries: &[Value]| {
                entries
                    .iter()
                    .map(|entry| {
                        json!({
                            "file": entry.get("file").cloned().unwrap_or(Value::Null),
                            "line": entry.get("line").cloned().unwrap_or(Value::Null),
                        })
                    })
                    .collect::<Vec<_>>()
            };
            let got_evidence = project(array(got, "evidence"));
            let wanted_evidence = project(wanted);
            if got_evidence == wanted_evidence {
                tally.add("tp", &[("language", language), ("axis_kind", "provenance")])?;
            } else {
                tally.add("fp", &[("language", language), ("axis_kind", "provenance")])?;
                findings.push(format!(
                    "edge {} -{}-> {} evidence is {got_evidence:?}, expected {wanted_evidence:?}",
                    edge_triple.0, edge_triple.1, edge_triple.2
                ));
            }
        }
        if let Some(wanted_len) = want.get("evidence_len").and_then(Value::as_u64) {
            let actual = u64::try_from(array(got, "evidence").len()).unwrap_or(u64::MAX);
            if actual == wanted_len {
                tally.add("tp", &[("language", language), ("axis_kind", "provenance")])?;
            } else {
                tally.add("fp", &[("language", language), ("axis_kind", "provenance")])?;
                findings.push(format!(
                    "edge {} -{}-> {} carries {actual} evidence entries, expected {wanted_len}",
                    edge_triple.0, edge_triple.1, edge_triple.2
                ));
            }
        }
        if let Some(wanted_reason) = want.get("reason")
            && got.get("reason") != Some(wanted_reason)
        {
            tier_disagreements.push(json!({
                "edge": [&edge_triple.0, &edge_triple.1, &edge_triple.2],
                "expected": wanted_reason,
                "reported": got.get("reason").cloned().unwrap_or(Value::Null),
            }));
        }
    }
    for (edge_triple, got) in &got_edges {
        if want_edges.contains_key(edge_triple) || !scoped_types.contains(&edge_triple.1) {
            continue;
        }
        let tier = got.get("reason").and_then(Value::as_str).unwrap_or("");
        tally.add(
            "fp",
            &[
                ("language", language),
                ("relation", &edge_triple.1),
                ("tier", tier),
                ("axis_kind", "edge"),
            ],
        )?;
        findings.push(format!(
            "unexpected edge {} -{}-> {} (tier {})",
            edge_triple.0,
            edge_triple.1,
            edge_triple.2,
            value_repr(got.get("reason"))
        ));
    }
    for forbidden in array(expected, "forbidden_edges") {
        let edge_triple = triple(forbidden, false)?;
        if let Some(got) = got_edges.get(&edge_triple) {
            let tier = got.get("reason").and_then(Value::as_str).unwrap_or("");
            tally.add(
                "fp",
                &[
                    ("language", language),
                    ("relation", &edge_triple.1),
                    ("tier", tier),
                    ("axis_kind", "edge"),
                ],
            )?;
            findings.push(format!(
                "forbidden edge emitted: {} -{}-> {}",
                edge_triple.0, edge_triple.1, edge_triple.2
            ));
        } else {
            tally.add(
                "tp",
                &[
                    ("language", language),
                    ("relation", &edge_triple.1),
                    ("axis_kind", "edge"),
                ],
            )?;
        }
    }

    if let Some(visibility) = expected.get("visibility").and_then(Value::as_object) {
        for (name, wanted) in visibility {
            let Some(node) = by_name.get(name) else {
                tally.add("fn", &[("language", language), ("axis_kind", "visibility")])?;
                findings.push(format!("visibility unmeasurable: no node named {name}"));
                continue;
            };
            let got = node.get("data").and_then(|data| data.get("visibility"));
            if got == Some(wanted) {
                tally.add("tp", &[("language", language), ("axis_kind", "visibility")])?;
            } else {
                tally.add("fp", &[("language", language), ("axis_kind", "visibility")])?;
                findings.push(format!(
                    "{name} classified {}, expected {}",
                    value_repr(got),
                    value_repr(Some(wanted))
                ));
            }
        }
    }
    if let Some(signatures) = expected.get("signatures").and_then(Value::as_object) {
        for (name, wanted) in signatures {
            let got = by_name
                .get(name)
                .and_then(|node| node.get("data"))
                .and_then(|data| data.get("signature"));
            if got == Some(wanted) {
                tally.add("tp", &[("language", language), ("axis_kind", "signature")])?;
            } else {
                tally.add("fp", &[("language", language), ("axis_kind", "signature")])?;
                findings.push(format!(
                    "{name} signature {}, expected {}",
                    value_repr(got),
                    value_repr(Some(wanted))
                ));
            }
        }
    }

    if expected.get("mentions").is_some() {
        let mention = |value: &Value, expected_value: bool| -> Result<(String, String, String)> {
            let context = if expected_value {
                "expected mention"
            } else {
                "mention"
            };
            Ok((
                required_string(value, "identifier", context)?,
                required_string(value, "kind", context)?,
                required_string(value, "source", context)?,
            ))
        };
        let want: BTreeSet<_> = array(expected, "mentions")
            .iter()
            .map(|value| mention(value, true))
            .collect::<Result<_>>()?;
        let got: BTreeSet<_> = array(produced, "mentions")
            .iter()
            .map(|value| mention(value, false))
            .collect::<Result<_>>()?;
        for missing in want.difference(&got) {
            tally.add("fn", &[("language", language), ("axis_kind", "mention")])?;
            findings.push(format!(
                "missing mention {} as {} on {}",
                missing.0, missing.1, missing.2
            ));
        }
        for _ in want.intersection(&got) {
            tally.add("tp", &[("language", language), ("axis_kind", "mention")])?;
        }
        if expected
            .get("exhaustive_mentions")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            for extra in got.difference(&want) {
                tally.add("fp", &[("language", language), ("axis_kind", "mention")])?;
                findings.push(format!(
                    "unexpected mention {} as {} on {}",
                    extra.0, extra.1, extra.2
                ));
            }
        }
    }

    let mut census = Map::new();
    if let Some(wanted) = expected.get("ambiguous_call_sites") {
        let reported = produced
            .get("stats")
            .and_then(|stats| stats.get("unresolved_calls"))
            .cloned();
        census.insert(
            "ambiguous_call_sites".to_owned(),
            json!({
                "expected": wanted,
                "reported": reported.clone().unwrap_or(Value::Null),
                "state": if reported.is_some() { "measured" } else { "not-computed" },
            }),
        );
    }
    if let Some(wanted) = expected.get("diagnostics").and_then(Value::as_object) {
        let diagnostics = array(produced, "diagnostics");
        census.insert(
            "diagnostics".to_owned(),
            json!({"reported": diagnostics.len(), "expected": wanted, "state": "measured"}),
        );
        let diagnostic_count = u64::try_from(diagnostics.len()).unwrap_or(u64::MAX);
        if let Some(minimum) = wanted.get("min").and_then(Value::as_u64)
            && diagnostic_count < minimum
        {
            findings.push(format!(
                "expected at least {minimum} diagnostic(s), got {}",
                diagnostics.len()
            ));
        }
        if let Some(maximum) = wanted.get("max").and_then(Value::as_u64)
            && diagnostic_count > maximum
        {
            let codes: Vec<_> = diagnostics
                .iter()
                .map(|item| item.get("code").cloned().unwrap_or(Value::Null))
                .collect();
            findings.push(format!(
                "expected at most {maximum} diagnostic(s), got {}: {codes:?}",
                diagnostics.len()
            ));
        }
        for code in wanted
            .get("codes")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice)
        {
            if !diagnostics
                .iter()
                .any(|item| item.get("code") == Some(code))
            {
                let got = optional_string_list(
                    diagnostics
                        .iter()
                        .map(|item| item.get("code").and_then(Value::as_str).map(str::to_owned)),
                );
                findings.push(format!(
                    "no diagnostic with code {}; got {got}",
                    code.as_str().map_or_else(|| code.to_string(), string_repr)
                ));
            }
        }
        for path in wanted
            .get("paths")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice)
        {
            if !diagnostics
                .iter()
                .any(|item| item.get("path") == Some(path))
            {
                let got = optional_string_list(
                    diagnostics
                        .iter()
                        .map(|item| item.get("path").and_then(Value::as_str).map(str::to_owned)),
                );
                findings.push(format!(
                    "no diagnostic naming {}; got {got}",
                    path.as_str().map_or_else(|| path.to_string(), string_repr)
                ));
            }
        }
    }

    let mut derived = Map::new();
    for (key, payload_key) in [
        ("paths", "paths"),
        ("impact", "impact"),
        ("test_selection", "test_selection"),
        ("derived_links", "links"),
    ] {
        let Some(wanted) = expected.get(key) else {
            continue;
        };
        if let Some(got) = produced.get(payload_key) {
            derived.insert(
                key.to_owned(),
                json!({"state": "measured", "matches": got == wanted}),
            );
        } else {
            derived.insert(
                key.to_owned(),
                json!({
                    "state": "not-computed",
                    "why": "the producer emits no such answer; an extractor is not a traversal engine, and a zero here would be a measurement nobody took",
                }),
            );
        }
    }
    for needle in array(expected, "forbidden_payload_substrings")
        .iter()
        .filter_map(Value::as_str)
    {
        if raw.contains(needle) {
            findings.push(format!(
                "payload contains {needle:?} - a value that cannot be compared between two machines"
            ));
        }
    }

    Ok(json!({
        "findings": findings,
        "tier_disagreements": tier_disagreements,
        "kind_census": kind_census,
        "census": census,
        "derived": derived,
        "population": {
            "nodes_produced": nodes.len(),
            "edges_produced": edges.len(),
            "nodes_expected": array(expected, "nodes").len(),
            "edges_expected": want_edges.len(),
        },
    }))
}

fn load_yaml_json(path: &Path) -> Result<Value> {
    let text = read_text(path)?;
    serde_yaml::from_str(&text).map_err(|source| CorpusError::Yaml {
        path: path.to_path_buf(),
        source,
    })
}

/// Complete score operation outcome.
#[derive(Clone, Debug, Serialize)]
pub struct ScoreReport {
    /// Score-report wire contract version.
    pub schema_version: u32,
    /// Identity of the manifest and fixture truth used by this run.
    pub corpus_revision: String,
    /// Per-case identities at the same revision.
    pub case_digests: BTreeMap<String, String>,
    /// Exact executable and ordered argument templates used by this run.
    pub producer_invocation: ProducerInvocation,
    /// Number of cases selected and scored.
    pub scored_cases: usize,
    /// Confusion matrix and its deterministic slices.
    pub confusion: Value,
    /// Per-case results keyed by slash-separated case identity.
    ///
    /// The inner values remain schema-flexible because different authored
    /// fixture families carry different census and derived-query projections.
    pub cases: BTreeMap<String, Value>,
}

/// Complete score operation outcome.
#[derive(Clone, Debug)]
pub struct ScoreRun {
    /// Version-2 score observation.
    pub report: ScoreReport,
    /// Whether a non-pending case failed or a pending case became stale.
    pub failed: bool,
    /// Human-readable stale-pending notices.
    pub stale: Vec<String>,
}

/// Score the selected cases through a structured producer invocation.
pub fn score(
    root: &Path,
    invocation: &ProducerInvocation,
    selected: &BTreeSet<String>,
) -> Result<ScoreRun> {
    let revision = corpus_revision(root)?;
    let digests = case_digests(root)?;
    let mut tally = Tally::default();
    let mut cases = BTreeMap::new();
    let mut failed = false;
    let mut stale = Vec::new();
    for (key, case_dir) in discover(root)? {
        let slash_key = key.slash_name();
        if !selected.is_empty() && !selected.contains(&slash_key) {
            continue;
        }
        let metadata: CaseMetadata = serde_yaml::from_str(&read_text(&case_dir.join("case.yaml"))?)
            .map_err(|source| CorpusError::Yaml {
                path: case_dir.join("case.yaml"),
                source,
            })?;
        let expected = load_yaml_json(&case_dir.join("expected.yaml"))?;
        let org = required_string(&expected, "org", "expected case")?;
        let repo = required_string(&expected, "repo", "expected case")?;
        let observation = invocation.run(root, &case_dir, &org, &repo)?;
        let mut result = score_case(
            &metadata,
            &expected,
            &observation.value,
            &mut tally,
            &observation.raw,
        )?;
        let result_object = result
            .as_object_mut()
            .expect("score_case always returns an object");
        if let Some(relation) = expected.get("relation").and_then(Value::as_str) {
            let variant_org = expected
                .get("variant_org")
                .and_then(Value::as_str)
                .unwrap_or(&org);
            let variant = case_dir.join("variant");
            let variant_dir = if variant.is_dir() {
                variant.as_path()
            } else {
                case_dir.as_path()
            };
            let second = invocation.run(root, variant_dir, variant_org, &repo)?;
            let relation_findings =
                compare_runs(relation, &observation.value, &second.value, &expected)?;
            result_object.insert(
                "relation".to_owned(),
                json!({"name": relation, "held": relation_findings.is_empty()}),
            );
            if relation_findings.is_empty() {
                tally.add(
                    "tp",
                    &[("language", &metadata.language), ("axis_kind", "relation")],
                )?;
            } else {
                for finding in relation_findings {
                    tally.add(
                        "fp",
                        &[("language", &metadata.language), ("axis_kind", "relation")],
                    )?;
                    add_finding(result_object, finding);
                }
            }
        }
        result_object.insert(
            "pending".to_owned(),
            metadata.pending.clone().map_or(Value::Null, Value::String),
        );
        result_object.insert(
            "derived_blocked_by".to_owned(),
            expected
                .get("derived_blocked_by")
                .cloned()
                .unwrap_or(Value::Null),
        );
        if expected
            .get("deterministic")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            let second = invocation.run(root, &case_dir, &org, &repo)?;
            let deterministic = observation.raw == second.raw;
            result_object.insert("deterministic".to_owned(), Value::Bool(deterministic));
            if !deterministic {
                add_finding(
                    result_object,
                    "two extractions of the same tree produced different bytes; the difference is in the producer, not the input"
                        .to_owned(),
                );
            }
        }
        result_object.insert(
            "kind".to_owned(),
            metadata.kind.clone().map_or(Value::Null, Value::String),
        );
        let has_findings = !array(&result, "findings").is_empty();
        if metadata.pending.is_some() {
            if !has_findings {
                stale.push(format!(
                    "STALE {slash_key}: pending on {} and passing; remove the marker",
                    metadata.pending.as_deref().unwrap_or_default()
                ));
                failed = true;
            }
        } else if has_findings {
            failed = true;
        }
        cases.insert(slash_key, result);
    }
    if corpus_revision(root)? != revision {
        return Err(CorpusError::Invariant(
            "the corpus changed while scoring; no observation can identify two revisions"
                .to_owned(),
        ));
    }
    Ok(ScoreRun {
        report: ScoreReport {
            schema_version: 2,
            corpus_revision: revision,
            case_digests: digests,
            producer_invocation: invocation.clone(),
            scored_cases: cases.len(),
            confusion: tally.as_value(),
            cases,
        },
        failed,
        stale,
    })
}
