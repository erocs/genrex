use rand::Rng;

use crate::error::GenrexError;

/// Trait for a regex AST token node.
pub trait RegexToken {
    /// Generate a string matching this token, using the provided PRNG and context.
    fn generate<R: Rng + ?Sized>(&self, rng: &mut R, ctx: &mut TokenContext) -> Result<String, GenrexError>;

    /// Returns a human-readable description of the token.
    fn describe(&self) -> String;
}

/// Context for token generation (captures, backreferences, etc).
pub struct TokenContext {
    /// Maximum additional repeats to use when a quantifier has an open-ended max (usize::MAX).
    pub max_repeat: usize,
    /// Captured group strings in appearance order; index = group number - 1 for backreferences.
    pub captures: Vec<String>,
}
 
impl TokenContext {
    /// Create a TokenContext with the default max_repeat.
    pub fn new() -> Self {
        TokenContext::new_with_max_repeat(32)
    }
 
    /// Create a TokenContext with a caller-provided max_repeat.
    pub fn new_with_max_repeat(max_repeat: usize) -> Self {
        TokenContext {
            max_repeat,
            captures: Vec::new(),
        }
    }
}
// Traits for the genrex library API interface.


// use crate::error::GenrexError; // removed duplicate/unnecessary import


/// Trait for generating random strings matching a regex.
pub trait RegexStringGenerator {
    /// Generate a single string matching the regex, or an error.
    ///
    /// # Errors
    /// Returns `GenrexError` if generation fails (invalid regex, no match, timeout, etc).
    fn generate_one(&mut self) -> Result<String, GenrexError>;

    /// Returns true if multiline mode is enabled.
    fn is_multiline(&self) -> bool;

    /// Generate `n` strings matching the regex, or an error.
    ///
    /// # Errors
    /// Returns `GenrexError` if generation fails (invalid regex, no match, timeout, etc).
    fn generate_n(&mut self, n: usize) -> Result<Vec<String>, GenrexError>;
}

/// Trait for configuring the generator.
pub trait GeneratorConfigurable {
    /// Set the minimum length for generated strings.
    fn min_len(&mut self, min: usize) -> &mut Self;

    /// Set the maximum length for generated strings.
    fn max_len(&mut self, max: usize) -> &mut Self;

    /// Set the maximum number of attempts for generation.
    fn max_attempts(&mut self, attempts: usize) -> &mut Self;

    /// Set an optional timeout (in milliseconds).
    fn timeout_ms(&mut self, ms: Option<u64>) -> &mut Self;

    /// Enable or disable multiline mode.
    fn multiline(&mut self, enabled: bool) -> &mut Self;
}

/// Trait for advanced generation strategies (future extensibility).
pub trait GenerationAgent {
    /// Generate a string using a custom strategy.
    ///
    /// # Errors
    /// Returns `GenrexError` if generation fails or the strategy is unsupported.
    fn generate_with_strategy(&mut self, strategy: &str) -> Result<String, GenrexError>;
}
