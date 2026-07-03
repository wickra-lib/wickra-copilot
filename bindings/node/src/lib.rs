//! Node.js bindings for `wickra-copilot` (napi-rs).
//!
//! Thin glue over the copilot core's data-driven surface: build a `Copilot` from
//! a spec JSON, drive it with a command JSON and read back the response JSON. The
//! same command protocol crosses every binding, so a Node front-end drives the
//! exact same deterministic core as the native CLI. The LLM adapter
//! (`copilot-llm`) is deliberately never exposed here, so the network and API key
//! stay off the binding surface.

#![allow(missing_debug_implementations)]
// napi exposes owned `String` arguments; the bodies only need to borrow them.
#![allow(clippy::needless_pass_by_value)]

use napi::Result;
use napi_derive::napi;

use copilot_core::Copilot as CoreCopilot;

/// Build a napi error from a message.
fn err(message: impl Into<String>) -> napi::Error {
    napi::Error::from_reason(message.into())
}

/// The library version.
#[napi]
pub fn version() -> String {
    CoreCopilot::version().to_string()
}

/// A copilot instance driven by JSON commands.
#[napi]
pub struct Copilot {
    inner: CoreCopilot,
}

#[napi]
impl Copilot {
    /// Build a copilot from a spec JSON string.
    #[napi(constructor)]
    pub fn new(spec_json: String) -> Result<Self> {
        CoreCopilot::new(&spec_json)
            .map(|inner| Self { inner })
            .map_err(|e| err(e.to_string()))
    }

    /// Apply a command JSON and return the resulting response JSON.
    #[napi]
    pub fn command(&mut self, cmd_json: String) -> Result<String> {
        self.inner
            .command_json(&cmd_json)
            .map_err(|e| err(e.to_string()))
    }

    /// The library version.
    #[napi]
    pub fn version(&self) -> String {
        CoreCopilot::version().to_string()
    }
}
