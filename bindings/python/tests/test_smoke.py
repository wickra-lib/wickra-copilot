"""Smoke test: construct a copilot, build a context, route a question."""

import json

from wickra_copilot import Copilot, __version__

SPEC = json.dumps(
    {
        "symbols": ["BTCUSDT"],
        "lookback": 3,
        "facts": ["price_move"],
    }
)

# BTC drops 6% over three bars -> one significant price-move fact.
FEEDS = {
    "BTCUSDT": {
        "symbol": "BTCUSDT",
        "candles": [
            {"ts": 1, "open": 100.0, "high": 100.0, "low": 100.0, "close": 100.0, "volume": 1.0},
            {"ts": 2, "open": 97.0, "high": 97.0, "low": 97.0, "close": 97.0, "volume": 1.0},
            {"ts": 3, "open": 94.0, "high": 94.0, "low": 94.0, "close": 94.0, "volume": 1.0},
        ],
    }
}


def test_build_context_roundtrip() -> None:
    copilot = Copilot(SPEC)
    context = json.loads(copilot.command(json.dumps({"cmd": "build_context", "feeds": FEEDS})))
    assert context["symbols"] == ["BTCUSDT"]
    fact = context["facts"][0]
    assert fact["kind"] == "price_move"
    assert abs(fact["value"] - (-6.0)) < 1e-9


def test_query_after_build() -> None:
    copilot = Copilot(SPEC)
    copilot.command(json.dumps({"cmd": "build_context", "feeds": FEEDS}))
    result = json.loads(copilot.command(json.dumps({"cmd": "query", "question": "why did BTC dump"})))
    kinds = {call["kind"] for call in result["tool_calls"]}
    assert "price_move" in kinds


def test_version_matches_module() -> None:
    assert Copilot.version() == __version__


def test_bad_spec_raises() -> None:
    try:
        Copilot("not json")
    except ValueError:
        return
    raise AssertionError("expected ValueError for a malformed spec")
