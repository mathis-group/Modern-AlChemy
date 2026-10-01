// Global Imports
use serde::{Serialize, Deserialize};

// Package Imports
use crate::config::generators::{b_tree_gen::BTreeGen, fontana_gen::FontanaGen};

/// Default struct for the GenConfig trait,
/// implemented by all generators
pub trait GenConfig {
    fn new() -> Self;
}

/// Configuration for the generators
#[warn(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Generator {
    /// Use the btree generator
    BTree(BTreeGen),

    /// Use Fontana's generator
    Fontana(FontanaGen),
}

impl Generator {
    /// Produce a new `Generator` struct with default values.
    pub fn new() -> Self {
        Self::BTree(BTreeGen::new())
    }
}