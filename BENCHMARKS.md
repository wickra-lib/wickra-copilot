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

Measured on one developer machine (release build, `parallel` feature), median
criterion estimates. Treat these as orders of magnitude, not guarantees — they
vary with CPU and toolchain.

A full `build_context` over a synthetic universe, every symbol requesting all six
fact kinds:

| Universe     | build_context (median) | Throughput    |
|--------------|-----------------------:|--------------:|
| 1 symbol     |                ~3.1 µs |  ~320 K sym/s |
| 10 symbols   |                 ~35 µs |  ~280 K sym/s |
| 100 symbols  |               ~0.27 ms |  ~375 K sym/s |
| 1,000 symbols |               ~2.5 ms |  ~400 K sym/s |

The build is **roughly linear in the number of symbols** — 10× the universe is
about 8–10× the time — because each symbol's facts are derived independently.
Per-symbol throughput is flat at a few hundred thousand symbol-contexts per
second, so a 1,000-symbol universe with all six facts assembles in about
**2.5 ms**. The parallel (rayon) and sequential builds produce a byte-identical
`MarketContext`; these figures are the parallel path.

## Caveats

These figures bound the context-build overhead only. End-to-end time in a real
`ask` run is dominated by the LLM round-trip, which these in-process benchmarks
deliberately exclude — the copilot's job is to build the grounding fast; the model
provider owns the rest.
