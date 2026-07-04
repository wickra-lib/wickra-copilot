<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-copilot)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codeql.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/codeql.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-copilot)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/best-practices.svg)](https://www.bestpractices.dev/)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/provenance.svg)](https://github.com/wickra-lib/wickra-copilot/attestations)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/docs.svg)](https://wickra.org)
[![Live demo](https://img.shields.io/badge/live%20demo-live.wickra.org-3b82f6)](https://live.wickra.org)

---

# Wickra Copilot

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — the trading assistant that cannot hallucinate the facts.**

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same data-driven core and ten-language binding surface also power [wickra-exchange](https://github.com/wickra-lib/wickra-exchange), [wickra-backtest](https://github.com/wickra-lib/wickra-backtest), [wickra-terminal](https://github.com/wickra-lib/wickra-terminal), [wickra-screener](https://github.com/wickra-lib/wickra-screener), [wickra-xray](https://github.com/wickra-lib/wickra-xray), [wickra-radar](https://github.com/wickra-lib/wickra-radar), [wickra-copilot](https://github.com/wickra-lib/wickra-copilot) and [wickra-shazam](https://github.com/wickra-lib/wickra-shazam).

Wickra Copilot is one data-driven core, [`copilot-core`](crates/copilot-core): a
serde `ContextSpec` is folded over real microstructure feeds ([`wickra-core`](https://github.com/wickra-lib/wickra)
+ [`wickra-exchange`](https://github.com/wickra-lib/wickra-exchange)) into a
`MarketContext` — a list of hard, numeric **facts**: price moves, order-book
imbalance, liquidation clusters, funding flips, open-interest changes and
volatility spikes. Each fact carries its own one-line human sentence. That
context is the grounding you hand to an LLM: ask *"Why did BTC just dump?"* and
the answer is anchored to the **real order book, liquidations and funding — not
vibes.**

Because the context is **data, not code**, the exact same `MarketContext` crosses
the C ABI and WASM unchanged — and stays byte-for-byte identical between the
parallel (rayon) and sequential (the WASM fallback) builds. The core is exposed
as a **JSON-over-C-ABI data API** (`Copilot::command`) in **Rust, Python,
Node.js, WASM, C, C++, C#, Go, Java and R**, with a reference CLI.

- **Deterministic core** — the `MarketContext` fact list is the only golden-tested
  surface; it is identical across all ten languages and both build profiles.
- **Separate LLM adapter** — the network call lives in a distinct crate
  ([`copilot-llm`](crates/copilot-llm)); it never crosses the C ABI. The
  deterministic core has no network, no key, no I/O.
- **Local tool, your own key** — not a hosted service and not a SaaS. It runs
  locally and calls an LLM endpoint with **your** API key, read from the
  environment. Ollama runs fully offline; OpenAI / Claude / Gemini use your own
  key over their endpoints. No vendor lock-in.
- **Read-only** — it reads market data and asks questions; it never places orders.

## Status

**Pre-release — functionally complete, CI-verified, not yet published.** The
deterministic core, the separate LLM adapter, the CLI, all ten language bindings,
the byte-exact golden corpus, property + fuzz tests, benchmarks and one runnable
example per language are in place and green across the full CI matrix (10
languages × 3 OS). Not yet released to any registry — track progress in
[ROADMAP.md](ROADMAP.md).

## Documentation

- [Architecture](ARCHITECTURE.md) — the deterministic core, the fact boundary, the LLM adapter, the binding surface.
- Fact & spec reference, the grounding rationale, and per-binding quickstarts under [`docs/`](docs); one runnable example per language under [`examples/`](examples).
- [ROADMAP.md](ROADMAP.md) · [BENCHMARKS.md](BENCHMARKS.md) · [THREAT_MODEL.md](THREAT_MODEL.md) · [SECURITY.md](SECURITY.md).

## Quickstart

```bash
# Build the market context from a spec + a per-symbol feed directory,
# and print its derived facts (the same bytes every binding returns):
cargo run -p wickra-copilot -- context --spec golden/specs/dump.json --feeds golden/feeds --format json

# Human-readable list of facts:
cargo run -p wickra-copilot -- context --spec golden/specs/dump.json --feeds golden/feeds

# Build the context and ask a local LLM to explain it (Ollama, no API key):
cargo run -p wickra-copilot -- ask --spec golden/specs/dump.json --feeds golden/feeds \
  --question "Why did BTC just dump?" --provider ollama
```

`--spec` is a `ContextSpec`; feeds are read either from `--feeds <dir>` (one
`<SYMBOL>.json` `FeedSnapshot` per symbol) or as one JSON object from `--stdin`.
The `context` subcommand is fully deterministic and offline; `ask` adds the LLM
adapter on top.

## ContextSpec / facts

A spec is a JSON (or TOML) document: the `symbols` to inspect, a `lookback`
window in bars, an optional `timeframe`, and the `facts` to derive. The builder
walks each symbol's feed, derives the requested facts, rounds every magnitude to
`1e-8`, and returns them sorted by magnitude (descending), then kind, symbol and
timestamp (ascending) — a total order, so the output is stable everywhere.

```json
{
  "symbols": ["BTCUSDT"],
  "lookback": 20,
  "timeframe": "1m",
  "facts": ["price_move", "orderbook_imbalance", "liquidation_cluster", "funding_flip", "oi_change", "volatility_spike"]
}
```

- **Fact kinds**: `price_move`, `orderbook_imbalance`, `liquidation_cluster`, `funding_flip`, `oi_change`, `volatility_spike`.
- **Fact** — `Fact { kind, symbol, value, magnitude, ts, human }`; `value` is
  signed, `magnitude` is its ranking key, and `human` is a ready-made sentence
  (e.g. `"BTCUSDT dropped -6.44% over the last 20 bars."`). The context is
  `MarketContext { facts, symbols, lookback }`, so it explains itself before any
  LLM sees it.

## Grounding, and why it is deterministic

The `MarketContext` is computed, not generated: it is a pure function of the
spec and the feeds. `command` drives a `Copilot` handle — `set_spec`,
`build_context`, `query`, `reset`, `version` — and `build_context` goes through
one shared code path whether facts are derived in parallel (rayon) or
sequentially. Facts sort by a total order (`f64::total_cmp` on magnitude, never a
partial float compare), so the JSON is **byte-identical** across all ten
languages and both build profiles. The LLM can be wrong about *interpretation*,
but it can never invent the numbers — they are pinned by the golden corpus.

## LLM adapter — choose your provider, keep your key

The network call is a separate, swappable crate, [`copilot-llm`](crates/copilot-llm),
consumed by the CLI's `ask` subcommand. It ships four provider presets plus a
`custom` one:

- **Ollama** (default) — fully local, no API key.
- **OpenAI**, **Claude**, **Gemini** — your own key, read from the environment
  (`WICKRA_COPILOT_API_KEY`, with `WICKRA_COPILOT_BASE_URL` / `_MODEL` overrides).

The adapter is read-only and never crosses the C ABI: language bindings surface
only the deterministic core. There is no SaaS, no telemetry, and your key stays
on your machine. See [docs/LLM_ADAPTER.md](docs/LLM_ADAPTER.md).

## Use in any language

The same `Copilot` handle — construct from a JSON spec, drive with
`command(json) -> json`, read `version` — is reachable from every binding:

```python
import json
from wickra_copilot import Copilot

spec = json.dumps({"symbols": ["BTCUSDT"], "lookback": 3, "facts": ["price_move"]})
feeds = {"BTCUSDT": {"symbol": "BTCUSDT", "candles": [
    {"ts": 1, "open": 100, "high": 100, "low": 100, "close": 100, "volume": 1},
    {"ts": 2, "open": 97,  "high": 97,  "low": 97,  "close": 97,  "volume": 1},
    {"ts": 3, "open": 94,  "high": 94,  "low": 94,  "close": 94,  "volume": 1}]}}

copilot = Copilot(spec)
context = json.loads(copilot.command(json.dumps({"cmd": "build_context", "feeds": feeds})))
# context is a JSON MarketContext: {"facts":[{"kind":"price_move","symbol":"BTCUSDT",...}],...}
```

The C ABI hub (`bindings/c`) backs C, C++, C#, Go, Java and R; Rust, Python,
Node.js and WASM are native. See each `bindings/<lang>/README.md` and the runnable
[`examples/`](examples).

## Project layout

```
crates/copilot-core    the deterministic core (ContextSpec, facts, MarketContext, command_json)
crates/copilot-llm     the separate LLM adapter (providers, prompt) — never crosses the C ABI
crates/copilot-cli     the CLI (bin: wickra-copilot; context + ask subcommands)
crates/copilot-bench   criterion benchmarks
bindings/{python,node,wasm,c,go,csharp,java,r}   the ten-language surface
golden/                a deterministic feed universe, specs, and byte-exact expected contexts
fuzz/                  cargo-fuzz targets (spec_parse, feed_parse, build_context, query)
examples/              one runnable "build a context" example per language, plus examples/ask (LLM demo)
```

## Building from source

```bash
cargo build --workspace
cargo test  --workspace --all-features
cargo test  --workspace --no-default-features   # sequential build path
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p wickra-copilot -- context --spec golden/specs/dump.json --feeds golden/feeds --format json
```

## Requirements

- **Rust** ≥ 1.86 (workspace MSRV; the Node binding needs ≥ 1.88).
- Binding toolchains as needed: Node ≥ 22, Python ≥ 3.9, a C toolchain, .NET 8,
  JDK 22+, Go 1.23, R — see each `bindings/<lang>/README.md`.
- The LLM `ask` path additionally needs a reachable provider: a local Ollama
  server, or an API key for OpenAI / Claude / Gemini.

## Benchmarks

`crates/copilot-bench` measures `build_context` scaling by universe size and
lookback, parallel vs sequential. See [BENCHMARKS.md](BENCHMARKS.md).

## Ecosystem

Part of the [Wickra](https://github.com/wickra-lib/wickra) family — each one a
data-driven core with a CLI and the same ten-language binding surface:

- [**wickra**](https://github.com/wickra-lib/wickra) — the core library: 514 O(1) streaming indicators across ten languages
- [**wickra-exchange**](https://github.com/wickra-lib/wickra-exchange) — unified market-data + execution across ten crypto exchanges
- [**wickra-backtest**](https://github.com/wickra-lib/wickra-backtest) — event-driven backtester over the Wickra core
- [**wickra-terminal**](https://github.com/wickra-lib/wickra-terminal) — the trading terminal: a TUI and a browser renderer over the stack
- [**wickra-screener**](https://github.com/wickra-lib/wickra-screener) — parallel multi-symbol screening over 514 streaming indicators
- [**wickra-xray**](https://github.com/wickra-lib/wickra-xray) — market-microstructure explorer: footprint, order-book heatmap, liquidation map, funding/OI divergence
- [**wickra-radar**](https://github.com/wickra-lib/wickra-radar) — perp-universe alert radar: OI delta, funding flip, book imbalance, liquidation clusters, OI/price divergence
- [**wickra-copilot**](https://github.com/wickra-lib/wickra-copilot) — local market copilot grounded in real order-book, liquidation and funding microstructure
- [**wickra-shazam**](https://github.com/wickra-lib/wickra-shazam) — match an asset's current microstructure fingerprint against its entire history

Docs at [docs.wickra.org](https://docs.wickra.org); the marketing site and
in-browser demo at [wickra.org](https://wickra.org).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
Commits are signed and in English; open a PR against `main`.

## Security

See [SECURITY.md](SECURITY.md) and [THREAT_MODEL.md](THREAT_MODEL.md). Report
vulnerabilities privately — never in a public issue.

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

## Disclaimer

Wickra Copilot is analysis software: it builds a deterministic market context and
relays it to a language model of your choosing. It is provided "as is", without
warranty of any kind. LLM output can be wrong and is **not financial advice**; the
copilot only reports facts and places no orders. Trading carries risk of loss;
review the code and use at your own discretion.
