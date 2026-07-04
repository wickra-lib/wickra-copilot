# Architecture (internals)

The top-level [ARCHITECTURE.md](../ARCHITECTURE.md) gives the high-level shape;
this page covers how the product actually turns a spec + a feed universe into a
grounded answer. There are **two clearly separated halves**: a deterministic core
(`copilot-core`) that everything is built on, and a non-deterministic LLM adapter
(`copilot-llm`) that only the CLI touches. The C ABI and the ten language
bindings wrap **only the core**.

## The pipeline

```
ContextSpec (JSON/TOML)           feeds: { symbol -> FeedSnapshot } (JSON)
   │  parse + validate               │  parse (candles/orderbook/trades/
   │  (≥1 symbol, ≥1 fact,            │   funding/open_interest/liquidations)
   │   lookback ≥ 1)                  ▼
   ▼                              per symbol: derive each requested fact
build_context(feeds, spec) ◄───────┘  (six deterministic reductions)
   │  round every magnitude to 1e-8
   │  sort by magnitude desc, then (kind, symbol, ts) asc
   ▼
MarketContext { facts, symbols, lookback }
   │  serde_json::to_string  (compact, fixed-precision floats)
   ▼
the exact bytes every binding returns from a build_context command
   │
   │   ── the deterministic boundary ends here ──
   ▼
copilot-llm: render_prompt(context) → provider.ask() → answer   (CLI `ask` only)
```

- **`ContextSpec`** (`crates/copilot-core/src/spec.rs`) — `symbols`, `lookback`, an optional `timeframe`, and the `facts` to derive. Construction rejects an empty symbol list, an empty fact list and a zero lookback.
- **`FeedSnapshot`** (`src/feed.rs`) — per-symbol `candles`, `orderbook`, `trades`, `funding`, `open_interest`, `liquidations`. `Candle::new` rejects non-finite values, so downstream code needs no `is_finite` guards.
- **`derive`** (`src/derive.rs`) — the six fact reductions, each a pure function of one snapshot. See [FACTS.md](FACTS.md).
- **`build_context`** (`src/builder.rs`) — derives every requested fact for every symbol, drops the insignificant ones, rounds and sorts.
- **`query`** (`src/tool.rs`) — routes a natural-language question to the fact kinds it needs, returning `ToolCall`s. See [TOOL_CALLING.md](TOOL_CALLING.md).

## Parallel vs sequential

With the default `parallel` feature the per-symbol derivations run across rayon;
without it (the WASM build, `--no-default-features`) they run sequentially. The
two are **byte-for-byte identical** — facts sort by a total order
(`f64::total_cmp` on magnitude, then kind, symbol and ts ascending), never a
partial float compare, and every magnitude is rounded to a fixed `1e-8`
precision. The golden suite runs under both feature sets in CI to prove it. See
[GROUNDING.md](GROUNDING.md).

## The command protocol

Every binding drives the core through one entry point, `Copilot::command`, whose
envelope is `{"cmd": "..."}` and whose reply is **always a JSON string**. A domain
error is returned in-band as `{"ok":false,"error":"..."}` — never a panic or a
thrown exception. The commands are:

| `cmd` | Reply |
|-------|-------|
| `set_spec` (`{"spec":…}`) | `{"ok":true}` |
| `build_context` (alias `facts`) (`{"feeds":…}`) | a `MarketContext` |
| `query` (`{"question":"…"}`) | `{"tool_calls":[…]}` |
| `reset` | `{"ok":true}` |
| `version` | `{"version":"0.1.0"}` |

Because the reply is the core's compact JSON verbatim, the context is
byte-identical across every language.

## The LLM boundary

`copilot-llm` is a **separate crate**, not reachable over the C ABI. It renders
the deterministic `MarketContext` into a prompt and calls one OpenAI-compatible
endpoint chosen by a `Provider` preset. The API key is read from the environment
and never logged. Only the CLI's `ask` subcommand links it; the bindings cannot
reach it, so the network and the key stay off the language surface. See
[LLM_ADAPTER.md](LLM_ADAPTER.md).

## Data-driven boundary

The context is **data, not code**: a `Fact` carries a `kind`, a `symbol`, a signed
`value`, a `magnitude`, a `ts` and a ready-made `human` sentence, and a consumer
decides what to do with it. That is why the same output crosses the C ABI and
WASM unchanged, and why a dashboard, an alerting bot or an LLM prompt can be built
in any language without linking the core's internals.
