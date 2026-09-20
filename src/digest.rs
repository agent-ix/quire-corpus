//! Corpus and per-case BLAKE2b-256 identity.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use blake2::Blake2bVar;
use blake2::digest::{Update, VariableOutput};
use serde::Serialize;
use walkdir::WalkDir;

use crate::bounds::discover;
use crate::error::read_bytes;
use crate::{CorpusError, Result};

/// The deterministic digest observation emitted by `digest`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DigestReport {
    /// The report's own schema version, for forward compatibility.
    pub schema_version: u32,
    /// The whole-corpus BLAKE2b-256 digest, over `corpus.yaml` and every fixture file.
    pub corpus_revision: String,
    /// Per-case BLAKE2b-256 digests, keyed by slash-separated case name.
    pub cases: BTreeMap<String, String>,
}

fn files_below(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| {
            CorpusError::Invariant(format!("cannot walk {}: {error}", root.display()))
        })?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}

fn digest_paths(root: &Path, paths: &[PathBuf]) -> Result<String> {
    let mut hasher = Blake2bVar::new(32)
        .map_err(|error| CorpusError::Invariant(format!("invalid digest size: {error}")))?;
    for path in paths {
        let relative = path
            .strip_prefix(root)
            .map_err(|error| CorpusError::Invariant(error.to_string()))?;
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update(&[0]);
        hasher.update(&read_bytes(path)?);
        hasher.update(&[0]);
    }
    let mut output = [0_u8; 32];
    hasher
        .finalize_variable(&mut output)
        .map_err(|error| CorpusError::Invariant(format!("cannot finalize digest: {error}")))?;
    Ok(hex(&output))
}

/// Derive every case digest in slash-key order.
pub fn case_digests(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (key, case_dir) in discover(root)? {
        out.insert(
            key.slash_name(),
            digest_paths(root, &files_below(&case_dir)?)?,
        );
    }
    Ok(out)
}

/// Derive the complete corpus revision from the manifest and fixture tree.
pub fn corpus_revision(root: &Path) -> Result<String> {
    let mut files = vec![root.join("corpus.yaml")];
    files.extend(files_below(&root.join("fixtures"))?);
    digest_paths(root, &files)
}

/// Derive the complete digest report.
pub fn report(root: &Path) -> Result<DigestReport> {
    Ok(DigestReport {
        schema_version: 1,
        corpus_revision: corpus_revision(root)?,
        cases: case_digests(root)?,
    })
}
