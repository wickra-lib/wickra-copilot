# Examples

A runnable "build a market context" example in every language. Each one builds a
copilot from the same spec (a single `price_move` fact over a 3-bar lookback),
feeds an inline three-bar dump (`100 → 97 → 94` on `BTCUSDT`) and prints the
version and the resulting `MarketContext`. The examples are self-contained: the
spec and feeds are inline, so there is no shared `data/` directory to load (the
cross-language golden fixtures live in [`../golden/`](../golden)).

| Language | Path | Run |
|----------|------|-----|
| Rust | [`rust/`](rust/) | `cargo run -p wickra-copilot-example` |
| Python | [`python/context.py`](python/context.py) | `pip install wickra-copilot && python examples/python/context.py` |
| Node.js | [`node/`](node/) | `cd examples/node && npm install && node context.js` |
| C / C++ | [`c/`](c/) | see below |
| Go | [`go/`](go/) | `cd examples/go && go run .` |
| C# | [`csharp/Context/`](csharp/Context/) | `dotnet run --project examples/csharp/Context` |
| Java | [`java/Context.java`](java/Context.java) | see the header comment |
| R | [`r/context.R`](r/context.R) | `Rscript examples/r/context.R` |

The native bindings (Python, Node.js) load their own compiled library. The bindings
that go through the C ABI (Go, C#, Java, R, and the C / C++ example itself) need the
C ABI library built first:

```bash
cargo build --release -p wickra-copilot-c
```

## C / C++

The C and C++ examples build with CMake and run under ctest:

```bash
cargo build --release -p wickra-copilot-c
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

On Windows the build copies `wickra_copilot.dll` next to each executable, since
there is no rpath.

## Asking a model

The [`ask/`](ask/) example goes one step further: it grounds the same context and
then asks a real LLM to answer *only* from those facts, through the separate
`copilot-llm` adapter. It is the one example that talks to the network, so it is
**not** part of CI — it compiles there but is only ever run locally. It defaults
to a local [Ollama](https://ollama.com) server (no API key):

```bash
cargo run -p wickra-copilot-ask-example

# or any OpenAI-compatible endpoint:
WICKRA_COPILOT_PROVIDER=openai \
  WICKRA_COPILOT_API_KEY=sk-... \
  WICKRA_COPILOT_MODEL=gpt-4o-mini \
  cargo run -p wickra-copilot-ask-example
```

The grounding context is deterministic; the model's answer is not, and is never
pinned by any test.

## Expected output

Every context example prints the version and the context, for example:

```text
wickra-copilot 0.1.0
{"facts":[{"kind":"price_move","symbol":"BTCUSDT","value":-6.0,"magnitude":6.0,"ts":3,"human":"BTCUSDT dropped -6.00% over the last 3 bars."}],"symbols":["BTCUSDT"],"lookback":3}
```
