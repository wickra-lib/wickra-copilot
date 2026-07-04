#![no_main]
//! Fuzz the context builder: a `{spec, feeds}` object is parsed and built. The
//! spec and the whole feed universe are attacker-controlled; the build must
//! never panic (an invalid spec is a clean `Err`, not a crash).

use std::collections::BTreeMap;

use copilot_core::{build_context, ContextSpec, FeedSnapshot};
use libfuzzer_sys::fuzz_target;
use serde::Deserialize;

#[derive(Deserialize)]
struct Input {
    spec: ContextSpec,
    feeds: BTreeMap<String, FeedSnapshot>,
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(input) = serde_json::from_str::<Input>(text) else {
        return;
    };
    // Bound the total work so the fuzzer cannot request an unbounded fold.
    let total: usize = input.feeds.values().map(|f| f.candles.len()).sum();
    if total > 5000 {
        return;
    }
    let _ = build_context(&input.feeds, &input.spec);
});
