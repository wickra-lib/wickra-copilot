package copilot

import (
	"encoding/json"
	"math"
	"strings"
	"testing"
)

const spec = `{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}`

func candle(ts int, close float64) map[string]any {
	return map[string]any{
		"ts": ts, "open": close, "high": close, "low": close, "close": close, "volume": 1.0,
	}
}

func TestVersion(t *testing.T) {
	if Version() == "" {
		t.Fatal("empty version")
	}
}

func TestBuildContextRoundtrip(t *testing.T) {
	c, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer c.Close()

	// BTC drops 6% over three bars -> one significant price-move fact.
	feeds := map[string]any{"BTCUSDT": map[string]any{
		"symbol":  "BTCUSDT",
		"candles": []map[string]any{candle(1, 100.0), candle(2, 97.0), candle(3, 94.0)},
	}}
	build, err := json.Marshal(map[string]any{"cmd": "build_context", "feeds": feeds})
	if err != nil {
		t.Fatal(err)
	}

	raw, err := c.Command(string(build))
	if err != nil {
		t.Fatal(err)
	}
	var ctx struct {
		Symbols []string `json:"symbols"`
		Facts   []struct {
			Kind  string  `json:"kind"`
			Value float64 `json:"value"`
		} `json:"facts"`
	}
	if err := json.Unmarshal([]byte(raw), &ctx); err != nil {
		t.Fatal(err)
	}
	if len(ctx.Symbols) != 1 || ctx.Symbols[0] != "BTCUSDT" {
		t.Fatalf("expected [BTCUSDT], got %+v", ctx.Symbols)
	}
	if len(ctx.Facts) != 1 || ctx.Facts[0].Kind != "price_move" {
		t.Fatalf("expected one price_move fact, got %+v", ctx.Facts)
	}
	if math.Abs(ctx.Facts[0].Value-(-6.0)) > 1e-9 {
		t.Fatalf("expected value -6.0, got %v", ctx.Facts[0].Value)
	}
}

func TestInvalidSpec(t *testing.T) {
	if _, err := New("not json"); err == nil {
		t.Fatal("expected an error for an invalid spec")
	}
}

func TestUnknownCommandIsInBandError(t *testing.T) {
	c, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer c.Close()

	// An unknown command is not a hard error: the C ABI returns a length and the
	// error surfaces in-band as {"ok":false,...} JSON.
	raw, err := c.Command(`{"cmd":"nope"}`)
	if err != nil {
		t.Fatalf("unexpected hard error: %v", err)
	}
	if !strings.Contains(raw, `"ok":false`) {
		t.Fatalf("expected an in-band error, got: %s", raw)
	}
}
