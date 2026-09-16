<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-copilot-go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — Go

---

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — for Go. `go get github.com/wickra-lib/wickra-copilot-go` — over the C ABI via cgo, prebuilt library bundled in the module.**

[Wickra Copilot](https://github.com/wickra-lib/wickra-copilot) grounds a market copilot in real order-book, liquidation and funding microstructure, deriving a deterministic `MarketContext` from feeds. This package is the Go binding: it exposes only the deterministic core over cgo — the LLM adapter is never reachable over the C ABI, so the network and API key stay off this surface.

## Install

Use the published **`wickra-copilot-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps
(a C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-copilot-go
```

`wickra-copilot-go` is generated from this directory by the release pipeline: it mirrors
the Go sources, the vendored C ABI header (`include/wickra_copilot.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Linux/macOS the library path is baked
in via rpath; on Windows the DLL must be discoverable at run time (next to the
executable or on `PATH`).

### Building from this repository (contributors)

This `bindings/go` directory is the development source. To build it directly,
compile the C ABI hub and stage the library into the per-platform directory cgo
links against:

```bash
cargo build -p wickra-copilot-c --release
mkdir -p bindings/go/lib/linux_amd64                    # match your GOOS_GOARCH
cp target/release/libwickra_copilot.so    bindings/go/lib/linux_amd64/   # Linux
cp target/release/libwickra_copilot.dylib bindings/go/lib/darwin_arm64/  # macOS (arm64)
cp target/release/wickra_copilot.dll      bindings/go/lib/windows_amd64/ # Windows
```

Then, with the library on the loader path, run `go test ./...` from this directory.

## Quick start

```go
package main

import (
	"encoding/json"
	"fmt"

	copilot "github.com/wickra-lib/wickra-copilot-go"
)

func main() {
	spec := `{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}`
	c, err := copilot.New(spec)
	if err != nil {
		panic(err)
	}
	defer c.Close()

	feeds := map[string]any{"BTCUSDT": map[string]any{
		"symbol": "BTCUSDT",
		"candles": []map[string]any{
			{"ts": 1, "open": 100.0, "high": 100.0, "low": 100.0, "close": 100.0, "volume": 1.0},
			{"ts": 2, "open": 97.0, "high": 97.0, "low": 97.0, "close": 97.0, "volume": 1.0},
			{"ts": 3, "open": 94.0, "high": 94.0, "low": 94.0, "close": 94.0, "volume": 1.0},
		},
	}}
	build, _ := json.Marshal(map[string]any{"cmd": "build_context", "feeds": feeds})
	out, err := c.Command(string(build))
	if err != nil {
		panic(err)
	}
	fmt.Println(out)
}
```

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of cgo over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-copilot/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-copilot>
- **Docs** (guides, spec reference, cookbook): <https://copilot.wickra.org>
- **Runnable example:** [`examples/go/`](https://github.com/wickra-lib/wickra-copilot/tree/main/examples/go)

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
