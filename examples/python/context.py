"""A runnable Python example: build a market context through the binding.

    pip install wickra-copilot
    python examples/python/context.py
"""

import json

from wickra_copilot import Copilot

SPEC = json.dumps(
    {
        "symbols": ["BTCUSDT"],
        "lookback": 3,
        "facts": ["price_move"],
    }
)

FEEDS = {
    "BTCUSDT": {
        "symbol": "BTCUSDT",
        "candles": [
            {"ts": 1, "open": 100, "high": 100, "low": 100, "close": 100, "volume": 1},
            {"ts": 2, "open": 97, "high": 97, "low": 97, "close": 97, "volume": 1},
            {"ts": 3, "open": 94, "high": 94, "low": 94, "close": 94, "volume": 1},
        ],
    }
}


def main() -> None:
    copilot = Copilot(SPEC)
    response = copilot.command(json.dumps({"cmd": "build_context", "feeds": FEEDS}))
    context = json.loads(response)

    print(f"wickra-copilot {Copilot.version()}")
    print(response)
    print(f"  facts: {len(context['facts'])}")


if __name__ == "__main__":
    main()
