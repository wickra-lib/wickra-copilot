//! The run pipeline: load a spec and a feed universe, build the context, and
//! render it — or, for `ask`, hand it to an LLM (§3.3).
//!
//! Only the `ask` path touches the network (through `copilot-llm`); `context`
//! is fully deterministic and offline. The public entry point is wired into
//! `main` in the next unit, so it is transiently unreferenced here.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::Path;

use copilot_core::{build_context, Config, ContextSpec, FeedSnapshot, MarketContext};
use copilot_llm::{ask, OpenAiCompatible};

use crate::args::{AskArgs, Cli, Command, FeedInput, Format};

/// Dispatch the parsed command and return the rendered output.
///
/// # Errors
/// Returns an error if a file cannot be read or parsed, no feed source is
/// given, the spec fails validation, or (for `ask`) the provider fails.
pub fn run(cli: Cli) -> Result<String, Box<dyn Error>> {
    match cli.command {
        Command::Context(input) => {
            let context = build(&input)?;
            Ok(match input.format {
                Format::Json => serde_json::to_string(&context)?,
                Format::Text => render_context(&context),
            })
        }
        Command::Ask(ask_args) => run_ask(&ask_args),
    }
}

/// Build the deterministic context from the shared feed inputs.
fn build(input: &FeedInput) -> Result<MarketContext, Box<dyn Error>> {
    let spec = load_spec(&input.spec)?;
    let feeds = load_feeds(input)?;
    Ok(build_context(&feeds, &spec)?)
}

/// Build the context, then ask the configured provider to explain it.
fn run_ask(ask_args: &AskArgs) -> Result<String, Box<dyn Error>> {
    let context = build(&ask_args.input)?;
    // The provider reads its model from the environment; a `--model` override
    // is applied the same way so the one HTTP implementation stays unchanged.
    if let Some(model) = &ask_args.model {
        std::env::set_var("WICKRA_COPILOT_MODEL", model);
    }
    let provider = OpenAiCompatible::from_provider(ask_args.provider.into())?;
    let answer = ask(&provider, &ask_args.question, &context)?;
    Ok(match ask_args.input.format {
        Format::Json => serde_json::to_string(&answer)?,
        Format::Text => format!("{}\n\n{}", answer.text, render_context(&answer.context)),
    })
}

/// Parse the spec config, choosing TOML or JSON by extension (JSON by default).
fn load_spec(path: &Path) -> Result<ContextSpec, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let is_toml = path.extension().and_then(|ext| ext.to_str()) == Some("toml");
    let config = if is_toml {
        Config::from_toml(&text)?
    } else {
        Config::from_json(&text)?
    };
    Ok(config.spec)
}

/// Load the feed universe from stdin (`--stdin`) or a directory (`--feeds`).
fn load_feeds(input: &FeedInput) -> Result<BTreeMap<String, FeedSnapshot>, Box<dyn Error>> {
    if input.stdin {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        Ok(serde_json::from_str(&text)?)
    } else if let Some(dir) = &input.feeds {
        load_feeds_dir(dir)
    } else {
        Err("either --feeds <dir> or --stdin is required".into())
    }
}

/// Assemble the universe from `<SYMBOL>.json` files in `dir` (one `FeedSnapshot`
/// each). The `BTreeMap` orders symbols by key, so the build is deterministic
/// regardless of directory-read order.
fn load_feeds_dir(dir: &Path) -> Result<BTreeMap<String, FeedSnapshot>, Box<dyn Error>> {
    let mut universe = BTreeMap::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let symbol = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or("feed file has no valid name")?
            .to_owned();
        let text = fs::read_to_string(&path)?;
        universe.insert(symbol, serde_json::from_str::<FeedSnapshot>(&text)?);
    }
    Ok(universe)
}

