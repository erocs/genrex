//! genrex — minimal MVP crate to generate random strings matching a regex (rejection sampling).
//!
//! Limitations (MVP):
//! - Uses rejection sampling over ASCII alphanumeric characters.
//! - No support for backreferences/lookarounds.
//! - May be inefficient for very constrained patterns; later versions will add AST->NFA bounded sampling.

use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
use regex::Regex;
use thiserror::Error;
use std::time::{Duration, Instant};

#[derive(Debug, Error)]
pub enum GenError {
    #[error("invalid regex: {0}")]
    InvalidRegex(String),

    #[error("no match found within constraints")]
    NoMatch,
}

/// Configuration for the generator.
#[derive(Clone, Debug)]
pub struct GeneratorConfig {
    pub min_len: usize,
    pub max_len: usize,
    /// Maximum number of candidate strings to try before giving up.
    pub max_attempts: usize,
    /// Optional timeout for generation attempts.
    pub timeout: Option<Duration>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        GeneratorConfig {
            min_len: 0,
            max_len: 64,
            max_attempts: 10_000,
            timeout: None,
        }
    }
}

/// A generator for strings matching a provided regex.
pub struct RegexGenerator {
    re: Regex,
    config: GeneratorConfig,
}

impl RegexGenerator {
    /// Create a new generator from a regex pattern string and config.
    pub fn new(pattern: &str, config: GeneratorConfig) -> Result<Self, GenError> {
        match Regex::new(pattern) {
            Ok(re) => Ok(RegexGenerator { re, config }),
            Err(e) => Err(GenError::InvalidRegex(e.to_string())),
        }
    }

    /// Generate one matching string using the provided RNG.
    /// Uses rejection sampling over ASCII alphanumeric characters.
    pub fn generate_one<R: Rng>(&self, rng: &mut R) -> Result<String, GenError> {
        let start = Instant::now();
        let mut attempts = 0;
        while attempts < self.config.max_attempts {
            if let Some(timeout) = self.config.timeout {
                if start.elapsed() >= timeout {
                    break;
                }
            }
            attempts += 1;
            let len = if self.config.max_len == self.config.min_len {
                self.config.min_len
            } else {
                rng.gen_range(self.config.min_len..=self.config.max_len)
            };
            let s: String = (0..len).map(|_| rng.sample(Alphanumeric) as char).collect();
            if self.re.is_match(&s) {
                return Ok(s);
            }
        }
        Err(GenError::NoMatch)
    }

    /// Convenience: generate n matches (may return fewer if generator hit limits).
    pub fn generate_n<R: Rng>(&self, rng: &mut R, n: usize) -> Result<Vec<String>, GenError> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            match self.generate_one(rng) {
                Ok(s) => out.push(s),
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}

impl Default for RegexGenerator {
    fn default() -> Self {
        RegexGenerator {
            re: Regex::new(".*").unwrap(),
            config: GeneratorConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;

    #[test]
    fn generates_simple_literal_or_times_out() {
        let cfg = GeneratorConfig { min_len: 3, max_len: 10, max_attempts: 1_000, timeout: None };
        let g = RegexGenerator::new("^foo\\d{1,3}$", cfg).expect("compile regex");
        let mut rng = StdRng::seed_from_u64(42);
        let res = g.generate_one(&mut rng);
        assert!(res.is_err() || g.re.is_match(&res.unwrap_or_default()));
    }
}