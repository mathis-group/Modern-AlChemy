// Global Imports
use serde::{Serialize, Deserialize};

/// Configuration for the reactor
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Recursive {
    /// Number of generations an experiment should be run for. Default: 1
    pub n_generations: usize,

    /// Percent of the population (rounded down) to remove between recursive generations. [0, 100]. Default: 0
    pub wipeout_percent: usize,

    /// The type of refill that should be performed after a wipeout
    /// {config_generator: Generator, custom_expression: string}. Default: config_generator
    pub refill_type: RefillType,

    /// The custom expression to be seeded in place of the wipeout 
    /// population up to sample_size. Default: None
    /// TODO: Extend this to accept multiple expressions for injection
    pub repopulation_expression: String
}

impl Recursive {
    /// Produce a new `Recursive` struct with default values.
    pub fn new() -> Self {
        Recursive {
            n_generations: 1,
            wipeout_percent: 0,
            refill_type: RefillType::ConfigGenerator,
            repopulation_expression: String::from("\\x.\\y.x y")
        }
    }
}

/// Configuration for the generators
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub enum RefillType {
    /// Use the generator specified in the config
    /// TODO: Extend this to use a different generator from the one provided in config
    ConfigGenerator,

    /// Use a custom expression specified in the config
    CustomExpression
}