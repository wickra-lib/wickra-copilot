"use strict";

// Tests over the wasm-pack (nodejs target) output, run by the WASM job after
// `wasm-pack build --target nodejs --out-dir pkg-node`:
//
//   golden   every golden/specs/*.json builds, through the WebAssembly core
//            over the shared golden/feeds.json, to the byte-identical
//            golden/expected/<spec>.json the native bindings produce;
//   smoke    the version matches the module export, an unknown command is an
//            in-band error, a spec that is not a spec is refused.
//
// The require is hard: a missing build must fail the job, not skip it, and a
// missing corpus is a failure, not a skip.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Copilot, version } = require("../pkg-node/wickra_copilot_wasm.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");
const SPECS = path.join(GOLDEN, "specs");
const EXPECTED = path.join(GOLDEN, "expected");

test("golden contexts are byte-identical through the wasm core", () => {
  const specs = fs.readdirSync(SPECS).filter((f) => f.endsWith(".json"));
  assert.ok(specs.length > 0, "golden corpus not found");
  const feeds = JSON.parse(fs.readFileSync(path.join(GOLDEN, "feeds.json"), "utf8"));
  for (const file of specs) {
    const spec = fs.readFileSync(path.join(SPECS, file), "utf8");
    const expected = fs.readFileSync(path.join(EXPECTED, file), "utf8").trim();
    const copilot = new Copilot(spec);
    const response = copilot.command(JSON.stringify({ cmd: "build_context", feeds }));
    assert.strictEqual(response.trim(), expected, `mismatch for ${file}`);
  }
});

test("the version is the crate's, on the module and on an instance", () => {
  const spec = fs.readFileSync(path.join(SPECS, "dump.json"), "utf8");
  const copilot = new Copilot(spec);
  assert.match(version(), /^\d+\.\d+\.\d+/);
  assert.strictEqual(copilot.version(), version());
});

test("a spec that is not a spec is refused", () => {
  assert.throws(() => new Copilot('{"not": "a spec"}'));
});

test("an unknown command is an in-band error", () => {
  const spec = fs.readFileSync(path.join(SPECS, "dump.json"), "utf8");
  const copilot = new Copilot(spec);
  const response = JSON.parse(copilot.command(JSON.stringify({ cmd: "no_such_command" })));
  assert.strictEqual(response.ok, false);
  assert.match(response.error, /unknown cmd/);
});
