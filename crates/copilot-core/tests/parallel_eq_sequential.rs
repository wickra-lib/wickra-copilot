//! Parallel/sequential equivalence (§6.3): `build_context` derives each symbol's
//! facts concurrently under the `parallel` feature (rayon) and one-at-a-time
//! without it, then sorts by a total order that is independent of the fold
//! order. The two paths must produce a byte-identical `MarketContext`.
//!
//! A single compilation only exercises one path, so this test pins the built
//! context against the blessed `golden/expected/*.json` (the same anchor the
//! golden test uses). CI runs it under **both** `--all-features` (parallel) and
//! `--no-default-features` (sequential); when both configurations reproduce the
//! blessed bytes, parallel and sequential are proven equal. Within a single run
//! it also asserts the fold is deterministic — building the same universe twice
//! yields the same bytes — which catches any nondeterminism (e.g. a `HashMap`
//! creeping into the fold) regardless of which path is active.

use std::fs;
use std::path::{Path, PathBuf};

use copilot_core::Copilot;

/// The repository-root `golden/` directory, resolved from this crate's manifest.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("golden")
}

/// Build a context for `spec_json` over the shared golden feeds and return the
/// compact reply string.
fn build(spec_json: &str, feeds: &str) -> String {
    let mut copilot = Copilot::new(spec_json).unwrap();
    let cmd = format!(r#"{{"cmd":"build_context","feeds":{feeds}}}"#);
    copilot.command_json(&cmd).unwrap()
}

#[test]
fn active_fold_matches_blessed_and_is_deterministic() {
    let golden = golden_dir();
    let feeds = fs::read_to_string(golden.join("feeds.json")).unwrap();

    let mut specs: Vec<PathBuf> = fs::read_dir(golden.join("specs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    specs.sort();
    assert!(!specs.is_empty(), "no golden specs found");

    for spec_path in specs {
        let name = spec_path.file_stem().unwrap().to_str().unwrap();
        let spec_json = fs::read_to_string(&spec_path).unwrap();
        let expected = fs::read_to_string(golden.join("expected").join(format!("{name}.json")))
            .unwrap_or_else(|e| panic!("{name}: missing expected: {e}"));

        // Whichever fold path is compiled must reproduce the blessed bytes...
        let first = build(&spec_json, &feeds);
        assert_eq!(first, expected.trim_end(), "{name}: fold != blessed");
        // ...and it must be deterministic across repeated builds.
        let second = build(&spec_json, &feeds);
        assert_eq!(first, second, "{name}: fold is nondeterministic");
    }
}
