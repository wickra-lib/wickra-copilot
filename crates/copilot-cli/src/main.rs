//! `wickra-copilot` — the command-line reference consumer.
//!
//! It builds a deterministic `MarketContext` from serialized microstructure
//! feeds and, with the `ask` subcommand, hands that context to a user-configured
//! LLM for a natural-language explanation.

use std::process::ExitCode;

use clap::Parser;

mod args;
mod run;

/// Parse the arguments, run the command, and print the result. Errors are
/// reported on stderr with a non-zero exit code.
fn main() -> ExitCode {
    let cli = args::Cli::parse();
    match run::run(cli) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
