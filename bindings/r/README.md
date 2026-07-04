# Wickra Copilot — R

R bindings for the `wickra-copilot` deterministic market-context core, over its C
ABI hub (`.Call`). Build a copilot from a spec JSON, drive it with command JSON,
read back the `MarketContext` — the same protocol as the CLI and every other
binding. Only the deterministic core is exposed; the LLM adapter is never
reachable over the C ABI, so the network and API key stay off this surface.

## Requirements

The package compiles against the `wickra-copilot` C ABI. Point the build at the
header and library with two environment variables (set by CI / the installer):

- `WKCOPILOT_INC` — the directory holding `wickra_copilot.h` (i.e. `bindings/c/include`).
- `WKCOPILOT_LIB` — the directory holding the built shared library (i.e. the
  Cargo `target/release` after `cargo build -p wickra-copilot-c --release`).

At run time the loader finds the shared library via `PATH` (Windows) or
`LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` (Linux/macOS).

## Usage

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

## API

| Function | Description |
|----------|-------------|
| `wkcopilot_new(spec_json)` | Build a copilot from a spec JSON (errors on an invalid spec). |
| `wkcopilot_command(copilot, cmd_json)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `wkcopilot_version()` | The library version. |

The handle is an external pointer with a finalizer, so it is freed
automatically. Domain errors (a bad spec, an unknown command) come back in-band
as `{"ok":false,"error":...}`; only unusable arguments and caught panics raise.

## Test

```sh
cargo build -p wickra-copilot-c --release
WKCOPILOT_INC=../c/include WKCOPILOT_LIB=../../target/release R CMD INSTALL .
Rscript tests/run_tests.R
```
