# Wickra Copilot — WASM

WASM bindings for the `wickra-copilot` deterministic market-context core,
compiled to WebAssembly with wasm-bindgen. Build a `Copilot` from a spec JSON,
drive it with command JSON, read back the `MarketContext` — the same protocol as
every other binding, running in the browser. Only the deterministic core is
exposed — the LLM adapter is never part of this surface, so the network and API
key stay out of the binding.

The core is built with `--no-default-features`, so the context folds
**sequentially** (no rayon thread pool in the browser sandbox) and byte-identical
to the native parallel build.

## Build

```bash
wasm-pack build --target web
```

This emits `pkg/` with the `.wasm` module and JS glue.

## Usage

```js
import init, { Copilot, version } from "./pkg/wickra_copilot_wasm.js";

await init();

const spec = JSON.stringify({ symbols: ["BTCUSDT"], lookback: 3, facts: ["price_move"] });
const feeds = { BTCUSDT: { symbol: "BTCUSDT", candles: [
  { ts: 1, open: 100, high: 100, low: 100, close: 100, volume: 1 },
  { ts: 2, open: 97, high: 97, low: 97, close: 97, volume: 1 },
  { ts: 3, open: 94, high: 94, low: 94, close: 94, volume: 1 },
] } };

const copilot = new Copilot(spec);
const ctx = JSON.parse(copilot.command(JSON.stringify({ cmd: "build_context", feeds })));

console.log(ctx.facts[0].human); // BTCUSDT dropped -6.00% over the last 3 bars.
console.log(version());
```

## API

| Member | Description |
|--------|-------------|
| `new Copilot(specJson)` | Build a copilot from a spec JSON (throws on an invalid spec). |
| `copilot.command(cmdJson)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `copilot.version()` / `version()` | The library version. |
