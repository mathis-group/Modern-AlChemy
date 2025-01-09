use clap::{Parser, ValueEnum};
use futures::executor::block_on;
use generators::BTreeGen;
use lambda_calculus::*;
use std::fs::{read_to_string, File};
use std::io::{self, BufRead, BufReader, Write};

/// Simulation analysis
mod analysis;

/// Global configuration
mod config;

/// Random expression generators
mod generators;

/// Main AlChemy simulation module
mod supercollider;

/// Experimental stuff
mod experiments;

/// Utilities
mod utils;

/// Lambda-calculus stuff
mod lambda;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Experiment {
    XorsetStability,
    XorsetSearch,
    AdditionSearch,
    SyncEntropyTest,
    EntropyTest,
    EntropySeries,
    SampleSimulate,
    SampleScan,
    MagicTestFunction,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Fail a reaction if it takes more than `reduction_cutoff` steps to reduce. If set, this
    /// flag overwrites the `reactor_config.reactor_config` configuration option.
    #[arg(short = 'f', long)]
    reduction_cutoff: Option<usize>,

    /// Generate a tape that snapshots the state of the reactor every `polling_interval`
    /// reactions. If set, this flag overwrites the `polling_interval` configuration option.
    #[arg(short, long)]
    polling_interval: Option<usize>,

    /// Number of reactions to run before printing out final soup. If set, this flag overwrites the
    /// `run_limit` configuration option.
    #[arg(short, long)]
    run_limit: Option<usize>,

    /// Explicit path to configuration file
    #[arg(short, long)]
    config_file: Option<String>,

    /// Dump out the current config and exit
    #[arg(long)]
    dump_config: bool,

    /// Run an experiment and exit
    #[arg(short, long)]
    experiment: Option<Experiment>,

    /// Make a default config file in the current directory and exit
    #[arg(short, long)]
    make_default_config: bool,

    /// Generate n lambda expresions and exit
    #[arg(long)]
    generate: Option<u32>,

    /// Read expressions from stdin instead of generating own expressions
    #[arg(long)]
    read_stdin: bool,

    /// Log each reaction
    #[arg(long)]
    log: bool,
}

fn get_config(cli: &Cli) -> std::io::Result<config::Config> {
    let mut config = if let Some(filename) = &cli.config_file {
        let contents = read_to_string(filename)?;
        config::Config::from_config_str(&contents)
    } else {
        config::Config::new()
    };

    if let Some(limit) = cli.run_limit {
        config.set_run_limit(limit);
    }
    if let Some(cutoff) = cli.reduction_cutoff {
        config.set_reduction_cutoff(cutoff);
    }
    if cli.polling_interval.is_some() {
        config.set_polling_interval(cli.polling_interval);
    }
    if cli.log {
        config.set_verbose_logging(cli.log)
    }

    Ok(config)
}

/// Read lambda expressions from stdin and return an iterator over them
pub fn read_inputs() -> impl Iterator<Item = Term> {
    let mut expression_strings = Vec::<String>::new();
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        match line {
            Ok(line) => expression_strings.push(line),
            Err(_) => break,
        }
    }

    let expressions = expression_strings
        .iter()
        .map(|s| lambda_calculus::parse(s, lambda_calculus::Classic).unwrap())
        .collect::<Vec<Term>>();
    expressions.into_iter()
}

pub fn generate_expressions_and_seed_soup(cfg: &config::Config) -> lambda::LambdaSoup {
    let expressions = match &cfg.generator_config {
        config::Generator::BTree(gen_cfg) => {
            let mut gen = generators::BTreeGen::from_config(gen_cfg);
            gen.generate_n(cfg.sample_size)
        }
        config::Generator::Fontana(gen_cfg) => {
            let gen = generators::FontanaGen::from_config(gen_cfg);
            std::iter::from_fn(move || gen.generate())
                .take(cfg.sample_size)
                .collect::<Vec<Term>>()
        }
    };
    let mut soup = lambda::LambdaSoup::from_config(&cfg.reactor_config);
    soup.add_lambda_expressions(expressions);
    soup
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if cli.make_default_config {
        let config_path = "config.json";
        let mut config_file = File::create(config_path)?;
        config_file.write_all(config::Config::new().to_config_str().as_bytes())?;
        return Ok(());
    }

    let config = get_config(&cli)?;

    if cli.dump_config {
        println!("{}", config.to_config_str());
        return Ok(());
    }

    if let Some(n) = cli.generate {
        let mut gen = BTreeGen::new();
        for _ in 0..n {
            println!("{:?}", gen.generate())
        }
        return Ok(());
    }

    if let Some(e) = cli.experiment {
        match e {
            Experiment::XorsetStability => {}
            Experiment::AdditionSearch => {
                block_on(experiments::look_for_add());
            }
            Experiment::MagicTestFunction => {
                block_on(experiments::look_for_magic());
            }
            Experiment::SyncEntropyTest => experiments::sync_entropy_test(),
            Experiment::SampleScan => experiments::one_sample_with_dist(),
            Experiment::XorsetSearch => {
                block_on(experiments::look_for_xorset());
            }
            Experiment::EntropyTest => {
                block_on(experiments::entropy_test());
            }
            Experiment::EntropySeries => {
                block_on(experiments::entropy_series());
            }
            Experiment::SampleSimulate => {
                block_on(experiments::simulate_sample());
            }
        }
        return Ok(());
    }

    let mut soup = if cli.read_stdin {
        let mut soup = lambda::LambdaSoup::from_config(&config.reactor_config);
        let expressions = read_inputs();
        soup.add_lambda_expressions(expressions);
        soup
    } else {
        generate_expressions_and_seed_soup(&config)
    };

    if let Some(polling_interval) = config.polling_interval {
        let tape =
            soup.simulate_and_record(config.run_limit, polling_interval, config.verbose_logging);
        for soup in tape.history() {
            println!("{}", soup.population_entropy());
        }
    } else {
        soup.simulate_for(config.run_limit, config.verbose_logging);
        soup.print();
    }

    Ok(())
}
