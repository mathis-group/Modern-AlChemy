// Global Imports
use std::env;
use std::fs::read_to_string;
use clap::Parser;

// Package Imports
use crate::experiments::enums::Experiment;
use crate::config::config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Fail a reaction if it takes more than `reduction_cutoff` steps to reduce. If set, this
    /// flag overwrites the `reactor_config.reactor_config` configuration option.
    #[arg(short = 'f', long)]
    pub reduction_cutoff: Option<usize>,

    /// Generate a tape that snapshots the state of the reactor every `polling_interval`
    /// reactions. If set, this flag overwrites the `polling_interval` configuration option.
    #[arg(short, long)]
    pub polling_interval: Option<usize>,

    /// Number of reactions to run before printing out final soup. If set, this flag overwrites the
    /// `run_limit` configuration option.
    #[arg(short, long)]
    pub run_limit: Option<usize>,

    /// Explicit path to configuration file
    #[arg(short, long)]
    pub config_file: Option<String>,

    /// Dump out the current config and exit
    #[arg(long)]
    pub dump_config: bool,

    /// Run an experiment and exit
    #[arg(short, long)]
    pub experiment: Option<Experiment>,

    /// Make a default config file in the current directory and exit
    #[arg(short, long)]
    pub make_default_config: bool,

    /// Generate n lambda expresions and exit
    #[arg(long)]
    pub generate: Option<usize>,

    /// Read expressions from stdin instead of generating own expressions
    #[arg(long)]
    pub read_stdin: bool,

    /// Log each reaction
    #[arg(long)]
    pub log: bool,
}

impl Cli {
    pub fn get_config(&mut self) -> std::io::Result<config::Config> {
        let mut config = 
        if let Some(filename) = &self.config_file {
            println!("{filename}");
            let cwd = env::current_dir()?;
            // Print the directory path using .display()
            println!("Current directory: {}", cwd.display());

            let contents = read_to_string(filename)?;
            config::Config::from_config_str(&contents)
        } else {
            config::Config::new()
        };

        if let Some(limit) = self.run_limit {
            config.set_run_limit(limit);
        }
        if let Some(cutoff) = self.reduction_cutoff {
            config.set_reduction_cutoff(cutoff);
        }
        if self.polling_interval.is_some() {
            config.set_polling_interval(self.polling_interval);
        }
        if self.log {
            config.set_verbose_logging(self.log)
        }

        Ok(config)
    }
}