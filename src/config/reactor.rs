// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::config_seed::ConfigSeed;

/// Configuration for the reactor
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Reactor {
    /// Set of reaction rules. Each rule must always be a lambda expressions
    /// with two arguments. Default: `["\x.\y.x y"]`.
    pub rules: Vec<String>,

    /// When set, remove all results that are structurally isomorphic to parents.
    /// Default: `true`.
    pub discard_copy_actions: bool,

    /// When set, remove all results that are structurally isomorphic to the identity function:
    /// `\x.x`. Default: `true`.
    pub discard_identity: bool,

    /// When set, remove all expressions that contain free variables. Default: `true`.
    pub discard_free_variable_expressions: bool,

    /// When set, remove the parents from the soup instead of returning them. Default: `true`.
    pub discard_parents: bool,

    /// When set maintain a constant population size after each reaction. If there are more
    /// elements than the population originally started with, then remove elements randomly from
    /// the soup until the original population remains. If there are fewer elements after a
    /// reaction, then do nothing. This behavior may change. Default: `true`.
    pub maintain_constant_population_size: bool,

    ///  The number of reductions allowed before AlChemy gives up and fails the reaction. Default:
    ///  `500`.
    pub reduction_cutoff: usize,

    /// The largest size of any expression during a reduction step. Defaults to `1024`.
    pub size_cutoff: usize,

    /// The seed for the reactor. If set to `None`, then a seed is chosen
    /// randomly. Default: `None`
    pub seed: ConfigSeed,
}

impl Reactor {
    /// Produce a new `ReactorConfig` struct with default values.
    pub fn new() -> Self {
        Reactor {
            rules: vec![String::from("\\x.\\y.x y")],

            discard_copy_actions: true,
            discard_identity: true,
            discard_free_variable_expressions: true,
            maintain_constant_population_size: true,
            discard_parents: false,
            reduction_cutoff: 500,
            size_cutoff: 500,
            seed: ConfigSeed(None),
        }
    }
}

// TODO: Eventually, all config objects will use `default` instead of `new`. For now, this just
// fixes a clippy lint
impl Default for Reactor {
    fn default() -> Self {
        Reactor::new()
    }
}