// src/python.rs
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde::{Deserialize, Serialize};

use lambda_calculus::{parse, term::Notation::Classic, Term};

use crate::config::{self, ConfigSeed, Reactor as RustReactor};
use crate::generators::{
    BTreeGen as RustBTreeGen, FontanaGen as RustFontanaGen, Standardization as RustStandardization,
};
use crate::lambda::recursive::{
    AlchemyCollider, LambdaCollisionError, LambdaCollisionOk, LambdaParticle,
};
use crate::supercollider::Soup as GenericSoup;
use crate::utils::{decode_hex, encode_hex};

// Concrete soup alias for the recursive lambda flavor
type RustSoup =
    GenericSoup<LambdaParticle, AlchemyCollider, LambdaCollisionOk, LambdaCollisionError>;

// ============ Errors exposed to Python ============

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
    RecursiveArgument,
    BadArgument,
}

impl From<LambdaCollisionError> for PyReactionError {
    fn from(error: LambdaCollisionError) -> Self {
        let kind = match error {
            LambdaCollisionError::ExceedsReductionLimit => ReactionErrorKind::ExceedsReductionLimit,
            LambdaCollisionError::NotEnoughExpressions => ReactionErrorKind::NotEnoughExpressions,
            LambdaCollisionError::IsIdentity => ReactionErrorKind::IsIdentity,
            LambdaCollisionError::IsParent => ReactionErrorKind::IsParent,
            LambdaCollisionError::HasFreeVariables => ReactionErrorKind::HasFreeVariables,
            LambdaCollisionError::ExceedsDepthLimit => ReactionErrorKind::ExceedsDepthLimit,
            LambdaCollisionError::RecursiveArgument => ReactionErrorKind::RecursiveArgument,
            LambdaCollisionError::BadArgument => ReactionErrorKind::BadArgument,
        };
        PyReactionError { kind }
    }
}

// ============ Reactor wrapper ============

#[pyclass]
pub struct PyReactor {
    pub(crate) inner: RustReactor,
}

#[pymethods]
impl PyReactor {
    #[new]
    fn new() -> Self {
        PyReactor { inner: RustReactor::new() }
    }
}

// ============ Standardization wrapper ============

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
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid standardization type")),
        };
        Ok(PyStandardization { standardization })
    }
}

impl From<PyStandardization> for RustStandardization {
    fn from(py_std: PyStandardization) -> Self { py_std.standardization }
}

// ============ Soup wrapper ============

#[pyclass]
pub struct PySoup {
    inner: RustSoup,
}

#[pymethods]
impl PySoup {
    #[new]
    fn new() -> Self { PySoup { inner: RustSoup::new() } }

    #[staticmethod]
    fn from_config(cfg: &PyReactor) -> Self {
        PySoup { inner: RustSoup::from_config(&cfg.inner) }
    }

    fn perturb(&mut self, expressions: Vec<String>) -> PyResult<()> {
        let terms = expressions.into_iter().filter_map(|s| parse(&s, Classic).ok());
        self.inner.add_lambda_expressions(terms);
        Ok(())
    }

    fn simulate_for(&mut self, n: usize, log: bool) -> usize { self.inner.simulate_for(n, log) }
    fn len(&self) -> usize { self.inner.len() }
    fn collisions(&self) -> usize { self.inner.collisions() }

    fn expressions(&self) -> Vec<String> {
        self.inner.lambda_expressions().map(|t| t.to_string()).collect()
    }
    fn unique_expressions(&self) -> Vec<String> {
        self.inner.unique_expressions().into_iter().map(|t| t.to_string()).collect()
    }
    fn expression_counts(&self) -> Vec<(String, u32)> {
        self.inner.expression_counts().into_iter().map(|(t, c)| (t.to_string(), c)).collect()
    }
    fn population_entropy(&self) -> f32 { self.inner.population_entropy() }
}

// ============ Generators ============

#[pyclass]
pub struct PyBTreeGen { inner: RustBTreeGen }

#[pymethods]
impl PyBTreeGen {
    #[new]
    fn new() -> Self { PyBTreeGen { inner: RustBTreeGen::new() } }

    #[staticmethod]
    fn from_config(
        size: u32,
        freevar_generation_probability: f64,
        max_free_vars: u32,
        std: PyStandardization,
    ) -> Self {
        let cfg = config::BTreeGen {
            size,
            freevar_generation_probability,
            n_max_free_vars: max_free_vars,
            standardization: std.into(),
            seed: ConfigSeed(Some([0; 32])),
        };
        PyBTreeGen { inner: RustBTreeGen::from_config(&cfg) }
    }

    fn generate(&mut self) -> String { self.inner.generate().to_string() }

    fn generate_n(&mut self, n: usize) -> Vec<String> {
        self.inner.generate_n(n).into_iter().map(|t| t.to_string()).collect()
    }
}

#[pyclass]
pub struct PyFontanaGen { inner: RustFontanaGen }

#[pymethods]
impl PyFontanaGen {
    #[staticmethod]
    fn from_config(
        abs_range: (f64, f64),
        app_range: (f64, f64),
        max_depth: u32,
        max_free_vars: u32,
    ) -> Self {
        let cfg = config::FontanaGen {
            abstraction_prob_range: abs_range,
            application_prob_range: app_range,
            max_depth,
            n_max_free_vars: max_free_vars,
            seed: ConfigSeed(Some([0; 32])),
        };
        PyFontanaGen { inner: RustFontanaGen::from_config(&cfg) }
    }

    fn generate(&self) -> Option<String> { self.inner.generate().map(|t| t.to_string()) }
}

// ============ Utilities ============

#[pyfunction]
fn decode_hex_py(hex_string: &str) -> PyResult<Vec<u8>> {
    decode_hex(hex_string).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn encode_hex_py(bytes: Vec<u8>) -> String { encode_hex(&bytes) }

// ============ Public registration hook ============

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySoup>()?;
    m.add_class::<PyReactor>()?;
    m.add_class::<PyReactionError>()?;
    m.add_class::<PyStandardization>()?;
    m.add_class::<PyBTreeGen>()?;
    m.add_class::<PyFontanaGen>()?;
    m.add_function(wrap_pyfunction!(decode_hex_py, m)?)?;
    m.add_function(wrap_pyfunction!(encode_hex_py, m)?)?;
    Ok(())
}
