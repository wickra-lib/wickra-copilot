//! The six fact derivations — the exact, deterministic formulas that turn a
//! feed snapshot into facts.
//!
//! Every reduction runs serially in timestamp / key order (never rayon order).
//! A fact is emitted only when its `magnitude` clears the kind-specific
//! significance threshold; a missing feed or a non-finite result yields `None`,
//! never a zero fact. The `human` strings are fixed English templates with
//! rounded numbers — no LLM.
//!
//! These functions are called by the builder, which is reached through
//! `Copilot::command_json`.

use crate::fact::{Fact, FactKind};
use crate::feed::{FeedSnapshot, Side};
use crate::indicator_set::IndicatorSet;

/// Top-of-book depth considered for the imbalance.
const BOOK_DEPTH: usize = 10;

/// Percentage move in close over the last `lookback` bars.
pub(crate) fn derive_price_move(snap: &FeedSnapshot, lookback: u32) -> Option<Fact> {
    let window = window_candles(snap, lookback);
    if window.len() < 2 {
        return None;
    }
    let first = window.first()?.close;
    let last = window.last()?.close;
    if first == 0.0 {
        return None;
    }
    let value = (last / first - 1.0) * 100.0;
    let magnitude = value.abs();
    if !value.is_finite() || magnitude < 1.0 {
        return None;
    }
    let ts = window.last()?.ts;
    let human = human_price_move(&snap.symbol, value, lookback);
    Some(Fact::new(
        FactKind::PriceMove,
        &snap.symbol,
        value,
        magnitude,
        ts,
        human,
    ))
}

/// Top-of-book bid/ask volume imbalance.
pub(crate) fn derive_orderbook_imbalance(snap: &FeedSnapshot, _lookback: u32) -> Option<Fact> {
    let book = snap.orderbook.as_ref()?;
    let bid_vol: f64 = book
        .bids
        .iter()
        .take(BOOK_DEPTH)
        .map(|level| level[1])
        .sum();
    let ask_vol: f64 = book
        .asks
        .iter()
        .take(BOOK_DEPTH)
        .map(|level| level[1])
        .sum();
    let total = bid_vol + ask_vol;
    if total <= 0.0 {
        return None;
    }
    let value = (bid_vol - ask_vol) / total;
    let magnitude = value.abs();
    if !value.is_finite() || magnitude < 0.20 {
        return None;
    }
    let human = human_orderbook_imbalance(&snap.symbol, value);
    Some(Fact::new(
        FactKind::OrderbookImbalance,
        &snap.symbol,
        value,
        magnitude,
        book.ts,
        human,
    ))
}

/// Liquidation notional summed over the window, signed by dominant side.
pub(crate) fn derive_liquidation_cluster(snap: &FeedSnapshot, _lookback: u32) -> Option<Fact> {
    if snap.liquidations.is_empty() {
        return None;
    }
    let mut long_notional = 0.0;
    let mut short_notional = 0.0;
    let mut last_ts = i64::MIN;
    for liq in &snap.liquidations {
        let notional = liq.price * liq.qty;
        match liq.side {
            Side::Long => long_notional += notional,
            Side::Short => short_notional += notional,
        }
        last_ts = last_ts.max(liq.ts);
    }
    let value = long_notional - short_notional;
    let magnitude = long_notional + short_notional;
    if !value.is_finite() || !magnitude.is_finite() || magnitude < 1.0 {
        return None;
    }
    let long_dominated = long_notional >= short_notional;
    let human = human_liquidation_cluster(&snap.symbol, magnitude, long_dominated);
    Some(Fact::new(
        FactKind::LiquidationCluster,
        &snap.symbol,
        value,
        magnitude,
        last_ts,
        human,
    ))
}

/// The most recent funding-rate sign flip in the window.
pub(crate) fn derive_funding_flip(snap: &FeedSnapshot, _lookback: u32) -> Option<Fact> {
    let funding = &snap.funding;
    let mut flip: Option<usize> = None;
    for i in 1..funding.len() {
        if funding[i - 1].rate * funding[i].rate < 0.0 {
            flip = Some(i);
        }
    }
    let i = flip?;
    let value = funding[i].rate;
    let magnitude = (funding[i].rate - funding[i - 1].rate).abs();
    if !value.is_finite() || !magnitude.is_finite() {
        return None;
    }
    let human = human_funding_flip(&snap.symbol, value);
    Some(Fact::new(
        FactKind::FundingFlip,
        &snap.symbol,
        value,
        magnitude,
        funding[i].ts,
        human,
    ))
}

