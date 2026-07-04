// A runnable Node.js example: build a market context through the binding.
//
//   ( cd bindings/node && npm install && npm run build )
//   ( cd examples/node && npm install && node context.js )

"use strict";

const { Copilot, version } = require("wickra-copilot");

const SPEC = JSON.stringify({
  symbols: ["BTCUSDT"],
  lookback: 3,
  facts: ["price_move"],
});

const FEEDS = {
  BTCUSDT: {
    symbol: "BTCUSDT",
    candles: [
      { ts: 1, open: 100, high: 100, low: 100, close: 100, volume: 1 },
      { ts: 2, open: 97, high: 97, low: 97, close: 97, volume: 1 },
      { ts: 3, open: 94, high: 94, low: 94, close: 94, volume: 1 },
    ],
  },
};

const copilot = new Copilot(SPEC);
const response = copilot.command(
  JSON.stringify({ cmd: "build_context", feeds: FEEDS }),
);
const context = JSON.parse(response);

console.log("wickra-copilot", version());
console.log(response);
console.log(`  facts: ${context.facts.length}`);
