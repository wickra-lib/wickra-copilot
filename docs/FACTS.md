# Facts

A `MarketContext` is a ranked list of **facts**. Each fact is a pure,
deterministic reduction of one `FeedSnapshot`, defined in
[`crates/copilot-core/src/derive.rs`](../crates/copilot-core/src/derive.rs). A
fact is emitted only when its `magnitude` clears a kind-specific significance
threshold; a missing feed or a non-finite result yields nothing, never a zero
fact. Every `human` string is a fixed English template with rounded numbers — no
LLM is involved in producing them.

## The `Fact` shape

```json
{ "kind": "price_move", "symbol": "BTCUSDT", "value": -6.44, "magnitude": 6.44,
  "ts": 1700002340, "human": "BTCUSDT dropped -6.44% over the last 20 bars." }
```

| Field | Meaning |
|-------|---------|
| `kind` | one of the six fact kinds below (snake_case) |
| `symbol` | the symbol the fact is about |
| `value` | the signed measurement (direction matters) |
| `magnitude` | the ranking key — `\|value\|` for most kinds; the significance threshold applies to this |
| `ts` | the timestamp the fact is anchored to |
| `human` | a byte-pinned English sentence describing the fact |

Facts are sorted by `magnitude` descending, then `kind`, `symbol` and `ts`
ascending — a total order, so the list is identical across every language and
both build profiles.

## The six kinds

### `price_move`

Percentage change in `close` over the last `lookback` candles:
`value = (last/first - 1) * 100`. Emitted when `|value| ≥ 1.0` (%). Anchored to
the last candle's `ts`.

> `BTCUSDT dropped -6.44% over the last 20 bars.` (rose / dropped by sign)

### `orderbook_imbalance`

Top-of-book (depth 10) bid/ask volume skew:
`value = (bid_vol - ask_vol) / (bid_vol + ask_vol)`, in `[-1, 1]`. Emitted when
`|value| ≥ 0.20`. Anchored to the order book's `ts`.

> `BTCUSDT order book is bid-heavy (imbalance +0.50).` (bid- / ask-heavy by sign)

### `liquidation_cluster`

Liquidation notional summed over the snapshot, signed by the dominant side:
`value = long_notional - short_notional`, `magnitude = long_notional +
short_notional` (total notional, the ranking key). Emitted when
`magnitude ≥ 1.0`. Anchored to the last liquidation `ts`.

> `BTCUSDT saw 3968000.00 notional liquidated (long-dominated).`

### `funding_flip`

The most recent funding-rate sign change in the window: `value` is the funding
rate after the flip, `magnitude` is the size of the rate step across it. Emitted
whenever a sign change exists and both numbers are finite. Anchored to the flip
`ts`.

> `BTCUSDT funding flipped to negative (-0.0002).` (positive / negative by sign)

### `oi_change`

Percentage change in open interest over the window:
`value = (last/first - 1) * 100`. Emitted when `|value| ≥ 1.0` (%). Anchored to
the last OI point's `ts`.

> `BTCUSDT open interest rose +5.00% over the window.` (rose / fell by sign)

### `volatility_spike`

Recent realised volatility relative to a longer baseline:
`value = stddev(last lookback bars) / stddev(last 2·lookback bars)`. Because the
recent window is a subset of the baseline, this ratio is capped near √2, so it
rarely clears the emission threshold of `≥ 1.5x`. When it does not, no fact is
produced — the golden `vol_spike` context is legitimately empty. Anchored to the
last candle's `ts`.

> `BTCUSDT volatility spiked to 2.00x its baseline.`

## Thresholds at a glance

| Kind | Value | Emit when | Book depth |
|------|-------|-----------|------------|
| `price_move` | `(last/first−1)·100` | `\|value\| ≥ 1.0` | — |
| `orderbook_imbalance` | `(bid−ask)/(bid+ask)` | `\|value\| ≥ 0.20` | 10 |
| `liquidation_cluster` | `long−short` notional | `total ≥ 1.0` | — |
| `funding_flip` | rate after flip | a sign change exists | — |
| `oi_change` | `(last/first−1)·100` | `\|value\| ≥ 1.0` | — |
| `volatility_spike` | recent/baseline stddev | `value ≥ 1.5` | — |

Every threshold, formula and `human` template is pinned by unit tests in
`derive.rs` (`human_templates_are_byte_exact` pins all eleven prose branches), so
they cannot drift without a failing test.

## See also

[Grounding](GROUNDING.md) · [Tool calling](TOOL_CALLING.md) ·
[Architecture](ARCHITECTURE.md) · [Cookbook](Cookbook.md).
