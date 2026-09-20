//! Typed, deterministic corpus bounds, identity, and scoring operations.
//!
//! Source repositories below `fixtures/**/input` are opaque fixture data. This
//! crate reads their bytes only for revision identity and passes their paths to
//! a selected producer; it does not interpret their source languages.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod bounds;
pub mod coverage;
pub mod digest;
pub mod error;
pub mod producer;
pub mod refresh;
pub mod score;

pub use error::{CorpusError, Result};
