// Global Imports
use std::marker::PhantomData;
use lambda_calculus::Term;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Package Imports
use crate::config::config::Config;
use crate::soupercollider::Soup;
use crate::lambda::{
    particle::LambdaParticle, 
    collider::AlchemyCollider, 
    generator::LambdaGenerator,
    result::{LambdaCollisionOk, LambdaCollisionError}
};

pub type LambdaSoup =
    Soup<LambdaParticle, AlchemyCollider, LambdaGenerator, LambdaCollisionOk, LambdaCollisionError>;

impl LambdaSoup {
    /// Generate an empty soup with the following configuration options:
    pub fn new() -> Self {
        LambdaSoup::from_config(&Config::new())
    }

    /// Generate an empty soup from a given `config` object.
    pub fn from_config(cfg: &Config) -> Self {
        let seed = cfg.reactor_config.seed.get();
        let rng = ChaCha8Rng::from_seed(seed);
        Self {
            expressions: Vec::new(),
            collider: AlchemyCollider::from_config(&cfg.reactor_config),
            generator: LambdaGenerator::from_config(&cfg.generator_config),
            maintain_constant_population_size: cfg.reactor_config.maintain_constant_population_size,
            discard_parents: cfg.reactor_config.discard_parents,
            rng,
            n_collisions: 0,
            t: PhantomData,
            e: PhantomData,
        }
    }

    pub fn add_lambda_expressions(&mut self, expressions: impl IntoIterator<Item = Term>) {
        self.expressions
            .extend(expressions.into_iter().map(|t| LambdaParticle {
                expr: t,
                recursive: false,
            }))
    }

    pub fn perturb_lambda_expressions<I>(&mut self, nterms: usize, expressions: I)
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
        self.add_lambda_expressions(expressions.into_iter().cycle().take(nterms))
    }

    pub fn add_test_expressions(&mut self, expressions: impl IntoIterator<Item = Term>) {
        self.expressions
            .extend(expressions.into_iter().map(|t| LambdaParticle {
                expr: t,
                recursive: true,
            }))
    }

    pub fn perturb_test_expressions<I>(&mut self, nterms: usize, expressions: I)
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
        self.add_test_expressions(expressions.into_iter().cycle().take(nterms))
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
