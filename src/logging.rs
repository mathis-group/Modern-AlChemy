// Global Imports
use std::fmt::{Debug, Display};

// Package Imports
use crate::soupercollider::Soup;
use crate::traits::{Particle, Collider, Generator};

/// A single logged reaction event, capturing parents, products, and outcome.
#[derive(Debug, Clone)]
pub struct ReactionRecord<P: Clone> {
    pub step: usize,
    pub left: P,
    pub right: P,
    pub products: Vec<P>,
    pub success: bool,
    pub error: Option<String>,
}

pub struct Tape<P, C, G> {
    pub soup: Soup<P, C, G>,
    pub history: Vec<Soup<P, C, G>>,
    pub polling_interval: usize,
}

impl<P, C, G> Tape<P, C, G>
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    pub fn final_state(&self) -> &Soup<P, C, G> {
        &self.soup
    }

    pub fn history(&self) -> impl Iterator<Item = &Soup<P, C, G>> {
        self.history.iter()
    }

    pub fn polling_interval(&self) -> usize {
        self.polling_interval
    }
}