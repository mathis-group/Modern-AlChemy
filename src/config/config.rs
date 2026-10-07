// Global Imports
use serde::{Deserialize, Serialize};

// Package Imports
use crate::config::{expression::Expression, recursive::Recursive, soup::Soup};

/// `Config` stores the global configuration of the program.
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// The expression type to run, tagged, carrying its expression-specific
    /// generator and reactor configuration.
    pub expression: Expression,

    /// The number of reactions to run for this simulation. Default: `100000`.
    pub run_limit: usize,

    /// The number of lambda expressions used to seed the generator. Default: `1000`
    pub sample_size: usize,

    /// Print out the state of the soup every `polling_interval` reactions. When set to `None`,
    /// never poll. Default: `None`.
    pub polling_interval: Option<usize>,

    /// When set, print out all logs for each individual reaction. Default: `false`.
    pub verbose_logging: bool,

    /// Configuration options for the soup's population and dilution flux.
    /// Shared by every expression type, so it sits outside `expression`.
    pub soup_config: Soup,

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
            expression: Expression::new(),
            soup_config: Soup::new(),
            recursive_config: Recursive::new(),
            run_limit: 100000,
            sample_size: 1000,
            polling_interval: None,
            verbose_logging: false,
        }
    }

    /// Override the reduction cutoff from the CLI.
    ///
    /// `reduction_cutoff` is a beta-reduction concept, so it only applies to
    /// expression types that reduce. The match is exhaustive by construction:
    /// adding a variant without that concept will stop this compiling, which is
    /// where the decision about what `--reduction-cutoff` should mean for it
    /// belongs.
    pub fn set_reduction_cutoff(&mut self, cutoff: usize) {
        match &mut self.expression {
            Expression::UntypedLambda(e) => e.reactor.reduction_cutoff = cutoff,
        }
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