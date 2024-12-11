use core::fmt;
use std::fmt::{Debug, Display};

use crate::collidable::{Collider, Particle};
use lambda_calculus::{abs, app, Term, Var};

pub struct LambdaParticle {
    expr: Term,
    recursive: bool,
}

pub struct AlchemyCollider {
    rlimit: usize,
    slimit: usize,
    disallow_recursive: bool,
    reaction_rules: Vec<Term>,
    discard_copy_actions: bool,
    discard_identity: bool,
    discard_free_variable_expressions: bool,
}

/// The result of composing a vector `v` of 2-ary lambda expressions with
/// the expressions A and B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaCollisionOk {
    pub results: Vec<Term>,
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
}

impl LambdaParticle {
    pub fn get_underlying_term(&self) -> &Term {
        &self.expr
    }
}

impl AlchemyCollider {
    pub fn reduce_with_limit(&self, expr: &mut Term) -> Result<usize, LambdaCollisionError> {
        let mut n = 0;
        for _ in 0..self.rlimit {
            if expr.reduce(lambda_calculus::HAP, 1) == 0 {
                break;
            }

            // WARNING: This is EXTREMELY expensive. Calling max_depth is log(depth), and is done
            // per reduction step. Remove when possible.
            let depth = expr.size();
            if depth > self.slimit {
                return Err(LambdaCollisionError::ExceedsDepthLimit);
            }
            n += 1;
        }
        Ok(n)
    }
}

impl Particle for LambdaParticle {
    fn compose(&self, other: &Self) -> Self {
        LambdaParticle {
            expr: lambda_calculus::app!(self.expr.clone(), other.expr.clone()),
            recursive: false,
        }
    }

    fn is_isomorphic_to(&self, other: &Self) -> bool {
        self.expr.is_isomorphic_to(&other.expr)
    }
}

impl Collider<LambdaParticle, LambdaCollisionOk, LambdaCollisionError> for AlchemyCollider {
    /// Return the result of ((`rule` `left`) `right`), up to a limit of
    /// `self.reduction_limit`.
    fn collide(
        &self,
        left: LambdaParticle,
        right: LambdaParticle,
    ) -> Result<LambdaCollisionOk, LambdaCollisionError> {
        if right.recursive {
            return Err(LambdaCollisionError::RecursiveArgument);
        }

        let mut lt = left.expr.clone();
        let rt = right.expr;

        if left.recursive {
            if self.disallow_recursive {
                return Err(LambdaCollisionError::RecursiveArgument);
            }
            lt = left.expr.unabs().unwrap();
        }

        // Record collision information
        let mut collision_results = Vec::with_capacity(self.reaction_rules.len());

        for rule in &self.reaction_rules {
            let mut expr = app!(rule.clone(), lt.clone(), rt.clone());
            let size = expr.size();
            let n = self.reduce_with_limit(&mut expr)?;

            if n == self.rlimit {
                return Err(LambdaCollisionError::ExceedsReductionLimit);
            }

            let identity = abs(Var(1));
            if expr.is_isomorphic_to(&identity) && self.discard_identity {
                return Err(LambdaCollisionError::IsIdentity);
            }

            let is_copy_action = expr.is_isomorphic_to(&lt) || expr.is_isomorphic_to(&rt);
            if is_copy_action && self.discard_copy_actions {
                return Err(LambdaCollisionError::IsParent);
            }

            if expr.has_free_variables() && self.discard_free_variable_expressions {
                return Err(LambdaCollisionError::HasFreeVariables);
            }

            collision_results.push((expr, size, n))
        }
        Ok(LambdaCollisionOk {
            results: collision_results.iter().map(|t| t.0.clone()).collect(),
            reductions: collision_results.iter().map(|t| t.1).collect(),
            sizes: collision_results.iter().map(|t| t.2).collect(),
            left_size: lt.size(),
            right_size: rt.size(),
        })
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
        }
    }
}

impl std::error::Error for LambdaCollisionError {}

impl fmt::Display for LambdaParticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{:?}", self.expr), f)
    }
}
