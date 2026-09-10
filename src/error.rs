//! Error contract shared by the library and CLI.

use std::path::PathBuf;

/// A rejected corpus declaration, observation, or host operation.
#[derive(Debug, thiserror::Error)]
pub enum CorpusError {
    /// A required file or directory could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// A file could not be written.
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// YAML did not satisfy the declared data shape.
    #[error("invalid YAML in {path}: {source}")]
    Yaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    /// JSON did not satisfy the declared data shape.
    #[error("invalid JSON from {context}: {source}")]
    Json {
        context: String,
        #[source]
        source: serde_json::Error,
    },
    /// A corpus invariant was false.
    #[error("{0}")]
    Invariant(String),
    /// A selected process could not be started.
    #[error("cannot execute {program}: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    /// A selected process returned a non-success status.
    #[error("{program} exited {status}: {stderr}")]
    Process {
        program: String,
        status: String,
        stderr: String,
    },
}

/// Result type for corpus operations.
pub type Result<T> = std::result::Result<T, CorpusError>;

/// Read a UTF-8 file with path context.
pub(crate) fn read_text(path: &std::path::Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| CorpusError::Read {
        path: path.to_path_buf(),
        source,
    })
}

/// Read bytes with path context.
pub(crate) fn read_bytes(path: &std::path::Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|source| CorpusError::Read {
        path: path.to_path_buf(),
        source,
    })
}
