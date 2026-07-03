//! Non-deterministic LLM adapter for the Wickra Copilot.
//!
//! `copilot-core` produces a deterministic `MarketContext` — a stable, ranked
//! list of hard facts. This crate is the *only* place that turns that context
//! into a natural-language answer: it renders the facts into a prompt and calls
//! the user's own OpenAI-compatible endpoint (Ollama, OpenAI, Claude or Gemini),
//! reading the API key from the environment.
//!
//! It is kept deliberately separate from the core for three reasons: the LLM
//! call is non-deterministic and must never enter the golden/CI comparison; the
//! network and API key stay out of the core and every language binding (this
//! crate is never exposed over the C ABI); and the endpoint is swappable without
//! touching the fact-derivation logic.
//!
//! The public surface is:
//! - [`LlmProvider`] — the swappable backend that turns messages into a reply.
//! - [`Message`] / [`Answer`] — the request messages and the assembled answer.
//! - [`ask`] — grounds a question in a `MarketContext` and calls the provider.
//!
//! The deterministic pieces of an [`Answer`] (`context` and `tool_calls`) come
//! straight from the core; only `text` and `model` come from the provider, and
//! `text` is never byte-compared. The concrete `OpenAiCompatible` provider (the
//! only place that touches the network) and the standalone prompt renderer are
//! added in the following units.

use serde::Serialize;

use copilot_core::{query, MarketContext, ToolCall};

/// An error from the LLM adapter.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provider failed to produce a completion.
    #[error("LLM provider error: {0}")]
    Provider(String),
}

/// The adapter's result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A single chat message in the OpenAI-compatible `chat/completions` shape.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// The role: `"system"` or `"user"`.
    pub role: String,
    /// The message body.
    pub content: String,
}

/// The copilot's answer to a question.
///
/// `context` and `tool_calls` are deterministic (they come from the core);
/// `text` and `model` come from the provider and are never part of any golden
/// comparison. There is intentionally no `Deserialize`: an `Answer` is only ever
/// produced here, never parsed back.
#[derive(Serialize, Clone, Debug)]
pub struct Answer {
    /// The provider's natural-language reply (non-deterministic).
    pub text: String,
    /// The grounding context the answer was built from (deterministic).
    pub context: MarketContext,
    /// The deterministic tool calls the question routes to.
    pub tool_calls: Vec<ToolCall>,
    /// The model that produced `text`.
    pub model: String,
}

/// A swappable LLM backend: it turns a rendered prompt into a reply.
///
/// The only implementation that touches the network is `OpenAiCompatible`
/// (added in a later unit); tests use a fake in-memory provider.
pub trait LlmProvider {
    /// Complete the given messages into a single reply string.
    ///
    /// # Errors
    /// Returns an error if the provider cannot produce a completion.
    fn complete(&self, messages: &[Message]) -> Result<String>;

    /// The model identifier this provider answers with, recorded on the
    /// [`Answer`] so callers know which model produced the text.
    fn model(&self) -> &str;
}

/// The fixed system instruction: answer strictly from the supplied facts.
const SYSTEM_PROMPT: &str =
    "You are a market analyst. Answer ONLY from the facts below. If a fact is absent, say you don't know.";

