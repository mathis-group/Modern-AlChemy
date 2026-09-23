// Global Imports
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

// Package Imports
use crate::utils::{decode_hex, encode_hex};

/// Represents a seed for serde RNGs in the configuration file. Mostly here because we want
/// to ser/de to/from a hex string.
#[warn(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct ConfigSeed(pub Option<[u8; 32]>);

impl ConfigSeed {
    /// Get the seed item
    pub fn get(&self) -> [u8; 32] {
        self.0.unwrap_or(thread_rng().gen())
    }

    pub fn seed(&self) -> Option<[u8; 32]> {
        self.0
    }

    pub fn new(seed: [u8; 32]) -> Self {
        ConfigSeed(Some(seed))
    }

    pub fn blank() -> Self {
        ConfigSeed(None)
    }
}

/// Manually serialize [u8; 32] to a hex string
impl Serialize for ConfigSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(seed) = self.seed() {
            serializer.serialize_str(&encode_hex(&seed))
        } else {
            serializer.serialize_none()
        }
    }
}

/// Manually deserialize a hex string to [u8; 32]
///
/// SAFETY: `panic!`s when hex string is malformed or of odd length
impl<'de> Deserialize<'de> for ConfigSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let seed_string: Option<&str> = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(if let Some(s) = seed_string {
            let hexvec = decode_hex(s).unwrap();
            ConfigSeed::new(hexvec.try_into().unwrap())
        } else {
            ConfigSeed::blank()
        })
    }
}
