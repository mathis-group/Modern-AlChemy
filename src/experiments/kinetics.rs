use async_std::task::{block_on, spawn};
use futures::stream::{FuturesUnordered, StreamExt};
use lambda_calculus::{data::num::binary::succ, Term};
use rand::random;

use crate::{
    config::{self, ConfigSeed},
    lambda::LambdaSoup,
    utils::dump_series_to_file,
};

use super::magic_test_function::{asymmetric_skip_sample, test_succ};

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

struct TestParams {
    id: Vec<usize>,
    seed: ConfigSeed,
    run_length: usize,
    polling_interval: usize,
    perturbation_interval: usize,
    count_each_poll: Vec<Term>,
}

async fn generalized_magic_test<F>(
    prefix: Vec<Term>,
    sample: Vec<Term>,
    tests: Vec<F>,
    n_prefix: usize,
    n_samples: usize,
    n_tests: usize,
    params: TestParams,
) -> (Vec<usize>, Vec<(usize, Vec<usize>)>)
where
    F: Fn() -> Term,
{
    let mut soup = experiment_soup(params.seed);

    let sample_iter = sample.iter().cycle();
    let test_iter = tests.iter().cycle().map(|f| f());

    soup.add_lambda_expressions(prefix.iter().cloned().take(n_prefix));
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

pub fn kinetic_succ_experiment() {
    let mut futures = FuturesUnordered::new();

    let sample_size = 5000;
    let good_fracs = [0.0002, 0.001, 0.02, 0.1];
    let test_fracs = [0.10, 0.20, 0.30, 0.40];

    for (i, good_frac) in good_fracs.iter().enumerate() {
        for (j, test_frac) in test_fracs.iter().enumerate() {
            for seed in 0..100 {
                let n_good = (good_frac * sample_size as f64) as usize;
                let n_test = (test_frac * sample_size as f64) as usize;
                let n_rest = sample_size - (n_good + n_test);

                let goods = vec![succ()];
                let tests = vec![|| test_succ(random::<usize>() % 20)];
                let sample = asymmetric_skip_sample(ConfigSeed::new([seed as u8; 32]));

                let params = TestParams {
                    id: vec![i, j, seed],
                    seed: ConfigSeed::new([seed as u8; 32]),
                    count_each_poll: vec![succ()],
                    perturbation_interval: 10,
                    polling_interval: 1000,
                    run_length: 100000,
                };

                let run =
                    generalized_magic_test(goods, sample, tests, n_good, n_rest, n_test, params);
                futures.push(spawn(run));
            }
        }
    }
    let fname = "kinetic-scc-output";
    while let Some((id, series)) = block_on(futures.next()) {
        dump_series_to_file(fname, &series, &id).expect("Cannot write to file");
    }
}
