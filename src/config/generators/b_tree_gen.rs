// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::config_seed::ConfigSeed;
use crate::config::generator::GenConfig;
use crate::enums::Standardization;

/// Configuration for the BTree generator
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct BTreeGen {
    /// The seed for the lambda expression generator. If set to `None`, then a seed is chosen
    /// randomly. Default: `None`
    pub seed: ConfigSeed,

    /// Number of nodes in the binary tree
    pub size: u32,

    /// Probability that a leaf vertex is a free variable
    pub freevar_generation_probability: f64,

    /// Size of the variable palette
    pub n_max_free_vars: u32,

    /// Standardization scheme. Defaults to prefix standardization (this is different from the
    /// paper!)
    pub standardization: Standardization,
}

impl GenConfig for BTreeGen {
    /// Produce a new `BTreeGenConfig` struct with default values.
    fn new() -> Self {
        BTreeGen {
            size: 20,
            freevar_generation_probability: 0.2,
            standardization: Standardization::Prefix,
            n_max_free_vars: 6,
            seed: ConfigSeed(None),
        }
    }
}