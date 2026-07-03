"use strict";

const { test } = require("node:test");
const assert = require("node:assert");
const { Copilot, version } = require("../index.js");

const SPEC = JSON.stringify({
  symbols: ["BTCUSDT"],
  lookback: 3,
  facts: ["price_move"],
});

// BTC drops 6% over three bars -> one significant price-move fact.
const FEEDS = {
  BTCUSDT: {
    symbol: "BTCUSDT",
    candles: [
      { ts: 1, open: 100.0, high: 100.0, low: 100.0, close: 100.0, volume: 1.0 },
      { ts: 2, open: 97.0, high: 97.0, low: 97.0, close: 97.0, volume: 1.0 },
      { ts: 3, open: 94.0, high: 94.0, low: 94.0, close: 94.0, volume: 1.0 },
    ],
  },
};

test("build_context roundtrip returns the market context", () => {
  const copilot = new Copilot(SPEC);
  const ctx = JSON.parse(copilot.command(JSON.stringify({ cmd: "build_context", feeds: FEEDS })));
  assert.deepStrictEqual(ctx.symbols, ["BTCUSDT"]);
  assert.strictEqual(ctx.facts[0].kind, "price_move");
  assert.ok(Math.abs(ctx.facts[0].value - -6.0) < 1e-9);
});

test("query routes to the price-move tool call", () => {
  const copilot = new Copilot(SPEC);
  copilot.command(JSON.stringify({ cmd: "build_context", feeds: FEEDS }));
  const result = JSON.parse(copilot.command(JSON.stringify({ cmd: "query", question: "why did BTC dump" })));
  const kinds = new Set(result.tool_calls.map((c) => c.kind));
  assert.ok(kinds.has("price_move"));
});

test("version matches the module-level function", () => {
  const copilot = new Copilot(SPEC);
  assert.strictEqual(copilot.version(), version());
});

test("a malformed spec throws", () => {
  assert.throws(() => new Copilot("not json"));
});
