//! Conformance tests: the JSON contract (§6) is stable and self-consistent —
//! every enum representation is `snake_case`, every public type round-trips
//! through serde, an unknown fact kind is rejected at parse time, domain errors
//! surface as in-band JSON, and the deterministic `human` prose is byte-stable.

use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use copilot_core::{
    Candle, ContextSpec, Copilot, Fact, FactKind, FeedSnapshot, FundingPoint, Liquidation,
    MarketContext, OiPoint, OrderbookL2, Side, ToolCall, Trade,
};

const SPEC: &str = r#"{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}"#;

/// Serialize, deserialize, and assert the value survives unchanged.
fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let json = serde_json::to_string(value).unwrap();
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(value, &back);
}

#[test]
fn fact_kind_tags_are_snake_case() {
    let cases = [
        (FactKind::PriceMove, r#""price_move""#),
        (FactKind::OrderbookImbalance, r#""orderbook_imbalance""#),
        (FactKind::LiquidationCluster, r#""liquidation_cluster""#),
        (FactKind::FundingFlip, r#""funding_flip""#),
        (FactKind::OiChange, r#""oi_change""#),
        (FactKind::VolatilitySpike, r#""volatility_spike""#),
    ];
    for (kind, tag) in cases {
        assert_eq!(serde_json::to_string(&kind).unwrap(), tag);
        round_trip(&kind);
    }
}

#[test]
fn side_tags_are_snake_case() {
    assert_eq!(serde_json::to_string(&Side::Long).unwrap(), r#""long""#);
    assert_eq!(serde_json::to_string(&Side::Short).unwrap(), r#""short""#);
    round_trip(&Side::Long);
    round_trip(&Side::Short);
}

#[test]
fn feed_types_round_trip() {
    round_trip(&Candle {
        ts: 1,
        open: 100.0,
        high: 101.0,
        low: 99.0,
        close: 100.5,
        volume: 12.0,
    });
    round_trip(&OrderbookL2 {
        ts: 2,
        bids: vec![[94.0, 3.0], [93.9, 2.0]],
        asks: vec![[94.1, 7.0], [94.2, 6.0]],
    });
    round_trip(&Trade {
        ts: 3,
        price: 100.0,
        qty: 1.5,
        buyer_maker: true,
    });
    round_trip(&FundingPoint {
        ts: 4,
        rate: -0.0002,
    });
    round_trip(&OiPoint {
        ts: 5,
        oi: 96_000_000.0,
    });
    round_trip(&Liquidation {
        ts: 6,
        side: Side::Long,
        price: 95.0,
        qty: 20_000.0,
    });
}

#[test]
fn feed_snapshot_round_trips_with_optional_fields() {
    // A bare snapshot (only candles) and a fully populated one both survive.
    round_trip(&FeedSnapshot {
        symbol: "BTCUSDT".into(),
        candles: vec![Candle {
            ts: 1,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1.0,
        }],
        orderbook: None,
        trades: vec![],
        funding: vec![],
        open_interest: vec![],
        liquidations: vec![],
    });
    round_trip(&FeedSnapshot {
        symbol: "ETHUSDT".into(),
        candles: vec![],
        orderbook: Some(OrderbookL2 {
            ts: 2,
            bids: vec![[52.4, 8.0]],
            asks: vec![[52.5, 3.0]],
        }),
        trades: vec![Trade {
            ts: 2,
            price: 50.0,
            qty: 2.0,
            buyer_maker: false,
        }],
        funding: vec![FundingPoint {
            ts: 2,
            rate: 0.0003,
        }],
        open_interest: vec![OiPoint { ts: 2, oi: 4.0e7 }],
        liquidations: vec![Liquidation {
            ts: 2,
            side: Side::Short,
            price: 52.0,
            qty: 15_000.0,
        }],
    });
}

#[test]
fn fact_and_context_round_trip() {
    let fact = Fact::new(
        FactKind::PriceMove,
        "BTCUSDT",
        -6.0,
        6.0,
        3,
        "BTCUSDT dropped -6.00% over the last 3 bars.",
    );
    round_trip(&fact);
    round_trip(&MarketContext {
        facts: vec![fact],
        symbols: vec!["BTCUSDT".into()],
        lookback: 3,
    });
}

#[test]
fn spec_round_trips_with_and_without_timeframe() {
    let with = ContextSpec {
        symbols: vec!["BTCUSDT".into()],
        lookback: 20,
        facts: vec![FactKind::PriceMove, FactKind::OiChange],
        timeframe: Some("1m".into()),
    };
    round_trip(&with);
    let without = ContextSpec {
        timeframe: None,
        ..with
    };
    round_trip(&without);
}

#[test]
fn tool_call_round_trips_with_and_without_kind() {
    round_trip(&ToolCall {
        tool: "get_fact".into(),
        symbol: "BTCUSDT".into(),
        kind: Some(FactKind::PriceMove),
    });
    round_trip(&ToolCall {
        tool: "get_fact".into(),
        symbol: "BTCUSDT".into(),
        kind: None,
    });
}

#[test]
fn unknown_fact_kind_is_rejected_at_parse() {
    // An unrecognised fact-kind tag must fail deserialization, not parse to a
    // silent default.
    assert!(
        ContextSpec::from_json(r#"{"symbols":["BTCUSDT"],"lookback":3,"facts":["teleport"]}"#)
            .is_err()
    );
}

#[test]
fn invalid_specs_are_rejected_on_construction() {
    // No symbols, zero lookback, no facts, and a duplicated fact kind each fail.
    assert!(Copilot::new(r#"{"symbols":[],"lookback":3,"facts":["price_move"]}"#).is_err());
    assert!(
        Copilot::new(r#"{"symbols":["BTCUSDT"],"lookback":0,"facts":["price_move"]}"#).is_err()
    );
    assert!(Copilot::new(r#"{"symbols":["BTCUSDT"],"lookback":3,"facts":[]}"#).is_err());
    assert!(Copilot::new(
        r#"{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move","price_move"]}"#
    )
    .is_err());
}

#[test]
fn bad_spec_command_yields_error_json() {
    let mut copilot = Copilot::new(SPEC).unwrap();
    let reply = copilot
        .command_json(
            r#"{"cmd":"set_spec","spec":{"symbols":[],"lookback":3,"facts":["price_move"]}}"#,
        )
        .unwrap();
    assert!(reply.contains(r#""ok":false"#), "{reply}");
}

#[test]
fn unknown_command_yields_error_json() {
    let mut copilot = Copilot::new(SPEC).unwrap();
    let reply = copilot.command_json(r#"{"cmd":"nope"}"#).unwrap();
    assert!(reply.contains(r#""ok":false"#), "{reply}");
    assert!(reply.contains("unknown cmd"), "{reply}");
}

#[test]
fn human_prose_is_byte_stable() {
    // A three-bar 100 -> 94 dump is a clean -6.00% move; the deterministic
    // template must render exactly this string, and the built context must
    // serialize to exactly these bytes.
    let mut feeds = BTreeMap::new();
    feeds.insert(
        "BTCUSDT".to_string(),
        FeedSnapshot {
            symbol: "BTCUSDT".into(),
            candles: vec![
                Candle {
                    ts: 1,
                    open: 100.0,
                    high: 100.0,
                    low: 100.0,
                    close: 100.0,
                    volume: 1.0,
                },
                Candle {
                    ts: 2,
                    open: 97.0,
                    high: 97.0,
                    low: 97.0,
                    close: 97.0,
                    volume: 1.0,
                },
                Candle {
                    ts: 3,
                    open: 94.0,
                    high: 94.0,
                    low: 94.0,
                    close: 94.0,
                    volume: 1.0,
                },
            ],
            orderbook: None,
            trades: vec![],
            funding: vec![],
            open_interest: vec![],
            liquidations: vec![],
        },
    );
    let mut copilot = Copilot::new(SPEC).unwrap();
    let ctx = copilot.build_context(&feeds).unwrap();
    assert_eq!(
        ctx.facts[0].human,
        "BTCUSDT dropped -6.00% over the last 3 bars."
    );
    assert_eq!(
        serde_json::to_string(&ctx).unwrap(),
        r#"{"facts":[{"kind":"price_move","symbol":"BTCUSDT","value":-6.0,"magnitude":6.0,"ts":3,"human":"BTCUSDT dropped -6.00% over the last 3 bars."}],"symbols":["BTCUSDT"],"lookback":3}"#
    );
}
