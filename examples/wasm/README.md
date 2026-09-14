# wickra-copilot WASM examples

Browser demos for the `wickra-copilot-wasm` binding.

The WASM build carries the whole grounding core: the same fact derivations
and the same context bytes the CLI and the other nine bindings produce. A spec
is data, not code, so the bytes on this page are the same ones
`examples/node/context.js` sends, and the facts are the same facts. The LLM
adapter is not part of the WASM build; the page stops at the deterministic
context and the tool calls a query derives from it.

## Build

The module ships as a `wasm-pack` `--target web` bundle. Build it once from the
repository root:

```bash
wasm-pack build bindings/wasm --target web --release
```

That writes `bindings/wasm/pkg/` with the `.wasm` binary, the JS loader and the
type declarations the page imports.

## Run

The page loads its module over `http://`, not `file://`, because ES module
imports and `WebAssembly.instantiateStreaming` both need a real origin. Serve the
repository root:

```bash
python -m http.server 8000
```

Then open `http://localhost:8000/examples/wasm/context.html`.

## Pages

| Page | What it does |
|------|--------------|
| `context.html` | Folds a three-bar BTC dump into a `MarketContext`, then asks the same question against the stored context and against the context passed inline, showing that both ways answer the same. The page counterpart of `examples/node/context.js`. |

## See also

- [examples/README.md](../README.md) — the same context in every other language.
- [bindings/wasm/README.md](../../bindings/wasm/README.md) — the binding itself.
