//! Rust-native producer and Quire stand-in used by the corpus qualification suite.

use std::process::ExitCode;

use serde_json::json;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|argument| argument == "coverage")
    {
        println!(
            "{}",
            json!({
                "totals": {"backed": 1, "total": 1},
                "status_lies": [],
                "unbacked_rows": [],
                "no_symbol_rows": [],
                "diagnostics": [],
            })
        );
        return ExitCode::SUCCESS;
    }
    if arguments.iter().any(|argument| argument == "--exit=3") {
        eprintln!("requested failure");
        return ExitCode::from(3);
    }
    if arguments
        .iter()
        .any(|argument| argument == "--invalid-json")
    {
        println!("not json");
        return ExitCode::SUCCESS;
    }
    if arguments
        .iter()
        .any(|argument| argument == "--mutate-input")
        && let Some(input) = arguments.last()
    {
        std::fs::write(
            std::path::Path::new(input).join("producer-mutation"),
            "changed",
        )
        .expect("write deliberate producer mutation");
    }
    let run = arguments
        .iter()
        .find_map(|argument| argument.strip_prefix("--counter-file="))
        .map_or(0, |path| {
            let path = std::path::Path::new(path);
            let current = std::fs::read_to_string(path)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            std::fs::write(path, (current + 1).to_string()).expect("write test counter");
            current
        });
    println!(
        "{}",
        json!({
            "nodes": [],
            "edges": [],
            "mentions": [],
            "diagnostics": [],
            "stats": {"unresolved_calls": 0},
            "observed_arguments": arguments,
            "run": run,
        })
    );
    ExitCode::SUCCESS
}
