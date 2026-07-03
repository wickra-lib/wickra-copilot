# Roadmap

`wickra-copilot` is built out in phases, mirroring the proven structure of the
Wickra exchange, backtester, terminal, screener, X-ray and radar repos. Each phase
lands as reviewed, CI-green pull requests. Status below is updated as phases
complete.

## Phases

0. **Scaffold** — workspace, governance, supply-chain config, `.github`
   scaffolding. *In progress.*
1. **`copilot-core`** — the `ContextSpec`, the feed model, the six facts
   (price move, order-book imbalance, liquidation cluster, funding flip, OI
   change, volatility spike), the deterministic `FactBuilder` into a
   `MarketContext`, the `ToolCatalog`, and the `command_json` boundary, with
   near-total coverage via inline tests.
2. **`copilot-llm`** — the separate, non-deterministic adapter: a provider
   abstraction with four presets (Ollama / OpenAI / Claude / Gemini, plus custom)
   over one OpenAI-compatible HTTP path, and a prompt renderer that grounds the
   model in the `MarketContext`. Exercised only by offline tests.
3. **`copilot-cli` + `copilot-desktop`** — the reference `wickra-copilot` binary
   (`context` builds the deterministic context; `ask` grounds an LLM), and a local
   desktop front-end.
4. **Bindings** — the C ABI hub first, then native Python, Node and WASM, then C,
   C++, C#, Go, Java and R over the hub; each exposes the `Copilot` handle +
   `command` + `version`, with a completeness guard.
5. **Golden corpus** — fixed deterministic feed snapshots and canonical specs
   whose blessed `MarketContext` is the byte-exact, cross-language parity corpus.
6. **Test rigor** — conformance, golden, `parallel == sequential`, property and
   fuzz tests, offline adapter tests, and a criterion benchmark suite.
7. **ABI harness + examples** — cbindgen header sync-check and one runnable
   example per language, plus an `ask` LLM demo (not run in CI).
8. **CI/CD** — the full workflow matrix (all languages), OpenSSF Scorecard, Best
   Practices, link check, and the release workflow.
9. **README, badges, docs** — the banner + badge treatment and the docs guides.

## Beyond 1.0

- Additional facts and richer per-fact parameters as the corpus grows.
- A live feed streamed from an exchange, still read-only.
- More provider presets as OpenAI-compatible endpoints proliferate.

## Non-goals

- **Indicator code in this repository.** Indicators come from the `wickra-core`
  registry; the copilot composes them, it does not reimplement them.
- **LLM non-determinism in the deterministic core.** The `MarketContext` is a
  serde data-model built with no RNG and a stable order, so it crosses the C ABI
  and WASM unchanged and is golden-tested; the LLM call is a strictly separate,
  never-golden adapter.
- **A hosted service or stored credentials.** The copilot runs locally, calls an
  LLM with the user's own key, reads only public market data, and places no
  orders.
