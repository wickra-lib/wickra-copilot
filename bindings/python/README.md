# Wickra Copilot — Python

Python bindings for [`copilot-core`](https://github.com/wickra-lib/wickra-copilot),
built with [PyO3] and [maturin]. The surface mirrors every other Wickra binding:
build a `Copilot` from a spec JSON, drive it with command JSONs, and read back a
ranked `MarketContext` of hard facts. Only the deterministic core is exposed —
the LLM adapter is never part of this surface, so the network and API key stay
out of the binding.

## Install

```sh
pip install wickra-copilot
```

## Usage

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

## Surface

- **`Copilot(spec_json)`** builds a copilot from a spec JSON (`""` or `"{}"` for
  an empty handle whose spec is set later). Raises `ValueError` on a malformed
  spec.
- **`copilot.command(cmd_json)`** applies a command JSON (`set_spec`,
  `build_context`, `facts`, `query`, `reset`, `version`) and returns the response
  JSON. A bad spec or unknown command comes back in-band as
  `{"ok": false, "error": ...}`.
- **`Copilot.version()`** returns the library version.

## Building from source

```sh
maturin develop --release   # inside a virtualenv
pytest
```

[PyO3]: https://pyo3.rs
[maturin]: https://www.maturin.rs
