"use strict";

// Operating-mode equivalence through the binding, over the golden corpus. The
// command protocol offers two ways to reach the same answer, and both must
// return the same bytes: `facts` is an alias of `build_context`, and `query`
// answers either against the context the handle stored from its last
// `build_context` or against a context passed inline. The core pins this in
// Rust (operating_modes.rs); this checks the boundary the WebAssembly
// core crosses. A missing corpus is a failure, not a skip.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Copilot } = require("../pkg-node/wickra_copilot_wasm.js");

const QUESTION = "what moved and why?";

function findGolden() {
  const g = path.resolve(__dirname, "..", "..", "..", "golden");
  return fs.existsSync(path.join(g, "specs")) ? g : null;
}

test("facts alias and inline query match the stored path", () => {
  const golden = findGolden();
  assert.ok(golden, "golden corpus not found");
  const feeds = JSON.parse(fs.readFileSync(path.join(golden, "feeds.json"), "utf8"));
  const specDir = path.join(golden, "specs");
  const files = fs.readdirSync(specDir).filter((f) => f.endsWith(".json")).sort();
  assert.ok(files.length > 0, "golden corpus not found");
  for (const file of files) {
    const spec = fs.readFileSync(path.join(specDir, file), "utf8");
    const expected = fs.readFileSync(path.join(golden, "expected", file), "utf8").trim();

    const stored = new Copilot(spec);
    const built = stored.command(JSON.stringify({ cmd: "build_context", feeds }));
    assert.strictEqual(built, expected, file);
    const fromStored = stored.command(JSON.stringify({ cmd: "query", question: QUESTION }));

    const fresh = new Copilot(spec);
    const fromInline = fresh.command(
      JSON.stringify({ cmd: "query", question: QUESTION, context: JSON.parse(built) }),
    );
    assert.strictEqual(fromInline, fromStored, file);

    const alias = new Copilot(spec);
    assert.strictEqual(alias.command(JSON.stringify({ cmd: "facts", feeds })), built, file);
  }
});
