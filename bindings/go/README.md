<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Copilot — the local market copilot for Go" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-copilot-go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — Go

---

**The deterministic market-context core for Go, over the Wickra C ABI hub via cgo.**

[Wickra Copilot](https://github.com/wickra-lib/wickra-copilot) grounds a market copilot in real order-book, liquidation and funding microstructure, deriving a deterministic `MarketContext` from feeds. This package is the Go binding: it exposes only the deterministic core over cgo — the LLM adapter is never reachable over the C ABI, so the network and API key stay off this surface.

## Install

Use the published **`wickra-copilot-go`** module, which bundles the prebuilt C ABI library
for every platform, so `go get` + `go build` works with no extra steps (a C
compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-copilot-go
```

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


`wickra-copilot-go` is generated from this directory by the release pipeline: it mirrors the
Go sources, the vendored C ABI header (`include/wickra_copilot.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Windows the DLL must be discoverable at
run time (next to the executable or on `PATH`).

## Building from this repository (contributors)

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

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-APACHE), at your option.
