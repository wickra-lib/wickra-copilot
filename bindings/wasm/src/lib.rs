//! WebAssembly bindings for `wickra-copilot` (wasm-bindgen).
//!
//! The deterministic market-context core, compiled to WebAssembly for the
//! browser: build a `Copilot` from a spec JSON, drive it with a command JSON and
//! read back the response JSON. The same command protocol crosses every binding,
//! so a browser front-end drives the exact same core as the native CLI. The LLM
//! adapter (`copilot-llm`) is never exposed here, so the network and API key stay
//! off the binding surface.
//!
//! The `parallel` feature of the core is disabled here: rayon's thread pool is
//! not available in a browser sandbox, so the context folds sequentially — which
//! is byte-identical to the parallel build, the exact cross-language golden
//! check.

use wasm_bindgen::prelude::*;

use copilot_core::Copilot as CoreCopilot;

/// A copilot instance driven by JSON commands.
#[wasm_bindgen]
pub struct Copilot {
    inner: CoreCopilot,
}

#[wasm_bindgen]
impl Copilot {
    /// Build a copilot from a spec JSON string.
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<Copilot, JsError> {
        CoreCopilot::new(spec_json)
            .map(|inner| Self { inner })
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Apply a command JSON and return the resulting response JSON.
    pub fn command(&mut self, cmd_json: &str) -> Result<String, JsError> {
        self.inner
            .command_json(cmd_json)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// The library version.
    #[wasm_bindgen(js_name = version)]
    pub fn instance_version(&self) -> String {
        CoreCopilot::version().to_string()
    }
}

/// The library version.
#[wasm_bindgen]
pub fn version() -> String {
    CoreCopilot::version().to_string()
}
