use lambda_calculus::data::num::church::{add, succ};
use rand::random;

use crate::{
    config::{self, ConfigSeed},
    generators::BTreeGen,
    lambda::reduce_with_limit,
    utils::dump_series_to_file,
};

fn experiment_gen(seed: ConfigSeed) -> BTreeGen {
    BTreeGen::from_config(&config::BTreeGen {
        size: 20,
        freevar_generation_probability: 0.2,
        standardization: crate::generators::Standardization::Prefix,
        n_max_free_vars: 6,
        seed,
    })
}

pub fn measure_initial_population() {
    for (i, term) in [succ(), add()].iter().enumerate() {
        let series = (0..1000)
            .map(|_| {
                let random_seed = ConfigSeed::new(random::<[u8; 32]>());
                let mut gen = experiment_gen(random_seed);
                gen.generate_n(10000)
                    .iter_mut()
                    .map(|mut t| {
                        let r = reduce_with_limit(&mut t, 1000, 8000);
                        (r, t)
                    })
                    .filter(|(r, t)| r.is_ok() && t.is_isomorphic_to(&term))
                    .count()
            })
            .collect::<Vec<_>>();
        dump_series_to_file("initial_population_counts", &series, &[i])
            .expect("Cannot write to file");
    }
}
