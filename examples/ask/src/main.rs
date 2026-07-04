//! A runnable LLM-adapter example: ground a question in the deterministic
//! `MarketContext`, then ask a real model to answer *only* from those facts.
//!
//! Unlike every other example this one talks to the network, so it is **not**
//! part of CI. It compiles in CI (the adapter surface must keep building) but is
//! only ever *run* locally. By default it targets a local Ollama server, which
//! needs no API key:
//!
//! ```bash
//! # start Ollama and pull a model first, then:
//! cargo run -p wickra-copilot-ask-example
//!
//! # or point at any OpenAI-compatible endpoint:
//! WICKRA_COPILOT_PROVIDER=openai \
//!   WICKRA_COPILOT_API_KEY=sk-... \
//!   WICKRA_COPILOT_MODEL=gpt-4o-mini \
//!   cargo run -p wickra-copilot-ask-example
//! ```
//!
//! The grounding context is deterministic and always printed; the model's answer
//! is non-deterministic and is never pinned by any test.

use std::collections::BTreeMap;

use copilot_core::{build_context, ContextSpec, FeedSnapshot};
use copilot_llm::{ask, OpenAiCompatible, Provider};

const SPEC: &str = r#"{
    "symbols": ["BTCUSDT"],
    "lookback": 3,
    "facts": ["price_move"]
}"#;

const FEEDS: &str = r#"{
    "BTCUSDT": {
        "symbol": "BTCUSDT",
        "candles": [
            {"ts": 1, "open": 100, "high": 100, "low": 100, "close": 100, "volume": 1},
            {"ts": 2, "open": 97, "high": 97, "low": 97, "close": 97, "volume": 1},
            {"ts": 3, "open": 94, "high": 94, "low": 94, "close": 94, "volume": 1}
        ]
    }
}"#;

const QUESTION: &str = "Why did BTC move?";

fn main() {
    // 1) Build the deterministic grounding context (no network).
    let spec = ContextSpec::from_json(SPEC).expect("valid spec");
    let feeds: BTreeMap<String, FeedSnapshot> = serde_json::from_str(FEEDS).expect("valid feeds");
    let context = build_context(&feeds, &spec).expect("build_context");

    println!("Grounding context:");
    for fact in &context.facts {
        println!("  - {}", fact.human);
    }

    // 2) Pick a provider (default: local Ollama, no key needed).
    let provider_name =
        std::env::var("WICKRA_COPILOT_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
    let provider = match Provider::from_str(&provider_name) {
        Ok(provider) => provider,
        Err(err) => {
            eprintln!("unknown provider {provider_name:?}: {err}");
            return;
        }
    };
    let client = match OpenAiCompatible::from_provider(provider) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("could not configure provider: {err}");
            return;
        }
    };

    // 3) Ask the model — the only networked step. A missing server or key is a
    //    clean error, never a panic.
    println!("\nQuestion: {QUESTION}");
    match ask(&client, QUESTION, &context) {
        Ok(answer) => {
            println!("Model ({}):", answer.model);
            println!("  {}", answer.text);
        }
        Err(err) => {
            eprintln!(
                "\nno answer: {err}\n\
                 (start a local Ollama server, or set WICKRA_COPILOT_PROVIDER / \
                 WICKRA_COPILOT_API_KEY for a hosted endpoint)"
            );
        }
    }
}
