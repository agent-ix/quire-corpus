//! Deterministic producer-criteria pin refresh.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use crate::error::read_text;
use crate::{CorpusError, Result};

fn git(repo: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(arguments)
        .output()
        .map_err(|source| CorpusError::Spawn {
            program: "git".to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(CorpusError::Process {
            program: "git".to_owned(),
            status: output
                .status
                .code()
                .map_or_else(|| "by signal".to_owned(), |code| code.to_string()),
            stderr: String::from_utf8_lossy(&output.stderr)
                .trim()
                .chars()
                .take(400)
                .collect(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| {
            CorpusError::Invariant(format!("cannot walk {}: {error}", root.display()))
        })?;
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|value| value == "md")
        {
            paths.push(entry.into_path());
        }
    }
    paths.sort();
    Ok(paths)
}

fn repository_name(remote: &str) -> Result<String> {
    let without_slash = remote.trim_end_matches('/');
    let trimmed = without_slash.strip_suffix(".git").unwrap_or(without_slash);
    let candidate = trimmed.rsplit(['/', ':']).next().unwrap_or_default();
    if candidate.is_empty()
        || !candidate.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        return Err(CorpusError::Invariant(format!(
            "cannot derive repository name from origin {remote:?}"
        )));
    }
    Ok(candidate.to_owned())
}

/// Result of refreshing one producer criterion pin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshOutcome {
    pub producer: String,
    pub revision: String,
    pub criteria: usize,
    pub path: PathBuf,
}

/// Refresh one producer's committed criterion snapshot.
pub fn refresh(corpus_root: &Path, producer_repo: &Path) -> Result<RefreshOutcome> {
    let spec = producer_repo.join("spec");
    if !spec.is_dir() {
        return Err(CorpusError::Invariant(format!(
            "{} is not a directory",
            spec.display()
        )));
    }
    let revision = git(producer_repo, &["rev-parse", "HEAD"])?;
    let branch = git(producer_repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let remote = git(producer_repo, &["remote", "get-url", "origin"])?;
    let producer = repository_name(&remote)?;
    let criterion = Regex::new(r"(?m)^\| ((?:FR|NFR|StR)-\d+-(?:AC|CON|VC)-\d+)")
        .map_err(|error| CorpusError::Invariant(error.to_string()))?;
    let mut criteria = BTreeSet::new();
    for document in markdown_files(&spec)? {
        for found in criterion.captures_iter(&read_text(&document)?) {
            criteria.insert(found[1].to_owned());
        }
    }
    let pin = corpus_root
        .join("producers")
        .join(format!("{producer}.criteria.yaml"));
    let header = std::fs::read_to_string(&pin)
        .ok()
        .and_then(|text| text.split("producer:").next().map(str::to_owned))
        .unwrap_or_default();
    let mut rendered = format!(
        "{header}producer: agent-ix/{producer}\nrevision: {revision}\nbranch: {branch}\ncriteria:\n"
    );
    for identifier in &criteria {
        rendered.push_str(&format!("- {identifier}\n"));
    }
    std::fs::write(&pin, rendered).map_err(|source| CorpusError::Write {
        path: pin.clone(),
        source,
    })?;
    Ok(RefreshOutcome {
        producer,
        revision,
        criteria: criteria.len(),
        path: pin,
    })
}
