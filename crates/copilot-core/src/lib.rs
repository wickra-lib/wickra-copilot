//! Data-driven core of the Wickra Copilot.
//!
//! A serde `ContextSpec` is folded over serialized microstructure feed snapshots
//! — order book, trades, funding, open interest and liquidations — into a
//! `MarketContext`: a deterministic list of hard facts (price move, order-book
//! imbalance, liquidation cluster, funding flip, OI change, volatility spike).
//! Symbols derive their facts in parallel (rayon) or sequentially (the WASM
//! fallback), producing a byte-identical `MarketContext`.
//!
//! The LLM call that turns that context into an answer lives in the separate,
//! non-deterministic `copilot-llm` adapter — never here.
//!
//! The public surface is assembled module by module through P-COP-1; the final
//! re-export block lands in `lib.rs` (P-COP-1.12).

mod error;
mod fact;
mod feed;
mod spec;

pub use error::{Error, Result};
pub use fact::{Fact, FactKind, MarketContext};
pub use feed::{
    Candle, FeedSnapshot, FundingPoint, Liquidation, OiPoint, OrderbookL2, Side, Trade,
};
pub use spec::ContextSpec;
