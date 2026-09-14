//! Operating-mode equivalence over the golden corpus.
//!
//! The command protocol offers two ways to reach the same answer, and both
//! must return the same bytes:
//!
//! - `facts` is an alias of `build_context`: the same feeds through either verb
//!   produce the blessed `expected/<spec>.json` context.
//! - `query` answers either against the context the handle stored from its last
//!   `build_context`, or against a context passed inline in the command. The
//!   tool calls must not depend on which way the context arrived.
//!
//! The bindings repeat this check at their own boundary; this is the in-core
//! anchor.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use wickra_copilot_core::Copilot;

const QUESTION: &str = "what moved and why?";

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("golden")
}

#[test]
fn facts_alias_and_inline_query_match_the_stored_path() {
    let golden = golden_dir();
    let feeds: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(golden.join("feeds.json")).unwrap()).unwrap();
    let mut specs: Vec<PathBuf> = fs::read_dir(golden.join("specs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    specs.sort();
    assert!(!specs.is_empty(), "no golden specs found");

    for spec_path in specs {
        let spec = fs::read_to_string(&spec_path).unwrap();
        let expected =
            fs::read_to_string(golden.join("expected").join(spec_path.file_name().unwrap()))
                .unwrap();

        // One handle builds and stores; its `query` reads the stored context.
        let mut stored = Copilot::new(&spec).unwrap();
        let built = stored
            .command_json(&json!({"cmd": "build_context", "feeds": feeds}).to_string())
            .unwrap();
        assert_eq!(built, expected.trim(), "{}", spec_path.display());
        let from_stored = stored
            .command_json(&json!({"cmd": "query", "question": QUESTION}).to_string())
            .unwrap();

        // A second handle never builds; its `query` carries the context inline.
        let mut fresh = Copilot::new(&spec).unwrap();
        let context: serde_json::Value = serde_json::from_str(&built).unwrap();
        let from_inline = fresh
            .command_json(
                &json!({"cmd": "query", "question": QUESTION, "context": context}).to_string(),
            )
            .unwrap();
        assert_eq!(from_inline, from_stored, "{}", spec_path.display());

        // The alias verb is the same computation.
        let mut alias = Copilot::new(&spec).unwrap();
        let via_facts = alias
            .command_json(&json!({"cmd": "facts", "feeds": feeds}).to_string())
            .unwrap();
        assert_eq!(via_facts, built, "{}", spec_path.display());
    }
}