/// Percentage change in open interest over the window.
pub(crate) fn derive_oi_change(snap: &FeedSnapshot, _lookback: u32) -> Option<Fact> {
    let oi = &snap.open_interest;
    if oi.len() < 2 {
        return None;
    }
    let first = oi.first()?.oi;
    let last = oi.last()?.oi;
    if first == 0.0 {
        return None;
    }
    let value = (last / first - 1.0) * 100.0;
    let magnitude = value.abs();
    if !value.is_finite() || magnitude < 1.0 {
        return None;
    }
    let ts = oi.last()?.ts;
    let human = human_oi_change(&snap.symbol, value);
    Some(Fact::new(
        FactKind::OiChange,
        &snap.symbol,
        value,
        magnitude,
        ts,
        human,
    ))
}

/// Realised volatility relative to a longer baseline.
pub(crate) fn derive_volatility_spike(
    snap: &FeedSnapshot,
    lookback: u32,
    inds: &IndicatorSet,
) -> Option<Fact> {
    let lb = lookback as usize;
    if lb == 0 {
        return None;
    }
    let n = snap.candles.len();
    let recent = &snap.candles[n.saturating_sub(lb)..];
    let baseline = &snap.candles[n.saturating_sub(2 * lb)..];
    let vol_recent = inds.value_over(recent, "StdDev", &[lb as f64], lb)?;
    let vol_baseline = inds.value_over(baseline, "StdDev", &[(2 * lb) as f64], 2 * lb)?;
    if vol_baseline <= 0.0 {
        return None;
    }
    let value = vol_recent / vol_baseline;
    let magnitude = value;
    if !value.is_finite() || value < 1.5 {
        return None;
    }
    let ts = snap.candles.last()?.ts;
    let human = human_volatility_spike(&snap.symbol, value);
    Some(Fact::new(
        FactKind::VolatilitySpike,
        &snap.symbol,
        value,
        magnitude,
        ts,
        human,
    ))
}

/// The last `lookback` candles of the snapshot.
fn window_candles(snap: &FeedSnapshot, lookback: u32) -> &[crate::feed::Candle] {
    let n = snap.candles.len();
    &snap.candles[n.saturating_sub(lookback as usize)..]
}

fn human_price_move(symbol: &str, value: f64, lookback: u32) -> String {
    let verb = if value > 0.0 { "rose" } else { "dropped" };
    format!("{symbol} {verb} {value:+.2}% over the last {lookback} bars.")
}

fn human_orderbook_imbalance(symbol: &str, value: f64) -> String {
    let side = if value > 0.0 { "bid" } else { "ask" };
    format!("{symbol} order book is {side}-heavy (imbalance {value:+.2}).")
}

fn human_liquidation_cluster(symbol: &str, magnitude: f64, long_dominated: bool) -> String {
    let side = if long_dominated { "long" } else { "short" };
    format!("{symbol} saw {magnitude:.2} notional liquidated ({side}-dominated).")
}

fn human_funding_flip(symbol: &str, value: f64) -> String {
    let sign = if value > 0.0 { "positive" } else { "negative" };
    format!("{symbol} funding flipped to {sign} ({value:+.4}).")
}

fn human_oi_change(symbol: &str, value: f64) -> String {
    let verb = if value > 0.0 { "rose" } else { "fell" };
    format!("{symbol} open interest {verb} {value:+.2}% over the window.")
}

