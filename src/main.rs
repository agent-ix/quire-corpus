//! `quire-corpus` command-line adapter.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ix_cli_kit::streams::{ColorChoice, Diagnostics, DiagnosticsFormat, Record};
use ix_cli_kit::{Outcome, json as kit_json};
use serde::Serialize;
use serde_json::Value;

use quire_corpus::bounds::{CriterionCoverage, audit};
use quire_corpus::coverage;
use quire_corpus::digest;
use quire_corpus::producer::ProducerInvocation;
use quire_corpus::refresh;
use quire_corpus::score;
use quire_corpus::{CorpusError, Result};

#[derive(Debug, Parser)]
#[command(name = "quire-corpus", version, about)]
struct Cli {
    /// Root containing corpus.yaml, fixtures/, producers/, and spec/.
    #[arg(long)]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Derive the inventory, controls, gaps, and criterion coverage.
    Bounds {
        /// Write the deterministic report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Derive the corpus revision and per-case digests.
    Digest {
        /// Write the deterministic report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Score a producer's canonical JSON records.
    Score {
        /// Producer executable; never evaluated by a shell.
        #[arg(long)]
        producer: String,
        /// Ordered producer argument template; repeat once per argument.
        #[arg(long = "producer-arg", allow_hyphen_values = true)]
        producer_arguments: Vec<String>,
        /// Score only this family/case/language key; repeat to select several.
        #[arg(long = "case")]
        cases: Vec<String>,
        /// Write the deterministic report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Refresh a producer's exact criteria pin.
    RefreshCriteria {
        /// Producer repository containing spec/ and an origin remote.
        #[arg(long)]
        producer_repo: PathBuf,
    },
    /// Run Quire coverage directly and reject false-green report shapes.
    CheckCoverage {
        /// Exact Quire executable to invoke.
        #[arg(long)]
        quire: PathBuf,
        /// Exact module directory supplied to Quire.
        #[arg(long)]
        module: PathBuf,
    },
}

/// Emit a report as canonical JSON on stdout.
///
/// The recursive key-sort and the encoder both moved to `ix-cli-kit`: three
/// repositories had written the same canonicaliser, and it cannot be replaced
/// by `BTreeMap` ordering because this crate enables `serde_json`'s
/// `preserve_order` feature while `quoin-core` deliberately does not.
fn print_json(value: &impl Serialize) -> Result<()> {
    let encoded = kit_json::encode_canonical(value, true).map_err(|error| CorpusError::Json {
        context: "corpus report".to_owned(),
        source: error.source,
    })?;
    ix_cli_kit::streams::emit_result(&encoded);
    Ok(())
}

fn run(cli: Cli) -> Result<bool> {
    match cli.command {
        Command::Bounds { json } => {
            let report = audit(&cli.root)?;
            if json {
                print_json(&report)?;
            } else {
                for (name, coverage) in &report.criterion_coverage {
                    match coverage {
                        CriterionCoverage::Unavailable { why } => {
                            println!("{name}: criteria unavailable — {why}");
                        }
                        CriterionCoverage::Measured {
                            total,
                            reached,
                            unreachable,
                            unreached,
                            ..
                        } => println!(
                            "{name}  {} reached, {} unreachable, {} unreached  of {total}",
                            reached.len(),
                            unreachable.len(),
                            unreached.len()
                        ),
                    }
                }
                println!("covered      {}", report.covered.len());
                println!("gap_count    {}", report.gap_count);
                println!("scoped out   {}", report.scoped_out.len());
                for key in &report.gaps {
                    println!("  GAP        {}", key.slash_name());
                }
                for key in &report.undeclared {
                    println!("  UNDECLARED {}", key.slash_name());
                }
                for problem in &report.problems {
                    println!("  PROBLEM    {problem}");
                }
            }
            Ok(report.failed())
        }
        Command::Digest { json } => {
            let report = digest::report(&cli.root)?;
            if json {
                print_json(&report)?;
            } else {
                println!("corpus_revision  {}", report.corpus_revision);
                println!("cases            {}", report.cases.len());
                for (key, value) in report.cases {
                    println!("  {}  {key}", &value[..16]);
                }
            }
            Ok(false)
        }
        Command::Score {
            producer,
            producer_arguments,
            cases,
            json,
        } => {
            let invocation = ProducerInvocation {
                executable: producer,
                arguments: producer_arguments,
            };
            let selected = cases.into_iter().collect::<BTreeSet<_>>();
            let run = score::score(&cli.root, &invocation, &selected)?;
            for stale in &run.stale {
                eprintln!("  {stale}");
            }
            if json {
                print_json(&run.report)?;
            } else {
                let revision = &run.report.corpus_revision;
                let scored = run.report.scored_cases;
                println!(
                    "corpus_revision {}  cases {scored}",
                    revision.chars().take(16).collect::<String>()
                );
                let totals = &run.report.confusion["total"]["total"];
                println!(
                    "tp {}  fp {}  fn {}  precision {}  recall {}",
                    display_json(&totals["tp"]),
                    display_json(&totals["fp"]),
                    display_json(&totals["fn"]),
                    display_json(&totals["precision"]),
                    display_json(&totals["recall"])
                );
                for (key, result) in &run.report.cases {
                    let findings = result["findings"].as_array().map_or(&[][..], Vec::as_slice);
                    let pending = !result["pending"].is_null();
                    let mark = if findings.is_empty() {
                        "ok  "
                    } else if pending {
                        "PEND"
                    } else {
                        "FAIL"
                    };
                    println!("  {mark} {key}");
                    for finding in findings.iter().filter_map(Value::as_str) {
                        println!("       {finding}");
                    }
                }
            }
            Ok(run.failed)
        }
        Command::RefreshCriteria { producer_repo } => {
            let outcome = refresh::refresh(&cli.root, &producer_repo)?;
            println!(
                "pinned {} criteria from {} at {}",
                outcome.criteria,
                outcome.producer,
                outcome.revision.chars().take(12).collect::<String>()
            );
            Ok(false)
        }
        Command::CheckCoverage { quire, module } => {
            let outcome = coverage::run_quire(&quire, &cli.root, &module)?;
            println!(
                "coverage: {}/{} targets backed, {} unbacked row(s), {} explained by method",
                outcome.backed, outcome.total, outcome.unbacked, outcome.explained
            );
            for problem in &outcome.problems {
                println!("FAIL: {problem}");
            }
            Ok(outcome.failed())
        }
    }
}

fn display_json(value: &Value) -> String {
    match value {
        Value::Null => "None".to_owned(),
        other => other.to_string(),
    }
}

fn main() -> ExitCode {
    // Findings present and the run itself breaking were both exit 1 here. They
    // are different facts and a caller could not tell them apart: a `Partial`
    // run produced a complete report on stdout, an `Internal` one produced
    // nothing. The shared taxonomy (ix-cli-kit) spells the difference.
    let outcome = match run(Cli::parse()) {
        Ok(false) => Outcome::Ok,
        Ok(true) => Outcome::Partial,
        Err(error) => {
            let out = Diagnostics::resolved(DiagnosticsFormat::Human, ColorChoice::Auto);
            Record::error("quire-corpus", &error.to_string()).emit(out);
            Outcome::Internal
        }
    };
    outcome.into()
}
