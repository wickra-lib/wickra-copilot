#![no_main]
//! Fuzz the feed-parsing path: arbitrary bytes are parsed as a single
//! `FeedSnapshot` and as a `{ symbol: FeedSnapshot }` universe. Neither must
//! panic; malformed input must surface as a clean `Err`.

use std::collections::BTreeMap;

use copilot_core::FeedSnapshot;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let _ = serde_json::from_str::<FeedSnapshot>(text);
    let _ = serde_json::from_str::<BTreeMap<String, FeedSnapshot>>(text);
});
