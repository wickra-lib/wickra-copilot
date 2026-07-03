//! Non-deterministic LLM adapter for the Wickra Copilot.
//!
//! `copilot-core` produces a deterministic `MarketContext` — a stable, ranked
//! list of hard facts. This crate is the *only* place that turns that context
//! into a natural-language answer: it renders the facts into a prompt and calls
//! the user's own OpenAI-compatible endpoint (Ollama, OpenAI, Claude or Gemini),
//! reading the API key from the environment.
//!
//! It is kept deliberately separate from the core for three reasons: the LLM
//! call is non-deterministic and must never enter the golden/CI comparison; the
//! network and API key stay out of the core and every language binding (this
//! crate is never exposed over the C ABI); and the endpoint is swappable without
//! touching the fact-derivation logic.
//!
//! The public surface — the `LlmProvider` trait, the `Message` / `Answer` data
//! types, the `ask` entry point and the `OpenAiCompatible` provider — is added in
//! the following units.
