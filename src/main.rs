// Global Imports
use std::fs::File;
use std::iter::repeat_n;
use std::io;
use std::io::Write;
use alchemy::traits::Generator;
use clap::Parser;

// Package Imports
use alchemy::config::config;
use alchemy::config::recursive::RefillType;
use alchemy::errors::ParsingError;
use alchemy::utils::{run_experiment, read_inputs, string_to_term};
use alchemy::cli::Cli;
use alchemy::lambda::soup::LambdaSoup;

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

    // TODO: Genericize this to a trait, have the config specify an expression type
    // and implement a generic soup with it's associated config params
    let mut soup = LambdaSoup::from_config(&config);

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
        // TODO: Make this a generic add_expressions function
        // to allow for any expression type to be added from the cli
        soup.add_lambda_expressions(expressions);
    } 
    // Or have the soup seeded with n expressions with the configured generator
    else {
        soup.seed_with_generator(config.sample_size);
    };

    // Setup the tape list vector to store each generations history
    let mut tape_list = Vec::new();

    
    // ------------- Recursive Experiment -------------
    // Iterate over each for n_generations
    for gen in 0..config.recursive_config.n_generations {
        println!("Generation {gen}");
        // If we have a polling interval configured, push our recordings to the tape struct list
        if let Some(polling_interval) = config.polling_interval {
            tape_list.push(soup.simulate_and_record(config.run_limit, polling_interval, config.verbose_logging));
        } 
        // Otherwise simulate normally without recording and print the soup at the end
        else {
            soup.simulate_for(config.run_limit, config.verbose_logging);
        }

        // Recursive wipeout, remove `wipeout_percent` of expressions from the soup
        let n_wipeout = soup.wipeout(config.recursive_config.wipeout_percent);
        
        // And repopulate with expressions of the specified refill_type
        match &config.recursive_config.refill_type {
            RefillType::ConfigGenerator => {
                // If configured generator was selected, add those expressions
                let repop_expressions = soup.generator.generate_n_particles(n_wipeout);
                println!("Injecting {:?}", repop_expressions);
                soup.expressions.extend(repop_expressions);
            }
            RefillType::CustomExpression => {
                // Or parse the provided `repopulation_expression` and fill the remaining slots
                if let Some(repop_expression) = &config.recursive_config.repopulation_expression {
                    let repop_expressions = repeat_n(string_to_term(repop_expression), n_wipeout);
                    println!("Injecting {:?}", repop_expressions);
                    soup.add_lambda_expressions(repop_expressions);
                } 
                // If a `repopulation_expression` was not provided but the CustomExpression type was used 
                // return an error
                else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData, 
                        ParsingError::NoRepopulationExpression
                    ));                
                }
            }
        }
        soup.print();
        println!("");
    }

    Ok(())
}