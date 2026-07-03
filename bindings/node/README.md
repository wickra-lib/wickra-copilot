# Wickra Copilot — Node.js

Node.js bindings for [`copilot-core`](https://github.com/wickra-lib/wickra-copilot),
built with [napi-rs]. The surface mirrors every other Wickra binding: build a
`Copilot` from a spec JSON, drive it with command JSONs, and read back a ranked
`MarketContext` of hard facts. Only the deterministic core is exposed — the LLM
adapter is never part of this surface, so the network and API key stay out of the
binding.

## Install

```sh
npm install wickra-copilot
```

## Usage

```js
const { Copilot } = require("wickra-copilot");

const spec = JSON.stringify({ symbols: ["BTCUSDT"], lookback: 3, facts: ["price_move"] });
const feeds = { BTCUSDT: { symbol: "BTCUSDT", candles: [
  { ts: 1, open: 100, high: 100, low: 100, close: 100, volume: 1 },
  { ts: 2, open: 97, high: 97, low: 97, close: 97, volume: 1 },
  { ts: 3, open: 94, high: 94, low: 94, close: 94, volume: 1 },
] } };

const copilot = new Copilot(spec);
const ctx = JSON.parse(copilot.command(JSON.stringify({ cmd: "build_context", feeds })));
console.log(ctx.facts[0].human); // BTCUSDT dropped -6.00% over the last 3 bars.
```

## Surface

- **`new Copilot(specJson)`** builds a copilot from a spec JSON (`""` or `"{}"`
  for an empty handle whose spec is set later). Throws on a malformed spec.
- **`copilot.command(cmdJson)`** applies a command JSON (`set_spec`,
  `build_context`, `facts`, `query`, `reset`, `version`) and returns the response
  JSON. A bad spec or unknown command comes back in-band as
  `{"ok": false, "error": ...}`.
- **`copilot.version()`** / **`version()`** return the library version.

## Building from source

```sh
npm install
npm run build   # napi build --platform --release; regenerates index.js/index.d.ts
npm test
```

[napi-rs]: https://napi.rs
