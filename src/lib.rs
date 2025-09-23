use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde::{Deserialize, Serialize};


// --- Module Imports ---
mod config;
mod soup;
mod experiments;
mod generators;
mod utils;
mod analysis;

use lambda_calculus::{parse, term::Notation::Classic, Term};
use std::io::{self, BufRead};

pub fn read_inputs() -> impl Iterator<Item = Term> {
    io::stdin()
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| parse(&line, Classic).ok())
}


// --- Exposing Core Configurations ---
use crate::config::{Reactor as RustReactor, ConfigSeed};
use crate::soup::{Soup, ReactionError};
use crate::generators::{BTreeGen as RustBTreeGen, FontanaGen as RustFontanaGen, Standardization as RustStandardization};
use crate::utils::{decode_hex, encode_hex};
use crate::experiments::{look_for_add, entropy_series, entropy_test, sync_entropy_test};


// --- Error Handling Wrappers ---
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyReactionError {
    kind: ReactionErrorKind,
}

#[derive(Debug, Clone)]
pub enum ReactionErrorKind {
    ExceedsReductionLimit,
    NotEnoughExpressions,
    IsIdentity,
    IsParent,
    HasFreeVariables,
    ExceedsDepthLimit,
}

impl From<ReactionError> for PyReactionError {
    fn from(error: ReactionError) -> Self {
        let kind = match error {
            ReactionError::ExceedsReductionLimit => ReactionErrorKind::ExceedsReductionLimit,
            ReactionError::NotEnoughExpressions => ReactionErrorKind::NotEnoughExpressions,
            ReactionError::IsIdentity => ReactionErrorKind::IsIdentity,
            ReactionError::IsParent => ReactionErrorKind::IsParent,
            ReactionError::HasFreeVariables => ReactionErrorKind::HasFreeVariables,
            ReactionError::ExceedsDepthLimit => ReactionErrorKind::ExceedsDepthLimit,
        };
        PyReactionError { kind }
    }
}

// --- Reactor Wrapper ---
#[pyclass]
pub struct PyReactor {
    inner: RustReactor,
}

#[pymethods]
impl PyReactor {
    #[new]
    fn new() -> Self {
        PyReactor {
            inner: RustReactor::new(),
        }
    }
}

// --- Standardization Wrapper ---
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyStandardization {
    standardization: RustStandardization,
}

#[pymethods]
impl PyStandardization {
    #[new]
    fn new(kind: &str) -> PyResult<Self> {
        let standardization = match kind {
            "prefix" => RustStandardization::Prefix,
            "postfix" => RustStandardization::Postfix,
            "none" => RustStandardization::None,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid standardization type"))
        };
        Ok(PyStandardization { standardization })
    }
}

impl From<PyStandardization> for RustStandardization {
    fn from(py_std: PyStandardization) -> Self {
        py_std.standardization
    }
}

// --- Soup Wrapper ---
#[pyclass]
pub struct PySoup {
    inner: Soup,
}

#[pymethods]
impl PySoup {
    #[new]
    fn new() -> Self {
        PySoup { inner: Soup::new() }
    }

    #[staticmethod]
    fn from_config(cfg: &PyReactor) -> Self {
        PySoup { inner: Soup::from_config(&cfg.inner) }
    }

    fn set_limit(&mut self, limit: usize) {
        self.inner.set_limit(limit);
    }

    fn perturb(&mut self, expressions: Vec<String>) -> PyResult<()> {
        let terms = expressions
            .into_iter()
            .filter_map(|s| parse(&s, Classic).ok());
        self.inner.perturb(terms);  // Pass iterator directly
        Ok(())
    }

    fn simulate_for(&mut self, n: usize, log: bool) -> usize {
        self.inner.simulate_for(n, log)
    }

