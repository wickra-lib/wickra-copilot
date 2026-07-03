//! The CLI config file — a [`ContextSpec`] wrapper.
//!
//! The CLI reads a JSON or TOML config with a top-level `spec` table and hands
//! the validated spec to the core. Parsing validates the spec, so a malformed
//! config is rejected at load time rather than mid-build.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::spec::ContextSpec;

/// A parsed config file.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    /// The context spec to build.
    pub spec: ContextSpec,
}

impl Config {
    /// Parse and validate a config from JSON.
    ///
    /// # Errors
    /// Returns an error if the config fails to parse or the spec fails to
    /// validate.
    pub fn from_json(s: &str) -> Result<Self> {
        let config: Config = serde_json::from_str(s)?;
        config.spec.validate()?;
        Ok(config)
    }

    /// Parse and validate a config from TOML.
    ///
    /// # Errors
    /// Returns an error if the config fails to parse or the spec fails to
    /// validate.
    pub fn from_toml(s: &str) -> Result<Self> {
        let config: Config = toml::from_str(s)?;
        config.spec.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_config() {
        let config = Config::from_json(
            r#"{ "spec": {
                "symbols": ["BTCUSDT"],
                "lookback": 20,
                "facts": ["price_move", "oi_change"]
            } }"#,
        )
        .unwrap();
        assert_eq!(config.spec.symbols, vec!["BTCUSDT"]);
        assert_eq!(config.spec.facts.len(), 2);
    }

    #[test]
    fn parses_toml_config() {
        let config = Config::from_toml(
            r#"
            [spec]
            symbols = ["BTCUSDT"]
            lookback = 20
            facts = ["price_move"]
            "#,
        )
        .unwrap();
        assert_eq!(config.spec.symbols, vec!["BTCUSDT"]);
    }

    #[test]
    fn rejects_an_invalid_spec() {
        // Empty facts fails validation.
        let err = Config::from_json(
            r#"{ "spec": { "symbols": ["BTCUSDT"], "lookback": 20, "facts": [] } }"#,
        );
        assert!(err.is_err());
    }
}
