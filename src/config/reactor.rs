// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::reactors::untyped_lambda::UntypedLambdaReactor;

/// Default struct for the GenConfig trait,
/// implemented by all generators
pub trait ReactorConfig {
    fn new() -> Self;
}

/// Configuration for the generators
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Reactor {
    /// Use the untyped lambda reactor
    UntypedLambda(UntypedLambdaReactor)
}

impl Reactor {
    /// Produce a new `Reactor` struct with default values.
    pub fn new() -> Self {
        Self::UntypedLambda(UntypedLambdaReactor::new())
    }
}