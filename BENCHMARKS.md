# Benchmarks

The copilot's deterministic cost is dominated by folding feed snapshots (order
book, trades, funding, open interest, liquidations) into facts and assembling a
`MarketContext` across a symbol universe. The benchmarks here measure that **core
context-build work**, so throughput scales predictably with the universe size and
the amount of feed data. The LLM call is not benchmarked — its latency belongs to
the provider and is not part of the deterministic core.

## What is measured

The `copilot-bench` crate (criterion) covers a context build across a matrix of:

- **Universe size** — the number of symbols folded before the context is built.
- **Feed length** — the number of feed events per symbol.
- **Mode** — the parallel (rayon) fold vs the sequential (WASM fallback) fold,
  which must produce a byte-identical `MarketContext`.

## Methodology

Run against fixed, in-process synthetic feed snapshots so the numbers are
reproducible and contain no I/O or network variance:

```bash
cargo bench -p copilot-bench
```

## Results

_To be filled in from the criterion run in the test-rigor / docs phase._ Figures
will be the median estimate on a single machine; treat them as orders of
magnitude, not guarantees — they vary with CPU and toolchain.

## Caveats

These figures bound the context-build overhead only. End-to-end time in a real
`ask` run is dominated by the LLM round-trip, which these in-process benchmarks
deliberately exclude — the copilot's job is to build the grounding fast; the model
provider owns the rest.
