// Global Imports
use serde::{Serialize, Deserialize};
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Standardization {
    Prefix,
    Postfix,
    None,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Experiment {
    // entropy.rs
    EntropyAndFailures,
    SyncEntropyAndFailures,
    EntropyTimeSeries,

    // search_by_behavior.rs
    XorsetSearch,
    NotXorsetSearch,

    // distribution.rs
    DistributionTimeSeries,

    // magic_test_function.rs
    AddSearchNoTest,
    AddSearchWithTest,
    SuccSearchWithTest,

    // kinetics.rs
    SuccKinetics,

    // discovery.rs
    MeasureInitialPopulation,
    AddSccPopulationFromRandomInputs,
    AddSccPopulationFromSkiInputs,
    AddSccPopulationFromSkipInputs,
    SccPopulationFromRandomInputsWithTests,
    AddPopulationFromRandomInputsWithTests,
    AddPopulationFromRandomInputsWithAddSuccTests,
    SccPopulationFromSkiInputsWithTests,
    AddPopulationFromSkiInputsWithTests,
    AddPopulationFromSkiInputsWithAddSuccTests,
    AddPopulationFromSkiInputsWithBatchedAddSuccTests,
    AddtwoPopulationFromSkiInputsWithAddtwoTests,
    AddPopulationFromSkipInputsWithAddSuccTests,
}