# Support

Thanks for using `wickra-copilot`. Here is where to go for help.

## Questions and usage help

- Read the [README](README.md) and the [contributing guide](CONTRIBUTING.md).
  The architecture, fact and provider guides live under `docs/` and the runnable
  per-language examples under `examples/` (added during the build-out).
- Open a [GitHub Discussion](https://github.com/wickra-lib/wickra-copilot/discussions)
  for questions and ideas.

## Bugs and feature requests

Open a [GitHub issue](https://github.com/wickra-lib/wickra-copilot/issues) using
the bug-report or feature-request template. Please include the version, the
binding/language you used, a minimal `ContextSpec` and a small sample feed
snapshot, and the expected vs actual `MarketContext`. For the LLM adapter, name
the provider (Ollama / OpenAI / Claude / Gemini) but **never paste your API key**.

## Security

Do **not** open a public issue for security problems. Report privately to
**support@wickra.org** or via GitHub private vulnerability reporting — see
[SECURITY.md](SECURITY.md).

## Note

`wickra-copilot` is a research and engineering tool: it builds a market context
and relays it to a language model of your choosing, and places no orders. LLM
output is not financial advice and comes with no warranty — review the code and
validate results before relying on them.
