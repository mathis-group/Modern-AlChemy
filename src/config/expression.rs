// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::expressions::untyped_lambda::UntypedLambdaExpression;

/// The expression type a simulation runs on, carrying the configuration that is
/// only meaningful for that type.
///
/// The variant tag *is* the expression type, so there is exactly one place in a
/// config file that names it, and its generator and reactor cannot be paired
/// with the wrong expression type.
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Expression {
    /// Untyped lambda calculus
    UntypedLambda(UntypedLambdaExpression),
}

impl Expression {
    /// Produce a new `Expression` with default values.
    pub fn new() -> Self {
        Self::UntypedLambda(UntypedLambdaExpression::new())
    }
}

impl Default for Expression {
    fn default() -> Self {
        Expression::new()
    }
}
