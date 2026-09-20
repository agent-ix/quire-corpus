//! Corpus inventory discovery and fail-closed bounds reporting.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::read_text;
use crate::{CorpusError, Result};

/// A stable `(family, case, language)` fixture identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct CaseKey(pub String, pub String, pub String);

impl CaseKey {
    /// Render the slash-separated public case key.
    #[must_use]
    pub fn slash_name(&self) -> String {
        format!("{}/{}/{}", self.0, self.1, self.2)
    }
}

/// One inventory cell, in either compact or reasoned form.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum DeclaredCell {
    /// Every listed language is required.
    Languages(Vec<String>),
    /// Required languages plus explicitly reasoned exclusions.
    Detailed {
        /// Languages this case must still cover.
        #[serde(default)]
        languages: Vec<String>,
        /// Languages this case will never cover, keyed to why.
        #[serde(default)]
        out_of_scope: BTreeMap<String, String>,
    },
}

impl DeclaredCell {
    fn parts(&self) -> Result<(&[String], &BTreeMap<String, String>)> {
        static EMPTY: std::sync::LazyLock<BTreeMap<String, String>> =
            std::sync::LazyLock::new(BTreeMap::new);
        match self {
            Self::Languages(languages) => Ok((languages, &EMPTY)),
            Self::Detailed {
                languages,
                out_of_scope,
            } => {
                for (language, reason) in out_of_scope {
                    if reason.trim().is_empty() {
                        return Err(CorpusError::Invariant(format!(
                            "out_of_scope[{language}] has no reason; an exclusion without one is a gap wearing a different word"
                        )));
                    }
                }
                Ok((languages, out_of_scope))
            }
        }
    }
}

/// A family name held for later work.
#[derive(Clone, Debug, Deserialize)]
pub struct ReservedFamily {
    /// Owning issue.
    pub issue_ref: String,
    /// Why the family is not populated yet.
    pub reason: String,
}

/// One producer's pinned criterion coverage declaration.
#[derive(Clone, Debug, Deserialize)]
pub struct ProducerDeclaration {
    /// Repository-relative pin file.
    pub criteria_pin: String,
    /// Reason name to prose containing criterion identifiers.
    #[serde(default)]
    pub unreachable: BTreeMap<String, String>,
}

/// The parts of `corpus.yaml` interpreted by the bounds engine.
#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    /// Declared case/language matrix.
    pub inventory: BTreeMap<String, BTreeMap<String, DeclaredCell>>,
    /// Reserved but deliberately unpopulated families.
    #[serde(default)]
    pub reserved_families: BTreeMap<String, ReservedFamily>,
    /// Producers whose criteria coverage is measured.
    #[serde(default)]
    pub producers: BTreeMap<String, ProducerDeclaration>,
}

/// The case metadata used by inventory and scoring.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct CaseMetadata {
    /// Fixture source language.
    #[serde(default)]
    pub language: String,
    /// Positive/control classification.
    pub kind: Option<String>,
    /// Owning issue.
    pub issue_ref: Option<String>,
    /// An issue that intentionally holds a failing expectation open.
    pub pending: Option<String>,
    /// Positive case named by a control.
    pub control_for: Option<String>,
    /// Control case named by a positive.
    pub control_for_pair: Option<String>,
    /// Non-local control language.
    pub control_language: Option<String>,
    /// Producer criteria reached by this fixture.
    #[serde(default)]
    pub criteria: Vec<String>,
}

/// Derived criterion coverage for one producer.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum CriterionCoverage {
    /// The committed pin could not be read.
    Unavailable {
        /// Human-readable reason the pin file could not be loaded.
        why: String,
    },
    /// Coverage was derived from the pin and fixture claims.
    Measured {
        /// Total criteria declared by the pin.
        total: usize,
        /// Criteria claimed by at least one fixture.
        reached: Vec<String>,
        /// Criteria declared unreachable, keyed to why.
        unreachable: BTreeMap<String, String>,
        /// Criteria reached by no fixture and declared unreachable by nothing.
        unreached: Vec<String>,
        /// Criteria declared unreachable that the pin no longer states.
        stale_unreachable: Vec<String>,
        /// Criteria claimed by a fixture but absent from the pin.
        claimed_but_undeclared: Vec<String>,
    },
}

