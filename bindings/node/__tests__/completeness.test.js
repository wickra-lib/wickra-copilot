"use strict";

// Parity guard: the Node binding must expose the full public surface of the
// copilot, so an export dropped in a refactor fails loudly here (mirrors the
// completeness check in the main wickra repo).

const { test } = require("node:test");
const assert = require("node:assert");
const wickra = require("../index.js");

test("module exposes Copilot and version", () => {
  assert.strictEqual(typeof wickra.Copilot, "function");
  assert.strictEqual(typeof wickra.version, "function");
});

test("Copilot exposes command and version", () => {
  for (const name of ["command", "version"]) {
    assert.strictEqual(
      typeof wickra.Copilot.prototype[name],
      "function",
      `Copilot is missing ${name}`,
    );
  }
});

// Strict parity: pin the exact public surface so a stray export (or a dropped
// one) fails here, matching the exact-surface guards in the Python and R
// bindings.
test("module surface is exactly {Copilot, version}", () => {
  assert.deepStrictEqual(Object.keys(wickra).sort(), ["Copilot", "version"]);
});

test("Copilot surface is exactly {command, version}", () => {
  const methods = Object.getOwnPropertyNames(wickra.Copilot.prototype)
    .filter((name) => name !== "constructor")
    .sort();
  assert.deepStrictEqual(methods, ["command", "version"]);
});
