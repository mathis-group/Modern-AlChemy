// Global Imports
use std::fs::File;
use std::io::Write;
use alchemy::traits::Generator;
use clap::Parser;

// Package Imports
use alchemy::config::config;
use alchemy::enums::ExpressionType;
use alchemy::utils::{run_experiment, read_inputs};
use alchemy::cli::Cli;
use alchemy::lambda::soup::LambdaSoup;
use alchemy::recursive_experiment::simulate_recursive_experiment;

fn main() -> std::io::Result<()> {
    let mut cli = Cli::parse();

    // Generate a default config and write it to `config.json`
    if cli.make_default_config {
        let config_path = "config.json";
        let mut config_file = File::create(config_path)?;
        config_file.write_all(config::Config::new().to_config_str().as_bytes())?;
        return Ok(());
    }

    let config = cli.get_config()?;

    // Print to config to the console
    if cli.dump_config {
        println!("{}", config.to_config_str());
        return Ok(());
    }

    // Run a specified experiment
    if let Some(e) = cli.experiment {
        run_experiment(e);
        return Ok(());
    }

    // Create the soup from the configured soup type
    let mut soup = 
        match config.expression_type {
            ExpressionType::UntypedLambda     => LambdaSoup::from_config(&config),
            ExpressionType::SimplyTypedLambda => todo!("typed lambda soup"),
            ExpressionType::Haskell           => todo!("haskell soup"),
        };

    // Generate & print n expressions from the configured generator
    if let Some(n) = cli.generate {
        let particles = soup.generator.generate_n_particles(n);
        for p in particles {
            println!("{:?}", p);
        }
        return Ok(());
    }

    // Add expressions input through the CLI
    if cli.read_stdin {
        let expressions = read_inputs();
        // Parse the expressions and add to the soup
        soup.add_expressions(expressions)?;
    } 
    // Or have the soup seeded with n expressions with the configured generator
    else {
        soup.seed_with_generator(config.sample_size);
    };

    // Perform the recursive experiment using the soup and config to produce a recording on the `tape_list`
    let _tape_list = simulate_recursive_experiment(&mut soup, config)?;

    Ok(())
}