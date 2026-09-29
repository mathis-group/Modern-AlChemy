// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::config_seed::ConfigSeed;

/// Configuration for the soup itself. These options govern the population and
/// its dilution flux, and are independent of the expression type being reacted
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Soup {
    /// When set, remove the parents from the soup instead of returning them.
    /// Default: `false`.
    pub discard_parents: bool,

    /// When set maintain a constant population size after each reaction. If there are more
    /// elements than the population originally started with, then remove elements randomly from
    /// the soup until the original population remains. If there are fewer elements after a
    /// reaction, then do nothing. This behavior may change. Default: `true`.
    pub maintain_constant_population_size: bool,

    /// The seed for the soup's reaction RNG. Generators carry their own seed
    /// separately. If set to `None`, then a seed is chosen randomly. Default: `None`
    pub seed: ConfigSeed,
}

impl Soup {
    /// Produce a new `Soup` config struct with default values.
    pub fn new() -> Self {
        Soup {
            discard_parents: false,
            maintain_constant_population_size: true,
            seed: ConfigSeed(None),
        }
    }
}

// TODO: Eventually, all config objects will use `default` instead of `new`. For now, this just
// fixes a clippy lint
impl Default for Soup {
    fn default() -> Self {
        Soup::new()
    }
}
