//! The deterministic prompt renderer.
//!
//! [`render_prompt`] turns a question and its grounding [`MarketContext`] into a
//! fixed pair of chat messages — a system instruction that pins the model to the
//! supplied facts, and a user message carrying those facts (one `human` line
//! each) followed by the question. It performs no I/O and no randomness, so the
//! byte output is stable and unit-testable; only `LlmProvider::complete` is
//! non-deterministic.

use copilot_core::MarketContext;

use crate::Message;

/// The fixed system instruction: answer strictly from the supplied facts.
const SYSTEM_PROMPT: &str =
    "You are a market analyst. Answer ONLY from the facts below. If a fact is absent, say you don't know.";

/// Render a deterministic prompt from a question and its grounding context.
///
/// Returns exactly two messages: the `system` instruction and the `user`
/// message. The user content is `"Facts:\n<lines>\n\nQuestion: <question>"`,
/// where `<lines>` is the facts' `human` prose joined by newlines, or
/// `"(no significant facts)"` when the context is empty. The byte layout is
/// pinned by the tests below.
#[must_use]
pub fn render_prompt(question: &str, ctx: &MarketContext) -> Vec<Message> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use copilot_core::{Fact, FactKind};

    /// A context with a single, known price-move fact — its `human` prose is
    /// fixed so the prompt bytes are fully determined.
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

    #[test]
    fn prompt_bytes_are_pinned() {
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
    fn multiple_facts_are_joined_by_newlines() {
        let ctx = MarketContext {
            facts: vec![
                Fact::new(FactKind::PriceMove, "BTCUSDT", -6.0, 6.0, 3, "line one"),
                Fact::new(FactKind::OiChange, "BTCUSDT", -4.0, 4.0, 3, "line two"),
            ],
            symbols: vec!["BTCUSDT".to_string()],
            lookback: 3,
        };
        let messages = render_prompt("q", &ctx);
        assert_eq!(
            messages[1].content,
            "Facts:\nline one\nline two\n\nQuestion: q"
        );
    }

    #[test]
    fn empty_context_renders_a_placeholder() {
        let ctx = MarketContext {
            facts: Vec::new(),
            symbols: Vec::new(),
            lookback: 0,
        };
        let messages = render_prompt("anything", &ctx);
        assert_eq!(
            messages[1].content,
            "Facts:\n(no significant facts)\n\nQuestion: anything"
        );
    }
}
