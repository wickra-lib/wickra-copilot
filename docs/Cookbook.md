# Cookbook

Short, runnable recipes. Each drives the same core through the JSON command
protocol; the CLI examples assume you have built the workspace (`cargo build`).

## Build a context from the CLI

```bash
cargo run -p wickra-copilot -- context \
  --spec golden/specs/dump.json --feeds golden/feeds --format json
```

The `--format json` output is exactly the bytes every binding returns from a
`build_context` command. Drop `--format json` for a human-readable list of facts.

## Feed the whole universe from stdin

Instead of a `--feeds` directory of `<SYMBOL>.json` files, pass one JSON object
`{"SYM": {…FeedSnapshot…}, …}` on stdin:

```bash
cargo run -p wickra-copilot -- context --spec golden/specs/dump.json --stdin < feeds.json
```

## Ask an LLM to explain the context

`ask` builds the same deterministic context, then routes the question and sends
the facts to a provider. Ollama is local and needs no key:

```bash
cargo run -p wickra-copilot -- ask \
  --spec golden/specs/dump.json --feeds golden/feeds \
  --question "Why did BTC just dump?" --provider ollama
```

For a hosted provider, export your key first (see [LLM_ADAPTER.md](LLM_ADAPTER.md)):

```bash
export WICKRA_COPILOT_API_KEY="sk-…"
cargo run -p wickra-copilot -- ask --spec golden/specs/dump.json --feeds golden/feeds \
  --question "Why did BTC just dump?" --provider openai
```

## Build a context (Python)

```python
import json
from wickra_copilot import Copilot

spec = json.dumps({"symbols": ["BTCUSDT"], "lookback": 3, "facts": ["price_move"]})
feeds = {"BTCUSDT": {"symbol": "BTCUSDT", "candles": [
    {"ts": 1, "open": 100, "high": 100, "low": 100, "close": 100, "volume": 1},
    {"ts": 2, "open": 97,  "high": 97,  "low": 97,  "close": 97,  "volume": 1},
    {"ts": 3, "open": 94,  "high": 94,  "low": 94,  "close": 94,  "volume": 1}]}}

copilot = Copilot(spec)
ctx = json.loads(copilot.command(json.dumps({"cmd": "build_context", "feeds": feeds})))
print(ctx["facts"][0]["human"])  # BTCUSDT dropped -6.00% over the last 3 bars.
```

`build_context` and its alias `facts` return the identical bytes.

## Route a question to fact kinds

`query` is deterministic and offline — it decides which facts a question is
about (see [TOOL_CALLING.md](TOOL_CALLING.md)):

```python
calls = json.loads(copilot.command(json.dumps({"cmd": "query", "question": "why did BTC dump?"})))
print(calls["tool_calls"])  # [{"tool":"get_fact","symbol":"BTCUSDT","kind":"price_move"}, …]
```

## Reset for a new spec or window

```python
copilot.command(json.dumps({"cmd": "set_spec", "spec": json.loads(spec)}))  # {"ok":true}
copilot.command('{"cmd":"reset"}')                                          # {"ok":true}
```

## Check the version

```bash
cargo run -p wickra-copilot -- --version
```

or, from any binding, `{"cmd":"version"}` → `{"version":"0.1.0"}`.

## See also

[Architecture](ARCHITECTURE.md) · [Facts](FACTS.md) · [Grounding](GROUNDING.md) ·
[LLM adapter](LLM_ADAPTER.md) · [Tool calling](TOOL_CALLING.md).
