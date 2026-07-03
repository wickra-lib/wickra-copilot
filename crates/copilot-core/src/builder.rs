//! The context builder — folds a feed universe into a sorted `MarketContext`.
//!
//! Each symbol derives its requested facts independently (in parallel under the
//! `parallel` feature, sequentially otherwise); the per-symbol results are
//! collected into a `BTreeMap` (never a `HashMap`) so insertion is key-ordered,
//! then flattened and sorted by `(magnitude desc, kind asc, symbol asc, ts asc)`.
//! Both feature builds produce a byte-identical `MarketContext`. It is reached
//! through `Copilot::command_json`.

use std::collections::BTreeMap;

use crate::derive::{
    derive_funding_flip, derive_liquidation_cluster, derive_oi_change, derive_orderbook_imbalance,
    derive_price_move, derive_volatility_spike,
};
use crate::error::Result;
use crate::fact::{Fact, FactKind, MarketContext};
use crate::feed::FeedSnapshot;
use crate::indicator_set::IndicatorSet;
use crate::spec::ContextSpec;

/// Build a `MarketContext` from a feed universe and a spec.
pub fn build_context(
    feeds: &BTreeMap<String, FeedSnapshot>,
    spec: &ContextSpec,
) -> Result<MarketContext> {
    spec.validate()?;
    let inds = IndicatorSet::new();

    let per_symbol = fold(feeds, spec, &inds);

    // Flatten serially in key order, then impose the total order so the result
    // is independent of the fold order (parallel == sequential).
    let mut facts: Vec<Fact> = per_symbol.values().flatten().cloned().collect();
    facts.sort_by(|a, b| {
        b.magnitude
            .total_cmp(&a.magnitude)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.symbol.cmp(&b.symbol))
            .then_with(|| a.ts.cmp(&b.ts))
    });

    let symbols: Vec<String> = per_symbol.keys().cloned().collect();
    Ok(MarketContext {
        facts,
        symbols,
        lookback: spec.lookback,
    })
}

/// Derive the requested facts for one symbol, in spec order.
fn derive_symbol(snap: &FeedSnapshot, spec: &ContextSpec, inds: &IndicatorSet) -> Vec<Fact> {
    let mut facts = Vec::new();
    for kind in &spec.facts {
        let fact = match kind {
            FactKind::PriceMove => derive_price_move(snap, spec.lookback),
            FactKind::OrderbookImbalance => derive_orderbook_imbalance(snap, spec.lookback),
            FactKind::LiquidationCluster => derive_liquidation_cluster(snap, spec.lookback),
            FactKind::FundingFlip => derive_funding_flip(snap, spec.lookback),
            FactKind::OiChange => derive_oi_change(snap, spec.lookback),
            FactKind::VolatilitySpike => derive_volatility_spike(snap, spec.lookback, inds),
        };
        if let Some(fact) = fact {
            facts.push(fact);
        }
    }
    facts
}

#[cfg(feature = "parallel")]
fn fold(
    feeds: &BTreeMap<String, FeedSnapshot>,
    spec: &ContextSpec,
    inds: &IndicatorSet,
) -> BTreeMap<String, Vec<Fact>> {
    use rayon::prelude::*;
    spec.symbols
        .par_iter()
        .filter_map(|symbol| {
            feeds
                .get(symbol)
                .map(|snap| (symbol.clone(), derive_symbol(snap, spec, inds)))
        })
        .collect()
}

