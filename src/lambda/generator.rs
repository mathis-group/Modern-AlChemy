// Global Imports

//Package Imports
use crate::config::generator::Generator as GeneratorConfig;
use crate::traits::Generator;
use crate::lambda::generators::{b_tree_gen::BTreeGen, fontana_gen::FontanaGen};
use crate::lambda::particle::LambdaParticle;

/// Marker layer: "this is a generator of lambda particles."
/// Carries no methods yet it's the home for lambda-specific
/// generator behavior when/if that appears.
/// We need it to have a single entry point for a particle specific
/// generator when multiple are available.
pub trait LambdaGen: Generator<LambdaParticle> {}

impl LambdaGen for BTreeGen {}
impl LambdaGen for FontanaGen {}

/// The closed set of lambda generators. This is what a Soup holds.
#[derive(Debug, Clone)]
pub enum LambdaGenerator {
    BTree(BTreeGen),
    Fontana(FontanaGen),
}

impl Generator<LambdaParticle> for LambdaGenerator {
    fn generate_n_particles(&mut self, n: usize) -> Vec<LambdaParticle> {
        match self {
            Self::BTree(g)   => g.generate_n_particles(n),
            Self::Fontana(g) => g.generate_n_particles(n),
        }
    }
}

impl LambdaGenerator {
    /// The single place a runtime config tag becomes a type.
    pub fn from_config(cfg: &GeneratorConfig) -> Self {
        match cfg {
            GeneratorConfig::BTree(c)   => Self::BTree(BTreeGen::from_config(c)),
            GeneratorConfig::Fontana(c) => Self::Fontana(FontanaGen::from_config(c)),
        }
    }
}