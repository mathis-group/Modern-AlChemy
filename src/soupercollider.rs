// Global Imports
use std::fmt::{Debug, Display};
use std::iter::repeat_n;

use rand::Rng;
use rand_chacha::ChaCha8Rng;

//Package Imports
use crate::{config::recursive::{Recursive, RefillType}, traits::{Collider, Generator, Particle, Residue}};
use crate::logging::{ReactionRecord, Tape};
use crate::errors::ParsingError;

/// The principal AlChemy object. The `Soup` struct contains a set of
/// lambda expressions, and rules for composing and filtering them.
#[derive(Debug, Clone)]
pub struct Soup<P, C, G> {
    // All of these pub(crate)s here are hacky
    pub expressions: Vec<P>,
    pub n_collisions: usize,
    pub collider: C,
    pub generator: G,

    pub maintain_constant_population_size: bool,
    pub discard_parents: bool,

    pub rng: ChaCha8Rng,
}

impl<P, C, G> Soup<P, C, G>
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone
{
    /// Introduce all expressions in `expressions` into the soup, without
    /// reduction.
    pub fn perturb(&mut self, expressions: impl IntoIterator<Item = P>) {
        self.expressions.extend(expressions)
    }

    /// Produce one atomic reaction on the soup.
    pub fn react(&mut self) -> Result<C::Product, C::Error> {
        let n_expr = self.expressions.len();

        // Remove two distinct expressions randomly from the soup
        let i = self.rng.gen_range(0..n_expr);
        let left = self.expressions.swap_remove(i);

        let j = self.rng.gen_range(0..n_expr - 1);
        let right = self.expressions.swap_remove(j);

        // Add collision results to soup
        let result = self.collider.collide(left.clone(), right.clone());

        if let Ok(ref t) = result {
            self.perturb(t.particles());

            // Remove additional expressions, if required.
            if self.maintain_constant_population_size {
                for _ in 0..t.count() {
                    let k = self.rng.gen_range(0..self.expressions.len());
                    self.expressions.swap_remove(k);
                }
            }
        }

        // Add removed parents back into the soup, if necessary
        if !self.discard_parents {
            self.expressions.push(left);
            self.expressions.push(right);
        }

        result.clone()
    }

    /// Produce one atomic reaction and return a full record of who
    /// reacted with whom to produce what.
    pub fn react_logged(&mut self, step: usize) -> ReactionRecord<P> {
        let n_expr = self.expressions.len();

        let i = self.rng.gen_range(0..n_expr);
        let left = self.expressions.swap_remove(i);

        let j = self.rng.gen_range(0..n_expr - 1);
        let right = self.expressions.swap_remove(j);

        let result = self.collider.collide(left.clone(), right.clone());

        let record = match &result {
            Ok(ref t) => {
                let products: Vec<P> = t.particles().collect();
                self.perturb(products.iter().cloned());

                if self.maintain_constant_population_size {
                    for _ in 0..t.count() {
                        let k = self.rng.gen_range(0..self.expressions.len());
                        self.expressions.swap_remove(k);
                    }
                }

                ReactionRecord {
                    step,
                    left: left.clone(),
                    right: right.clone(),
                    products,
                    success: true,
                    error: None,
                }
            }
            Err(ref e) => ReactionRecord {
                step,
                left: left.clone(),
                right: right.clone(),
                products: vec![],
                success: false,
                error: Some(format!("{}", e)),
            },
        };

        if !self.discard_parents {
            self.expressions.push(left);
            self.expressions.push(right);
        }

        record
    }

    /// Simulate for `n` collisions, returning a full reaction log.
    /// Note: this allocates a Vec of n records. For very large n,
    /// consider using `simulate_for_logged_filtered` instead.
    pub fn simulate_for_logged(&mut self, n: usize) -> Vec<ReactionRecord<P>> {
        let mut log = Vec::with_capacity(n);
        for step in 0..n {
            log.push(self.react_logged(step));
        }
        log
    }

    /// Simulate for `n` collisions, returning only successful reaction records.
    pub fn simulate_for_logged_filtered(&mut self, n: usize) -> Vec<ReactionRecord<P>> {
        let mut log = Vec::new();
        for step in 0..n {
            let record = self.react_logged(step);
            if record.success {
                log.push(record);
            }
        }
        log
    }

    fn log_message_from_reaction(reaction: &Result<C::Product, C::Error>) -> String {
        match reaction {
            Ok(result) => format!("successful with {}", result),
            Err(message) => format!("failed because {}", message),
        }
    }

    /// Simulate the soup for `n` collisions. If `log` is set, then print
    /// out a log message for each reaction. Returns the number of successful reactions
    /// (the fraction of failed reactions).
    pub fn simulate_for(&mut self, n: usize, log: bool) -> usize {
        let mut n_successes = 0;
        for i in 0..n {
            let reaction = self.react();
            if reaction.is_ok() {
                n_successes += 1;
            }

            if log {
                let message = Self::log_message_from_reaction(&reaction);
                println!("reaction {:?} {}", i, message)
            }
        }
        n_successes
    }

    pub fn simulate_and_poll<F, R>(
        &mut self,
        n: usize,
        polling_interval: usize,
        log: bool,
        poller: F,
    ) -> Vec<R>
    where
        F: Fn(&Self) -> R,
    {
        let mut data: Vec<R> = Vec::new();
        for i in 0..n {
            let reaction = self.react();
            if (i % polling_interval) == 0 {
                data.push(poller(self))
            }
            if log {
                let message = Self::log_message_from_reaction(&reaction);
                println!("reaction {:?} {}", i, message)
            }
        }
        data
    }

    pub fn simulate_and_poll_with_killer<F, R>(
        &mut self,
        n: usize,
        polling_interval: usize,
        log: bool,
        killpoller: F,
    ) -> Vec<R>
    where
        F: Fn(&Self) -> (R, bool),
    {
        let mut data: Vec<R> = Vec::new();
        for i in 0..n {
            let reaction = self.react();
            if (i % polling_interval) == 0 {
                let (datum, should_kill) = killpoller(self);
                data.push(datum);
                if should_kill {
                    return data;
                };
            }
            if log {
                let message = Self::log_message_from_reaction(&reaction);
                println!("reaction {:?} {}", i, message)
            }
        }
        data
    }

    /// Simulate the soup for `n` collisions, recording the state of the soup every
    /// `polling_interval` reactions. If `log` is set, then print out a log message for each
    /// reaction
    pub fn simulate_and_record(
        &mut self,
        n: usize,
        polling_interval: usize,
        log: bool,
    ) -> Tape<P, C, G> {
        let mut history: Vec<Self> = Vec::new();
        for i in 0..n {
            let reaction = self.react();
            if (i % polling_interval) == 0 {
                history.push(self.clone())
            }
            if log {
                let message = Self::log_message_from_reaction(&reaction);
                println!("reaction {:?} {}", i, message)
            }
        }

        Tape::<P, C, G> {
            soup: self.clone(),
            history,
            polling_interval,
        }
    }

    /// Cull `wipeout_percent` of the population (rounded down) and replace with
    /// `ConfigGenerator` generated or `custom_expression` expressions
    // TODO: Move this to another file for recursive_experiments. Should not be a member of the soup.
    pub fn wipeout(&mut self, wipeout_percent: usize) -> usize {
        // Get the number of expressions we currently have
        let n_expr = self.expressions.len();

        // Calculate how many expressions will be culled based on the wipeout percent
        // TODO: Gotta be a better way to do this type conversion/casting/floor thing
        let n_wipeout = (self.expressions.len() as f64 * ((wipeout_percent as f64) / 100.0)) as usize;
        
        // Cull one expression for each wipeout we have
        for execution_count in 0..n_wipeout {
            let cull_index = self.rng.gen_range(0..n_expr - execution_count);
            let removed_expression = self.expressions.swap_remove(cull_index);
            println!("Removed Expression: {removed_expression}")
        }
        n_wipeout
    }

    /// Repopulate the soup based with n new particles either from the configured generator
    /// or the provided `repopulation_expression`
    // TODO: Move this to another file for recursive_experiments. Should not be a member of the soup.
    pub fn repopulate(&mut self, n: usize, recursive_config: &Recursive) -> Result<(), ParsingError> {
        match recursive_config.refill_type {
            // Refill with the configured generator
            RefillType::ConfigGenerator => self.seed_with_generator(n),
            // Refill with the custom expression provided
            RefillType::CustomExpression => {
                let particle = P::parse(&recursive_config.repopulation_expression)?;
                self.perturb(repeat_n(particle, n));
            }
        }
        Ok(())
    }

    /// Generate and add n particles to an existing soup
    pub fn seed_with_generator(&mut self, n: usize) {
        let particles = self.generator.generate_n_particles(n);
        self.perturb(particles);
    }

    /// Adds a list of String expressions to the soup, generic to Particle type
    /// Can be used by any expression type which implements the Particle trait
   pub fn add_expressions(&mut self, sources: Vec<String>) -> Result<(), ParsingError> {
        for s in sources {
            self.expressions.push(P::parse(&s)?);
        }
        Ok(())
    }

    /// Print out all expressions within the soup. Defaults to Church notation.
    pub fn print(&self) {
        for expression in &self.expressions {
            println!("{}", expression)
        }
    }

    /// Get an iterator over all expressions.
    pub fn expressions(&self) -> impl Iterator<Item = &P> {
        self.expressions.iter()
    }

    /// Get the number of expressions in the soup.
    pub fn len(&self) -> usize {
        self.expressions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }

    /// Get the number of successful collisions
    pub fn collisions(&self) -> usize {
        self.n_collisions
    }
}