/// Render a deterministic prompt from a question and its grounding context.
///
/// This is a private helper for now; it is promoted to a public, byte-pinned
/// `prompt::render_prompt` in the next unit.
fn build_messages(question: &str, ctx: &MarketContext) -> Vec<Message> {
    let facts = if ctx.facts.is_empty() {
        "(no significant facts)".to_string()
    } else {
        ctx.facts
            .iter()
            .map(|fact| fact.human.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    };
    vec![
        Message {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!("Facts:\n{facts}\n\nQuestion: {question}"),
        },
    ]
}

/// Ground a question in a `MarketContext` and ask the provider.
///
/// The prompt is rendered deterministically from the context, the provider is
/// called for the reply text, and the answer is assembled with the deterministic
/// `context` and `tool_calls` from the core. Only `text` and `model` are
/// non-deterministic.
///
/// # Errors
/// Returns an error if the provider fails to complete the prompt.
pub fn ask(provider: &dyn LlmProvider, question: &str, ctx: &MarketContext) -> Result<Answer> {
    let messages = build_messages(question, ctx);
    let text = provider.complete(&messages)?;
    Ok(Answer {
        text,
        context: ctx.clone(),
        tool_calls: query(question, ctx),
        model: provider.model().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use copilot_core::{build_context, ContextSpec, FactKind, FeedSnapshot};
    use std::collections::BTreeMap;

    /// An offline provider that echoes a fixed reply — no network, deterministic.
    struct FakeProvider {
        reply: String,
        model: String,
    }

    impl LlmProvider for FakeProvider {
        fn complete(&self, _messages: &[Message]) -> Result<String> {
            Ok(self.reply.clone())
        }
        fn model(&self) -> &str {
            &self.model
        }
    }

    /// A provider that always fails, to exercise error propagation.
    struct FailingProvider {
        model: String,
    }

    impl LlmProvider for FailingProvider {
        fn complete(&self, _messages: &[Message]) -> Result<String> {
            Err(Error::Provider("boom".to_string()))
        }
        fn model(&self) -> &str {
            &self.model
        }
    }

    fn context() -> MarketContext {
        // BTC drops 6% over three bars -> a single significant price-move fact.
        let feeds: BTreeMap<String, FeedSnapshot> = serde_json::from_str(
            r#"{"BTCUSDT":{"symbol":"BTCUSDT","candles":[
                {"ts":1,"open":100.0,"high":100.0,"low":100.0,"close":100.0,"volume":1.0},
                {"ts":2,"open":97.0,"high":97.0,"low":97.0,"close":97.0,"volume":1.0},
                {"ts":3,"open":94.0,"high":94.0,"low":94.0,"close":94.0,"volume":1.0}]}}"#,
        )
        .unwrap();
        let spec = ContextSpec {
            symbols: vec!["BTCUSDT".to_string()],
            lookback: 3,
            facts: vec![FactKind::PriceMove],
            timeframe: None,
        };
        build_context(&feeds, &spec).unwrap()
    }

    #[test]
    fn ask_fills_deterministic_context_and_tool_calls() {
        let ctx = context();
        let provider = FakeProvider {
            reply: "BTC dropped 6% over the window.".to_string(),
            model: "fake-1".to_string(),
        };
        let answer = ask(&provider, "why did BTC dump", &ctx).unwrap();
        assert_eq!(answer.text, "BTC dropped 6% over the window.");
        assert_eq!(answer.model, "fake-1");
        // context is the core's, verbatim.
        assert_eq!(answer.context, ctx);
        // the question routes to a price-move tool call the core produced.
        assert!(!answer.tool_calls.is_empty());
        assert!(answer
            .tool_calls
            .iter()
            .any(|call| call.kind == Some(FactKind::PriceMove)));
    }

    #[test]
    fn ask_prompt_carries_system_rule_and_facts() {
        let ctx = context();
        let messages = build_messages("why did BTC dump", &ctx);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, SYSTEM_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains("Question: why did BTC dump"));
        // the fact's human prose is grounded into the prompt.
        assert!(messages[1].content.contains(&ctx.facts[0].human));
    }

    #[test]
    fn empty_context_still_renders_a_prompt() {
        let ctx = MarketContext {
            facts: Vec::new(),
            symbols: Vec::new(),
            lookback: 0,
        };
        let messages = build_messages("anything", &ctx);
        assert!(messages[1].content.contains("(no significant facts)"));
    }

    #[test]
    fn failing_provider_propagates() {
        let ctx = context();
        let provider = FailingProvider {
            model: "none".to_string(),
        };
        let err = ask(&provider, "why", &ctx).unwrap_err();
        assert!(matches!(err, Error::Provider(_)));
    }
}
