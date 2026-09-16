<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/pypi.svg)](https://pypi.org/project/wickra-copilot/)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — Python

---

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — for Python. `pip install wickra-copilot` — prebuilt wheels for Linux, macOS and Windows, nothing to compile.**

Python bindings for [`wickra-copilot-core`](https://github.com/wickra-lib/wickra-copilot),
built with [PyO3] and [maturin]. The surface mirrors every other Wickra binding:
build a `Copilot` from a spec JSON, drive it with command JSONs, and read back a
ranked `MarketContext` of hard facts. Only the deterministic core is exposed —
the LLM adapter is never part of this surface, so the network and API key stay
out of the binding.

## Install

```bash
pip install wickra-copilot
```

Pre-built wheels ship for Linux, macOS and Windows — there is nothing to
compile and no C library to track down.

### Building from this repository (contributors)

```sh
maturin develop --release   # inside a virtualenv
pytest
```

## Quick start

```python
import json
from wickra_copilot import Copilot

spec = json.dumps({
    "symbols": ["BTCUSDT"],
    "lookback": 3,
    "facts": ["price_move"],
})

feeds = {"BTCUSDT": {"symbol": "BTCUSDT", "candles": [
    {"ts": 1, "open": 100.0, "high": 100.0, "low": 100.0, "close": 100.0, "volume": 1.0},
    {"ts": 2, "open": 97.0, "high": 97.0, "low": 97.0, "close": 97.0, "volume": 1.0},
    {"ts": 3, "open": 94.0, "high": 94.0, "low": 94.0, "close": 94.0, "volume": 1.0},
]}}

copilot = Copilot(spec)
context = json.loads(copilot.command(json.dumps({"cmd": "build_context", "feeds": feeds})))
print(context["facts"][0]["human"])  # BTCUSDT dropped -6.00% over the last 3 bars.
```

### Surface

- **`Copilot(spec_json)`** builds a copilot from a spec JSON (`""` or `"{}"` for
  an empty handle whose spec is set later). Raises `ValueError` on a malformed
  spec.
- **`copilot.command(cmd_json)`** applies a command JSON (`set_spec`,
  `build_context`, `facts`, `query`, `reset`, `version`) and returns the response
  JSON. A bad spec or unknown command comes back in-band as
  `{"ok": false, "error": ...}`.
- **`Copilot.version()`** returns the library version.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of PyO3, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-copilot/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-copilot>
- **Docs** (guides, spec reference, cookbook): <https://copilot.wickra.org>
- **Runnable example:** [`examples/python/`](https://github.com/wickra-lib/wickra-copilot/tree/main/examples/python)

Wickra Copilot ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-copilot/blob/main/SECURITY.md>.

## Disclaimer

Wickra Copilot is analysis software: it builds a deterministic market context and
relays it to a language model of your choosing. It is provided "as is", without
warranty of any kind. LLM output can be wrong and is **not financial advice**; the
copilot only reports facts and places no orders. Trading carries risk of loss;
review the code and use at your own discretion.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-MIT) at your option.

[PyO3]: https://pyo3.rs
[maturin]: https://www.maturin.rs
