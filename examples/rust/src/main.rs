//! A runnable Rust example: build a market context with the native
//! `build_context` API and print it.
//!
//! ```bash
//! cargo run -p wickra-copilot-example
//! ```

use std::collections::BTreeMap;

use copilot_core::{build_context, ContextSpec, FeedSnapshot};

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

fn main() {
    let spec: ContextSpec = ContextSpec::from_json(SPEC).expect("valid spec");
    let feeds: BTreeMap<String, FeedSnapshot> = serde_json::from_str(FEEDS).expect("valid feeds");

    let context = build_context(&feeds, &spec).expect("build_context");

    println!("wickra-copilot {}", copilot_core::version());
    println!(
        "{}",
        serde_json::to_string(&context).expect("serialize context")
    );
    println!("  facts: {}", context.facts.len());
}
