//! Indicator resolution — grounds indicator-backed facts on the `wickra-core`
//! registry via the `wickra-backtest-core` factory (the only name -> indicator
//! resolver in the ecosystem).
//!
//! Only the volatility fact needs an indicator (realised volatility, a `StdDev`
//! of close). `value_over` builds a fresh indicator, folds a candle slice
//! through it serially, and returns the final value — deterministic, no shared
//! state between calls.

use wickra_backtest_core::registry::{build, BarInput};
use wickra_backtest_core::Candle as BtCandle;

use crate::feed::Candle;

/// The resolver seam. Stateless: each `value_over` builds and folds a fresh
/// indicator so repeated derivations never share warmup state.
pub(crate) struct IndicatorSet;

impl IndicatorSet {
    /// A new resolver.
    pub(crate) fn new() -> Self {
        Self
    }

    /// Fold `candles` through the named registry indicator and return its final
    /// value, or `None` if there are fewer than `window` candles, the registry
    /// does not know the indicator, or the indicator never produced a finite
    /// value. A fresh indicator is built each call, so the fold is deterministic
    /// and independent of prior calls.
    #[allow(clippy::unused_self)]
    pub(crate) fn value_over(
        &self,
        candles: &[Candle],
        name: &str,
        params: &[f64],
        window: usize,
    ) -> Option<f64> {
        if candles.len() < window {
            return None;
        }
        let mut indicator = build(name, params).ok()?;
        let mut last = None;
        for candle in candles {
            let bar = BtCandle {
                time: candle.ts,
                open: candle.open,
                high: candle.high,
                low: candle.low,
                close: candle.close,
                volume: candle.volume,
            };
            let input = BarInput {
                candle: &bar,
                reference: None,
                deriv: None,
                orderbook: None,
                trades: &[],
                cross_section: None,
            };
            if let Some(value) = indicator.update(&input) {
                if value.is_finite() {
                    last = Some(value);
                }
            }
        }
        last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(close: f64) -> Candle {
        Candle {
            ts: 0,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
        }
    }

    #[test]
    fn resolves_and_folds_a_stddev() {
        let set = IndicatorSet::new();
        let candles: Vec<Candle> = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0].map(candle).to_vec();
        let value = set.value_over(&candles, "StdDev", &[3.0], 3);
        assert!(value.is_some());
        assert!(value.unwrap() >= 0.0);
    }

    #[test]
    fn too_few_candles_yields_none() {
        let set = IndicatorSet::new();
        let candles = vec![candle(1.0)];
        assert!(set.value_over(&candles, "StdDev", &[14.0], 14).is_none());
    }

    #[test]
    fn unknown_indicator_yields_none() {
        let set = IndicatorSet::new();
        let candles: Vec<Candle> = [1.0, 2.0, 3.0].map(candle).to_vec();
        assert!(set.value_over(&candles, "NotAnIndicator", &[], 1).is_none());
    }
}
