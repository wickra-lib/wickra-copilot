//! Criterion benchmarks for `build_context`: how the fold scales with the
//! universe size (1 / 10 / 100 / 1000 symbols), each requesting all six fact
//! kinds. The same benchmark, run with and without the `parallel` feature,
//! measures the rayon path against the sequential one.

use std::collections::BTreeMap;
use std::hint::black_box;

use copilot_core::{
    build_context, Candle, ContextSpec, FactKind, FeedSnapshot, FundingPoint, Liquidation, OiPoint,
    OrderbookL2, Side, Trade,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const LOOKBACK: usize = 20;
const BARS: usize = 40;

/// A synthetic snapshot for one symbol with enough data for every fact to fire:
/// 40 candles on a varied (non-geometric) path that dumps over the window, an
/// ask-heavy book, a funding sign flip, an open-interest drop and a liquidation
/// cluster. `seed` perturbs the level so symbols are not identical.
fn snapshot_for(symbol: String, seed: usize) -> FeedSnapshot {
    let base = 100.0 + f64::from(u32::try_from(seed % 50).unwrap());
    let mut candles = Vec::with_capacity(BARS);
    for index in 0..BARS {
        let step = f64::from(u32::try_from(index).unwrap());
        let ts = i64::try_from(index).unwrap() + 1;
        let close = if index < LOOKBACK {
            base + 0.3 * step.sin()
        } else {
            let progress = f64::from(u32::try_from(index - LOOKBACK).unwrap()) / 19.0;
            base - 6.0 * progress + 0.5 * (step * 1.1).sin()
        };
        candles.push(Candle {
            ts,
            open: close,
            high: close + 0.5,
            low: close - 0.5,
            close,
            volume: 10.0 + step,
        });
    }
    let last_ts = i64::try_from(BARS).unwrap();
    FeedSnapshot {
        symbol,
        candles,
        orderbook: Some(OrderbookL2 {
            ts: last_ts,
            bids: vec![[base - 6.0, 3.0], [base - 6.1, 2.0]],
            asks: vec![[base - 5.9, 7.0], [base - 5.8, 6.0]],
        }),
        trades: vec![Trade {
            ts: last_ts,
            price: base - 6.0,
            qty: 1.5,
            buyer_maker: true,
        }],
        funding: vec![
            FundingPoint {
                ts: 6,
                rate: 0.0003,
            },
            FundingPoint {
                ts: 18,
                rate: -0.0002,
            },
        ],
        open_interest: vec![
            OiPoint { ts: 1, oi: 1.0e8 },
            OiPoint {
                ts: last_ts,
                oi: 0.96e8,
            },
        ],
        liquidations: vec![
            Liquidation {
                ts: last_ts - 2,
                side: Side::Long,
                price: base - 5.0,
                qty: 20_000.0,
            },
            Liquidation {
                ts: last_ts - 1,
                side: Side::Long,
                price: base - 6.0,
                qty: 22_000.0,
            },
        ],
    }
}

/// A feed universe of `symbols` distinct symbols.
fn universe(symbols: usize) -> BTreeMap<String, FeedSnapshot> {
    (0..symbols)
        .map(|i| {
            let name = format!("SYM{i:05}");
            (name.clone(), snapshot_for(name, i))
        })
        .collect()
}

/// A spec over the given symbols requesting all six fact kinds.
fn spec(symbols: usize) -> ContextSpec {
    ContextSpec {
        symbols: (0..symbols).map(|i| format!("SYM{i:05}")).collect(),
        lookback: u32::try_from(LOOKBACK).unwrap(),
        facts: vec![
            FactKind::PriceMove,
            FactKind::OrderbookImbalance,
            FactKind::LiquidationCluster,
            FactKind::FundingFlip,
            FactKind::OiChange,
            FactKind::VolatilitySpike,
        ],
        timeframe: Some("1m".into()),
    }
}

fn bench_build_context(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("build_context");
    group.sample_size(10);
    for &symbols in &[1usize, 10, 100, 1_000] {
        let feeds = universe(symbols);
        let spec = spec(symbols);
        group.throughput(Throughput::Elements(u64::try_from(symbols).unwrap()));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{symbols}sym")),
            &(&feeds, &spec),
            |bencher, (feeds, spec)| {
                bencher.iter(|| black_box(build_context(black_box(feeds), black_box(spec))));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_build_context);
criterion_main!(benches);