/// Complete derived bounds report.
#[derive(Clone, Debug, Serialize)]
pub struct BoundsReport {
    /// Per-producer criterion coverage, keyed by producer name.
    pub criterion_coverage: BTreeMap<String, CriterionCoverage>,
    /// Declared cases that exist on disk.
    pub covered: Vec<CaseKey>,
    /// Declared cases missing from disk.
    pub gaps: Vec<CaseKey>,
    /// The number of entries in `gaps`.
    pub gap_count: usize,
    /// `(family, case, language, reason)` tuples excluded from scope.
    pub scoped_out: Vec<(String, String, String, String)>,
    /// Cases found on disk but not declared in the manifest.
    pub undeclared: Vec<CaseKey>,
    /// Human-readable descriptions of every inventory or coverage defect found.
    pub problems: Vec<String>,
    /// Positive cases named as the target of a control case.
    pub controls: Vec<CaseKey>,
}

#[derive(Debug, Deserialize)]
struct CriteriaPin {
    revision: Option<String>,
    #[serde(default)]
    criteria: BTreeSet<String>,
}

/// Load the corpus manifest from `root/corpus.yaml`.
pub fn load_manifest(root: &Path) -> Result<Manifest> {
    let path = root.join("corpus.yaml");
    let text = read_text(&path)?;
    serde_yaml::from_str(&text).map_err(|source| CorpusError::Yaml { path, source })
}

fn load_case_metadata(path: &Path) -> Result<CaseMetadata> {
    let text = read_text(path)?;
    serde_yaml::from_str(&text).map_err(|source| CorpusError::Yaml {
        path: path.to_path_buf(),
        source,
    })
}

/// Discover every `fixtures/<family>/<case>/<language>/case.yaml`.
pub fn discover(root: &Path) -> Result<BTreeMap<CaseKey, PathBuf>> {
    let fixtures = root.join("fixtures");
    let mut case_files = Vec::new();
    for entry in WalkDir::new(&fixtures).follow_links(false) {
        let entry = entry.map_err(|error| {
            CorpusError::Invariant(format!("cannot walk {}: {error}", fixtures.display()))
        })?;
        if entry.file_type().is_file() && entry.file_name() == "case.yaml" {
            case_files.push(entry.into_path());
        }
    }
    case_files.sort();

    let mut found = BTreeMap::new();
    for case_file in case_files {
        let relative = case_file
            .strip_prefix(&fixtures)
            .map_err(|error| CorpusError::Invariant(error.to_string()))?;
        let parts: Vec<_> = relative.iter().collect();
        if parts.len() != 4 {
            return Err(CorpusError::Invariant(format!(
                "{} is not at the four-segment path fixtures/<family>/<case>/<language>/case.yaml",
                case_file.display()
            )));
        }
        let key = CaseKey(
            parts[0].to_string_lossy().into_owned(),
            parts[1].to_string_lossy().into_owned(),
            parts[2].to_string_lossy().into_owned(),
        );
        let parent = case_file.parent().ok_or_else(|| {
            CorpusError::Invariant(format!("{} has no parent", case_file.display()))
        })?;
        found.insert(key, parent.to_path_buf());
    }
    Ok(found)
}

fn parse_unreachable(declaration: &ProducerDeclaration) -> Result<BTreeMap<String, String>> {
    let criterion = Regex::new(r"(?:FR|NFR|StR)-\d+-(?:AC|CON|VC)-\d+")
        .map_err(|error| CorpusError::Invariant(error.to_string()))?;
    let mut out = BTreeMap::new();
    for (reason, text) in &declaration.unreachable {
        for found in criterion.find_iter(text) {
            out.insert(found.as_str().to_owned(), reason.clone());
        }
    }
    Ok(out)
}

