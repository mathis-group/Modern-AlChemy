// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::config_seed::ConfigSeed;
use crate::config::generator::GenConfig;

#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
/// Generator-specific configuration derived from Walter Fontana's original scheme.
pub struct FontanaGen {
    /// The seed for the lambda expression generator. If set to `None`, then a seed is chosen
    /// randomly. Default: `None`
    pub seed: ConfigSeed,

    /// Minimum depth before leaf nodes can be generated
    pub min_depth: u32,

    /// Maximum depth of the generated trees
    pub max_depth: u32,

    /// Probability range of an abstraction being generated. Linearly changes from start to end,
    /// varying with depth
    pub abstraction_prob_range: (f64, f64),

    /// Probability range of an application being generated. Linearly changes from start to end,
    /// varying with depth
    pub application_prob_range: (f64, f64),
}

impl GenConfig for FontanaGen {
    fn new() -> Self {
        FontanaGen {
            seed: ConfigSeed(None),
            min_depth: 0,
            max_depth: 10,
            application_prob_range: (0.5, 0.3),
            abstraction_prob_range: (0.3, 0.5),
        }
    }
}