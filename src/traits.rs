// Global Imports
use std::fmt::Display;

// Package Imports
use crate::errors::ParsingError;

pub trait Particle: Sized {
    fn compose(&self, other: &Self) -> Self;

    fn is_isomorphic_to(&self, other: &Self) -> bool;

    fn parse(s: &String) -> Result<Self, ParsingError>;
}

pub trait Collider<P>
where
    P: Particle,
{
    type Product: Residue<P> + Display + Clone;
    type Error: std::error::Error + Display + Clone;

    fn collide(&self, left: P, right: P) -> Result<Self::Product, Self::Error>;
}

pub trait Generator<P>
where 
    P: Particle,
{
    fn generate_n_particles(&mut self, n: usize) -> Vec<P>;
}

pub trait Residue<P>
where
    P: Particle,
{
    fn particles(&self) -> impl Iterator<Item = P>;
    fn count(&self) -> usize;
}