use async_std::task::{block_on, spawn};
use futures::stream::{FuturesUnordered, StreamExt};
use lambda_calculus::{data::num::binary::succ, Term};
use rand::random;

use crate::{
    config::{self, ConfigSeed},
    experiments::magic_test_function::ski_sample,
    lambda::LambdaSoup,
    utils::dump_series_to_file,
};

use super::magic_test_function::{test_succ, test_succ_seq};

fn experiment_soup(seed: ConfigSeed) -> LambdaSoup {
    LambdaSoup::from_config(&config::Reactor {
        rules: vec![String::from("\\x.\\y.x y")],
        discard_copy_actions: false,
        discard_identity: false,
        discard_free_variable_expressions: true,
        maintain_constant_population_size: true,
        discard_parents: false,
        reduction_cutoff: 8000,
        size_cutoff: 1000,
        seed,
    })
}

pub(super) struct RunParams {
    pub id: Vec<usize>,
    pub seed: ConfigSeed,
    pub run_length: usize,
    pub polling_interval: usize,
    pub perturbation_interval: usize,
    pub count_each_poll: Vec<Term>,
}

// Returns (id, populations), where id is a vec of usizes and populations is a vec of
// (count, isomorphics). Here, count is the current population of recursive functions in the soup,
// and isomorphics is a list of populations of terms isomorphic to terms in params.count_each_poll.
pub(super) async fn general_test_run<F>(
    prefix: Vec<Term>,
    sample: Vec<Term>,
    tests: Vec<F>,
    n_prefix: usize,
    n_samples: usize,
    n_tests: usize,
    params: RunParams,
) -> (Vec<usize>, Vec<(usize, Vec<usize>)>)
where
    F: Fn() -> Term,
{
    let mut soup = experiment_soup(params.seed);

    let prefix_iter = prefix.iter().cycle();
    let sample_iter = sample.iter().cycle();
    let test_iter = tests.iter().cycle().map(|f| f());

    soup.add_lambda_expressions(prefix_iter.cloned().take(n_prefix));
    soup.add_lambda_expressions(sample_iter.cloned().take(n_samples));
    soup.add_test_expressions(test_iter.clone().take(n_tests));

    let populations = (0..params.perturbation_interval)
        .flat_map(|i| {
            let pops = soup.simulate_and_poll(
                params.run_length / params.perturbation_interval,
                params.polling_interval,
                false,
                |s| {
                    let isomorphics = params
                        .count_each_poll
                        .iter()
                        .map(|t| s.population_of(t))
                        .collect();
                    let n_recursive = s.expressions().filter(|e| e.is_recursive()).count();
                    (n_recursive, isomorphics)
                },
            );

            let n_remaining = n_tests - soup.expressions().filter(|e| e.is_recursive()).count();
            soup.perturb_test_expressions(n_remaining, test_iter.clone().take(n_remaining));
            println!("Soup {:?} {}0% done", params.id, i + 1);

            pops
        })
        .collect();
    (params.id, populations)
}

pub(super) async fn general_run(
    prefix: Vec<Term>,
    sample: Vec<Term>,
    n_prefix: usize,
    n_samples: usize,
    params: RunParams,
) -> (Vec<usize>, Vec<(usize, Vec<usize>)>) {
    let mut soup = experiment_soup(params.seed);

    let prefix_iter = prefix.iter().cycle();
    let sample_iter = sample.iter().cycle();

    soup.add_lambda_expressions(prefix_iter.cloned().take(n_prefix));
    soup.add_lambda_expressions(sample_iter.cloned().take(n_samples));

    let populations = (0..params.perturbation_interval)
        .flat_map(|i| {
            let pops = soup.simulate_and_poll(
                params.run_length / params.perturbation_interval,
                params.polling_interval,
                false,
                |s| {
                    let isomorphics = params
                        .count_each_poll
                        .iter()
                        .map(|t| s.population_of(t))
                        .collect();
                    let n_recursive = s.expressions().filter(|e| e.is_recursive()).count();
                    (n_recursive, isomorphics)
                },
            );

            println!("Soup {:?} {}0% done", params.id, i + 1);
            pops
        })
        .collect();
    (params.id, populations)
}

pub fn kinetic_succ_experiment() {
    let mut futures = FuturesUnordered::new();

    let sample_size = 5000;
    let good_fracs = [0.0002, 0.001, 0.02, 0.1, 0.5];
    let test_fracs = [0.10, 0.20, 0.30, 0.40];

    for (i, good_frac) in good_fracs.iter().enumerate() {
        for (j, test_frac) in test_fracs.iter().enumerate() {
            for seed in 0..100 {
                let n_good = (good_frac * sample_size as f64) as usize;
                let n_test = (test_frac * sample_size as f64) as usize;
                let n_rest = sample_size - (n_good + n_test);

                let goods = vec![succ()];
                let tests = vec![|| test_succ_seq((0..4).map(|_| random::<usize>() % 20))];
                let samples = ski_sample();
                let params = RunParams {
                    id: vec![i, j, seed],
                    seed: ConfigSeed::new([seed as u8; 32]),
                    count_each_poll: vec![succ()],
                    perturbation_interval: 10,
                    polling_interval: 1000,
                    run_length: 100000,
                };

                let run = general_test_run(goods, samples, tests, n_good, n_rest, n_test, params);
                futures.push(spawn(run));
            }
        }
    }
    let fname = "kinetic-scc-output";
    while let Some((id, series)) = block_on(futures.next()) {
        dump_series_to_file(fname, &series, &id).expect("Cannot write to file");
    }
}
