// Global Imports
use lambda_calculus::Term;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Package Imports
use crate::config::config::Config;
use crate::config::reactor::Reactor;
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
        // `main` has already dispatched on `expression_type` to reach a lambda
        // soup, so the reactor config must be the untyped lambda variant. This
        // binding is irrefutable while `Reactor` has one variant; adding another
        // makes it a compile error, which is where the decision about a mismatch
        // between `expression_type` and `reactor_config` belongs.
        let Reactor::UntypedLambda(reactor_cfg) = &cfg.reactor_config;

        let seed = cfg.soup_config.seed.get();
        let rng = ChaCha8Rng::from_seed(seed);
        Self {
            expressions: Vec::new(),
            collider: LambdaCollider::from_config(reactor_cfg),
            generator: LambdaGenerator::from_config(&cfg.generator_config),
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
