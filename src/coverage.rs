//! Validation of Quire coverage observations.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::{CorpusError, Result};

#[derive(Clone, Debug, Deserialize)]
struct Totals {
    backed: u64,
    total: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct Row {
    document: String,
    row_id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Diagnostic {
    reason: String,
    message: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CoverageReport {
    totals: Totals,
    status_lies: Vec<Row>,
    unbacked_rows: Vec<Row>,
    no_symbol_rows: Vec<Row>,
    diagnostics: Vec<Diagnostic>,
}

/// Coverage gate outcome with human-readable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageOutcome {
    /// Test Matrix rows backed by a real tracking tag.
    pub backed: u64,
    /// Total Test Matrix rows reconciled.
    pub total: u64,
    /// Count of rows with no backing tag.
    pub unbacked: usize,
    /// Count of unbacked rows whose verification method explains the gap.
    pub explained: usize,
    /// Human-readable descriptions of every false-green condition found.
    pub problems: Vec<String>,
}

impl CoverageOutcome {
    /// Whether the report contains a false-green condition.
    #[must_use]
    pub fn failed(&self) -> bool {
        !self.problems.is_empty()
    }
}

/// Validate a complete JSON observation written by `quire coverage`.
pub fn validate_report(bytes: &[u8]) -> Result<CoverageOutcome> {
    let report: CoverageReport =
        serde_json::from_slice(bytes).map_err(|source| CorpusError::Json {
            context: "quire coverage".to_owned(),
            source,
        })?;
    let explained: BTreeSet<_> = report
        .no_symbol_rows
        .iter()
        .map(|row| (row.document.as_str(), row.row_id.as_str()))
        .collect();
    let mut problems = Vec::new();
    if report.totals.total == 0 {
        problems.push("nothing was reconciled — the matrix or module path is wrong".to_owned());
    }
    for row in &report.status_lies {
        problems.push(format!(
            "{} claims a passing status and is unbacked ({})",
            row.row_id, row.document
        ));
    }
    for row in &report.unbacked_rows {
        if !explained.contains(&(row.document.as_str(), row.row_id.as_str())) {
            problems.push(format!(
                "{} is unbacked and its verification method does not explain why ({})",
                row.row_id, row.document
            ));
        }
    }
    const HOLLOW: [&str; 5] = [
        "id-column-matches-nothing",
        "archetype-matches-nothing",
        "section-holds-no-table",
        "marker-form-mismatch",
        "hollow-denominator",
    ];
    for diagnostic in &report.diagnostics {
        if HOLLOW.contains(&diagnostic.reason.as_str()) {
            let compact = diagnostic
                .message
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let short: String = compact.chars().take(160).collect();
            problems.push(format!("{}: {short}", diagnostic.reason));
        }
    }
    Ok(CoverageOutcome {
        backed: report.totals.backed,
        total: report.totals.total,
        unbacked: report.unbacked_rows.len(),
        explained: explained.len(),
        problems,
    })
}

/// Run a selected Quire executable directly and validate its complete report.
pub fn run_quire(quire: &Path, root: &Path, module: &Path) -> Result<CoverageOutcome> {
    let output = Command::new(quire)
        .args(["coverage", "--scope"])
        .arg(root)
        .args(["--module"])
        .arg(module)
        .arg("--json")
        .output()
        .map_err(|source| CorpusError::Spawn {
            program: quire.display().to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(CorpusError::Process {
            program: quire.display().to_string(),
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
    validate_report(&output.stdout)
}
