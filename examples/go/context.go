// A runnable Go example: build a market context through the binding.
//
//	cargo build --release -p wickra-copilot-c
//	# stage the library under bindings/go/lib/<goos>_<goarch>/ (CI does this)
//	cd examples/go && go run .
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra-copilot-go"
)

const spec = `{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}`

const build = `{"cmd":"build_context","feeds":{"BTCUSDT":{"symbol":"BTCUSDT","candles":[` +
	`{"ts":1,"open":100,"high":100,"low":100,"close":100,"volume":1},` +
	`{"ts":2,"open":97,"high":97,"low":97,"close":97,"volume":1},` +
	`{"ts":3,"open":94,"high":94,"low":94,"close":94,"volume":1}]}}}`

func main() {
	copilot, err := wickra.New(spec)
	if err != nil {
		panic(err)
	}
	defer copilot.Close()

	context, err := copilot.Command(build)
	if err != nil {
		panic(err)
	}

	fmt.Println("wickra-copilot", wickra.Version())
	fmt.Println(context)
}
