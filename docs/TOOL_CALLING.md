# Tool calling

Before the LLM ever runs, the copilot decides **which facts a question is about**.
That routing is deterministic, golden-pinned, and defined in
[`crates/copilot-core/src/tool.rs`](../crates/copilot-core/src/tool.rs). It is the
`query` command of the core.

## The `query` command

```json
{ "cmd": "query", "question": "why did BTC dump?" }
```

returns

```json
{ "tool_calls": [
  { "tool": "get_fact", "symbol": "BTCUSDT", "kind": "price_move" },
  { "tool": "get_fact", "symbol": "BTCUSDT", "kind": "liquidation_cluster" }
] }
```

A `ToolCall` names the `tool` (always `get_fact` today), the `symbol` to read,
and the fact `kind`. The list contains one call per `(kind, symbol)` that both the
question routes to **and** exists in the current context, sorted by `(kind,
symbol)` ascending. Build the context first (`build_context`); `query` reads it.

## The routing table

The question is lowercased and matched, case-insensitively, against a fixed
keyword table. Every matched keyword contributes its fact kinds; the union is
requested.

| Keyword (substring) | Fact kinds requested |
|---------------------|----------------------|
| `dump`, `crash`, `sell` | `price_move`, `liquidation_cluster` |
| `drop`, `fall`, `rise`, `move` | `price_move` |
| `pump`, `rally`, `moon` | `price_move`, `oi_change` |
| `surge` | `price_move`, `volatility_spike` |
| `liquidat`, `cascade` | `liquidation_cluster` |
| `leverage` | `oi_change`, `liquidation_cluster` |
| `funding` | `funding_flip` |
| `open interest` | `oi_change` |
| `order book`, `orderbook`, `imbalance`, `book` | `orderbook_imbalance` |
| `volatil` | `volatility_spike` |

If a question matches **no** keyword, the router falls back to **all six** fact
kinds — the context speaks for itself rather than returning nothing.

## Determinism

Routing uses ordered set operations (a `BTreeSet` keyed by `(kind, symbol)`), so
the result both deduplicates and comes out in a fixed order. The same question
against the same context always yields the same tool calls, in every language.
The golden corpus and the conformance tests pin the routing.

## From routing to an answer

`query` is the deterministic first step. The CLI's `ask` subcommand runs it, reads
the routed facts out of the context, renders them into a prompt, and hands that to
the LLM adapter ([LLM_ADAPTER.md](LLM_ADAPTER.md)). The routing decides *what the
model is allowed to see*; the model only reasons over facts that were actually
derived from the feed.

## See also

[Facts](FACTS.md) · [LLM adapter](LLM_ADAPTER.md) · [Grounding](GROUNDING.md) ·
[Cookbook](Cookbook.md).
