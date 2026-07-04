//! Property-based invariants: for random feed universes and random (valid)
//! specs, `build_context` never panics and every `MarketContext` obeys its
//! contract — every fact's `symbol` is one the spec asked for, every fact's
//! `kind` is one the spec requested, every `value`/`magnitude` is finite, the
//! facts are in total order (magnitude descending by `total_cmp`, then
//! `(kind, symbol, ts)` ascending), and the reported `symbols` are a subset of
//! the spec's symbols.
//!
//! The parallel-vs-sequential byte-equality is a compile-time property (the
//! `parallel` feature switches the fold with no runtime toggle), pinned by
//! running the golden suite under both feature sets in CI.

use std::collections::BTreeMap;

use copilot_core::{
    build_context, Candle, ContextSpec, FactKind, FeedSnapshot, FundingPoint, Liquidation, OiPoint,
    OrderbookL2, Side, Trade,
};
use proptest::prelude::*;

fn arb_ts() -> impl Strategy<Value = i64> {
    0i64..100_000
}

/// A finite, sane price.
fn arb_price() -> impl Strategy<Value = f64> {
    1.0f64..100_000.0
}

fn arb_qty() -> impl Strategy<Value = f64> {
    0.0f64..1_000.0
}

fn arb_candle() -> impl Strategy<Value = Candle> {
    (arb_ts(), arb_price(), arb_price(), arb_qty()).prop_map(|(ts, open, close, volume)| Candle {
        ts,
        open,
        high: open.max(close) + 0.5,
        low: open.min(close) - 0.5,
        close,
        volume,
    })
}

fn arb_level() -> impl Strategy<Value = [f64; 2]> {
    (arb_price(), arb_qty()).prop_map(|(price, qty)| [price, qty])
}

fn arb_orderbook() -> impl Strategy<Value = OrderbookL2> {
    (
        arb_ts(),
        prop::collection::vec(arb_level(), 0..4),
        prop::collection::vec(arb_level(), 0..4),
    )
        .prop_map(|(ts, bids, asks)| OrderbookL2 { ts, bids, asks })
}

fn arb_trade() -> impl Strategy<Value = Trade> {
    (arb_ts(), arb_price(), arb_qty(), any::<bool>()).prop_map(|(ts, price, qty, buyer_maker)| {
        Trade {
            ts,
            price,
            qty,
            buyer_maker,
        }
    })
}

fn arb_funding() -> impl Strategy<Value = FundingPoint> {
    (arb_ts(), -0.01f64..0.01).prop_map(|(ts, rate)| FundingPoint { ts, rate })
}

fn arb_oi() -> impl Strategy<Value = OiPoint> {
    (arb_ts(), 0.0f64..1_000_000_000.0).prop_map(|(ts, oi)| OiPoint { ts, oi })
}

fn arb_liquidation() -> impl Strategy<Value = Liquidation> {
    (
        arb_ts(),
        prop_oneof![Just(Side::Long), Just(Side::Short)],
        arb_price(),
        arb_qty(),
    )
        .prop_map(|(ts, side, price, qty)| Liquidation {
            ts,
            side,
            price,
            qty,
        })
}

fn arb_snapshot(symbol: String) -> impl Strategy<Value = FeedSnapshot> {
    (
        prop::collection::vec(arb_candle(), 0..12),
        prop::option::of(arb_orderbook()),
        prop::collection::vec(arb_trade(), 0..4),
        prop::collection::vec(arb_funding(), 0..4),
        prop::collection::vec(arb_oi(), 0..4),
        prop::collection::vec(arb_liquidation(), 0..4),
    )
        .prop_map(
            move |(candles, orderbook, trades, funding, open_interest, liquidations)| {
                FeedSnapshot {
                    symbol: symbol.clone(),
                    candles,
                    orderbook,
                    trades,
                    funding,
                    open_interest,
                    liquidations,
                }
            },
        )
}

const ALL_KINDS: [FactKind; 6] = [
    FactKind::PriceMove,
    FactKind::OrderbookImbalance,
    FactKind::LiquidationCluster,
    FactKind::FundingFlip,
    FactKind::OiChange,
    FactKind::VolatilitySpike,
];

/// A valid spec over 1..5 symbols `S0..`, a positive lookback, and a non-empty
/// distinct subset of the fact kinds; plus the matching feed universe.
fn arb_case() -> impl Strategy<Value = (BTreeMap<String, FeedSnapshot>, ContextSpec)> {
    (
        1usize..5,
        1u32..50,
        proptest::sample::subsequence(ALL_KINDS.to_vec(), 1..=ALL_KINDS.len()),
    )
        .prop_flat_map(|(n, lookback, facts)| {
            let symbols: Vec<String> = (0..n).map(|i| format!("S{i}")).collect();
            let snaps = symbols
                .iter()
                .cloned()
                .map(arb_snapshot)
                .collect::<Vec<_>>();
            (Just(symbols), Just(lookback), Just(facts), snaps)
        })
        .prop_map(|(symbols, lookback, facts, snaps)| {
            let feeds: BTreeMap<String, FeedSnapshot> =
                symbols.iter().cloned().zip(snaps).collect();
            let spec = ContextSpec {
                symbols,
                lookback,
                facts,
                timeframe: None,
            };
            (feeds, spec)
        })
}

proptest! {
    #[test]
    fn build_context_upholds_the_contract((feeds, spec) in arb_case()) {
        // A validated spec always builds; a panic here is a real defect.
        let ctx = build_context(&feeds, &spec).expect("a valid spec must build");

        // Reported symbols are a subset of the spec's symbols.
        for symbol in &ctx.symbols {
            prop_assert!(spec.symbols.contains(symbol), "stray symbol {}", symbol);
        }
        prop_assert_eq!(ctx.lookback, spec.lookback);

        let mut prev: Option<(f64, FactKind, &str, i64)> = None;
        for fact in &ctx.facts {
            prop_assert!(spec.symbols.contains(&fact.symbol), "stray fact symbol");
            prop_assert!(spec.facts.contains(&fact.kind), "stray fact kind");
            prop_assert!(fact.value.is_finite(), "non-finite value");
            prop_assert!(fact.magnitude.is_finite(), "non-finite magnitude");
            prop_assert!(fact.magnitude >= 0.0, "negative magnitude");

            // Total order: magnitude descending (total_cmp), then
            // (kind, symbol, ts) ascending — exactly as the builder sorts.
            if let Some((prev_mag, prev_kind, prev_sym, prev_ts)) = prev {
                let cmp = prev_mag.total_cmp(&fact.magnitude);
                let ordered = cmp == std::cmp::Ordering::Greater
                    || (cmp == std::cmp::Ordering::Equal
                        && (prev_kind, prev_sym, prev_ts)
                            <= (fact.kind, fact.symbol.as_str(), fact.ts));
                prop_assert!(ordered, "facts out of order");
            }
            prev = Some((fact.magnitude, fact.kind, fact.symbol.as_str(), fact.ts));
        }
    }
}
