//! The `Copilot` handle and the `command_json` protocol — the single
//! JSON-in / JSON-out entry point every binding drives.
//!
//! Only the deterministic context builder and the tool router go through this
//! surface; the LLM `ask` lives in the separate `copilot-llm` crate so the core
//! and bindings stay network- and key-free. Domain errors are returned in-band
//! as `{"ok":false,"error":...}`, never as a panic.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::builder;
use crate::error::{Error, Result};
use crate::fact::MarketContext;
use crate::feed::FeedSnapshot;
use crate::spec::ContextSpec;
use crate::tool::{self, ToolCall};

/// A copilot handle: the active spec plus the last-built context.
pub struct Copilot {
    spec: ContextSpec,
    last: Option<MarketContext>,
}

impl Copilot {
    /// Build a copilot from a spec JSON. `""` or `"{}"` create an empty handle
    /// whose spec must be set (via `set_spec`) before building a context.
    pub fn new(spec_json: &str) -> Result<Self> {
        let trimmed = spec_json.trim();
        let spec = if trimmed.is_empty() || trimmed == "{}" {
            ContextSpec {
                symbols: Vec::new(),
                lookback: 0,
                facts: Vec::new(),
                timeframe: None,
            }
        } else {
            ContextSpec::from_json(spec_json)?
        };
        Ok(Self { spec, last: None })
    }

    /// The library version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Replace the active spec.
    pub fn set_spec(&mut self, spec: ContextSpec) {
        self.spec = spec;
    }

    /// Build a `MarketContext` from the feed universe and the active spec,
    /// caching it as the last context.
    pub fn build_context(
        &mut self,
        feeds: &BTreeMap<String, FeedSnapshot>,
    ) -> Result<MarketContext> {
        let context = builder::build_context(feeds, &self.spec)?;
        self.last = Some(context.clone());
        Ok(context)
    }

    /// Route a question against the last-built context (empty if none built).
    #[must_use]
    pub fn query(&self, question: &str) -> Vec<ToolCall> {
        self.last
            .as_ref()
            .map(|ctx| tool::query(question, ctx))
            .unwrap_or_default()
    }

    /// Clear the last-built context, keeping the spec.
    pub fn reset(&mut self) {
        self.last = None;
    }

    /// Apply a command JSON and return the response JSON. Domain errors are
    /// returned in-band as `{"ok":false,"error":...}`.
    ///
    /// # Errors
    /// Reserved for a serialization failure of the error envelope, which cannot
    /// occur for the fixed shapes used here.
    pub fn command_json(&mut self, cmd_json: &str) -> Result<String> {
        Ok(self
            .dispatch(cmd_json)
            .unwrap_or_else(|e| error_json(&e.to_string())))
    }

    fn dispatch(&mut self, cmd_json: &str) -> Result<String> {
        let value: Value = serde_json::from_str(cmd_json)?;
        let cmd = value
            .get("cmd")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::BadSpec("missing \"cmd\"".into()))?;
        match cmd {
            "set_spec" => {
                let spec: ContextSpec = serde_json::from_value(field(&value, "spec")?)?;
                spec.validate()?;
                self.set_spec(spec);
                Ok(ok_json())
            }
            "build_context" | "facts" => {
                let feeds: BTreeMap<String, FeedSnapshot> =
                    serde_json::from_value(field(&value, "feeds")?)?;
                Ok(serde_json::to_string(&self.build_context(&feeds)?)?)
            }
            "query" => {
                let question = str_field(&value, "question")?.to_string();
                let calls = if let Some(inline) = value.get("context") {
                    let ctx: MarketContext = serde_json::from_value(inline.clone())?;
                    tool::query(&question, &ctx)
                } else {
                    match &self.last {
                        Some(ctx) => tool::query(&question, ctx),
                        None => {
                            return Err(Error::BadSpec(
                                "no context built; call build_context first".into(),
                            ))
                        }
                    }
                };
                Ok(json!({ "tool_calls": calls }).to_string())
            }
            "reset" => {
                self.reset();
                Ok(ok_json())
            }
            "version" => Ok(json!({ "version": Self::version() }).to_string()),
            other => Err(Error::BadSpec(format!("unknown cmd: {other}"))),
        }
    }
}

/// Clone a named field out of the envelope, erroring if absent.
fn field(value: &Value, name: &str) -> Result<Value> {
    value
        .get(name)
        .cloned()
        .ok_or_else(|| Error::BadSpec(format!("missing \"{name}\"")))
}

/// Read a named string field, erroring if absent or not a string.
fn str_field<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::BadSpec(format!("missing string \"{name}\"")))
}

