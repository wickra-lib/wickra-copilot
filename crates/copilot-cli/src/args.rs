//! Command-line arguments for `wickra-copilot` (§3.2).
//!
//! Two subcommands share the same feed inputs: `context` builds and prints the
//! deterministic market context, and `ask` additionally sends it to a
//! configured LLM. The runner that consumes these arguments lands in the next
//! unit, so the fields are transiently unread here.
#![allow(dead_code)]

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use copilot_llm::Provider;

/// Build a deterministic market context from microstructure feeds, and
/// optionally ask an LLM to explain it.
#[derive(Parser, Debug)]
#[command(name = "wickra-copilot", version, about)]
pub struct Cli {
    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The `wickra-copilot` subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Build the market context and print its derived facts.
    Context(FeedInput),
    /// Build the context and ask a configured LLM to explain it.
    Ask(AskArgs),
}

/// The spec and feed inputs shared by both subcommands.
#[derive(Args, Debug)]
pub struct FeedInput {
    /// Path to the spec file (JSON or TOML, chosen by extension).
    #[arg(long)]
    pub spec: PathBuf,

    /// Directory of per-symbol feed files (`<SYMBOL>.json`, one `FeedSnapshot`
    /// each); the symbol is the file name without its extension.
    #[arg(long, conflicts_with = "stdin")]
    pub feeds: Option<PathBuf>,

    /// Read the whole feed universe as one JSON object (`{"SYM":{...},...}`)
    /// from stdin instead of `--feeds`.
    #[arg(long, conflicts_with = "feeds")]
    pub stdin: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    pub format: Format,
}

/// Arguments for `ask`: the shared feed inputs plus the question and provider.
#[derive(Args, Debug)]
pub struct AskArgs {
    /// The shared spec and feed inputs.
    #[command(flatten)]
    pub input: FeedInput,

    /// The question to ask about the market context.
    #[arg(long)]
    pub question: String,

    /// Which LLM provider preset to use. Defaults to the local Ollama server,
    /// which needs no API key.
    #[arg(long, value_enum, default_value_t = ProviderArg::Ollama)]
    pub provider: ProviderArg,

    /// Override the provider's default model.
    #[arg(long)]
    pub model: Option<String>,
}

/// How to render the context.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// An aligned human-readable table of facts.
    Text,
    /// The `MarketContext` serialized as JSON.
    Json,
}

/// The LLM provider presets exposed on the command line.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderArg {
    /// Local Ollama server — no API key required.
    Ollama,
    /// OpenAI.
    Openai,
    /// Anthropic's OpenAI-compatible endpoint.
    Claude,
    /// Google Gemini's OpenAI-compatible endpoint.
    Gemini,
    /// A user-supplied endpoint, configured from the environment.
    Custom,
}

impl From<ProviderArg> for Provider {
    fn from(arg: ProviderArg) -> Self {
        match arg {
            ProviderArg::Ollama => Self::Ollama,
            ProviderArg::Openai => Self::Openai,
            ProviderArg::Claude => Self::Claude,
            ProviderArg::Gemini => Self::Gemini,
            ProviderArg::Custom => Self::Custom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_parses_feed_inputs() {
        let cli = Cli::try_parse_from([
            "wickra-copilot",
            "context",
            "--spec",
            "s.json",
            "--feeds",
            "feeddir",
            "--format",
            "json",
        ])
        .unwrap();
        let Command::Context(input) = cli.command else {
            panic!("expected the context subcommand");
        };
        assert_eq!(input.spec, PathBuf::from("s.json"));
        assert_eq!(input.feeds, Some(PathBuf::from("feeddir")));
        assert!(!input.stdin);
        assert_eq!(input.format, Format::Json);
    }

    #[test]
    fn ask_defaults_provider_to_ollama_and_format_to_text() {
        let cli = Cli::try_parse_from([
            "wickra-copilot",
            "ask",
            "--spec",
            "s.json",
            "--stdin",
            "--question",
            "why did BTC dump",
        ])
        .unwrap();
        let Command::Ask(ask) = cli.command else {
            panic!("expected the ask subcommand");
        };
        assert!(ask.input.stdin);
        assert_eq!(ask.input.format, Format::Text);
        assert_eq!(ask.question, "why did BTC dump");
        assert_eq!(ask.provider, ProviderArg::Ollama);
        assert!(ask.model.is_none());
    }

    #[test]
    fn feeds_and_stdin_conflict() {
        let err = Cli::try_parse_from([
            "wickra-copilot",
            "context",
            "--spec",
            "s.json",
            "--feeds",
            "feeddir",
            "--stdin",
        ]);
        assert!(err.is_err());
    }

    #[test]
    fn provider_arg_maps_to_the_llm_provider() {
        assert_eq!(Provider::from(ProviderArg::Ollama), Provider::Ollama);
        assert_eq!(Provider::from(ProviderArg::Openai), Provider::Openai);
        assert_eq!(Provider::from(ProviderArg::Claude), Provider::Claude);
        assert_eq!(Provider::from(ProviderArg::Gemini), Provider::Gemini);
        assert_eq!(Provider::from(ProviderArg::Custom), Provider::Custom);
    }
}
