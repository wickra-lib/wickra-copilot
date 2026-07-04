"use strict";

// Cross-language golden parity: build the copilot from each committed
// `golden/specs/*.json`, run `build_context` over the shared `golden/feeds.json`,
// and assert the context equals `golden/expected/<spec>.json` byte-for-byte.
// Because every binding returns the core's compact `command_json` string
// verbatim, byte equality is the exact cross-language parity check.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Copilot } = require("../index.js");

function findGolden() {
  let dir = __dirname;
  for (let i = 0; i < 8; i++) {
    const g = path.join(dir, "golden");
    if (fs.existsSync(path.join(g, "specs"))) {
      return g;
    }
    dir = path.dirname(dir);
  }
  return null;
}

test("golden contexts are byte-identical", (t) => {
  const golden = findGolden();
  if (!golden) {
    t.skip("golden fixtures not present");
    return;
  }
  const feeds = fs.readFileSync(path.join(golden, "feeds.json"), "utf8");
  const specDir = path.join(golden, "specs");
  for (const file of fs.readdirSync(specDir).filter((f) => f.endsWith(".json"))) {
    const spec = fs.readFileSync(path.join(specDir, file), "utf8");
    const expected = fs
      .readFileSync(path.join(golden, "expected", file), "utf8")
      .trim();
    const copilot = new Copilot(spec);
    const response = copilot.command(
      JSON.stringify({ cmd: "build_context", feeds: JSON.parse(feeds) }),
    );
    assert.strictEqual(response.trim(), expected, `mismatch for ${file}`);
  }
});
