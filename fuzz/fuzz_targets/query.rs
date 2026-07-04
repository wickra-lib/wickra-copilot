#![no_main]
//! Fuzz the tool router: a `{context, question}` object is parsed and routed.
//! The context and the question are attacker-controlled; `query` is pure
//! keyword routing and must never panic — it only ever returns tool calls for
//! kinds and symbols already present in the context.

use copilot_core::{query, MarketContext};
use libfuzzer_sys::fuzz_target;
use serde::Deserialize;

#[derive(Deserialize)]
struct Input {
    context: MarketContext,
    question: String,
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(input) = serde_json::from_str::<Input>(text) else {
        return;
    };
    let _ = query(&input.question, &input.context);
});
