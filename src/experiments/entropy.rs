#![allow(clippy::all)]
#![allow(warnings)]

use async_std::task::{block_on, spawn};
use futures::{stream::FuturesUnordered, StreamExt};
use lambda_calculus::Term;

use crate::config::{
    config::Config, 
    config_seed::ConfigSeed,
    generators::b_tree_gen::BTreeGen as BTreeGenConfig
};

use crate::lambda::{
    generators::b_tree_gen::BTreeGen,
    utils::reduce_with_limit
};

use crate::lambda::soup::LambdaSoup;

use crate::config::generators::b_tree_gen::Standardization;
use crate::record::RecordingType;
use crate::experiments::utils::experiment_soup;


fn experiment_gen(seed: ConfigSeed) -> BTreeGen {
    BTreeGen::from_config(&BTreeGenConfig {
        size: 20,
        freevar_generation_probability: 0.2,
        standardization: Standardization::Prefix,
        n_max_free_vars: 6,
        seed,
    })
}

async fn simulate_soup(
    sample: impl Iterator<Item = Term>,
    id: usize,
    run_length: usize,
) -> (LambdaSoup, usize, f32) {
    let mut soup = experiment_soup(ConfigSeed::new([0; 32]));
    soup.add_lambda_expressions(sample, false);
    
    let mut n_successes = 0;
    if let Some(tape)= soup.simulate(run_length, RecordingType::SuccessOnly, None) {
        n_successes = tape.reaction_record.len();
    };
    let failure_rate = 1f32 - n_successes as f32 / run_length as f32;
    (soup, id, failure_rate)
}

async fn simulate_soup_and_produce_entropies(
    sample: impl Iterator<Item = Term>,
    id: usize,
    run_length: usize,
    polling_interval: usize,
) -> (usize, Vec<f32>) {
    let mut seed: [u8; 32] = [0; 32];
    let bytes = id.to_le_bytes();
    seed[..bytes.len()].copy_from_slice(&bytes);
    let mut soup = experiment_soup(ConfigSeed::new([0; 32]));
    soup.add_lambda_expressions(sample, false);
    let data = soup.simulate_and_poll(run_length, polling_interval, false, |s: &LambdaSoup| {
        s.population_entropy()
    });
    (id, data)
}

pub fn entropy_time_series() {
    let mut generator = experiment_gen(ConfigSeed::new([0; 32]));
    let mut futures = FuturesUnordered::new();
    let run_length = 10000000;
    let polling_interval = 1000;
    let polls = run_length / polling_interval;
    for i in 0..1000 {
        let sample = generator.generate_n(10000);
        futures.push(spawn(simulate_soup_and_produce_entropies(
            sample.into_iter(),
            i,
            run_length,
            polling_interval,
        )));
    }

    print!("Soup, ");
    for i in 0..polls {
        print!("{}, ", i)
    }
    println!();
    while let Some((id, data)) = block_on(futures.next()) {
        print!("{}, ", id);
        for i in data {
            print!("{}, ", i)
        }
        println!();
    }
}

pub fn entropy_and_failures() {
    let mut generator = experiment_gen(ConfigSeed::new([0; 32]));
    let mut futures = FuturesUnordered::new();
    for i in 0..1000 {
        let sample = generator.generate_n(10000);
        futures.push(spawn(simulate_soup(sample.into_iter(), i, 10000000)));
    }

    let mut data = Vec::new();
    println!("Soup, Entropy, Failure rate");
    while let Some((soup, id, failure_rate)) = block_on(futures.next()) {
        let entropy = soup.population_entropy();
        println!("{}, {}, {}", id, entropy, failure_rate);
        data.push(entropy);
    }
}

pub fn sync_entropy_and_failures() {
    let mut generator = experiment_gen(ConfigSeed::new([0; 32]));

    for i in 0..100 {
        let sample = generator.generate_n(1000);
        let mut soup = experiment_soup(ConfigSeed::new([0; 32]));
        soup.add_lambda_expressions(sample, false);
        soup.simulate(100000, RecordingType::None, None);
        let entropy = soup.population_entropy();
        println!("{}: {}", i, entropy);
    }
}
