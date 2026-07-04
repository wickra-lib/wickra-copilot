# LLM adapter

The LLM adapter is the **only** networked part of the whole workspace. It lives in
its own crate, [`crates/copilot-llm`](../crates/copilot-llm), and is reachable
**only** from the CLI's `ask` subcommand — never over the C ABI, so the language
bindings cannot touch the network or a key. The deterministic core
([GROUNDING.md](GROUNDING.md)) has no dependency on it.

## How it works

1. The core builds a deterministic `MarketContext` (see [FACTS.md](FACTS.md)).
2. `render_prompt(context)` turns that context into a fixed prompt — the facts
   and their `human` sentences, plus the question.
3. A `Provider` preset selects one OpenAI-compatible chat endpoint, and a single
   `OpenAiCompatible` HTTP client drives it.
4. The model's answer is returned alongside the context and the tool calls that
   were routed for the question.

One HTTP implementation drives every provider; the preset only changes the base
URL, the default model and whether an API key is required.

## Providers

Choose with `--provider` on the CLI (default `ollama`). Every preset except
`custom` carries a default base URL and model; `custom` takes both from the
environment.

| Preset | Default base URL | Default model | API key |
|--------|------------------|---------------|---------|
| `ollama` | `http://localhost:11434/v1` | `llama3` | none (local) |
| `openai` | `https://api.openai.com/v1` | (OpenAI default) | required |
| `claude` | `https://api.anthropic.com/v1` | `claude-3-5-sonnet-latest` | required |
| `gemini` | `https://generativelanguage.googleapis.com/v1beta/openai` | `gemini-1.5-flash` | required |
| `custom` | `WICKRA_COPILOT_BASE_URL` | `WICKRA_COPILOT_MODEL` | as needed |

All four hosted presets are reached through their **OpenAI-compatible** endpoints,
so one client covers them. `ollama` runs fully offline and needs no key.

## Environment

The adapter reads exactly three environment variables; nothing is hard-coded and
nothing is written to disk:

| Variable | Purpose |
|----------|---------|
| `WICKRA_COPILOT_API_KEY` | the bearer key. Empty is allowed (e.g. Ollama). Never logged, never in `Debug` output. |
| `WICKRA_COPILOT_BASE_URL` | override the preset's base URL (required for `custom`). |
| `WICKRA_COPILOT_MODEL` | override the preset's default model. |

The key is your own, read from the environment at call time, and used only to
authenticate the single request to the endpoint you chose. There is **no SaaS,
no telemetry, no proxy** — the request goes straight from your machine to your
provider.

## Usage

```bash
# Local, offline, no key:
cargo run -p wickra-copilot -- ask \
  --spec golden/specs/dump.json --feeds golden/feeds \
  --question "Why did BTC just dump?" --provider ollama

# A hosted provider with your own key:
export WICKRA_COPILOT_API_KEY="sk-…"
cargo run -p wickra-copilot -- ask \
  --spec golden/specs/dump.json --feeds golden/feeds \
  --question "Why did BTC just dump?" --provider openai
```

Prefer to keep everything deterministic and offline? Use the `context`
subcommand instead of `ask` — it prints the facts and never calls a model.

## Why it is separate

Keeping the adapter out of the core and off the C ABI means:

- the golden-tested surface has no network and no key to leak;
- a security review of the bindings never has to reason about outbound requests;
- the provider can be swapped (or the whole adapter removed) without touching a
  single fact.

The adapter is read-only: it reads market context and asks a question. It never
places an order. Its output is **not financial advice** and can be wrong — only
the facts under it are pinned.

## See also

[Grounding](GROUNDING.md) · [Tool calling](TOOL_CALLING.md) · [Architecture](ARCHITECTURE.md).
