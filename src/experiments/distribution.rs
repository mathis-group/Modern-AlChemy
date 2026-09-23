#![allow(clippy::all)]
#![allow(warnings)]

use std::collections::HashMap;

use lambda_calculus::Term;

use crate::config::{
    config::Config, 
    config_seed::ConfigSeed,
    reactor::Reactor
};

use crate::lambda::{particle::LambdaParticle, soup::LambdaSoup};

use crate::utils::read_particles;

fn experiment_soup(seed: ConfigSeed) -> LambdaSoup {
    LambdaSoup::from_config(&Config {
        reactor_config: Reactor {
            rules: vec![String::from("\\x.\\y.x y")],
            discard_copy_actions: false,
            discard_identity: false,
            discard_free_variable_expressions: true,
            maintain_constant_population_size: true,
            discard_parents: false,
            reduction_cutoff: 8000,
            size_cutoff: 1000,
            seed,
        },
        ..Default::default()
    })
}

pub fn one_sample_with_dist() {
    let run_length = 1000000;
    let polling_interval = 1000;
    let polls = run_length / polling_interval;
    let sample: Vec<LambdaParticle> = read_particles().expect("invalid expression on stdin");    
    let mut soup = experiment_soup(ConfigSeed::new([0; 32]));

    soup.perturb(sample.into_iter().cycle().take(10000));
    let counts = soup.simulate_and_poll(run_length, polling_interval, false, |s| {
        s.expression_counts()
    });

    let mut map = HashMap::<Term, Vec<u32>>::new();
    for (i, count) in counts.iter().enumerate() {
        for (term, val) in count.iter() {
            map.entry(term.clone())
                .or_insert(vec![0; i.try_into().unwrap()])
                .push(*val);
        }
        for (term, vals) in map.iter_mut() {
            if !count.contains_key(term) {
                vals.push(0);
            }
        }
    }

    print!("Term, ");
    for i in 0..polls {
        print!("{}, ", i)
    }
    println!();
    for (term, vec) in map.iter() {
        print!("{}, ", term);
        for c in vec {
            print!("{}, ", c);
        }
        println!();
    }
}
