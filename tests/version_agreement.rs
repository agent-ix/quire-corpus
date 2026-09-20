//! The built binary's version surfaces must agree with each other.
//!
//! Provenance: `agent-ix/quire-research#69`. The check itself lives in
//! `ix-cli-kit`; the ecosystem's only prior implementation was
//! `quoin/scripts/check-version-agreement.mjs`, in Node, and no Rust CLI had
//! one. It runs the **built binary** on purpose: a test that reads the same
//! constant the binary bakes in always agrees with itself.
//!
//! Not `#[trace]`-tagged: this repository traces every test to a criterion,
//! and no criterion covers version agreement yet. Inventing a TC id and an
//! acceptance-criterion id here would mint untracked symbols. The spec cycle
//! on #69 mints them.

use std::path::Path;

use ix_cli_kit::version::Agreement;

const CLI: &str = env!("CARGO_BIN_EXE_quire-corpus");

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
