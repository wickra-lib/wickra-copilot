//! Input feed types — the deterministic, serde JSON boundary the context
//! builder consumes.
//!
//! A `FeedSnapshot` is already-serialized microstructure data (as
//! `wickra-exchange` delivers it, or a replay/CSV capture). The core consumes
//! snapshots and **never opens a socket** — that is the CLI's job under the
//! optional `live` feature — which is what keeps the core WASM-capable and
//! golden-testable.
//!
//! The field names mirror the `wickra-exchange` order-book / trade / funding /
//! open-interest / liquidation stream shapes. They are defined here (rather than
//! re-exported) because these types are the canonical JSON boundary of the
//! copilot and must carry serde derives: `wickra_core::Candle` is not
//! `Serialize`/`Deserialize`, so a local serde `Candle` is defined (the same
//! choice `xray-core` made).

use serde::{Deserialize, Serialize};

/// A single OHLCV bar. `ts` is an `i64` timestamp (seconds or milliseconds,
/// consistent with the feed); Wickra never inspects it numerically.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Candle {
    /// Bar timestamp.
    pub ts: i64,
    /// Bar open price.
    pub open: f64,
    /// Bar high price.
    pub high: f64,
    /// Bar low price.
    pub low: f64,
    /// Bar close price.
    pub close: f64,
    /// Bar volume.
    pub volume: f64,
}

/// A serialized snapshot of one symbol's microstructure feeds up to some
/// timestamp — the deterministic input to the context builder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FeedSnapshot {
    /// The perp symbol these feeds belong to.
    pub symbol: String,
    /// The candle history up to the snapshot timestamp.
    pub candles: Vec<Candle>,
    /// The top-of-book depth at the latest timestamp, if present.
    #[serde(default)]
    pub orderbook: Option<OrderbookL2>,
    /// Aggregated trades within the lookback window.
    #[serde(default)]
    pub trades: Vec<Trade>,
    /// The funding-rate series.
    #[serde(default)]
    pub funding: Vec<FundingPoint>,
    /// The open-interest series.
    #[serde(default)]
    pub open_interest: Vec<OiPoint>,
    /// Liquidation events within the window.
    #[serde(default)]
    pub liquidations: Vec<Liquidation>,
}

/// A level-2 order-book snapshot: `[price, size]` pairs, bids descending and
/// asks ascending.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OrderbookL2 {
    /// Snapshot timestamp.
    pub ts: i64,
    /// Resting bids, `[price, size]`, price descending.
    pub bids: Vec<[f64; 2]>,
    /// Resting asks, `[price, size]`, price ascending.
    pub asks: Vec<[f64; 2]>,
}

/// A single aggregated trade.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Trade {
    /// Trade timestamp.
    pub ts: i64,
    /// Trade price.
    pub price: f64,
    /// Trade quantity.
    pub qty: f64,
    /// Whether the buyer was the maker (i.e. a sell aggressor hit the bid).
    pub buyer_maker: bool,
}

/// A single funding-rate observation.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct FundingPoint {
    /// Observation timestamp.
    pub ts: i64,
    /// The funding rate (a small signed fraction).
    pub rate: f64,
}

/// A single open-interest observation.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct OiPoint {
    /// Observation timestamp.
    pub ts: i64,
    /// Open interest, in contracts or base units.
    pub oi: f64,
}

/// A single liquidation event.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Liquidation {
    /// Liquidation timestamp.
    pub ts: i64,
    /// Which side was liquidated.
    pub side: Side,
    /// The liquidation price.
    pub price: f64,
    /// The liquidated quantity.
    pub qty: f64,
}

/// The side of a liquidated position.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// A long position was liquidated (forced sell).
    Long,
    /// A short position was liquidated (forced buy).
    Short,
}
