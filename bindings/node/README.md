<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/npm.svg)](https://www.npmjs.com/package/wickra-copilot)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — Node.js

---

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — for Node.js. `npm install wickra-copilot` — prebuilt native binary, no system dependencies.**

Node.js bindings for [`wickra-copilot-core`](https://github.com/wickra-lib/wickra-copilot),
built with [napi-rs]. The surface mirrors every other Wickra binding: build a
`Copilot` from a spec JSON, drive it with command JSONs, and read back a ranked
`MarketContext` of hard facts. Only the deterministic core is exposed — the LLM
adapter is never part of this surface, so the network and API key stay out of the
binding.

## Install

```bash
npm install wickra-copilot
```

The native addon ships as a prebuilt binary per platform (Linux, macOS,
Windows — x64 and arm64), selected automatically through optional
dependencies. There is nothing to compile.

### Building from this repository (contributors)

```sh
npm install
npm run build   # napi build --platform --release; regenerates index.js/index.d.ts
npm test
```

## Quick start

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

### Surface

- **`new Copilot(specJson)`** builds a copilot from a spec JSON (`""` or `"{}"`
  for an empty handle whose spec is set later). Throws on a malformed spec.
- **`copilot.command(cmdJson)`** applies a command JSON (`set_spec`,
  `build_context`, `facts`, `query`, `reset`, `version`) and returns the response
  JSON. A bad spec or unknown command comes back in-band as
  `{"ok": false, "error": ...}`.
- **`copilot.version()`** / **`version()`** return the library version.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of napi-rs, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-copilot/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-copilot>
- **Docs** (guides, spec reference, cookbook): <https://copilot.wickra.org>
- **Runnable example:** [`examples/node/`](https://github.com/wickra-lib/wickra-copilot/tree/main/examples/node)

Wickra Copilot ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-copilot/blob/main/SECURITY.md>.

## Disclaimer

Wickra Copilot is analysis software: it builds a deterministic market context and
relays it to a language model of your choosing. It is provided "as is", without
warranty of any kind. LLM output can be wrong and is **not financial advice**; the
copilot only reports facts and places no orders. Trading carries risk of loss;
review the code and use at your own discretion.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-MIT) at your option.

[napi-rs]: https://napi.rs
