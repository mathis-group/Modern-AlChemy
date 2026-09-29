// Global Imports
use std::fmt::{Debug, Display};

// Package Imports
use crate::soupercollider::Soup;
use crate::record::Tape;
use crate::traits::{Particle, Collider, Generator};
use crate::config::{
    config::Config,
    config_seed::ConfigSeed,
    reactor::Reactor,
    reactors::untyped_lambda::UntypedLambdaReactor,
    soup::Soup as SoupConfig,
};
use crate::lambda::soup::LambdaSoup;

/// The standard experiment soup, shared by every experiment in this module.
///
/// All five experiment modules previously carried a byte-identical copy of this
/// function. Hoisting it means a config schema change touches one call site
/// instead of five.
///
/// Note where `seed` lands: it configures the *soup's* reaction RNG, which lives
/// in `soup_config`, not in the reactor. Generators carry their own seed.
pub fn experiment_soup(seed: ConfigSeed) -> LambdaSoup {
    LambdaSoup::from_config(&Config {
        reactor_config: Reactor::UntypedLambda(UntypedLambdaReactor {
            rules: vec![String::from("\\x.\\y.x y")],
            discard_copy_actions: false,
            discard_identity: false,
            discard_free_variable_expressions: true,
            reduction_cutoff: 8000,
            size_cutoff: 1000,
        }),
        soup_config: SoupConfig {
            discard_parents: false,
            maintain_constant_population_size: true,
            seed,
        },
        ..Default::default()
    })
}

impl<P, C, G> Soup<P, C, G> 
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone
    {
    
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
            let reaction = self.react(i, false);
            if (i % polling_interval) == 0 {
                data.push(poller(self))
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
            let _reaction = self.react(i, false);
            if (i % polling_interval) == 0 {
                let (datum, should_kill) = killpoller(self);
                data.push(datum);
                if should_kill {
                    return data;
                };
            }
        }
        data
    }

}