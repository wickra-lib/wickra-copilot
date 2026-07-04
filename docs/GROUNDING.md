# Grounding — why the copilot cannot hallucinate the facts

An LLM asked *"why did BTC dump?"* with no data will produce a fluent,
confident, and possibly invented answer. Wickra Copilot removes the invention
from the part that matters: the **facts** the model reasons over are computed, not
generated. This page explains why that grounding is trustworthy.

## The context is a pure function

`build_context(spec, feeds)` is deterministic. Given the same spec and the same
feeds it returns the same `MarketContext`, byte for byte, forever. There is no
clock, no randomness, no network and no hidden state in the core — the fact list
is a mathematical function of its inputs, defined by the six reductions in
[FACTS.md](FACTS.md).

Concretely, the core guarantees:

- **Fixed formulas.** Each fact kind is a closed-form reduction with a fixed
  significance threshold. A fact is emitted only when the data actually clears
  that threshold, and the number in the fact is the number from the feed.
- **Rounded, total-ordered output.** Every magnitude is rounded to `1e-8` and the
  facts are sorted by a total order (`f64::total_cmp` on magnitude, then kind,
  symbol, ts). No float comparison is left partial, so the serialization is
  stable.
- **Parallel ≡ sequential.** The default rayon build and the sequential
  (`--no-default-features`, WASM) build produce the **same bytes**. A dedicated
  test (`parallel_eq_sequential`) runs the golden corpus under both and asserts
  byte equality; CI runs the whole suite under both feature sets.
- **Cross-language identity.** Because every binding returns the core's compact
  JSON verbatim, the `MarketContext` is identical in Rust, Python, Node.js, WASM,
  C, C++, C#, Go, Java and R. The cross-language golden tests pin this.

## Honest facts, not flattering ones

The core never manufactures a signal to look useful. Two examples:

- **The `vol_spike` golden context is empty.** Realised volatility over a window
  divided by volatility over a window that *contains* it is capped near √2, so it
  rarely reaches the `1.5x` emission threshold. Rather than lower the bar, the
  golden `vol_spike` spec produces **no facts** — an honest golden.
- **Below-threshold moves vanish.** A 0.2% price move is not a fact. The context
  contains only what is significant; the absence of a fact is itself information.

## Where the model can still be wrong

Grounding pins the **facts**, not the **interpretation**. The LLM adapter
([LLM_ADAPTER.md](LLM_ADAPTER.md)) takes the deterministic context and writes
prose about it. That prose can still be wrong: the model may misweigh a fact,
draw a bad causal link, or editorialize. What it *cannot* do is invent a
liquidation that did not happen or a funding flip that did not occur — those
numbers come from the feed, through a pinned formula, before the model sees them.

This is the whole design: make the ground truth deterministic and testable, keep
the fallible language layer separate and swappable, and never let the second
rewrite the first. The copilot's answer is **not financial advice** and may be
wrong in its reasoning — but the facts under it are real.

## See also

[Facts](FACTS.md) · [LLM adapter](LLM_ADAPTER.md) · [Architecture](ARCHITECTURE.md).