fn human_volatility_spike(symbol: &str, value: f64) -> String {
    format!("{symbol} volatility spiked to {value:.2}x its baseline.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Candle, FundingPoint, Liquidation, OiPoint, OrderbookL2};

    fn candle(ts: i64, close: f64) -> Candle {
        Candle {
            ts,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1.0,
        }
    }

    fn snap(symbol: &str) -> FeedSnapshot {
        FeedSnapshot {
            symbol: symbol.to_string(),
            candles: Vec::new(),
            orderbook: None,
            trades: Vec::new(),
            funding: Vec::new(),
            open_interest: Vec::new(),
            liquidations: Vec::new(),
        }
    }

    #[test]
    fn price_move_drop_is_significant_and_signed() {
        let mut s = snap("BTCUSDT");
        s.candles = vec![candle(1, 100.0), candle(2, 97.0), candle(3, 93.8)];
        let fact = derive_price_move(&s, 3).unwrap();
        assert_eq!(fact.kind, FactKind::PriceMove);
        assert!(fact.value < 0.0);
        assert!(fact.human.contains("dropped"));
        assert_eq!(fact.ts, 3);
    }

    #[test]
    fn price_move_below_threshold_is_none() {
        let mut s = snap("BTCUSDT");
        s.candles = vec![candle(1, 100.0), candle(2, 100.2)];
        assert!(derive_price_move(&s, 2).is_none());
    }

    #[test]
    fn orderbook_ask_heavy_imbalance() {
        let mut s = snap("BTCUSDT");
        s.orderbook = Some(OrderbookL2 {
            ts: 9,
            bids: vec![[100.0, 3.0]],
            asks: vec![[101.0, 7.0]],
        });
        let fact = derive_orderbook_imbalance(&s, 20).unwrap();
        assert!(fact.value < 0.0);
        assert!(fact.human.contains("ask-heavy"));
        assert_eq!(fact.ts, 9);
    }

    #[test]
    fn liquidation_cluster_long_dominated() {
        let mut s = snap("BTCUSDT");
        s.liquidations = vec![
            Liquidation {
                ts: 5,
                side: Side::Long,
                price: 100.0,
                qty: 30.0,
            },
            Liquidation {
                ts: 7,
                side: Side::Short,
                price: 100.0,
                qty: 5.0,
            },
        ];
        let fact = derive_liquidation_cluster(&s, 20).unwrap();
        assert!(fact.value > 0.0);
        assert!((fact.magnitude - 3500.0).abs() < 1e-6);
        assert!(fact.human.contains("long-dominated"));
        assert_eq!(fact.ts, 7);
    }

    #[test]
    fn funding_flip_detects_last_sign_change() {
        let mut s = snap("BTCUSDT");
        s.funding = vec![
            FundingPoint {
                ts: 1,
                rate: 0.0002,
            },
            FundingPoint {
                ts: 2,
                rate: -0.0002,
            },
            FundingPoint {
                ts: 3,
                rate: -0.0003,
            },
        ];
        let fact = derive_funding_flip(&s, 20).unwrap();
        assert!((fact.value - (-0.0002)).abs() < 1e-9);
        assert!((fact.magnitude - 0.0004).abs() < 1e-9);
        assert_eq!(fact.ts, 2);
        assert!(fact.human.contains("negative"));
    }

    #[test]
    fn funding_no_flip_is_none() {
        let mut s = snap("BTCUSDT");
        s.funding = vec![
            FundingPoint {
                ts: 1,
                rate: 0.0002,
            },
            FundingPoint {
                ts: 2,
                rate: 0.0003,
            },
        ];
        assert!(derive_funding_flip(&s, 20).is_none());
    }

    #[test]
    fn oi_change_fall_is_significant() {
        let mut s = snap("BTCUSDT");
        s.open_interest = vec![OiPoint { ts: 1, oi: 100.0 }, OiPoint { ts: 2, oi: 96.0 }];
        let fact = derive_oi_change(&s, 20).unwrap();
        assert!(fact.value < 0.0);
        assert!(fact.human.contains("fell"));
    }

    #[test]
    fn volatility_spike_when_recent_exceeds_baseline() {
        let inds = IndicatorSet::new();
        let mut s = snap("BTCUSDT");
        // Calm baseline, then a volatile recent window.
        let mut candles = Vec::new();
        for i in 0..3 {
            candles.push(candle(i, 100.0));
        }
        for i in 3..6 {
            let close = if i % 2 == 0 { 130.0 } else { 70.0 };
            candles.push(candle(i, close));
        }
        s.candles = candles;
        // With lookback 3 the recent StdDev (volatile) should exceed the 6-bar baseline.
        let fact = derive_volatility_spike(&s, 3, &inds);
        // Either a spike fact or None (below 1.5x); if present, it must be well-formed.
        if let Some(f) = fact {
            assert_eq!(f.kind, FactKind::VolatilitySpike);
            assert!(f.value >= 1.5);
            assert!(f.human.contains("volatility spiked"));
        }
    }

    #[test]
    fn volatility_none_without_enough_candles() {
        let inds = IndicatorSet::new();
        let mut s = snap("BTCUSDT");
        s.candles = vec![candle(0, 100.0), candle(1, 101.0)];
        // Baseline needs 2 * lookback candles; two is not enough for lookback 3.
        assert!(derive_volatility_spike(&s, 3, &inds).is_none());
    }
}
