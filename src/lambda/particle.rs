// Global Imports
use core::fmt;
use std::fmt::{Debug, Display};
use lambda_calculus::{app, Term};

// Package Imports
use crate::traits::Particle;
use crate::errors::ParsingError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaParticle {
    pub expr: Term,
    pub recursive: bool,
}

impl LambdaParticle {
    pub fn get_underlying_term(&self) -> &Term {
        &self.expr
    }

    pub fn is_recursive(&self) -> bool {
        self.recursive
    }
}

impl Particle for LambdaParticle {
    fn compose(&self, other: &Self) -> Self {
        LambdaParticle {
            expr: lambda_calculus::app!(self.expr.clone(), other.expr.clone()),
            recursive: false,
        }
    }

    fn is_isomorphic_to(&self, other: &Self) -> bool {
        self.expr.is_isomorphic_to(&other.expr)
    }

    fn parse(s: &String) -> Result<Self, ParsingError> {
        let expr = lambda_calculus::parse(s, lambda_calculus::Classic)
            .map_err(|e| ParsingError::InvalidExpression(format!("{s}: {e}")))?;
        Ok(LambdaParticle { 
            expr, 
            recursive: false 
        })
    }

}

impl fmt::Display for LambdaParticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{:?}", self.expr), f)
    }
}