fn ok_json() -> String {
    json!({ "ok": true }).to_string()
}

fn error_json(message: &str) -> String {
    json!({ "ok": false, "error": message }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::FactKind;
    use crate::feed::{Candle, OiPoint};

    fn spec_json() -> String {
        r#"{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move","oi_change"]}"#.to_string()
    }

    fn feeds_json() -> String {
        // BTC: -6% move, -4% OI.
        r#"{"feeds":{"BTCUSDT":{"symbol":"BTCUSDT",
            "candles":[{"ts":1,"open":100.0,"high":100.0,"low":100.0,"close":100.0,"volume":1.0},
                       {"ts":2,"open":97.0,"high":97.0,"low":97.0,"close":97.0,"volume":1.0},
                       {"ts":3,"open":94.0,"high":94.0,"low":94.0,"close":94.0,"volume":1.0}],
            "open_interest":[{"ts":1,"oi":100.0},{"ts":3,"oi":96.0}]}}}"#
            .to_string()
    }

    #[test]
    fn version_command() {
        let mut c = Copilot::new(&spec_json()).unwrap();
        let out = c.command_json(r#"{"cmd":"version"}"#).unwrap();
        assert_eq!(out, format!(r#"{{"version":"{}"}}"#, Copilot::version()));
    }

    #[test]
    fn build_context_then_query() {
        let mut c = Copilot::new(&spec_json()).unwrap();
        let ctx = c.command_json(&feeds_json_cmd("build_context")).unwrap();
        assert!(ctx.contains("\"facts\""));
        assert!(ctx.contains("price_move"));
        let q = c
            .command_json(r#"{"cmd":"query","question":"why did BTC dump"}"#)
            .unwrap();
        assert!(q.contains("tool_calls"));
        assert!(q.contains("price_move"));
    }

    fn feeds_json_cmd(cmd: &str) -> String {
        feeds_json().replacen('{', &format!("{{\"cmd\":\"{cmd}\","), 1)
    }

    #[test]
    fn facts_is_an_alias_for_build_context() {
        let mut c = Copilot::new(&spec_json()).unwrap();
        let a = c.command_json(&feeds_json_cmd("build_context")).unwrap();
        let b = c.command_json(&feeds_json_cmd("facts")).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn query_before_build_is_an_error() {
        let mut c = Copilot::new(&spec_json()).unwrap();
        let out = c
            .command_json(r#"{"cmd":"query","question":"why"}"#)
            .unwrap();
        assert!(out.contains("\"ok\":false"));
    }

    #[test]
    fn unknown_command_is_error_json() {
        let mut c = Copilot::new(&spec_json()).unwrap();
        let out = c.command_json(r#"{"cmd":"nope"}"#).unwrap();
        assert!(out.contains("\"ok\":false"));
        assert!(out.contains("unknown cmd"));
    }

    #[test]
    fn set_spec_then_build() {
        let mut c = Copilot::new("").unwrap();
        let out = c
            .command_json(&format!(r#"{{"cmd":"set_spec","spec":{}}}"#, spec_json()))
            .unwrap();
        assert_eq!(out, r#"{"ok":true}"#);
        let ctx = c.command_json(&feeds_json_cmd("build_context")).unwrap();
        assert!(ctx.contains("price_move"));
    }

    #[test]
    fn reset_clears_context() {
        let mut c = Copilot::new(&spec_json()).unwrap();
        c.command_json(&feeds_json_cmd("build_context")).unwrap();
        assert_eq!(
            c.command_json(r#"{"cmd":"reset"}"#).unwrap(),
            r#"{"ok":true}"#
        );
        let out = c
            .command_json(r#"{"cmd":"query","question":"why"}"#)
            .unwrap();
        assert!(out.contains("\"ok\":false"));
    }

    #[test]
    fn native_api_round_trip() {
        let mut c = Copilot::new(&spec_json()).unwrap();
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
                        open: 94.0,
                        high: 94.0,
                        low: 94.0,
                        close: 94.0,
                        volume: 1.0,
                    },
                ],
                orderbook: None,
                trades: Vec::new(),
                funding: Vec::new(),
                open_interest: vec![OiPoint { ts: 1, oi: 100.0 }, OiPoint { ts: 2, oi: 95.0 }],
                liquidations: Vec::new(),
            },
        );
        let ctx = c.build_context(&feeds).unwrap();
        assert!(ctx.facts.iter().any(|f| f.kind == FactKind::PriceMove));
        assert!(!c.query("why did BTC dump").is_empty());
    }
}
