//! The built binary's version surfaces must agree with each other.
//!
//! It runs the built binary so the observed surfaces are independent of this
//! test's expectation.

use std::path::Path;

use ix_cli_kit::version::Agreement;
use ix_trace_rs::trace;

const CLI: &str = env!("CARGO_BIN_EXE_quire-corpus");

#[trace("TC-062", "FR-007-AC-7")]
#[test]
fn version_surfaces_of_the_built_cli_agree() {
    let report = Agreement::new()
        .surface("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))
        .command("--version", Path::new(CLI), &["--version"])
        .command("--help", Path::new(CLI), &["--help"])
        .against_git_tag(Path::new(env!("CARGO_MANIFEST_DIR")))
        .assert_agreed();

    assert_eq!(report.version, env!("CARGO_PKG_VERSION"));
}
