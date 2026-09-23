// Global Imports
use lambda_calculus::{abs, app, Term, Var};

// Package Imports
use crate::config::reactor::Reactor;
use crate::traits::Collider;
use crate::lambda::particle::LambdaParticle;
use crate::lambda::result::{LambdaCollisionOk, LambdaCollisionError};
use crate::lambda::utils::{has_two_args, uses_both_arguments, is_truthy, reduce_with_limit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlchemyCollider {
    rlimit: usize,
    slimit: usize,
    disallow_recursive: bool,
    reaction_rules: Vec<Term>,
    discard_copy_actions: bool,
    discard_identity: bool,
    discard_free_variable_expressions: bool,
}

impl AlchemyCollider {
    pub fn from_config(cfg: &Reactor) -> Self {
        Self {
            rlimit: cfg.reduction_cutoff,
            slimit: cfg.size_cutoff,
            disallow_recursive: false,
            reaction_rules: cfg
                .rules
                .iter()
                .map(|r| lambda_calculus::parse(r, lambda_calculus::Classic).unwrap())
                .collect(),
            discard_copy_actions: cfg.discard_copy_actions,
            discard_identity: cfg.discard_identity,
            discard_free_variable_expressions: cfg.discard_free_variable_expressions,
        }
    }

    fn recursive_collide(
        &self,
        left: LambdaParticle,
        right: LambdaParticle,
    ) -> Result<LambdaCollisionOk, LambdaCollisionError> {
        assert!(left.recursive);
        let has_good_signature = uses_both_arguments(&right.expr) && has_two_args(&right.expr);
        if is_truthy(&right.expr) || !has_good_signature {
            return Err(LambdaCollisionError::BadArgument);
        }
        let lt = left.expr.clone();
        let left_size = lt.size();
        let rt = right.expr.clone();
        let right_size = rt.size();

        let mut expr = app!(lt, rt.clone());
        let n = reduce_with_limit(&mut expr, 32000, 16000)?;

        if expr.is_isomorphic_to(&lambda_calculus::data::boolean::tru()) {
            Ok(LambdaCollisionOk {
                results: vec![right.clone(); 100],
                reductions: vec![n],
                sizes: vec![expr.size()],
                left_size,
                right_size,
            })
        } else {
            Ok(LambdaCollisionOk {
                results: vec![left],
                reductions: vec![n],
                sizes: vec![expr.size()],
                left_size,
                right_size,
            })
        }
    }

    fn nonrecursive_collide(
        &self,
        left: LambdaParticle,
        right: LambdaParticle,
    ) -> Result<LambdaCollisionOk, LambdaCollisionError> {
        assert!(!left.recursive);
        let lt = left.expr;
        let rt = right.expr;
        if right.recursive {
            return Err(LambdaCollisionError::RecursiveArgument);
        }
        let mut collision_results = Vec::with_capacity(self.reaction_rules.len());

        for rule in &self.reaction_rules {
            let mut expr = app!(rule.clone(), lt.clone(), rt.clone());
            let n = reduce_with_limit(&mut expr, self.rlimit, self.slimit)?;
            let size = expr.size();

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

            let expr = LambdaParticle {
                expr,
                recursive: false,
            };

            collision_results.push((expr, n, size))
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

impl Collider<LambdaParticle> for AlchemyCollider {

    type Product = LambdaCollisionOk;
    type Error   = LambdaCollisionError;

    /// Return the result of ((`rule` `left`) `right`), up to a limit of
    /// `self.reduction_limit`.
    fn collide(
        &self,
        left: LambdaParticle,
        right: LambdaParticle,
    ) -> Result<LambdaCollisionOk, LambdaCollisionError> {
        if left.recursive {
            self.recursive_collide(left, right)
        } else {
            self.nonrecursive_collide(left, right)
        }
    }
}