<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/npm.svg)](https://www.npmjs.com/package/wickra-copilot-wasm)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — WASM

---

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — for WASM. `npm install wickra-copilot-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

WASM bindings for the `wickra-copilot` deterministic market-context core,
compiled to WebAssembly with wasm-bindgen. Build a `Copilot` from a spec JSON,
drive it with command JSON, read back the `MarketContext` — the same protocol as
every other binding, running in the browser. Only the deterministic core is
exposed — the LLM adapter is never part of this surface, so the network and API
key stay out of the binding.

The core is built with `--no-default-features`, so the context folds
**sequentially** (no rayon thread pool in the browser sandbox) and byte-identical
to the native parallel build.

## Install

```bash
npm install wickra-copilot-wasm
```

### Building from this repository (contributors)

```bash
wasm-pack build --target web
```

This emits `pkg/` with the `.wasm` module and JS glue.

## Quick start

```js
import init, { Copilot, version } from "wickra-copilot-wasm";

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

### API

| Member | Description |
|--------|-------------|
| `new Copilot(specJson)` | Build a copilot from a spec JSON (throws on an invalid spec). |
| `copilot.command(cmdJson)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `copilot.version()` / `version()` | The library version. |

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of wasm-bindgen, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-copilot/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-copilot>
- **Docs** (guides, spec reference, cookbook): <https://copilot.wickra.org>
- **Runnable example:** [`examples/wasm/`](https://github.com/wickra-lib/wickra-copilot/tree/main/examples/wasm)

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