#[cfg(not(feature = "parallel"))]
fn fold(
    feeds: &BTreeMap<String, FeedSnapshot>,
    spec: &ContextSpec,
    inds: &IndicatorSet,
) -> BTreeMap<String, Vec<Fact>> {
    spec.symbols
        .iter()
        .filter_map(|symbol| {
            feeds
                .get(symbol)
                .map(|snap| (symbol.clone(), derive_symbol(snap, spec, inds)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Candle, OiPoint};

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

    fn snap(symbol: &str, closes: &[f64], oi: &[f64]) -> FeedSnapshot {
        FeedSnapshot {
            symbol: symbol.to_string(),
            candles: closes
                .iter()
                .enumerate()
                .map(|(i, &c)| candle(i64::try_from(i).unwrap(), c))
                .collect(),
            orderbook: None,
            trades: Vec::new(),
            funding: Vec::new(),
            open_interest: oi
                .iter()
                .enumerate()
                .map(|(i, &v)| OiPoint {
                    ts: i64::try_from(i).unwrap(),
                    oi: v,
                })
                .collect(),
            liquidations: Vec::new(),
        }
    }

    fn spec() -> ContextSpec {
        ContextSpec {
            symbols: vec!["BTCUSDT".into(), "ETHUSDT".into()],
            lookback: 3,
            facts: vec![FactKind::PriceMove, FactKind::OiChange],
            timeframe: None,
        }
    }

    fn feeds() -> BTreeMap<String, FeedSnapshot> {
        let mut m = BTreeMap::new();
        // BTC: -6% move, -4% OI.
        m.insert(
            "BTCUSDT".into(),
            snap("BTCUSDT", &[100.0, 97.0, 94.0], &[100.0, 96.0]),
        );
        // ETH: -2% move, no OI change.
        m.insert(
            "ETHUSDT".into(),
            snap("ETHUSDT", &[100.0, 99.0, 98.0], &[100.0, 100.0]),
        );
        m
    }

    #[test]
    fn facts_are_sorted_by_magnitude_desc() {
        let ctx = build_context(&feeds(), &spec()).unwrap();
        assert!(!ctx.facts.is_empty());
        for pair in ctx.facts.windows(2) {
            assert!(pair[0].magnitude >= pair[1].magnitude);
        }
        // The BTC price move (~6%) is the most significant fact.
        assert_eq!(ctx.facts[0].kind, FactKind::PriceMove);
        assert_eq!(ctx.facts[0].symbol, "BTCUSDT");
    }

    #[test]
    fn symbols_are_the_processed_ones_sorted() {
        let ctx = build_context(&feeds(), &spec()).unwrap();
        assert_eq!(
            ctx.symbols,
            vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
        );
        assert_eq!(ctx.lookback, 3);
    }

    #[test]
    fn missing_feed_is_skipped_not_an_error() {
        let mut sp = spec();
        sp.symbols.push("MISSING".into());
        let ctx = build_context(&feeds(), &sp).unwrap();
        assert!(!ctx.symbols.contains(&"MISSING".to_string()));
    }

    #[test]
    fn invalid_spec_errors() {
        let mut sp = spec();
        sp.symbols.clear();
        assert!(build_context(&feeds(), &sp).is_err());
    }

    #[test]
    fn equal_magnitudes_break_by_kind_then_symbol() {
        // Craft three facts that all round to magnitude 5.0:
        //   BTC price move (+5%), BTC OI change (+5%), ETH price move (+5%).
        // ETH OI is flat, so it produces no OI fact.
        let mut m = BTreeMap::new();
        m.insert(
            "BTCUSDT".into(),
            snap("BTCUSDT", &[100.0, 105.0], &[100.0, 105.0]),
        );
        m.insert(
            "ETHUSDT".into(),
            snap("ETHUSDT", &[100.0, 105.0], &[100.0, 100.0]),
        );
        let sp = ContextSpec {
            symbols: vec!["BTCUSDT".into(), "ETHUSDT".into()],
            lookback: 3,
            facts: vec![FactKind::PriceMove, FactKind::OiChange],
            timeframe: None,
        };
        let ctx = build_context(&m, &sp).unwrap();
        // All three tie at magnitude 5.0, so ordering falls to the tie-breakers.
        assert_eq!(ctx.facts.len(), 3);
        for f in &ctx.facts {
            assert!((f.magnitude - 5.0).abs() < 1e-9);
        }
        // kind asc (PriceMove < OiChange), then symbol asc (BTCUSDT < ETHUSDT).
        let order: Vec<(FactKind, &str)> = ctx
            .facts
            .iter()
            .map(|f| (f.kind, f.symbol.as_str()))
            .collect();
        assert_eq!(
            order,
            vec![
                (FactKind::PriceMove, "BTCUSDT"),
                (FactKind::PriceMove, "ETHUSDT"),
                (FactKind::OiChange, "BTCUSDT"),
            ]
        );
    }
}