/// An aligned, human-readable table of the derived facts.
fn render_context(context: &MarketContext) -> String {
    if context.facts.is_empty() {
        return format!("no facts ({} symbol(s) scanned)", context.symbols.len());
    }
    let kind_width = context
        .facts
        .iter()
        .map(|fact| format!("{:?}", fact.kind).len())
        .max()
        .unwrap_or(4)
        .max(4);
    let symbol_width = context
        .facts
        .iter()
        .map(|fact| fact.symbol.len())
        .max()
        .unwrap_or(6)
        .max(6);
    let mut lines = vec![format!(
        "{:<kind_width$}  {:<symbol_width$}  {:>12}  human",
        "kind", "symbol", "value"
    )];
    for fact in &context.facts {
        lines.push(format!(
            "{:<kind_width$}  {:<symbol_width$}  {:>12.4}  {}",
            format!("{:?}", fact.kind),
            fact.symbol,
            fact.value,
            fact.human,
        ));
    }
    lines.push(format!(
        "{} fact(s), {} symbol(s) scanned",
        context.facts.len(),
        context.symbols.len()
    ));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // A config whose spec asks for BTC price moves.
    const SPEC: &str = r#"{"spec":{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}}"#;
    // BTC drops 6% over three bars -> one significant price-move fact.
    const BTC_FEED: &str = r#"{"symbol":"BTCUSDT","candles":[
        {"ts":1,"open":100.0,"high":100.0,"low":100.0,"close":100.0,"volume":1.0},
        {"ts":2,"open":97.0,"high":97.0,"low":97.0,"close":97.0,"volume":1.0},
        {"ts":3,"open":94.0,"high":94.0,"low":94.0,"close":94.0,"volume":1.0}]}"#;

    fn built_context() -> MarketContext {
        let spec = Config::from_json(SPEC).unwrap().spec;
        let mut feeds = BTreeMap::new();
        feeds.insert(
            "BTCUSDT".to_string(),
            serde_json::from_str::<FeedSnapshot>(BTC_FEED).unwrap(),
        );
        build_context(&feeds, &spec).unwrap()
    }

    #[test]
    fn render_context_lists_facts() {
        let text = render_context(&built_context());
        assert!(text.contains("PriceMove"));
        assert!(text.contains("BTCUSDT"));
        assert!(text.contains("dropped"));
        assert!(text.contains("1 fact(s), 1 symbol(s) scanned"));
    }

    #[test]
    fn render_context_reports_no_facts() {
        let empty = MarketContext {
            facts: Vec::new(),
            symbols: vec!["BTCUSDT".to_string()],
            lookback: 3,
        };
        assert_eq!(render_context(&empty), "no facts (1 symbol(s) scanned)");
    }

    #[test]
    fn context_from_a_directory_matches_a_direct_build() {
        let base = std::env::temp_dir().join("wickra-copilot-run-dir-test");
        let feeds = base.join("feeds");
        fs::create_dir_all(&feeds).unwrap();
        fs::write(base.join("spec.json"), SPEC).unwrap();
        fs::write(feeds.join("BTCUSDT.json"), BTC_FEED).unwrap();

        let cli = Cli {
            command: Command::Context(FeedInput {
                spec: base.join("spec.json"),
                feeds: Some(feeds.clone()),
                stdin: false,
                format: Format::Json,
            }),
        };
        let out = run(cli).unwrap();
        let expected = serde_json::to_string(&built_context()).unwrap();
        assert_eq!(out, expected);

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn missing_feed_source_errors() {
        let base = std::env::temp_dir().join("wickra-copilot-run-nosrc-test");
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("spec.json"), SPEC).unwrap();

        let cli = Cli {
            command: Command::Context(FeedInput {
                spec: base.join("spec.json"),
                feeds: None,
                stdin: false,
                format: Format::Text,
            }),
        };
        let err = run(cli).unwrap_err();
        assert!(err.to_string().contains("--feeds"));

        fs::remove_dir_all(&base).unwrap();
    }
}
