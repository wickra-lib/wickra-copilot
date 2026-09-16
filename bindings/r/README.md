<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![r-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — R

---

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — for R. `install.packages("wickracopilot", repos = "https://wickra-lib.r-universe.dev")` — over the C ABI via `.Call`, prebuilt library fetched on install.**

R bindings for the `wickra-copilot` deterministic market-context core, over its C
ABI hub (`.Call`). Build a copilot from a spec JSON, drive it with command JSON,
read back the `MarketContext` — the same protocol as the CLI and every other
binding. Only the deterministic core is exposed; the LLM adapter is never
reachable over the C ABI, so the network and API key stay off this surface.

## Install

From r-universe, which builds the package with the prebuilt C ABI library bundled:

```r
install.packages("wickracopilot", repos = "https://wickra-lib.r-universe.dev")
```

A C toolchain (Rtools on Windows) is required for the thin `.Call` glue layer.

### Requirements

The package compiles against the `wickra-copilot` C ABI. Point the build at the
header and library with two environment variables (set by CI / the installer):

- `WKCOPILOT_INC` — the directory holding `wickra_copilot.h` (i.e. `bindings/c/include`).
- `WKCOPILOT_LIB` — the directory holding the built shared library (i.e. the
  Cargo `target/release` after `cargo build -p wickra-copilot-c --release`).

At run time the loader finds the shared library via `PATH` (Windows) or
`LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` (Linux/macOS).

### Building from this repository (contributors)

```sh
cargo build -p wickra-copilot-c --release
WKCOPILOT_INC=../c/include WKCOPILOT_LIB=../../target/release R CMD INSTALL .
Rscript tests/run_tests.R
```

## Quick start

```r
library(wickracopilot)

spec <- '{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}'
copilot <- wkcopilot_new(spec)

feeds <- paste0(
  '{"cmd":"build_context","feeds":{"BTCUSDT":{"symbol":"BTCUSDT","candles":[',
  '{"ts":1,"open":100,"high":100,"low":100,"close":100,"volume":1},',
  '{"ts":2,"open":97,"high":97,"low":97,"close":97,"volume":1},',
  '{"ts":3,"open":94,"high":94,"low":94,"close":94,"volume":1}]}}}'
)
cat(wkcopilot_command(copilot, feeds), "\n")
cat(wkcopilot_version(), "\n")
```

### API

| Function | Description |
|----------|-------------|
| `wkcopilot_new(spec_json)` | Build a copilot from a spec JSON (errors on an invalid spec). |
| `wkcopilot_command(copilot, cmd_json)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `wkcopilot_version()` | The library version. |

The handle is an external pointer with a finalizer, so it is freed
automatically. Domain errors (a bad spec, an unknown command) come back in-band
as `{"ok":false,"error":...}`; only unusable arguments and caught panics raise.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of R's native `.Call` interface over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-copilot/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-copilot>
- **Docs** (guides, spec reference, cookbook): <https://copilot.wickra.org>
- **Runnable example:** [`examples/r/`](https://github.com/wickra-lib/wickra-copilot/tree/main/examples/r)

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
