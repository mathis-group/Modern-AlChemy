// src/python.rs
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde::{Deserialize, Serialize};
use rand::Rng; // Import Rng for random seed generation

use lambda_calculus::{parse, term::Notation::Classic};

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

// ============ Helper for Seed Parsing ============

fn parse_seed(seed_hex: Option<String>) -> PyResult<[u8; 32]> {
    match seed_hex {
        Some(s) => {
            let bytes = decode_hex(&s).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid hex seed: {}", e))
            })?;
            bytes.try_into().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err("Seed must be exactly 32 bytes (64 hex chars)")
            })
        }
        None => {
            // Generate random seed if not provided
            let mut rng = rand::thread_rng();
            Ok(rng.gen())
        }
    }
}

// ============ Errors exposed to Python ============

#[pyclass]
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PyReactionError {
    kind: ReactionErrorKind,
}

#[derive(Debug, Clone, Copy)]
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

impl ReactionErrorKind {
    fn as_str(&self) -> &'static str {
        match self {
            ReactionErrorKind::ExceedsReductionLimit => "exceeds_reduction_limit",
            ReactionErrorKind::NotEnoughExpressions => "not_enough_expressions",
            ReactionErrorKind::IsIdentity => "is_identity",
            ReactionErrorKind::IsParent => "is_parent",
            ReactionErrorKind::HasFreeVariables => "has_free_variables",
            ReactionErrorKind::ExceedsDepthLimit => "exceeds_depth_limit",
            ReactionErrorKind::RecursiveArgument => "recursive_argument",
            ReactionErrorKind::BadArgument => "bad_argument",
        }
    }
}

#[pymethods]
impl PyReactionError {
    #[getter]
    fn kind(&self) -> &'static str {
        self.kind.as_str()
    }
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
        PyReactor {
            inner: RustReactor::new(),
        }
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
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Invalid standardization type",
                ))
            }
        };
        Ok(PyStandardization { standardization })
    }
}

impl From<PyStandardization> for RustStandardization {
    fn from(py_std: PyStandardization) -> Self {
        py_std.standardization
    }
}

// ============ Soup wrapper ============

#[pyclass]
pub struct PySoup {
    inner: RustSoup,
}

#[pymethods]
impl PySoup {
    #[new]
    fn new() -> Self {
        PySoup {
            inner: RustSoup::new(),
        }
    }

    #[staticmethod]
    fn from_config(cfg: &PyReactor) -> Self {
        PySoup {
            inner: RustSoup::from_config(&cfg.inner),
        }
    }

    fn perturb(&mut self, expressions: Vec<String>) {
        let terms = expressions
            .into_iter()
            .filter_map(|s| parse(&s, Classic).ok());
        self.inner.add_lambda_expressions(terms);
    }

    fn simulate_for(&mut self, n: usize, log: bool) -> usize {
        self.inner.simulate_for(n, log)
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn collisions(&self) -> usize {
        self.inner.collisions()
    }

    fn expressions(&self) -> Vec<String> {
        self.inner
            .lambda_expressions()
            .map(|t| t.to_string())
            .collect()
    }
    fn unique_expressions(&self) -> Vec<String> {
        self.inner
            .unique_expressions()
            .into_iter()
            .map(|t| t.to_string())
            .collect()
    }
    fn expression_counts(&self) -> Vec<(String, u32)> {
        self.inner
            .expression_counts()
            .into_iter()
            .map(|(t, c)| (t.to_string(), c))
            .collect()
    }
    fn population_entropy(&self) -> f32 {
        self.inner.population_entropy()
    }
}

// ============ Generators ============

#[pyclass]
pub struct PyBTreeGen {
    inner: RustBTreeGen,
}

#[pymethods]
impl PyBTreeGen {
    #[new]
    fn new() -> Self {
        PyBTreeGen {
            inner: RustBTreeGen::new(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (size, freevar_generation_probability, max_free_vars, std, seed=None))]
    fn from_config(
        size: u32,
        freevar_generation_probability: f64,
        max_free_vars: u32,
        std: PyStandardization,
        seed: Option<String>,
    ) -> PyResult<Self> {
        let seed_bytes = parse_seed(seed)?;
        
        let cfg = config::BTreeGen {
            size,
            freevar_generation_probability,
            n_max_free_vars: max_free_vars,
            standardization: std.into(),
            seed: ConfigSeed::new(seed_bytes),
        };
        Ok(PyBTreeGen {
            inner: RustBTreeGen::from_config(&cfg),
        })
    }

    fn generate(&mut self) -> String {
        self.inner.generate().to_string()
    }

    #[pyo3(signature = (n, unique=false))]
    fn generate_n(&mut self, n: usize, unique: bool) -> Vec<String> {
        if unique {
            self.inner
                .generate_n_unique(n)
                .into_iter()
                .map(|t| t.to_string())
                .collect()
        } else {
            self.inner
                .generate_n(n)
                .into_iter()
                .map(|t| t.to_string())
                .collect()
        }
    }
}

#[pyclass]
pub struct PyFontanaGen {
    inner: RustFontanaGen,
}

#[pymethods]
impl PyFontanaGen {
    /// Build a Fontana generator from config values
    #[staticmethod]
    #[pyo3(signature = (abs_range, app_range, min_depth, max_depth, seed=None))]
    pub fn from_config(
        abs_range: (f64, f64),
        app_range: (f64, f64),
        min_depth: u32,
        max_depth: u32,
        seed: Option<String>,
    ) -> PyResult<Self> {
        let seed_bytes = parse_seed(seed)?;

        let cfg = config::FontanaGen {
            abstraction_prob_range: abs_range,
            application_prob_range: app_range,
            min_depth,
            max_depth,
            seed: ConfigSeed::new(seed_bytes),
        };
        Ok(PyFontanaGen {
            inner: RustFontanaGen::from_config(&cfg),
        })
    }

    /// Generate a single lambda term
    pub fn generate(&mut self) -> String {
        self.inner.generate().to_string()
    }

    /// Convenience: generate N terms
    #[pyo3(signature = (n, unique=false))]
    pub fn generate_n(&mut self, n: usize, unique: bool) -> Vec<String> {
        if unique {
            self.inner
                .generate_n_unique(n)
                .into_iter()
                .map(|t| t.to_string())
                .collect()
        } else {
            self.inner
                .generate_n(n)
                .into_iter()
                .map(|t| t.to_string())
                .collect()
        }
    }
}

// ============ Utilities ============

#[allow(clippy::useless_conversion)]
#[pyfunction]
fn decode_hex_py(hex_string: &str) -> PyResult<Vec<u8>> {
    decode_hex(hex_string).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn encode_hex_py(bytes: Vec<u8>) -> String {
    encode_hex(&bytes)
}

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