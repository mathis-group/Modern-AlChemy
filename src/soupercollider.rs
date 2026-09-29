// Global Imports
use std::fmt::{Debug, Display};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

// Package Imports
use crate::{traits::{Collider, Generator, Particle, Residue}};
use crate::record::{Tape, ReactionRecord};
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

    /// Simulate the soup for `n` collisions. If `log` is set, then print
    /// out a log message for each reaction. Returns the number of successful reactions
    /// (the fraction of failed reactions).
    pub fn simulate(
        &mut self, 
        n: usize, 
        record: bool,
        filter: bool,
        polling_interval: Option<usize>, 
    ) -> Option<Tape<P, C, G>> {
        // Setup our recording lists
        let mut history: Vec<Self> = Vec::new();
        let mut reaction_record: Vec<ReactionRecord<P>> = Vec::new();

        // Iterate for n simulation steps
        for i in 0..n {
            let reaction = self.react(i, record)?;

            // If we have recording enabled
            if record {
                // If we have filtering disabled or have a successful reaction
                if !filter || reaction.success {
                    // Log the reaction to the reaction_record list
                    reaction_record.push(reaction);
                }
                // If we have a polling interval, copy the entire soup every polling_interval steps
                if let Some(interval) = polling_interval && (i % interval) == 0 {
                    history.push(self.clone())
                }
            }
        }

        // If we weren't recording return None
        if !record {
            return None
        }

        // If we don't have a whole divisor between number of steps and our configured polling interval
        // push the final value of the soup to the tape. If we did have a whole divisor, this would have been
        // caught by the last loop of the polling interval check above.
        if let Some(interval) = polling_interval && (n % interval) != 0 {
            history.push(self.clone());
        }

        // Return the Tape with the soup snapshots and reaction_record
        Some(Tape::<P, C, G> {
            soup_history: history,
            reaction_record
        })
    }

    /// Produce one atomic reaction on the soup.
    pub fn react(&mut self, step: usize, record: bool) -> Option<ReactionRecord<P>> {
        let n_expr = self.len();
        let mut products: Vec<P> = Vec::new();

        // Remove two distinct expressions randomly from the soup
        let left = self.expressions.swap_remove(self.rng.gen_range(0..n_expr));
        let right = self.expressions.swap_remove(self.rng.gen_range(0..n_expr - 1));

        // Add collision results to soup
        let result = self.collider.collide(left.clone(), right.clone());

        // Add removed parents back into the soup, if necessary
        if !self.discard_parents {
            self.expressions.push(left.clone());
            self.expressions.push(right.clone());
        }

        // If we had a successful result, add the products of the reduction to the soup
        if let Ok(ref t) = result {
            products.extend(t.particles());
            self.perturb(products.iter().cloned());

            // If we have set a constant population size, remove the
            // number of added expressions
            if self.maintain_constant_population_size {
                self.cull(t.count());
            }
        }

        // If logging is disabled return None
        if !record {
            return None;
        }

        // Otherwise return the recorded result of the reaction
        Some(ReactionRecord {
            step,
            left: left.clone(),
            right: right.clone(),
            products: if result.is_ok() { products } else { vec![] },
            success: result.is_ok(),
            error: if let Err(ref e) = result { Some(format!("{}", e)) } else { None },
        })
    }

    /// Introduce all expressions in `expressions` into the soup, without
    /// reduction.
    pub fn perturb(&mut self, expressions: impl IntoIterator<Item = P>) {
        self.expressions.extend(expressions)
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

    /// Removes n random expressions from the soup
    pub fn cull(&mut self, n: usize) {
        let n_expr = self.expressions.len();

        // Cull one expression for each wipeout we have
        for execution_count in 0..n {
            let cull_index = self.rng.gen_range(0..n_expr - execution_count);
            let _removed_expression = self.expressions.swap_remove(cull_index);
        }
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
