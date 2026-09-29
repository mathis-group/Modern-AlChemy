// Global Imports
use std::fmt::{Debug, Display};

// Package Imports
use crate::soupercollider::Soup;
use crate::traits::{Particle, Collider, Generator};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecordingType { 
    None, 
    All, 
    SuccessOnly 
}

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

pub struct Tape<P, C, G> 
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    pub soup_history: Vec<Soup<P, C, G>>,
    pub reaction_record: Vec<ReactionRecord<P>>
}

impl<P, C, G> Tape<P, C, G>
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone,
{
    pub fn final_state(&self) -> Option<&Soup<P, C, G>> {
        self.history().last()
    }

    pub fn history(&self) -> impl Iterator<Item = &Soup<P, C, G>> {
        self.soup_history.iter()
    }
}