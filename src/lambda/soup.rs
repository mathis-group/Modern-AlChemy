// Global Imports
use lambda_calculus::Term;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Package Imports
use crate::config::config::Config;
use crate::config::expression::Expression;
use crate::soupercollider::Soup;
use crate::lambda::{
    particle::LambdaParticle, 
    collider::LambdaCollider, 
    generator::LambdaGenerator
};

pub type LambdaSoup = Soup<LambdaParticle, LambdaCollider, LambdaGenerator>;

impl LambdaSoup {
    /// Generate an empty soup with the following configuration options:
    pub fn new() -> Self {
        LambdaSoup::from_config(&Config::new())
    }

    /// Generate an empty soup from a given `config` object.
    pub fn from_config(cfg: &Config) -> Self {
        let Expression::UntypedLambda(expr_cfg) = &cfg.expression;

        let seed = cfg.soup_config.seed.get();
        let rng = ChaCha8Rng::from_seed(seed);
        Self {
            expressions: Vec::new(),
            collider: LambdaCollider::from_config(&expr_cfg.reactor),
            generator: LambdaGenerator::from_config(&expr_cfg.generator),
            maintain_constant_population_size: cfg.soup_config.maintain_constant_population_size,
            discard_parents: cfg.soup_config.discard_parents,
            rng,
            n_collisions: 0,
        }
    }

    pub fn perturb_lambda_expressions<I>(&mut self, nterms: usize, expressions: I, is_test: bool)
    where
        I: IntoIterator<Item = Term>,
        <I as IntoIterator>::IntoIter: Clone,
    {
        if self.maintain_constant_population_size {
            for _ in 0..nterms {
                let k = self.rng.gen_range(0..self.expressions.len());
                self.expressions.swap_remove(k);
            }
        }
        self.add_lambda_expressions(expressions.into_iter().cycle().take(nterms), is_test)
    }

    pub fn add_lambda_expressions(&mut self, expressions: impl IntoIterator<Item = Term>, is_test: bool) {
        self.expressions
            .extend(expressions.into_iter().map(|t| LambdaParticle {
                expr: t,
                recursive: is_test,
            }))
    }

    pub fn lambda_expressions(&self) -> impl Iterator<Item = &Term> {
        self.expressions.iter().map(|e| e.get_underlying_term())
    }

    pub fn population_of(&self, item: &Term) -> usize {
        self.lambda_expressions()
            .filter(|p| p.is_isomorphic_to(item))
            .count()
    }
}
