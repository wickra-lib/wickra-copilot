#!/usr/bin/env python3
"""Deterministically generate the golden feed snapshots.

Every value is a fixed function of the bar index (no randomness), so the feeds
are reproducible byte-for-byte across machines. Run from the repo root:

    python golden/generate_feeds.py

It writes one `<SYMBOL>.json` FeedSnapshot per symbol into golden/feeds/. The
formula is documented in golden/README.md. These feeds are the deterministic
input to the golden specs; the expected MarketContext outputs are blessed from
the real core, never hand-edited.
"""

import json
import math
import os

BARS = 40
T0 = 1700000000
STEP = 60  # 1-minute bars


def candles(base, drift, wobble):
    """40 one-minute candles. `close` walks from ~base by `drift` over the last
    20 bars (the lookback window) with a fixed sinusoidal `wobble`; the first 20
    bars stay near `base` so the move is concentrated in the window."""
    out = []
    for i in range(BARS):
        if i < 20:
            close = base + 0.3 * math.sin(i)
        else:
            close = base + drift * ((i - 20) / 19.0) + wobble * math.sin(i * 1.1)
        close = round(close, 4)
        out.append({
            "ts": T0 + i * STEP,
            "open": close,
            "high": round(close + 0.5, 4),
            "low": round(close - 0.5, 4),
            "close": close,
            "volume": round(10.0 + i, 4),
        })
    return out


def snapshot(symbol, base, drift, wobble, book, funding, oi, liqs):
    last_ts = T0 + (BARS - 1) * STEP
    return {
        "symbol": symbol,
        "candles": candles(base, drift, wobble),
        "orderbook": {"ts": last_ts, "bids": book[0], "asks": book[1]},
        "trades": [
            {"ts": last_ts - 120, "price": base, "qty": 1.5, "buyer_maker": True},
            {"ts": last_ts - 60, "price": base, "qty": 2.0, "buyer_maker": False},
        ],
        "funding": funding,
        "open_interest": oi,
        "liquidations": liqs,
    }


FEEDS = {
    # BTC dumps ~6% over the window; ask-heavy book; funding flips +->-; OI -4%;
    # a long-liquidation cascade.
    "BTCUSDT": snapshot(
        "BTCUSDT", 100.0, -6.0, 0.5,
        book=([[94.0, 3.0], [93.9, 2.0]], [[94.1, 7.0], [94.2, 6.0]]),
        funding=[
            {"ts": T0 + 600, "rate": 0.0003},
            {"ts": T0 + 1200, "rate": 0.0001},
            {"ts": T0 + 1800, "rate": -0.0002},
        ],
        oi=[{"ts": T0, "oi": 100000000.0}, {"ts": T0 + 2340, "oi": 96000000.0}],
        liqs=[
            {"ts": T0 + 2040, "side": "long", "price": 95.0, "qty": 20000.0},
            {"ts": T0 + 2100, "side": "long", "price": 94.0, "qty": 22000.0},
        ],
    ),
    # ETH pumps ~5% over the window; bid-heavy book; funding flips -->+; OI +6%;
    # a short-liquidation cluster.
    "ETHUSDT": snapshot(
        "ETHUSDT", 50.0, 5.0, 0.4,
        book=([[52.4, 8.0], [52.3, 7.0]], [[52.5, 3.0], [52.6, 2.0]]),
        funding=[
            {"ts": T0 + 600, "rate": -0.0002},
            {"ts": T0 + 1200, "rate": -0.0001},
            {"ts": T0 + 1800, "rate": 0.0003},
        ],
        oi=[{"ts": T0, "oi": 40000000.0}, {"ts": T0 + 2340, "oi": 42400000.0}],
        liqs=[
            {"ts": T0 + 2040, "side": "short", "price": 52.0, "qty": 15000.0},
            {"ts": T0 + 2100, "side": "short", "price": 52.4, "qty": 12000.0},
        ],
    ),
}


def main():
    out_dir = os.path.join(os.path.dirname(__file__), "feeds")
    os.makedirs(out_dir, exist_ok=True)
    for symbol, snap in FEEDS.items():
        path = os.path.join(out_dir, f"{symbol}.json")
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            json.dump(snap, fh, indent=2)
            fh.write("\n")
        print(f"wrote {path}")


if __name__ == "__main__":
    main()
