//! Python bindings for `wickra-copilot`, exposed under the `wickra_copilot`
//! package.
//!
//! Thin glue over the copilot core's data-driven surface: build a [`Copilot`]
//! from a spec JSON, drive it with a command JSON and read back the response
//! JSON. The same command protocol crosses every binding, so a Python front-end
//! drives the exact same deterministic core as the native CLI. The LLM adapter
//! (`copilot-llm`) is deliberately never exposed here, so the network and API
//! key stay off the binding surface.

// PyO3 protocol methods take `self` by value/ref regardless of use.
#![allow(clippy::needless_pass_by_value)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use copilot_core::Copilot;

/// A copilot instance driven by JSON commands.
///
/// `unsendable`: a handle owns a mutable spec and the last-built context, so it
/// is bound to the thread that created it.
#[pyclass(name = "Copilot", unsendable)]
struct PyCopilot {
    inner: Copilot,
}

#[pymethods]
impl PyCopilot {
    /// Build a copilot from a spec JSON string (`""` or `"{}"` for an empty
    /// handle whose spec is set later via a `set_spec` command).
    #[new]
    fn new(spec_json: &str) -> PyResult<Self> {
        Copilot::new(spec_json)
            .map(|inner| Self { inner })
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Apply a command JSON and return the resulting response JSON.
    fn command(&mut self, cmd_json: &str) -> PyResult<String> {
        self.inner
            .command_json(cmd_json)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// The library version.
    #[staticmethod]
    fn version() -> &'static str {
        copilot_core::version()
    }
}

/// The native module (`wickra_copilot._wickra_copilot`).
#[pymodule]
fn _wickra_copilot(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyCopilot>()?;
    Ok(())
}
