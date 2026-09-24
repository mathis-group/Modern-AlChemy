// Global Imports
use std::fmt::Display;
use std::iter::repeat_n;

// Package Imports
use crate::config::{config::Config, recursive::{Recursive, RefillType}};
use crate::traits::{Particle, Collider, Generator};
use crate::soupercollider::Soup;
use crate::logging::Tape;
use crate::errors::ParsingError;

/// Simulate a recursive experiment on a Soup 
pub fn simulate_recursive_experiment<P, C, G>( 
    soup: &mut Soup<P, C, G>, 
    config: Config
) -> Result<Vec<Tape<P, C, G>>, ParsingError> 
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    // ------------- Recursive Experiment -------------
    // Setup the tape list vector to store each generations history
    let mut tape_list = Vec::new();

    // Iterate over each for n_generations
    for gen in 0..config.recursive_config.n_generations {
        println!("Generation {gen}");
        // If we have a polling interval configured, push our recordings to the tape struct list
        if let Some(polling_interval) = config.polling_interval {
            tape_list.push(soup.simulate_and_record(config.run_limit, polling_interval, config.verbose_logging));
        } 
        // Otherwise simulate normally without recording and print the soup at the end
        else {
            soup.simulate_for(config.run_limit, config.verbose_logging);
        }

        // Recursive wipeout, remove `wipeout_percent` of expressions from the soup
        let n_wipeout = wipeout(soup, config.recursive_config.wipeout_percent);
        
        // And repopulate with expressions of the specified refill_type
        let _repopulate_result = repopulate(soup, n_wipeout, &config.recursive_config)?;
        
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
    // TODO: Gotta be a better way to do this type conversion/casting/floor thing
    let n_wipeout = (soup.expressions.len() as f64 * ((wipeout_percent as f64) / 100.0)) as usize;
    
    // Remove that many records from the soup
    soup.cull(n_wipeout);

    n_wipeout
}

/// Repopulate the soup based with n new particles either from the configured generator
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