// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::generator::Generator;
use crate::config::reactors::untyped_lambda::UntypedLambdaReactor;

/// Configuration specific to untyped lambda calculus.
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct UntypedLambdaExpression {
    /// How the initial population of lambda terms is generated.
    pub generator: Generator,

    /// Reaction chemistry for untyped lambda terms.
    pub reactor: UntypedLambdaReactor,
}

impl UntypedLambdaExpression {
    /// Produce a new `UntypedLambdaExpression` with default values.
    pub fn new() -> Self {
        UntypedLambdaExpression {
            generator: Generator::new(),
            reactor: UntypedLambdaReactor::default(),
        }
    }
}

impl Default for UntypedLambdaExpression {
    fn default() -> Self {
        UntypedLambdaExpression::new()
    }
}
