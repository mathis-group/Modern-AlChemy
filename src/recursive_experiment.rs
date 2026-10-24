// Global Imports
use std::fmt::Display;
use std::iter::repeat_n;

// Package Imports
use crate::config::{config::Config, recursive::{Recursive, RefillType}};
use crate::traits::{Particle, Collider, Generator};
use crate::soupercollider::Soup;
use crate::record::{Tape, RecordingType};
use crate::errors::ParsingError;

/// Simulate a recursive experiment on a Soup 
pub fn simulate_recursive_experiment<P, C, G>( 
    soup: &mut Soup<P, C, G>, 
    recording: RecordingType,
    config: Config
) -> Result<Vec<Tape<P, C, G>>, ParsingError> 
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    run_generations(soup, recording, config, |soup, n, recording, polling| {
        soup.simulate(n, recording, polling)
    })
}

/// Python persistence metadata without changing the public Rust Tape shape.
pub(crate) struct RecordedGeneration<P, C, G>
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    pub tape: Tape<P, C, G>,
    pub history_collisions: Vec<usize>,
    pub initial_population: Vec<P>,
    pub final_population: Vec<P>,
}

pub(crate) fn simulate_recursive_experiment_recorded<P, C, G>(
    soup: &mut Soup<P, C, G>,
    config: Config,
) -> Result<Vec<RecordedGeneration<P, C, G>>, ParsingError>
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    run_generations(soup, RecordingType::All, config, |soup, n, recording, polling| {
        let initial_population = soup.expressions.clone();
        let mut history_collisions = Vec::new();
        let tape = soup.simulate_recorded(n, recording, polling, Some(&mut history_collisions))?;
        Some(RecordedGeneration {
            tape,
            history_collisions,
            initial_population,
            final_population: soup.expressions.clone(),
        })
    })
}

// Share the native generation/wipeout loop; only the new private path captures
// boundaries. Existing public Rust calls keep their original allocation behavior.
fn run_generations<P, C, G, R>(
    soup: &mut Soup<P, C, G>,
    recording: RecordingType,
    config: Config,
    mut simulate_generation: impl FnMut(
        &mut Soup<P, C, G>, usize, RecordingType, Option<usize>,
    ) -> Option<R>,
) -> Result<Vec<R>, ParsingError>
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    // ------------- Recursive Experiment -------------
    // Setup the tape list vector to store each generations history
    let mut tape_list = Vec::new();

    // Iterate over each for n_generations
    for i in 0..config.recursive_config.n_generations {
        println!("Generation {i}");
        // Run the simulation and record it to the tape list if configured.
        if let Some(tape) = simulate_generation(
            soup, config.run_limit, recording, config.polling_interval,
        ) {
            tape_list.push(tape);
        };

        // Run the wipeout and repopulate for every generation except the last.
        if i != config.recursive_config.n_generations - 1 {
            // Recursive wipeout, remove `wipeout_percent` of expressions from the soup
            let n_wipeout = wipeout(soup, config.recursive_config.wipeout_percent);

            // And repopulate with expressions of the specified refill_type
            let _repopulate_result = repopulate(soup, n_wipeout, &config.recursive_config)?;
        }

        soup.print();
        println!("");
    }
    Ok(tape_list)
}

/// Cull `wipeout_percent` of the population (rounded down) and replace with
/// `ConfigGenerator` generated or `custom_expression` expressions
fn wipeout<P, C, G>(
    soup: &mut Soup<P, C, G>,
    wipeout_percent: usize
) -> usize
where 
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    // Calculate how many expressions will be culled based on the wipeout percent
    let n_wipeout = soup.expressions.len() * wipeout_percent / 100;

    // Remove that many records from the soup
    soup.cull(n_wipeout);

    n_wipeout
}

/// Repopulate the soup with n new particles either from the configured generator
/// or the provided `repopulation_expression`
fn repopulate<P, C, G>(
    soup: &mut Soup<P, C, G>,
    n: usize, 
    recursive_config: &Recursive,
) -> Result<(), ParsingError> 
where 
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    match recursive_config.refill_type {
        // Refill with the configured generator
        RefillType::ConfigGenerator => soup.seed_with_generator(n),
        // Refill with the custom expression provided
        RefillType::CustomExpression => {
            let particle = P::parse(&recursive_config.repopulation_expression)?;
            soup.perturb(repeat_n(particle, n));
        }
    }
    Ok(())
}
