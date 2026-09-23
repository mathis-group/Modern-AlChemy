
// Global Imports
use core::fmt;
use std::fmt::{Debug, Display};

// Package Imports
use crate::traits::Residue;
use crate::lambda::particle::LambdaParticle;

/// The result of composing a vector `v` of 2-ary lambda expressions with
/// the expressions A and B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaCollisionOk {
    pub results: Vec<LambdaParticle>,
    pub reductions: Vec<usize>,
    pub sizes: Vec<usize>,

    /// Size of A
    pub left_size: usize,

    /// Size of B
    pub right_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LambdaCollisionError {
    ExceedsReductionLimit,
    NotEnoughExpressions,
    IsIdentity,
    IsParent,
    HasFreeVariables,
    ExceedsDepthLimit,
    RecursiveArgument,
    BadArgument,
}

impl Residue<LambdaParticle> for LambdaCollisionOk {
    fn particles(&self) -> impl Iterator<Item = LambdaParticle> {
        self.results.iter().cloned()
    }

    fn count(&self) -> usize {
        self.results.len()
    }
}

impl fmt::Display for LambdaCollisionOk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt("no message", f)
    }
}

impl fmt::Display for LambdaCollisionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LambdaCollisionError::IsIdentity => {
                Display::fmt("collision result is identity function", f)
            }
            LambdaCollisionError::IsParent => {
                Display::fmt("collision result is isomorphic to parent", f)
            }
            LambdaCollisionError::ExceedsReductionLimit => {
                Display::fmt("collision exceeds reduction limit", f)
            }
            LambdaCollisionError::NotEnoughExpressions => {
                Display::fmt("not enough expressions for further reactions", f)
            }
            LambdaCollisionError::HasFreeVariables => {
                Display::fmt("collision result has free variables", f)
            }

            LambdaCollisionError::ExceedsDepthLimit => {
                Display::fmt("expression exceeds depth limit during reduction", f)
            }
            LambdaCollisionError::RecursiveArgument => Display::fmt("argument is recursive", f),
            LambdaCollisionError::BadArgument => Display::fmt(
                "argument is truth-like or doesn't use all of own arguments",
                f,
            ),
        }
    }
}

impl std::error::Error for LambdaCollisionError {}