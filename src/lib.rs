// src/lib.rs
use pyo3::prelude::*;

// Re-export your Rust modules for the CLI and for external users
pub mod config;
pub mod experiments;
pub mod generators;
pub mod utils;
pub mod analysis;
pub mod lambda;
pub mod supercollider;

// New Python wrapper module
mod python;

#[pymodule]
fn alchemy(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    python::register(m)
}

// If you still need this for the CLI, keep it here:
use lambda_calculus::{parse, term::Notation::Classic, Term};
use std::io::{self, BufRead};



pub fn read_inputs() -> impl Iterator<Item = Term> {
    io::stdin()
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| parse(&line, Classic).ok())
}

