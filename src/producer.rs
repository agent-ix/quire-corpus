//! Structured producer process boundary.

use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use regex::{Captures, Regex};
use serde::Serialize;
use serde_json::Value;

use crate::{CorpusError, Result};

/// A version-2 producer executable and ordered argument templates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProducerInvocation {
    /// Executable passed directly to the operating system.
    pub executable: String,
    /// Ordered arguments in which `{org}`, `{repo}`, and `{input}` are replaced.
    pub arguments: Vec<String>,
}

/// Parsed JSON plus the producer's exact standard-output bytes as UTF-8 text.
#[derive(Clone, Debug)]
pub struct ProducerObservation {
    pub value: Value,
    pub raw: String,
}

fn limited(text: &str, characters: usize) -> String {
    text.chars().take(characters).collect()
}

fn render_argument(template: &str, org: &str, repo: &str, input: &str) -> String {
    static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\{(?:org|repo|input)\}").expect("the fixed placeholder regex is valid")
    });
    PLACEHOLDER
        .replace_all(template, |matched: &Captures<'_>| {
            if matched[0] == *"{org}" {
                org
            } else if matched[0] == *"{repo}" {
                repo
            } else {
                input
            }
        })
        .into_owned()
}

impl ProducerInvocation {
    /// Execute a producer directly, without a shell.
    pub fn run(
        &self,
        corpus_root: &Path,
        case_dir: &Path,
        org: &str,
        repo: &str,
    ) -> Result<ProducerObservation> {
        let joined_input = case_dir.join("input");
        let input = if case_dir.file_name().is_some_and(|name| name == "variant") {
            case_dir
        } else {
            joined_input.as_path()
        };
        let input = input.to_string_lossy();
        let arguments: Vec<_> = self
            .arguments
            .iter()
            .map(|argument| render_argument(argument, org, repo, &input))
            .collect();
        let output = Command::new(&self.executable)
            .args(&arguments)
            .current_dir(corpus_root)
            .output()
            .map_err(|source| CorpusError::Spawn {
                program: self.executable.clone(),
                source,
            })?;
        if !output.status.success() {
            return Err(CorpusError::Process {
                program: self.executable.clone(),
                status: output
                    .status
                    .code()
                    .map_or_else(|| "by signal".to_owned(), |code| code.to_string()),
                stderr: limited(&String::from_utf8_lossy(&output.stderr), 400),
            });
        }
        let raw = String::from_utf8(output.stdout).map_err(|error| {
            CorpusError::Invariant(format!(
                "producer output on {} is not UTF-8: {error}",
                case_dir.display()
            ))
        })?;
        let value = serde_json::from_str(&raw).map_err(|source| CorpusError::Json {
            context: format!("producer on {}", case_dir.display()),
            source,
        })?;
        Ok(ProducerObservation { value, raw })
    }
}
