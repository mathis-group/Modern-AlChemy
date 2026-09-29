// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::reactor::ReactorConfig;

/// Configuration for the reactor
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct UntypedLambdaReactor {
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

    ///  The number of reductions allowed before AlChemy gives up and fails the reaction. Default:
    ///  `500`.
    pub reduction_cutoff: usize,

    /// The largest size of any expression during a reduction step. Default: `500`.
    pub size_cutoff: usize,
}

impl ReactorConfig for UntypedLambdaReactor {
    /// Produce a new `ReactorConfig` struct with default values.
    fn new() -> Self {
        UntypedLambdaReactor {
            rules: vec![String::from("\\x.\\y.x y")],
            discard_copy_actions: true,
            discard_identity: true,
            discard_free_variable_expressions: true,
            reduction_cutoff: 500,
            size_cutoff: 500,
        }
    }
}

// TODO: Eventually, all config objects will use `default` instead of `new`. For now, this just
// fixes a clippy lint
impl Default for UntypedLambdaReactor {
    fn default() -> Self {
        UntypedLambdaReactor::new()
    }
}
