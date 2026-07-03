# Contributing to wickra-copilot

Thanks for your interest. Issues, bug reports, ideas and pull requests are all
welcome at <https://github.com/wickra-lib/wickra-copilot>. For larger changes,
open an issue first so we can agree on the approach.

## Orientation

- The core — the `ContextSpec` and the fold that turns serialized microstructure
  feeds (order book, trades, funding, open interest, liquidations) into a
  `MarketContext`, a list of hard facts (price moves, order-book imbalance,
  liquidation clusters, funding flips, OI changes, volatility spikes) — lives in
  `crates/copilot-core`. The context is **data, not code**: a serde data-model,
  so the same context crosses the C ABI and WASM unchanged, and stays
  byte-identical between the parallel (rayon) and sequential builds.
- The LLM adapter is a **separate, swappable crate** (`crates/copilot-llm`): it
  renders the context into a prompt and calls the user's own LLM endpoint
  (Ollama / OpenAI / Claude / Gemini over the OpenAI-compatible interface, key
  from the environment). The LLM call is non-deterministic and is **never**
  golden-tested — it is strictly separated from the deterministic core.
- The reference consumer is `crates/copilot-cli` (the `wickra-copilot` binary,
  `context` and `ask` subcommands); a local `crates/copilot-desktop` front-end
  drives the same core.
- Every language binding lives under `bindings/<lang>/` and exposes the same
  data-driven surface: a `Copilot` handle plus `command(json) -> json` and
  `version`. Bindings must preserve the **golden-parity invariant**: given the
  spec + feed snapshots in `golden/`, the same command produces the
  byte-identical `MarketContext` in `golden/expected/`.

## The dev loop

Every change runs green locally before a commit:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings   # WASM path (no rayon, no adapter networking)
cargo test --workspace --all-features
cargo test -p copilot-core --no-default-features                              # sequential == parallel (context builder)
cargo deny check
```

`cargo fmt --all` and the `clippy -D warnings` gate are enforced in CI on three
operating systems, across both the default (rayon `parallel`) and
`--no-default-features` (sequential / WASM) feature sets — the context builder
must produce a byte-identical `MarketContext` either way. The LLM adapter's
network path is exercised only by offline tests; it is not part of the
determinism gate.

## Conventions

- **Commits are signed** and follow Conventional Commits (`feat:`, `fix:`,
  `chore:`, `docs:`…). One logical change per commit. Open a PR against `main`;
  do not push to `main` directly.
- **All public artifacts are in English** — code, comments, commit messages, PR
  titles and bodies, issues and docs.
- **No secrets, ever** — not in code, tests, fixtures, logs, issues or PRs. The
  LLM API key is read from the environment, never committed, never logged, and
  never transmitted anywhere except the user-configured LLM endpoint. Market
  feeds read only public data.
- **Production code only** — no mocks outside `#[cfg(test)]`, no TODO stubs, and
  no defensive branches that can never run (they fail coverage).

## Adding a fact

Facts are a serde enum, so extending the context means adding a variant, not a
closure. A new fact kind is added to `crates/copilot-core/src/fact.rs` and
derived in `src/derive.rs`, with a serde round-trip test and a golden fixture.
Indicators that ground a fact (volatility, OI deltas) come from the
[Wickra](https://github.com/wickra-lib/wickra) core registry by name and
parameters — no indicator code lives here. See the guides under `docs/`.

## Developer Certificate of Origin

Contributions are accepted under the [DCO](DCO); sign off your commits with
`git commit -s`. By contributing you agree your work is dual-licensed under
`MIT OR Apache-2.0`.
