# Architecture

`wickra-copilot` is one data-driven core with many thin consumers, plus a
separate, swappable LLM adapter. The core turns a serde `ContextSpec` folded over
serialized microstructure feeds into a `MarketContext` — a list of hard **facts**.
Because the context is data, not instructions, the exact same result is produced
natively, across the C ABI and in WASM, byte-for-byte identical — and stays
identical between the parallel (rayon) and sequential builds. The LLM call that
turns that context into an answer is a distinct, non-deterministic layer, kept
strictly out of the deterministic core.

## The layers

```
CONSUMERS  CLI: crates/copilot-cli (context | ask)  ·  desktop: crates/copilot-desktop  ·  any language via its binding (command JSON)
      ▲ MarketContext JSON (deterministic)              │  Answer (LLM, non-deterministic — only via `ask` / the adapter)
CORE  crates/copilot-core:  ContextSpec (JSON) + FeedSnapshot (JSON)
                            → FactBuilder (O(1), deterministic) → MarketContext { Vec<Fact> }
                            + ToolCatalog (function-calling interface)          ← deterministic, golden-tested
      ▼ data-driven JSON API in ten languages (like screener command_json / backtest run_json)
ADAPTER  crates/copilot-llm:  provider abstraction + prompt renderer (MarketContext → messages)
                             → calls the user's own LLM endpoint, key from the environment    *** separate, non-deterministic, never golden ***
BINDINGS  python · node · wasm · c (C-ABI hub) → c / c++ / c# / go / java / r
CORES  wickra-core (indicators grounding facts) · wickra-exchange (microstructure feeds = fact inputs) · wickra-data (Candle, optional)
```

Each binding ships the same surface — a `Copilot` handle plus
`command(json) -> json` and `version` — with its own README, tests, a runnable
example, and a completeness guard.

## The core is data-driven

A `Fact` is a serde data-model, not a list of instructions, and the spec that
selects and thresholds the facts is **data too**: a `ContextSpec` never carries a
Rust closure. Closures cannot cross the C ABI or a WASM boundary; a serde
data-model can, so a Python, Go or browser consumer builds the identical
`MarketContext` a Rust consumer would.

## The six facts

Each fact is a hard, attributable observation derived from the feeds — a
`kind` + `symbol` + `value` + `magnitude` + `ts` + a human-readable string:

- **`price_move`** — a significant move in price over a window.
- **`orderbook_imbalance`** — resting bid/ask volume skew at the top of book.
- **`liquidation_cluster`** — a burst of liquidation notional in a short window.
- **`funding_flip`** — the funding rate crossing zero or reaching an extreme.
- **`oi_change`** — a relative change in open interest.
- **`volatility_spike`** — a jump in realised volatility.

The facts are the **grounding**: an answer is built from the real order book,
liquidations and funding, not from the model's priors.

## Determinism is the moat

The `MarketContext` is byte-identical across all ten languages and between the
parallel and sequential builds: `BTreeMap` in every output path, the fact vector
stably sorted (severity then symbol key), no RNG, and reductions run serially in
key order rather than rayon order. That determinism is what lets the golden corpus
pin the context byte-for-byte. **The LLM call is never part of it** — it is
non-deterministic by nature and lives entirely in the separate `copilot-llm`
adapter.

## The command boundary

Every consumer talks to the core through a single JSON-in / JSON-out function,
`Copilot::command`. The binding does no logic of its own — it forwards the command
string and returns the core's response verbatim. That verbatim pass-through is
what makes the golden corpus a **cross-language** parity corpus: the same command
produces a byte-identical `MarketContext` in every language, with no per-language
JSON reformatting.

## The LLM adapter

`copilot-llm` renders a `MarketContext` into a prompt and calls the user's chosen
LLM over the OpenAI-compatible `chat/completions` interface. Four providers are
selectable (`ollama` / `openai` / `claude` / `gemini`, plus `custom`); each preset
only sets the `base_url` (overridable via the environment), so one HTTP
implementation targets four endpoints. Ollama is local and needs no key; the cloud
providers use the user's own key, read from the environment. It is not a hosted
service and places no orders — it reads market data and asks questions.

## Indicators come from the Wickra core

No indicator mathematics lives in this repository. Where a fact needs a derived
series (realised volatility, OI deltas), `IndicatorSet` resolves each building
block from the `wickra-core` registry by name and parameters (the same resolver
the backtester uses), so `wickra-copilot` inherits all 514 indicators and any
future additions for free.

## Integration with the rest of Wickra

`wickra-copilot` sits beside the other Wickra consumers — the terminal, the
screener, the X-ray, the radar and the backtester — over the same core. It depends
on `wickra-core` (indicators) and on `wickra-exchange`, whose order-book, trade,
funding, open-interest and liquidation streams define the shapes of the copilot's
input feeds; `wickra-data` (`Candle` + CSV) is optional. It only reads and
analyses market data — it never places orders and holds no order-secret material.
