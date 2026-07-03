//! Facts and the market context — the deterministic output of the builder.
//!
//! A `Fact` is a hard, attributable observation derived from the feeds; a
//! `MarketContext` is the sorted list of facts the copilot hands to an LLM as
//! grounding. Every numeric value is rounded to a fixed precision so the JSON is
//! byte-identical across languages and builds. Nothing here calls an LLM.

use serde::{Deserialize, Serialize};

/// The fixed output precision for `value` and `magnitude`.
pub(crate) const QUANTUM: f64 = 1e-8;

/// Round `x` to the nearest multiple of `quantum` (the fixed output precision).
/// Keeps `value` / `magnitude` byte-stable across languages.
#[must_use]
pub(crate) fn round_to(x: f64, quantum: f64) -> f64 {
    (x / quantum).round() * quantum
}

/// Which fact the builder may derive. `Ord` is load-bearing: it breaks sort ties
/// deterministically by `(kind, symbol)`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum FactKind {
    /// Percentage move in price over the lookback (from candles).
    PriceMove,
    /// Top-of-book bid/ask volume imbalance, `(bid - ask) / (bid + ask)`.
    OrderbookImbalance,
    /// Liquidation notional summed over the window, with the dominant side.
    LiquidationCluster,
    /// The funding rate flipping sign within the window.
    FundingFlip,
    /// Percentage change in open interest over the lookback.
    OiChange,
    /// Realised volatility relative to its baseline (registry indicator).
    VolatilitySpike,
}

/// A single derived fact. `value` is the signed core number (e.g. `-6.2` for a
/// `-6.2%` move); `magnitude` is a non-negative strength used for ranking; `ts`
/// is the reference timestamp; `human` is deterministic, template-generated
/// English prose (never an LLM).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Fact {
    /// Which fact this is.
    pub kind: FactKind,
    /// The symbol this fact is about.
    pub symbol: String,
    /// The signed core value.
    pub value: f64,
    /// The non-negative ranking strength.
    pub magnitude: f64,
    /// The reference timestamp (window end).
    pub ts: i64,
    /// Deterministic, template-generated English prose.
    pub human: String,
}

impl Fact {
    /// Build a fact, rounding `value` and `magnitude` to the fixed output
    /// precision so the serialized JSON is byte-stable.
    #[must_use]
    pub fn new(
        kind: FactKind,
        symbol: impl Into<String>,
        value: f64,
        magnitude: f64,
        ts: i64,
        human: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            symbol: symbol.into(),
            value: round_to(value, QUANTUM),
            magnitude: round_to(magnitude, QUANTUM),
            ts,
            human: human.into(),
        }
    }
}

/// The grounding context handed to an LLM: the sorted facts, the symbols
/// actually processed, and the lookback the spec requested.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MarketContext {
    /// The derived facts, sorted (magnitude desc, then kind, symbol, ts).
    pub facts: Vec<Fact>,
    /// The symbols actually processed, sorted ascending.
    pub symbols: Vec<String>,
    /// The lookback window the spec requested.
    pub lookback: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_to_snaps_trailing_drift() {
        assert!((round_to(0.1 + 0.2, QUANTUM) - 0.3).abs() < 1e-12);
    }

    #[test]
    fn fact_new_rounds_value_and_magnitude() {
        let f = Fact::new(FactKind::PriceMove, "BTCUSDT", -6.200_000_004, 6.2, 42, "x");
        assert!((f.value - (-6.2)).abs() < 1e-9);
        assert!((f.magnitude - 6.2).abs() < 1e-9);
    }

    #[test]
    fn fact_kind_json_is_snake_case() {
        let j = serde_json::to_string(&FactKind::OrderbookImbalance).unwrap();
        assert_eq!(j, "\"orderbook_imbalance\"");
        let k: FactKind = serde_json::from_str("\"liquidation_cluster\"").unwrap();
        assert_eq!(k, FactKind::LiquidationCluster);
    }

    #[test]
    fn fact_kind_orders_by_declaration() {
        assert!(FactKind::PriceMove < FactKind::VolatilitySpike);
    }

    #[test]
    fn fact_round_trips_through_json() {
        let f = Fact::new(FactKind::OiChange, "ETHUSDT", -4.0, 4.0, 7, "y");
        let s = serde_json::to_string(&f).unwrap();
        let back: Fact = serde_json::from_str(&s).unwrap();
        assert_eq!(f, back);
    }
}
