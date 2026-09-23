// Global Imports
use std::fmt::Debug;
use lambda_calculus::Term;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Package Imports
use crate::config::generators::fontana_gen::FontanaGen as FontanaGenConfig;
use crate::traits::Generator;
use crate::lambda::particle::LambdaParticle;

#[derive(Debug, Clone)]
pub struct FontanaGen {
    min_depth: u32,
    max_depth: u32,
    abs_prob: (f32, f32),
    app_prob: (f32, f32),
    abs_incr: f32,
    app_incr: f32,
    seed: [u8; 32],
    rng: ChaCha8Rng,
}

impl Generator<LambdaParticle> for FontanaGen {
    fn generate_n_particles(&mut self, n: usize) -> Vec<LambdaParticle> {
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            let expr = self.generate();
            let particle = LambdaParticle { expr, recursive: false };
            v.push(particle);
        }
        v
    }
}

impl FontanaGen {
    pub fn new(
        min_depth: u32,
        mut max_depth: u32,
        mut abs_prob: (f32, f32),
        mut app_prob: (f32, f32),
        seed: [u8; 32],
    ) -> FontanaGen {
        max_depth = max_depth.max(1);
        abs_prob.0 = abs_prob.0.clamp(0.0, 1.0);
        abs_prob.1 = abs_prob.1.clamp(0.0, 1.0);
        app_prob.0 = app_prob.0.clamp(0.0, 1.0);
        app_prob.1 = app_prob.1.clamp(0.0, 1.0);

        let steps = (max_depth - 1).max(1);
        let abs_incr = (abs_prob.1 - abs_prob.0) / (steps as f32);
        let app_incr = (app_prob.1 - app_prob.0) / (steps as f32);

        FontanaGen {
            min_depth: min_depth.min(max_depth.saturating_sub(1)),
            max_depth,
            abs_prob,
            app_prob,
            abs_incr,
            app_incr,
            seed,
            rng: ChaCha8Rng::from_seed(seed),
        }
    }

    pub fn from_config(cfg: &FontanaGenConfig) -> FontanaGen {
        let seed = cfg.seed.get();

        FontanaGen::new(
            cfg.min_depth,
            cfg.max_depth,
            (
                cfg.abstraction_prob_range.0 as f32,
                cfg.abstraction_prob_range.1 as f32,
            ),
            (
                cfg.application_prob_range.0 as f32,
                cfg.application_prob_range.1 as f32,
            ),
            seed,
        )
    }

    pub fn generate(&mut self) -> Term {
        self.rand_lambda(0, 0, self.abs_prob.0, self.app_prob.0)
    }

    pub fn generate_n(&mut self, n: usize) -> Vec<Term> {
        (0..n).map(|_| self.generate()).collect()
    }

    pub fn generate_n_unique(&mut self, n: usize) -> Vec<Term> {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut results: Vec<Term> = Vec::with_capacity(n);

        let max_attempts = n * 20;
        let mut attempts = 0;

        while results.len() < n && attempts < max_attempts {
            let term = self.generate();
            let key = term.to_string();
            if seen.insert(key) {
                results.push(term);
            }
            attempts += 1;
        }

        results
    }

    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    pub fn rand_lambda(&mut self, depth: u32, abs_depth: u32, p_abs: f32, p_app: f32) -> Term {
        // Terminal case: max depth reached
        if depth >= self.max_depth {
            if abs_depth > 0 {
                return self.sample_variable(abs_depth);
            }
            return Term::Abs(Box::new(Term::Var(1)));
        }

        let next_abs = p_abs + self.abs_incr;
        let next_app = p_app + self.app_incr;

        let (p_abs_eff, p_app_eff) = Self::clamp_probabilities(p_abs, p_app);
        let coin: f32 = self.rng.gen();

        if coin <= p_abs_eff {
            return Term::Abs(Box::new(self.rand_lambda(
                depth + 1,
                abs_depth + 1,
                next_abs,
                next_app,
            )));
        }

        // Below min_depth or no binders: can't emit Var, must pick Abs or App
        if abs_depth == 0 || depth < self.min_depth {
            return Term::App(Box::new((
                self.rand_lambda(depth + 1, abs_depth, next_abs, next_app),
                self.rand_lambda(depth + 1, abs_depth, next_abs, next_app),
            )));
        }

        if coin <= p_abs_eff + p_app_eff {
            return Term::App(Box::new((
                self.rand_lambda(depth + 1, abs_depth, next_abs, next_app),
                self.rand_lambda(depth + 1, abs_depth, next_abs, next_app),
            )));
        }

        self.sample_variable(abs_depth)
    }

    fn sample_variable(&mut self, abs_depth: u32) -> Term {
        debug_assert!(
            abs_depth > 0,
            "sample_variable called with no enclosing abstractions"
        );
        Term::Var(self.rng.gen_range(1..=abs_depth) as usize)
    }

    fn clamp_probabilities(p_abs: f32, p_app: f32) -> (f32, f32) {
        let abs = p_abs.clamp(0.0, 1.0);
        let remaining = 1.0 - abs;
        let app = p_app.clamp(0.0, remaining);
        (abs, app)
    }
}