    fn expressions(&self) -> Vec<String> {
        self.inner
            .expressions()
            .map(|term| term.to_string())
            .collect()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn collisions(&self) -> usize {
        self.inner.collisions()
    }

    fn unique_expressions(&self) -> Vec<String> {
        self.inner
            .unique_expressions()
            .into_iter()
            .map(|term| term.to_string())
            .collect()
    }

    fn expression_counts(&self) -> Vec<(String, u32)> {
        self.inner
            .expression_counts()
            .into_iter()
            .map(|(term, count)| (term.to_string(), count))
            .collect()
    }

    fn population_entropy(&self) -> f32 {
        self.inner.population_entropy()
    }
}

// --- BTreeGen Wrapper ---
#[pyclass]
pub struct PyBTreeGen {
    inner: RustBTreeGen,
}

#[pymethods]
impl PyBTreeGen {
    #[new]
    fn new() -> Self {
        PyBTreeGen { inner: RustBTreeGen::new() }
    }

    #[staticmethod]
    fn from_config(size: u32, freevar_generation_probability: f64, max_free_vars: u32, std: PyStandardization) -> Self {
        let config = config::BTreeGen {
            size,
            freevar_generation_probability,
            n_max_free_vars: max_free_vars,
            standardization: std.into(),
            seed: ConfigSeed(Some([0; 32])),
        };
        PyBTreeGen { inner: RustBTreeGen::from_config(&config) }
    }

    fn generate(&mut self) -> String {
        self.inner.generate().to_string()
    }

    fn generate_n(&mut self, n: usize) -> Vec<String> {
        self.inner.generate_n(n).into_iter().map(|t| t.to_string()).collect()
    }
}

// --- FontanaGen Wrapper ---
#[pyclass]
pub struct PyFontanaGen {
    inner: RustFontanaGen,
}

#[pymethods]
impl PyFontanaGen {
    #[staticmethod]
    fn from_config(abs_range: (f64, f64), app_range: (f64, f64), max_depth: u32, max_free_vars: u32) -> Self {
        let config = config::FontanaGen {
            abstraction_prob_range: abs_range,
            application_prob_range: app_range,
            max_depth,
            n_max_free_vars: max_free_vars,
            seed: ConfigSeed(Some([0; 32])),
        };
        PyFontanaGen { inner: RustFontanaGen::from_config(&config) }
    }

    fn generate(&self) -> Option<String> {
        self.inner.generate().map(|term| term.to_string())
    }
}

// --- Utilities ---
#[pyfunction]
fn decode_hex_py(hex_string: &str) -> PyResult<Vec<u8>> {
    decode_hex(hex_string).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn encode_hex_py(bytes: Vec<u8>) -> String {
    encode_hex(&bytes)
}

// --- Experiment Functions ---
#[pyfunction]
fn run_look_for_add() -> PyResult<()> {
    async_std::task::block_on(look_for_add());
    Ok(())
}

#[pyfunction]
fn run_entropy_series() -> PyResult<()> {
    async_std::task::block_on(entropy_series());
    Ok(())
}

#[pyfunction]
fn run_entropy_test() -> PyResult<()> {
    async_std::task::block_on(entropy_test());
    Ok(())
}

#[pyfunction]
fn run_sync_entropy_test() -> PyResult<()> {
    sync_entropy_test();
    Ok(())
}

// --- Python Module Initialization ---
#[pymodule]
fn alchemy(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register classes
    m.add_class::<PySoup>()?;
    m.add_class::<PyReactor>()?;
    m.add_class::<PyReactionError>()?;
    m.add_class::<PyStandardization>()?;
    m.add_class::<PyBTreeGen>()?;
    m.add_class::<PyFontanaGen>()?;

    // Register functions
    m.add_function(wrap_pyfunction!(decode_hex_py, m)?)?;
    m.add_function(wrap_pyfunction!(encode_hex_py, m)?)?;
    m.add_function(wrap_pyfunction!(run_look_for_add, m)?)?;
    m.add_function(wrap_pyfunction!(run_entropy_series, m)?)?;
    m.add_function(wrap_pyfunction!(run_entropy_test, m)?)?;
    m.add_function(wrap_pyfunction!(run_sync_entropy_test, m)?)?;

    Ok(())
}
