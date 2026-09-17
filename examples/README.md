# Wickra Copilot examples

A runnable "build a market context" example in every language. Each one builds a
copilot from the same spec (a single `price_move` fact over a 3-bar lookback),
feeds an inline three-bar dump (`100 → 97 → 94` on `BTCUSDT`) and prints the
version and the resulting `MarketContext`. The examples are self-contained: the
spec and feeds are inline, so there is no shared `data/` directory to load (the
cross-language golden fixtures live in [`../golden/`](../golden)).

## Rust — `examples/rust/`

As the CI examples job runs it, from the repository root:

```bash
cargo run -q --manifest-path examples/rust/Cargo.toml
```

| Example | What it does |
| --- | --- |
| `src/main.rs` | A runnable Rust example: build a market context with the native `build_context` API and print it. |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-copilot-c --release`), then build and run
the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `context.c` | A minimal C example: build a market context through the wickra-copilot C ABI. |
| `context.cpp` | A minimal C++ example: build a market context, then ask the same question against the stored context and against the context passed inline -- both through the C++ hull. |

## C# — `examples/csharp/`

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Context
```

| Example | What it does |
| --- | --- |
| `Context/Program.cs` | A runnable .NET example: build a market context through the binding. |

## Go — `examples/go/`

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

| Example | What it does |
| --- | --- |
| `context.go` | A runnable Go example: build a market context through the binding. |

## R — `examples/r/`

As the CI examples job runs it, from the repository root:

```bash
R CMD INSTALL bindings/r
Rscript examples/r/context.R
```

| Example | What it does |
| --- | --- |
| `context.R` | A runnable R example: build a market context through the binding. |

## Java — `examples/java/`

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q package -DskipTests
javac -cp bindings/java/target/classes examples/java/Context.java -d examples/java/out
java --enable-native-access=ALL-UNNAMED  -Dnative.lib.dir="$PWD/target/release"  -cp "bindings/java/target/classes:examples/java/out" Context
```

| Example | What it does |
| --- | --- |
| `Context.java` | A runnable Java example: build a market context through the binding. |

## Python — `examples/python/`

As the CI examples job runs it, from the repository root:

```bash
python -m pip install --require-hashes -r .github/requirements/ci-dev-py3.txt
( cd bindings/python && maturin build --release --out dist )
python -m pip install --no-index --find-links bindings/python/dist wickra-copilot
python examples/python/context.py
```

| Example | What it does |
| --- | --- |
| `context.py` | A runnable Python example: build a market context through the binding. |

## Node.js — `examples/node/`

As the CI examples job runs it, from the repository root:

```bash
( cd bindings/node && npm install --no-audit --no-fund && npx napi build --platform --release )
( cd examples/node && npm install --no-audit --no-fund )
node examples/node/context.js
```

| Example | What it does |
| --- | --- |
| `context.js` | A runnable Node.js example: build a market context through the binding. |

## WASM — `examples/wasm/`

Build the WASM package, serve the repository root, and open the page in a browser;
the module script inside it is what runs (CI parses it with `node --check`):

```bash
wasm-pack build bindings/wasm --target web
python -m http.server 8000     # then open http://localhost:8000/examples/wasm/
```

| Example | What it does |
| --- | --- |
| `context.html` | A runnable example against this binding. |

## Example datasets

The examples are self-contained: the spec and the input are inline, so there is
no shared `data/` directory to load. The cross-language golden fixtures, which
every binding is checked against byte for byte, live in [`../golden/`](../golden).

## Asking a model

The [`ask/`](ask/) example goes one step further: it grounds the same context and
then asks a real LLM to answer *only* from those facts, through the separate
`wickra-copilot-llm` adapter. It is the one example that talks to the network, so it is
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
