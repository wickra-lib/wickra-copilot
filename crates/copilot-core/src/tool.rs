//! The deterministic tool / function-calling interface.
//!
//! `query` is rule- and keyword-based — never an LLM. It maps a question's
//! keywords to fact kinds, keeps only the `(kind, symbol)` pairs that actually
//! exist in the `MarketContext`, and returns a stably sorted `Vec<ToolCall>` an
//! LLM can "execute" by reading the matching facts back out of the context. The
//! keyword table is fixed here and pinned by the golden corpus. No network, no
//! non-determinism. It is reached through `Copilot::command_json`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::fact::{FactKind, MarketContext};

/// A routed tool call: read one `kind` of fact for one `symbol`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ToolCall {
    /// The tool to invoke (always `"get_fact"` today).
    pub tool: String,
    /// The symbol to read a fact for.
    pub symbol: String,
    /// The fact kind to read.
    #[serde(default)]
    pub kind: Option<FactKind>,
}

/// Every fact kind, used when a question matches no keyword.
const ALL_KINDS: &[FactKind] = &[
    FactKind::PriceMove,
    FactKind::OrderbookImbalance,
    FactKind::LiquidationCluster,
    FactKind::FundingFlip,
    FactKind::OiChange,
    FactKind::VolatilitySpike,
];

/// The fixed keyword -> fact-kind routing table (golden-pinned). Matched
/// case-insensitively as substrings of the question.
const KEYWORD_TABLE: &[(&str, &[FactKind])] = &[
    ("dump", &[FactKind::PriceMove, FactKind::LiquidationCluster]),
    (
        "crash",
        &[FactKind::PriceMove, FactKind::LiquidationCluster],
    ),
    ("sell", &[FactKind::PriceMove, FactKind::LiquidationCluster]),
    ("drop", &[FactKind::PriceMove]),
    ("fall", &[FactKind::PriceMove]),
    ("pump", &[FactKind::PriceMove, FactKind::OiChange]),
    ("rally", &[FactKind::PriceMove, FactKind::OiChange]),
    ("moon", &[FactKind::PriceMove, FactKind::OiChange]),
    ("rise", &[FactKind::PriceMove]),
    ("surge", &[FactKind::PriceMove, FactKind::VolatilitySpike]),
    ("liquidat", &[FactKind::LiquidationCluster]),
    ("cascade", &[FactKind::LiquidationCluster]),
    ("funding", &[FactKind::FundingFlip]),
    ("open interest", &[FactKind::OiChange]),
    (
        "leverage",
        &[FactKind::OiChange, FactKind::LiquidationCluster],
    ),
    ("order book", &[FactKind::OrderbookImbalance]),
    ("orderbook", &[FactKind::OrderbookImbalance]),
    ("imbalance", &[FactKind::OrderbookImbalance]),
    ("book", &[FactKind::OrderbookImbalance]),
    ("volatil", &[FactKind::VolatilitySpike]),
    ("move", &[FactKind::PriceMove]),
];

/// Route a question against the context. Returns the tool calls for every
/// requested fact kind that exists in the context, sorted by `(kind, symbol)`.
pub fn query(question: &str, ctx: &MarketContext) -> Vec<ToolCall> {
    let lowered = question.to_lowercase();
    let mut kinds: BTreeSet<FactKind> = BTreeSet::new();
    for (keyword, mapped) in KEYWORD_TABLE {
        if lowered.contains(keyword) {
            kinds.extend(mapped.iter().copied());
        }
    }
    if kinds.is_empty() {
        kinds.extend(ALL_KINDS.iter().copied());
    }

    // A BTreeSet keyed by (kind, symbol) both dedups and yields the calls in
    // (kind asc, symbol asc) order.
    let mut pairs: BTreeSet<(FactKind, String)> = BTreeSet::new();
    for fact in &ctx.facts {
        if kinds.contains(&fact.kind) {
            pairs.insert((fact.kind, fact.symbol.clone()));
        }
    }
    pairs
        .into_iter()
        .map(|(kind, symbol)| ToolCall {
            tool: "get_fact".to_string(),
            symbol,
            kind: Some(kind),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::Fact;

    fn ctx(facts: Vec<Fact>) -> MarketContext {
        let mut symbols: Vec<String> = facts.iter().map(|f| f.symbol.clone()).collect();
        symbols.sort();
        symbols.dedup();
        MarketContext {
            facts,
            symbols,
            lookback: 20,
        }
    }

    fn fact(kind: FactKind, symbol: &str) -> Fact {
        Fact::new(kind, symbol, 1.0, 1.0, 1, "x")
    }

    #[test]
    fn dump_routes_to_price_move_and_liquidation() {
        let c = ctx(vec![
            fact(FactKind::PriceMove, "BTCUSDT"),
            fact(FactKind::LiquidationCluster, "BTCUSDT"),
            fact(FactKind::FundingFlip, "BTCUSDT"),
        ]);
        let calls = query("why did BTC dump", &c);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].kind, Some(FactKind::PriceMove));
        assert_eq!(calls[1].kind, Some(FactKind::LiquidationCluster));
        assert!(calls.iter().all(|c| c.tool == "get_fact"));
    }

    #[test]
    fn query_is_case_insensitive() {
        let c = ctx(vec![fact(FactKind::PriceMove, "BTCUSDT")]);
        assert_eq!(query("DUMP", &c).len(), 1);
    }

    #[test]
    fn only_present_kinds_are_returned() {
        // "dump" maps to price_move + liquidation_cluster, but only price_move exists.
        let c = ctx(vec![fact(FactKind::PriceMove, "BTCUSDT")]);
        let calls = query("dump", &c);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].kind, Some(FactKind::PriceMove));
    }

    #[test]
    fn no_keyword_uses_all_kinds() {
        let c = ctx(vec![
            fact(FactKind::FundingFlip, "BTCUSDT"),
            fact(FactKind::OiChange, "ETHUSDT"),
        ]);
        let calls = query("give me the context", &c);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn calls_are_sorted_by_kind_then_symbol() {
        let c = ctx(vec![
            fact(FactKind::PriceMove, "ETHUSDT"),
            fact(FactKind::PriceMove, "BTCUSDT"),
        ]);
        let calls = query("move", &c);
        assert_eq!(calls[0].symbol, "BTCUSDT");
        assert_eq!(calls[1].symbol, "ETHUSDT");
    }
}
