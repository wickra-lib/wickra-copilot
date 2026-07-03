<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-copilot)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

<!-- Skeleton README (P-COP-0.12). The full ~20-badge block (CI, CodeQL, codecov,
     crates.io/PyPI/npm/NuGet/Maven/Go/R-universe, Scorecard, Best-Practices,
     Provenance, Docs) and the finished sections are assembled in P-COP-9.1, once
     the per-product badge SVGs are generated in the .github repo (P-COP-9.2).
     Until then this stays link-clean (no 404s on the repo page). -->

---

# Wickra Copilot

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — the trading assistant that cannot hallucinate.**

Wickra Copilot builds a **deterministic, structured market context** from real
microstructure feeds ([`wickra-core`](https://github.com/wickra-lib/wickra) +
[`wickra-exchange`](https://github.com/wickra-lib/wickra-exchange)) — a list of
hard facts (price moves, order-book imbalance, liquidation clusters, funding
flips, open-interest changes, volatility spikes) — and hands that context to an
LLM as grounding. Ask *"Why did BTC just dump?"* and the answer comes from the
**real order book, liquidations and funding — not vibes.**

- **Deterministic core** — the `MarketContext` fact list is byte-identical across
  all ten languages and between the parallel and sequential builds. It is the
  only part that is golden-tested; the LLM call is a separate, swappable adapter.
- **Local tool, your own key** — not a hosted service and not a SaaS. It runs
  locally and calls an LLM endpoint with **your** API key, read from the
  environment. Ollama runs fully offline; OpenAI / Claude / Gemini use your own
  key over their OpenAI-compatible endpoints. No vendor lock-in.
- **Read-only** — it reads market data and asks questions; it never places orders.

The core is one library (`copilot-core`), usable from **Rust, Python, Node.js,
WASM, C, C++, C#, Go, Java and R** over a JSON-over-C-ABI boundary, plus a
reference CLI and a local desktop front-end.

## Status

**Pre-release — under active construction.** This repository is being built out
phase by phase (scaffold → core → LLM adapter → CLI → ten language bindings →
golden corpus → tests → CI → docs). It is not yet published to any registry.

## Documentation

The full documentation — the `ContextSpec` / fact reference, the `MarketContext`
data-model, the provider/adapter guide, and per-binding quickstarts — is
finalized in this README and under `docs/` during the documentation phase.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

## Disclaimer

Wickra Copilot is analysis software: it builds a market context and relays it to
a language model of your choosing. LLM output can be wrong and is **not financial
advice**; the copilot places no orders. Trading carries risk; use at your own
discretion.
