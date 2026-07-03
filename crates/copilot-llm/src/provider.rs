//! The OpenAI-compatible provider — the only place in the whole workspace that
//! touches the network.
//!
//! A [`Provider`] preset selects an endpoint (`Ollama`, `Openai`, `Claude`,
//! `Gemini`) or defers entirely to the environment (`Custom`). One
//! [`OpenAiCompatible`] HTTP implementation drives them all: the preset only
//! chooses the base URL and default model, each overridable from the
//! environment. The API key is read from `WICKRA_COPILOT_API_KEY` and never
//! appears in `Debug`, logs, or the answer — the manual `Debug` impl below
//! redacts it.

use std::env;
use std::fmt;

use serde::Deserialize;

use crate::{Error, LlmProvider, Message, Result};

/// An endpoint preset. Every preset except `Custom` carries a default base URL
/// and model; `Custom` takes both from the environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    /// Local Ollama server — no API key required.
    Ollama,
    /// OpenAI.
    Openai,
    /// Anthropic's OpenAI-compatible endpoint.
    Claude,
    /// Google Gemini's OpenAI-compatible endpoint.
    Gemini,
    /// A user-supplied endpoint, configured entirely from the environment.
    Custom,
}

impl Provider {
    /// Parse a provider name (case-insensitive).
    ///
    /// # Errors
    /// Returns an error if the name is not one of the known presets.
    // Named `from_str` per the copilot provider contract; it is a fallible
    // preset lookup, not the `std::str::FromStr` conversion.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(name: &str) -> Result<Self> {
        match name.to_ascii_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" => Ok(Self::Openai),
            "claude" => Ok(Self::Claude),
            "gemini" => Ok(Self::Gemini),
            "custom" => Ok(Self::Custom),
            other => Err(Error::Provider(format!("unknown provider: {other}"))),
        }
    }

    /// The preset's default base URL, or `""` for `Custom` (which must be
    /// configured via `WICKRA_COPILOT_BASE_URL`).
    fn default_base_url(self) -> &'static str {
        match self {
            Self::Ollama => "http://localhost:11434/v1",
            Self::Openai => "https://api.openai.com/v1",
            Self::Claude => "https://api.anthropic.com/v1",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
            Self::Custom => "",
        }
    }

    /// The preset's default model, overridable via `WICKRA_COPILOT_MODEL`.
    fn default_model(self) -> &'static str {
        match self {
            Self::Ollama => "llama3",
            Self::Openai => "gpt-4o-mini",
            Self::Claude => "claude-3-5-sonnet-latest",
            Self::Gemini => "gemini-1.5-flash",
            Self::Custom => "default",
        }
    }
}

/// A single OpenAI-compatible chat endpoint.
///
/// The API key is intentionally excluded from every observable surface: the
/// `Debug` impl redacts it, and it never reaches an [`crate::Answer`].
pub struct OpenAiCompatible {
    base_url: String,
    model: String,
    api_key: String,
}

impl OpenAiCompatible {
    /// Build a provider from a preset, applying the environment overrides.
    ///
    /// `base_url` is `WICKRA_COPILOT_BASE_URL` if set, otherwise the preset
    /// default; `model` is `WICKRA_COPILOT_MODEL` if set, otherwise the preset
    /// default; the API key is `WICKRA_COPILOT_API_KEY` (empty is allowed, e.g.
    /// for a local Ollama server).
    ///
    /// # Errors
    /// Returns an error if `provider` is [`Provider::Custom`] and
    /// `WICKRA_COPILOT_BASE_URL` is not set.
    pub fn from_provider(provider: Provider) -> Result<Self> {
        let base_url = match env::var("WICKRA_COPILOT_BASE_URL") {
            Ok(url) if !url.is_empty() => url,
            _ => {
                let default = provider.default_base_url();
                if default.is_empty() {
                    return Err(Error::Provider(
                        "the custom provider requires WICKRA_COPILOT_BASE_URL".to_string(),
                    ));
                }
                default.to_string()
            }
        };
        let model = match env::var("WICKRA_COPILOT_MODEL") {
            Ok(m) if !m.is_empty() => m,
            _ => provider.default_model().to_string(),
        };
        let api_key = env::var("WICKRA_COPILOT_API_KEY").unwrap_or_default();
        Ok(Self {
            base_url,
            model,
            api_key,
        })
    }
}

impl fmt::Debug for OpenAiCompatible {
    /// Redacts the API key so it can never leak through a debug print.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAiCompatible")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

/// The subset of the `chat/completions` response we read: the first choice's
/// message content.
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

impl LlmProvider for OpenAiCompatible {
    fn complete(&self, messages: &[Message]) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({ "model": self.model, "messages": messages });
        let client = reqwest::blocking::Client::new();
        let mut request = client.post(&url).json(&body);
        if !self.api_key.is_empty() {
            request = request.bearer_auth(&self.api_key);
        }
        let response = request
            .send()
            .map_err(|e| Error::Provider(e.to_string()))?
            .error_for_status()
            .map_err(|e| Error::Provider(e.to_string()))?;
        let parsed: ChatResponse = response
            .json()
            .map_err(|e| Error::Provider(e.to_string()))?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| Error::Provider("response contained no choices".to_string()))
    }

    fn model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_parses_presets_case_insensitively() {
        assert_eq!(Provider::from_str("ollama").unwrap(), Provider::Ollama);
        assert_eq!(Provider::from_str("OpenAI").unwrap(), Provider::Openai);
        assert_eq!(Provider::from_str("Claude").unwrap(), Provider::Claude);
        assert_eq!(Provider::from_str("gemini").unwrap(), Provider::Gemini);
        assert_eq!(Provider::from_str("custom").unwrap(), Provider::Custom);
        assert!(Provider::from_str("nope").is_err());
    }

    #[test]
    fn presets_carry_their_default_base_urls() {
        assert_eq!(
            Provider::Ollama.default_base_url(),
            "http://localhost:11434/v1"
        );
        assert_eq!(
            Provider::Openai.default_base_url(),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            Provider::Claude.default_base_url(),
            "https://api.anthropic.com/v1"
        );
        assert_eq!(
            Provider::Gemini.default_base_url(),
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
        // Custom has no default; it must come from the environment.
        assert_eq!(Provider::Custom.default_base_url(), "");
    }

    #[test]
    fn debug_redacts_the_api_key() {
        let provider = OpenAiCompatible {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: "sk-supersecret-key".to_string(),
        };
        let rendered = format!("{provider:?}");
        assert!(!rendered.contains("sk-supersecret-key"));
        assert!(rendered.contains("<redacted>"));
        // The non-secret fields are still visible.
        assert!(rendered.contains("gpt-4o-mini"));
        assert_eq!(provider.model(), "gpt-4o-mini");
    }
}
