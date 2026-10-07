// src/lib.rs
use pyo3::prelude::*;

// Re-export your Rust modules for the CLI and for external users
pub mod config;
pub mod experiments;
pub mod lambda;
pub mod soupercollider;
pub mod record;
pub mod traits;
pub mod utils;
pub mod cli;
pub mod errors;
pub mod recursive_experiment;

// New Python wrapper module
mod python;

#[pymodule]
fn alchemy(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    python::register(m)
}