/// Derive the complete inventory, control, and criterion-coverage report.
pub fn audit(root: &Path) -> Result<BoundsReport> {
    let manifest = load_manifest(root)?;
    let on_disk = discover(root)?;
    let mut cells = BTreeSet::new();
    let mut gaps = BTreeSet::new();
    let mut scoped_out = BTreeSet::new();
    let mut problems = BTreeSet::new();

    for (family, cases) in &manifest.inventory {
        for (case, declaration) in cases {
            let (languages, exclusions) = declaration.parts()?;
            for language in languages {
                let key = CaseKey(family.clone(), case.clone(), language.clone());
                if on_disk.contains_key(&key) {
                    cells.insert(key);
                } else {
                    gaps.insert(key);
                }
            }
            for (language, reason) in exclusions {
                scoped_out.insert((
                    family.clone(),
                    case.clone(),
                    language.clone(),
                    reason.clone(),
                ));
                let key = CaseKey(family.clone(), case.clone(), language.clone());
                if on_disk.contains_key(&key) {
                    problems.insert(format!(
                        "{}/{}/{} is scoped out and also has a fixture; one of the two is wrong",
                        family, case, language
                    ));
                }
            }
        }
    }

    let declared: BTreeSet<_> = cells.union(&gaps).cloned().collect();
    let undeclared: BTreeSet<_> = on_disk
        .keys()
        .filter(|key| !declared.contains(*key))
        .cloned()
        .collect();
    let mut controls = BTreeSet::new();
    let mut claimed = BTreeSet::new();

    for (key, path) in &on_disk {
        let metadata = load_case_metadata(&path.join("case.yaml"))?;
        claimed.extend(metadata.criteria);
        if metadata.kind.as_deref() == Some("control")
            && let Some(control_for) = metadata.control_for.filter(|value| !value.is_empty())
        {
            controls.insert(CaseKey(key.0.clone(), control_for, key.2.clone()));
        }
        if metadata
            .issue_ref
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            problems.insert(format!("{} has no issue_ref", key.slash_name()));
        }
        if metadata
            .pending
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            problems.insert(format!(
                "{} is pending on nothing; a marker with no issue is a case nobody comes back to",
                key.slash_name()
            ));
        }
        if !path.join("expected.yaml").is_file() {
            problems.insert(format!("{} has no expected.yaml", key.slash_name()));
        }
        if !path.join("input").is_dir() {
            problems.insert(format!("{} has no input tree", key.slash_name()));
        }
        if metadata.kind.as_deref() == Some("positive")
            && let Some(pair) = metadata.control_for_pair
        {
            let language = metadata.control_language.unwrap_or_else(|| key.2.clone());
            if !on_disk.contains_key(&CaseKey(key.0.clone(), pair.clone(), language.clone())) {
                problems.insert(format!(
                    "{} names control {pair:?} in {language}, which does not exist",
                    key.slash_name()
                ));
            }
        }
    }

    let mut criterion_coverage = BTreeMap::new();
    for (name, declaration) in &manifest.producers {
        let pin_path = root.join(&declaration.criteria_pin);
        let Ok(pin_text) = std::fs::read_to_string(&pin_path) else {
            criterion_coverage.insert(
                name.clone(),
                CriterionCoverage::Unavailable {
                    why: format!("{} is not readable from here", pin_path.display()),
                },
            );
            continue;
        };
        let pin: CriteriaPin =
            serde_yaml::from_str(&pin_text).map_err(|source| CorpusError::Yaml {
                path: pin_path.clone(),
                source,
            })?;
        let _revision = pin.revision;
        let unreachable = parse_unreachable(declaration)?;
        let reached: Vec<_> = claimed.intersection(&pin.criteria).cloned().collect();
        let unreached: Vec<_> = pin
            .criteria
            .difference(&claimed)
            .filter(|identifier| !unreachable.contains_key(*identifier))
            .cloned()
            .collect();
        let stale: Vec<_> = unreachable
            .keys()
            .filter(|identifier| !pin.criteria.contains(*identifier))
            .cloned()
            .collect();
        let phantom: Vec<_> = claimed.difference(&pin.criteria).cloned().collect();
        for identifier in &unreached {
            problems.insert(format!(
                "{name} {identifier} is reached by no case and declared unreachable by nothing"
            ));
        }
        for identifier in &stale {
            problems.insert(format!(
                "{name} {identifier} is declared unreachable and the producer no longer states it"
            ));
        }
        for identifier in &phantom {
            problems.insert(format!(
                "{name} {identifier} is claimed by a case and stated by no requirement"
            ));
        }
        criterion_coverage.insert(
            name.clone(),
            CriterionCoverage::Measured {
                total: pin.criteria.len(),
                reached,
                unreachable,
                unreached,
                stale_unreachable: stale,
                claimed_but_undeclared: phantom,
            },
        );
    }

    Ok(BoundsReport {
        criterion_coverage,
        covered: cells.into_iter().collect(),
        gaps: gaps.iter().cloned().collect(),
        gap_count: gaps.len(),
        scoped_out: scoped_out.into_iter().collect(),
        undeclared: undeclared.into_iter().collect(),
        problems: problems.into_iter().collect(),
        controls: controls.into_iter().collect(),
    })
}

impl BoundsReport {
    /// Whether any fail-closed inventory condition is present.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.gap_count != 0 || !self.undeclared.is_empty() || !self.problems.is_empty()
    }
}
