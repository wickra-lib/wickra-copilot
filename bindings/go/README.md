# Wickra Copilot — Go

Go bindings for the `wickra-copilot` deterministic market-context core over its C
ABI hub. Build a `Copilot` from a spec JSON, drive it with command JSON, read
back the `MarketContext` — the same protocol as every other binding. Only the
deterministic core is exposed; the LLM adapter is never reachable over the C ABI,
so the network and API key stay off this surface.

## Install

```sh
go get github.com/wickra-lib/wickra-copilot-go
```

The binding is cgo over the C ABI: it needs the prebuilt native library staged
under `lib/<goos>_<goarch>/` and the header under `include/` (both shipped in the
release module).

## Usage

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

## API

| Function | Description |
|----------|-------------|
| `New(specJSON)` | Build a copilot from a spec JSON (error on an invalid spec). |
| `(*Copilot).Command(cmdJSON)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `(*Copilot).Close()` | Free the handle (also run by a finalizer). |
| `Version()` | The library version. |

Domain errors (a bad spec, an unknown command) come back in-band as
`{"ok":false,"error":...}`; only unusable arguments and caught panics are hard
errors.
