# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The `copilot-core` deterministic core: `ContextSpec` (JSON/TOML), the six fact
  derivations (price move, order-book imbalance, liquidation cluster, funding
  flip, open-interest change, volatility spike), each with a fixed significance
  threshold and a byte-pinned English `human` sentence, assembled into a ranked
  `MarketContext`, and the `Copilot::command` JSON-over-C-ABI protocol
  (`set_spec`, `build_context`/`facts`, `query`, `reset`, `version`). The
  parallel (rayon) and sequential builds are byte-for-byte identical.
- `copilot-llm`: a separate LLM adapter — never reachable over the C ABI — with
  four provider presets (Ollama, OpenAI, Claude, Gemini) plus a custom endpoint,
  driven by one OpenAI-compatible client, configured through the
  `WICKRA_COPILOT_API_KEY` / `_BASE_URL` / `_MODEL` environment variables. Local
  by default, read-only, no SaaS.
- `wickra-copilot` CLI: `context` builds and prints the deterministic facts;
  `ask` builds the context, routes the question and asks a configured provider to
  explain it (`--spec`, `--feeds` / `--stdin`, `--format`, `--provider`).
- Ten-language surface: native Rust, Python (PyO3), Node.js (napi) and WASM
  (wasm-bindgen), plus a C ABI hub (cbindgen) backing C, C++, C#, Go, Java and R.
  Only the deterministic core is exposed.
- Question routing: `query` maps a natural-language question to the fact kinds it
  needs through a fixed keyword table, returning deterministic `ToolCall`s.
- A deterministic golden corpus (feed universe, specs, byte-exact expected
  contexts) and cross-language byte-equality tests across every binding.
- Test rigor: conformance, golden, parallel-equals-sequential, property-based
  invariants, four cargo-fuzz targets, and the `copilot-bench` criterion suite.
- One runnable "build a context" example per language, an `ask` LLM demo, and the
  core documentation set under `docs/` (architecture, facts, grounding, LLM
  adapter, tool calling, cookbook).
- CI/CD: a multi-OS test matrix across ten languages, CodeQL, OpenSSF Scorecard,
  zizmor, link-check, benchmark and metadata-audit workflows, plus an authored
  (tag-gated) release workflow.
- Repository scaffolding: Cargo workspace, supply-chain configuration
  (`deny.toml`, `osv-scanner.toml`, `lychee.toml`), lint configuration
  (`clippy.toml`), `repo-metadata.toml`, and dual `MIT OR Apache-2.0` licensing.

[Unreleased]: https://github.com/wickra-lib/wickra-copilot/commits/main
