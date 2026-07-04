# Golden fixtures

Cross-language parity fixtures for the **grounding core** — the deterministic
step that turns feeds into facts. Every binding builds a copilot from each
`specs/*.json`, runs `build_context` over the shared feed universe, and asserts
the response equals `expected/<spec>.json` **byte-for-byte**. Because each
binding returns the core's compact `command_json` string verbatim, byte equality
is the exact cross-language check — the same bytes must come out of Rust,
Python, Node.js, WASM, C, C++, C#, Go, Java and R.

> **The golden covers only the grounding core, never the LLM.** It pins the
> `MarketContext` (the facts and their `human` strings) that the copilot feeds to
> a model. The model's answer is non-deterministic and network-bound and is
> deliberately **not** part of any golden. The `copilot-llm` tests only pin the
> rendered prompt bytes and the API-key redaction, offline.

## Files

- **`generate_feeds.py`** — the deterministic feed generator. Every value is a
  fixed function of the bar index (no randomness), so the feeds reproduce
  byte-for-byte across machines. Run from the repo root: `python
  golden/generate_feeds.py`.
- **`feeds/<SYMBOL>.json`** — one `FeedSnapshot` per symbol. This directory form
  is what the CLI reads in `--feeds <dir>` mode (symbol = filename stem).
- **`feeds.json`** — the same universe as a single `{ "<symbol>": FeedSnapshot }`
  map. This is the inline form a `build_context` command takes
  (`{"cmd":"build_context","feeds":<feeds.json>}`) and the shape the
  cross-language golden tests feed to every binding.
- **`specs/*.json`** — five bare `ContextSpec` fixtures
  (`{symbols, lookback, timeframe, facts}`), the shape `ContextSpec::from_json`
  loads. `build_context` selects symbols by `spec.symbols`, so feeding the full
  universe to a single-symbol spec yields only that symbol's facts.
- **`expected/<spec>.json`** — the blessed `MarketContext` for each spec, one
  line of compact JSON with a trailing newline.

### The feed formula

`candles(base, drift, wobble)` emits 40 one-minute bars. The first 20 stay near
`base` (`base + 0.3·sin(i)`); the last 20 — the lookback window — walk by `drift`
with a fixed sinusoidal `wobble`: `close = base + drift·((i−20)/19) +
wobble·sin(1.1·i)`. Each symbol also carries a synthetic order book, two trades,
a funding series, two open-interest points and a liquidation pair:

- **`BTCUSDT`** — dumps ~6% over the window; ask-heavy book; funding flips
  `+ → −`; open interest −4%; a long-liquidation cascade.
- **`ETHUSDT`** — pumps ~9% over the window; bid-heavy book; funding flips
  `− → +`; open interest +6%; a short-liquidation cluster.

### The specs

- **`dump.json`** — `BTCUSDT`, all six fact kinds (the worked example: five
  facts fire; see below).
- **`pump.json`** — `ETHUSDT`, `price_move` + `oi_change`.
- **`funding_flip.json`** — both symbols, `funding_flip` (one per symbol).
- **`liquidation.json`** — both symbols, `liquidation_cluster`.
- **`vol_spike.json`** — `BTCUSDT`, `volatility_spike`. This spec yields an
  **empty** fact set (`{"facts":[],…}`). It is a deliberate, honest golden: with
  the recent window a subset of the baseline window, the volatility ratio is
  mathematically capped below the significance threshold and the fact never
  fires. The golden reflects the real (empty) output rather than a doctored one.

## Regenerating the blessed output

The expected files are `serde_json::to_string(&MarketContext)` — the same string
`command_json`'s `build_context` returns and the CLI's `--format json` prints.
After an intentional core change, re-bless from the real core (never hand-edit):

```bash
cargo build -p wickra-copilot
for spec in golden/specs/*.json; do
  name=$(basename "$spec")
  # the CLI's --spec expects the Config wrapper; the golden specs are bare
  python -c "import json,sys; print(json.dumps({'spec': json.load(open(sys.argv[1]))}))" "$spec" > /tmp/cfg.json
  cargo run -q -p wickra-copilot -- context --spec /tmp/cfg.json --stdin --format json \
    < golden/feeds.json > "golden/expected/$name"
done
```

The blessed files carry a trailing newline from the CLI's `println!`; the
`command_json` reply does not, so the in-core golden test trims the expectation.
Keep fact values unambiguous (no build-dependent signed zeros) so the bytes match
across every toolchain and optimization level.
