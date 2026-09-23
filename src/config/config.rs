// Global Imports
use serde::{Deserialize, Serialize};

// Package Imports
use crate::config::{generator::Generator, reactor::Reactor, recursive::Recursive};

/// `Config` stores the global configuration of the program.
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// The number of reactions to run for this simulation. Default: `100000`.
    pub run_limit: usize,

    /// The number of lambda expressions used to seed the generator. Default: `1000`
    pub sample_size: usize,

    /// Print out the state of the soup every `polling_interval` reactions. When set to `None`,
    /// never poll. Default: `None`.
    pub polling_interval: Option<usize>,

    /// When set, print out all logs for each individual reaction. Default: `false`.
    pub verbose_logging: bool,

    /// Configuration options for the random expression generator.
    pub generator_config: Generator,

    /// Configuration options for the lambda reactor.
    pub reactor_config: Reactor,

    /// Configuration options for recursive experiments
    pub recursive_config: Recursive
}

impl Config {
    /// Create a config object from a string
    pub fn from_config_str(s: &str) -> Config {
        serde_json::from_str(s).unwrap()
    }

    /// Convert the config object to a string
    pub fn to_config_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Produce a new `Config` struct with default values.
    pub fn new() -> Self {
        Config {
            reactor_config: Reactor::new(),
            generator_config: Generator::new(),
            recursive_config: Recursive::new(),
            run_limit: 100000,
            sample_size: 1000,
            polling_interval: None,
            verbose_logging: false,
        }
    }

    pub fn set_reduction_cutoff(&mut self, cutoff: usize) {
        self.reactor_config.reduction_cutoff = cutoff;
    }

    pub fn set_run_limit(&mut self, limit: usize) {
        self.run_limit = limit;
    }

    pub fn set_polling_interval(&mut self, interval: Option<usize>) {
        self.polling_interval = interval;
    }

    pub fn set_verbose_logging(&mut self, logging: bool) {
        self.verbose_logging = logging;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}