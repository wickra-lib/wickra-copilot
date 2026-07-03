//! The context specification — a serde `ContextSpec` and its validation.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::fact::FactKind;

/// The data-driven request the context builder answers: which perp symbols to
/// look at, how long a window to fold, and which facts to derive.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContextSpec {
    /// The perp symbols to build context for; at least one is required.
    pub symbols: Vec<String>,
    /// The lookback window (bars / window length) for the derivations.
    pub lookback: u32,
    /// Which fact kinds to derive; at least one, with no duplicates.
    pub facts: Vec<FactKind>,
    /// The candle timeframe (e.g. `"1h"`); documentary only.
    #[serde(default)]
    pub timeframe: Option<String>,
}

impl ContextSpec {
    /// Parse a spec from JSON and validate it.
    pub fn from_json(s: &str) -> Result<Self> {
        let spec: ContextSpec = serde_json::from_str(s)?;
        spec.validate()?;
        Ok(spec)
    }

    /// Parse a spec from TOML and validate it.
    pub fn from_toml(s: &str) -> Result<Self> {
        let spec: ContextSpec = toml::from_str(s)?;
        spec.validate()?;
        Ok(spec)
    }

    /// Check the spec is structurally sound: at least one symbol, a positive
    /// lookback, and at least one fact kind with no duplicates.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.symbols.is_empty() {
            return Err(Error::BadSpec("at least one symbol is required".into()));
        }
        if self.lookback == 0 {
            return Err(Error::BadSpec("lookback must be greater than zero".into()));
        }
        if self.facts.is_empty() {
            return Err(Error::BadSpec("at least one fact kind is required".into()));
        }
        let mut seen = BTreeSet::new();
        for kind in &self.facts {
            if !seen.insert(*kind) {
                return Err(Error::BadSpec(format!("duplicate fact kind: {kind:?}")));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(symbols: Vec<&str>, lookback: u32, facts: Vec<FactKind>) -> ContextSpec {
        ContextSpec {
            symbols: symbols.into_iter().map(String::from).collect(),
            lookback,
            facts,
            timeframe: None,
        }
    }

    #[test]
    fn valid_spec_passes() {
        assert!(spec(vec!["BTCUSDT"], 20, vec![FactKind::PriceMove])
            .validate()
            .is_ok());
    }

    #[test]
    fn empty_symbols_rejected() {
        assert!(spec(vec![], 20, vec![FactKind::PriceMove])
            .validate()
            .is_err());
    }

    #[test]
    fn zero_lookback_rejected() {
        assert!(spec(vec!["BTCUSDT"], 0, vec![FactKind::PriceMove])
            .validate()
            .is_err());
    }

    #[test]
    fn empty_facts_rejected() {
        assert!(spec(vec!["BTCUSDT"], 20, vec![]).validate().is_err());
    }

    #[test]
    fn duplicate_facts_rejected() {
        assert!(spec(
            vec!["BTCUSDT"],
            20,
            vec![FactKind::PriceMove, FactKind::PriceMove]
        )
        .validate()
        .is_err());
    }

    #[test]
    fn from_json_parses_defaults_and_validates() {
        let json = r#"{"symbols":["BTCUSDT"],"lookback":20,"facts":["price_move","oi_change"]}"#;
        let spec = ContextSpec::from_json(json).unwrap();
        assert_eq!(spec.symbols, vec!["BTCUSDT".to_string()]);
        assert_eq!(spec.lookback, 20);
        assert!(spec.timeframe.is_none());
    }
}
