// src/python.rs
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use rand::Rng;
use serde::{Deserialize, Serialize};

use lambda_calculus::{parse, term::Notation::Classic};

use crate::config::generators::b_tree_gen::Standardization as RustStandardization;
use crate::config::config::{Config as RustConfig};
use crate::config::{config_seed::ConfigSeed, reactors::untyped_lambda::UntypedLambdaReactor as RustReactor};
use crate::config::generators::{b_tree_gen::BTreeGen as BTreeGenConfig, fontana_gen::FontanaGen as FontanaGenConfig};
use crate::lambda::generator::LambdaGenerator;
use crate::lambda::generators::{ b_tree_gen::BTreeGen as RustBTreeGen, fontana_gen::FontanaGen as RustFontanaGen };
use crate::lambda::{
    collider::LambdaCollider,
    particle::LambdaParticle,
    result::LambdaCollisionError
};

use crate::record::RecordingType;
use crate::soupercollider::Soup as GenericSoup;
use crate::utils::{decode_hex, encode_hex};

// Concrete soup alias for the recursive lambda flavor
type RustSoup = GenericSoup<LambdaParticle, LambdaCollider, LambdaGenerator>;

// ============ Helper for Seed Parsing ============

fn parse_seed(seed_hex: Option<String>) -> PyResult<[u8; 32]> {
    match seed_hex {
        Some(s) => {
            let bytes = decode_hex(&s).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid hex seed: {}", e))
            })?;
            bytes.try_into().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "Seed must be exactly 32 bytes (64 hex chars)",
                )
            })
        }
        None => {
            let mut rng = rand::thread_rng();
            Ok(rng.r#gen())
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

// ============ Reaction Record exposed to Python ============

#[pyclass]
#[derive(Clone)]
pub struct PyReactionRecord {
    #[pyo3(get)]
    pub step: usize,
    #[pyo3(get)]
    pub left: String,
    #[pyo3(get)]
    pub right: String,
    #[pyo3(get)]
    pub products: Vec<String>,
    #[pyo3(get)]
    pub success: bool,
    #[pyo3(get)]
    pub error: Option<String>,
}

#[pymethods]
impl PyReactionRecord {
    fn __repr__(&self) -> String {
        if self.success {
            format!(
                "ReactionRecord(step={}, left=..., right=..., products=[{}])",
                self.step,
                self.products.len()
            )
        } else {
            format!(
                "ReactionRecord(step={}, FAILED: {})",
                self.step,
                self.error.as_deref().unwrap_or("unknown")
            )
        }
    }
}


// ============ Config wrapper ============

#[pyclass]
pub struct PyConfig {
    pub(crate) inner: RustConfig,
}

#[pymethods]
impl PyConfig {
    #[new]
    fn new() -> Self {
        PyConfig { inner: RustConfig::new() }
    }

    /// Same JSON the CLI reads.
    #[staticmethod]
    fn from_json(s: &str) -> Self {
        PyConfig { inner: RustConfig::from_config_str(s) }
    }

    fn to_json(&self) -> String {
        self.inner.to_config_str()
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
            inner: RustReactor::default(),
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
    fn from_config(cfg: &PyConfig) -> Self {
        PySoup {
            inner: RustSoup::from_config(&cfg.inner),
        }
    }

    fn perturb(&mut self, expressions: Vec<String>) {
        let terms = expressions
            .into_iter()
            .filter_map(|s| parse(&s, Classic).ok());
        self.inner.add_lambda_expressions(terms, false);
    }

    fn simulate_for(&mut self, n: usize, log: bool) -> usize {
        if let Some(tape) = self.inner.simulate(n, RecordingType::SuccessOnly, None) {
            return tape.reaction_record.len()
        };
        0
    }

    /// Simulate for `n` collisions and return a list of ReactionRecord
    /// objects capturing every reaction (parents + products).
    fn simulate_for_logged(&mut self, n: usize) -> Vec<PyReactionRecord> {
        let mut reaction_record = Vec::new();
        if let Some(tape) = self.inner.simulate(n, RecordingType::All, None) {
            reaction_record
            .extend(
                tape.reaction_record.into_iter()
                .map(|r| PyReactionRecord {
                    step: r.step,
                    left: r.left.to_string(),
                    right: r.right.to_string(),
                    products: r.products.iter().map(|p| p.to_string()).collect(),
                    success: r.success,
                    error: r.error,
                })
            )
        }
        reaction_record
    }

    /// Simulate for `n` collisions and return only successful reaction records.
    /// Uses less memory than `simulate_for_logged` for large n.
    fn simulate_for_logged_filtered(&mut self, n: usize) -> Vec<PyReactionRecord> {
        let mut reaction_record = Vec::new();
        if let Some(tape) = self.inner.simulate(n, RecordingType::SuccessOnly, None) {
            reaction_record
            .extend(
                tape.reaction_record.into_iter()
                .map(|r| PyReactionRecord {
                    step: r.step,
                    left: r.left.to_string(),
                    right: r.right.to_string(),
                    products: r.products.iter().map(|p| p.to_string()).collect(),
                    success: r.success,
                    error: r.error,
                })
            )
        }
        reaction_record
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

        let cfg = BTreeGenConfig {
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
    /// Build a Fontana generator from config values.
    /// No free variable parameters — FontanaGen only produces closed terms.
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

        let cfg = FontanaGenConfig {
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

// ============ Recursive experiments ============

/// One generation's existing native Tape, without changing its recording policy.
#[pyclass]
#[derive(Clone)]
pub struct PyGenerationResult {
    #[pyo3(get)]
    pub generation: usize,
    #[pyo3(get)]
    pub history: Vec<Vec<String>>,
    #[pyo3(get)]
    pub history_collisions: Vec<usize>,
    #[pyo3(get)]
    pub initial_expressions: Vec<String>,
    #[pyo3(get)]
    pub final_expressions: Vec<String>,
    #[pyo3(get)]
    pub reactions: Vec<PyReactionRecord>,
}

/// Run the Rust generation loop using CLI-format JSON and an optional population.
/// History contains the native polled snapshots, not guaranteed final states.
#[pyfunction]
#[pyo3(signature = (config_json, initial_expressions=None))]
fn simulate_recursive_experiment(
    py: Python<'_>,
    config_json: &str,
    initial_expressions: Option<Vec<String>>,
) -> PyResult<Vec<PyGenerationResult>> {
    use pyo3::exceptions::PyValueError;

    let config: RustConfig = serde_json::from_str(config_json)
        .map_err(|error| PyValueError::new_err(format!("Invalid experiment config: {error}")))?;
    if config.recursive_config.n_generations == 0 {
        return Err(PyValueError::new_err("n_generations must be at least 1"));
    }
    if config.recursive_config.wipeout_percent > 100 {
        return Err(PyValueError::new_err(
            "wipeout_percent must be between 0 and 100",
        ));
    }
    if config.polling_interval == Some(0) {
        return Err(PyValueError::new_err(
            "polling_interval must be positive or null",
        ));
    }
    let population_size = initial_expressions
        .as_ref()
        .map_or(config.sample_size, |expressions| expressions.len());
    if population_size < 2 {
        return Err(PyValueError::new_err(
            "The initial population must contain at least 2 expressions",
        ));
    }

    // Only owned Rust configuration/results cross this closure. No Python
    // objects or callbacks are accessed while the GIL is released.
    let result: Result<Vec<PyGenerationResult>, String> = py.allow_threads(move || {
        let mut soup = RustSoup::from_config(&config);
        match initial_expressions {
            Some(expressions) => soup
                .add_expressions(expressions)
                .map_err(|error| error.to_string())?,
            None => soup.seed_with_generator(config.sample_size),
        }

        let tapes = crate::recursive_experiment::simulate_recursive_experiment_recorded(
            &mut soup, config,
        )
        .map_err(|error| error.to_string())?;

        Ok(tapes
            .into_iter()
            .enumerate()
            .map(|(index, generation)| {
                let tape = generation.tape;
                PyGenerationResult {
                    generation: index + 1,
                    history_collisions: generation.history_collisions,
                    initial_expressions: generation
                        .initial_population
                        .into_iter()
                        .map(|particle| particle.get_underlying_term().to_string())
                        .collect(),
                    final_expressions: generation
                        .final_population
                        .into_iter()
                        .map(|particle| particle.get_underlying_term().to_string())
                        .collect(),
                    history: tape
                        .soup_history
                        .into_iter()
                        .map(|snapshot| {
                            snapshot
                                .lambda_expressions()
                                .map(|term| term.to_string())
                                .collect()
                        })
                        .collect(),
                    reactions: tape
                        .reaction_record
                        .into_iter()
                        .map(|reaction| PyReactionRecord {
                            step: reaction.step,
                            left: reaction.left.to_string(),
                            right: reaction.right.to_string(),
                            products: reaction.products.iter().map(|p| p.to_string()).collect(),
                            success: reaction.success,
                            error: reaction.error,
                        })
                        .collect(),
                }
            })
            .collect())
    });
    result.map_err(PyValueError::new_err)
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
    m.add_class::<PyConfig>()?;
    m.add_class::<PyReactor>()?;
    m.add_class::<PyReactionError>()?;
    m.add_class::<PyReactionRecord>()?;
    m.add_class::<PyGenerationResult>()?;
    m.add_class::<PyStandardization>()?;
    m.add_class::<PyBTreeGen>()?;
    m.add_class::<PyFontanaGen>()?;
    m.add_function(wrap_pyfunction!(decode_hex_py, m)?)?;
    m.add_function(wrap_pyfunction!(encode_hex_py, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_recursive_experiment, m)?)?;
    Ok(())
}
