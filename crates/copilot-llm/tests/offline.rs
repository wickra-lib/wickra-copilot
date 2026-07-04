//! Offline integration tests for the LLM adapter's public surface. Everything
//! here runs without a network — the only provider used is a local fake, and the
//! real `OpenAiCompatible` is only inspected for key redaction, never called. CI
//! runs these; they pin the deterministic prompt and the grounding the adapter
//! attaches to an answer, but they are deliberately **not** a golden of the
//! model's text (that is non-deterministic and network-bound).

use copilot_core::{Fact, FactKind, MarketContext};
use copilot_llm::{ask, render_prompt, LlmProvider, Message, OpenAiCompatible, Provider, Result};

/// A single-fact context whose `human` prose is fixed, so the rendered prompt is
/// fully determined.
fn context() -> MarketContext {
    MarketContext {
        facts: vec![Fact::new(
            FactKind::PriceMove,
            "BTCUSDT",
            -6.0,
            6.0,
            3,
            "BTCUSDT dropped -6.00% over the last 3 bars.",
        )],
        symbols: vec!["BTCUSDT".to_string()],
        lookback: 3,
    }
}

/// A provider that answers from memory — no network, no key.
struct FakeProvider {
    model: String,
}

impl LlmProvider for FakeProvider {
    fn complete(&self, _messages: &[Message]) -> Result<String> {
        Ok("BTC dumped because of a long-liquidation cascade.".to_string())
    }

    fn model(&self) -> &str {
        &self.model
    }
}

#[test]
fn renders_the_pinned_prompt() {
    let messages = render_prompt("why did BTC dump", &context());
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "system");
    assert_eq!(
        messages[0].content,
        "You are a market analyst. Answer ONLY from the facts below. If a fact is absent, say you don't know."
    );
    assert_eq!(messages[1].role, "user");
    assert_eq!(
        messages[1].content,
        "Facts:\nBTCUSDT dropped -6.00% over the last 3 bars.\n\nQuestion: why did BTC dump"
    );
}

#[test]
fn ask_attaches_the_grounding_and_routes_tool_calls() {
    let ctx = context();
    let provider = FakeProvider {
        model: "fake-model".to_string(),
    };
    let answer = ask(&provider, "why did BTC dump", &ctx).unwrap();

    assert_eq!(
        answer.text,
        "BTC dumped because of a long-liquidation cascade."
    );
    assert_eq!(answer.model, "fake-model");
    // The grounding context is carried through verbatim.
    assert_eq!(answer.context, ctx);
    // The question routes to at least the price-move fact that is present.
    assert!(answer
        .tool_calls
        .iter()
        .any(|call| call.kind == Some(FactKind::PriceMove)));
}

#[test]
fn provider_debug_never_leaks_the_key() {
    // Feed a sentinel secret through the environment and assert it never reaches
    // a debug print: the manual Debug impl redacts the key field, so a leaked
    // secret can never surface in a log line. This test is the only one in the
    // binary that touches the copilot environment variables.
    std::env::set_var("WICKRA_COPILOT_API_KEY", "SECRET-SENTINEL-123");
    let provider = OpenAiCompatible::from_provider(Provider::Openai).unwrap();
    std::env::remove_var("WICKRA_COPILOT_API_KEY");

    let debug = format!("{provider:?}");
    assert!(!debug.contains("SECRET-SENTINEL-123"), "{debug}");
    assert!(debug.contains("<redacted>"), "{debug}");
}
