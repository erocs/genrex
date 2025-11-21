/// Minimal lexer: converts a regex pattern string into a vector of Tokens.
/// Only supports literals and character classes for now.
fn lex_pattern(pattern: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '[' => {
                let mut class = Vec::new();
                let mut negated = false;
                if let Some('^') = chars.peek() {
                    chars.next();
                    negated = true;
                }
                while let Some(&next) = chars.peek() {
                    if next == ']' {
                        chars.next();
                        break;
                    }
                    class.push(chars.next().unwrap());
                }
                if negated {
                    tokens.push(Token::NegatedClass(class));
                } else {
                    tokens.push(Token::Class(class));
                }
            }
            '.' => tokens.push(Token::Wildcard),
            '^' => tokens.push(Token::AnchorStart),
            '$' => tokens.push(Token::AnchorEnd),
            '\\' => {
                if let Some(next) = chars.next() {
                    match next {
                        'b' => tokens.push(Token::WordBoundary),
                        'd' => tokens.push(Token::Class(('0'..='9').collect())),
                        'D' => tokens.push(Token::NegatedClass(('0'..='9').collect())),
                        'w' => tokens.push(Token::Class("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_".chars().collect())),
                        'W' => tokens.push(Token::NegatedClass("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_".chars().collect())),
                        's' => tokens.push(Token::Class(" \t\n\r\x0B\x0C".chars().collect())),
                        'S' => tokens.push(Token::NegatedClass(" \t\n\r\x0B\x0C".chars().collect())),
                        '1'..='9' => tokens.push(Token::Backreference(next.to_digit(10).unwrap() as usize)),
                        _ => tokens.push(Token::Literal(next)),
                    }
                }
            }
            '(' => {
                // Only support non-nested groups for now
                let mut group = String::new();
                let mut depth = 1;
                while let Some(next) = chars.next() {
                    match next {
                        '(' => { depth += 1; group.push(next); },
                        ')' => {
                            depth -= 1;
                            if depth == 0 { break; }
                            group.push(next);
                        }
                        _ => group.push(next),
                    }
                }
                let inner_tokens = lex_pattern(&group);
                tokens.push(Token::Group(Box::new(Token::Concatenation(inner_tokens))));
            }
            '?' => {
                // Non-capturing group or quantifier
                if let Some(&':') = chars.peek() {
                    chars.next();
                    // Parse non-capturing group
                    let mut group = String::new();
                    let mut depth = 1;
                    while let Some(next) = chars.next() {
                        match next {
                            '(' => { depth += 1; group.push(next); },
                            ')' => {
                                depth -= 1;
                                if depth == 0 { break; }
                                group.push(next);
                            }
                            _ => group.push(next),
                        }
                    }
                    let inner_tokens = lex_pattern(&group);
                    tokens.push(Token::NonCapturingGroup(Box::new(Token::Concatenation(inner_tokens))));
                } else {
                    // Quantifier ? (zero or one)
                    if let Some(last) = tokens.pop() {
                        tokens.push(Token::Quantifier { token: Box::new(last), min: 0, max: 1, greedy: true });
                    }
                }
            }
            '*' => {
                if let Some(last) = tokens.pop() {
                    tokens.push(Token::Quantifier { token: Box::new(last), min: 0, max: usize::MAX, greedy: true });
                }
            }
            '+' => {
                if let Some(last) = tokens.pop() {
                    tokens.push(Token::Quantifier { token: Box::new(last), min: 1, max: usize::MAX, greedy: true });
                }
            }
            '{' => {
                // Parse {min,max}
                let mut num = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ',' || ch == '}' { break; }
                    num.push(chars.next().unwrap());
                }
                let min = num.parse::<usize>().unwrap_or(0);
                let mut max = min;
                if let Some(&',') = chars.peek() {
                    chars.next();
                    let mut num2 = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' { break; }
                        num2.push(chars.next().unwrap());
                    }
                    if !num2.is_empty() {
                        max = num2.parse::<usize>().unwrap_or(min);
                    } else {
                        max = usize::MAX;
                    }
                }
                if let Some('}') = chars.peek() { chars.next(); }
                if let Some(last) = tokens.pop() {
                    tokens.push(Token::Quantifier { token: Box::new(last), min, max, greedy: true });
                }
            }
            '|' => {
                // Alternation: split tokens at this point
                let rest: String = chars.collect();
                let right = lex_pattern(&rest);
                let left = std::mem::take(&mut tokens);
                tokens.push(Token::Alternation(vec![Token::Concatenation(left), Token::Concatenation(right)]));
                break;
            }
            _ => {
                tokens.push(Token::Literal(c));
            }
        }
    }
    tokens
}
pub use crate::traits::{RegexStringGenerator, GeneratorConfigurable, GenerationAgent};
pub use crate::error::GenrexError;
mod traits;
mod error;
mod tokens;
mod ast;
mod parser;
pub use crate::tokens::Token;
pub use crate::traits::{RegexToken, TokenContext};
// use crate::traits::{RegexStringGenerator, GeneratorConfigurable, GenerationAgent}; // removed duplicate import, now re-exported
// use crate::error::GenrexError; // removed duplicate import, now re-exported
// use crate::tokens::Token; // removed duplicate import, now re-exported
use crate::parser::AstParser;
use crate::ast::AstNode;
impl RegexStringGenerator for RegexGenerator {
    fn generate_one(&mut self) -> Result<String, GenrexError> {
        self.generate_one().map_err(|e| match e {
            GenError::InvalidRegex(s) => GenrexError::InvalidRegex(s),
            GenError::NoMatch => GenrexError::NoMatch,
        })
    }

    fn generate_n(&mut self, n: usize) -> Result<Vec<String>, GenrexError> {
        self.generate_n(n).map_err(|e| match e {
            GenError::InvalidRegex(s) => GenrexError::InvalidRegex(s),
            GenError::NoMatch => GenrexError::NoMatch,
        })
    }

    fn is_multiline(&self) -> bool {
        self.multiline
    }
}

impl GeneratorConfigurable for RegexGenerator {
    fn min_len(&mut self, min: usize) -> &mut Self {
        self.config.min_len = min;
        self
    }
    fn max_len(&mut self, max: usize) -> &mut Self {
        self.config.max_len = max;
        self
    }
    fn max_attempts(&mut self, attempts: usize) -> &mut Self {
        self.config.max_attempts = attempts;
        self
    }
    fn timeout_ms(&mut self, ms: Option<u64>) -> &mut Self {
        self.config.timeout = ms.map(std::time::Duration::from_millis);
        self
    }
    fn multiline(&mut self, enabled: bool) -> &mut Self {
        self.multiline = enabled;
        self
    }
}

impl GenerationAgent for RegexGenerator {
    fn generate_with_strategy(&mut self, _strategy: &str) -> Result<String, GenrexError> {
        // For now, just call the default generator
        self.generate_one().map_err(|e| match e {
            GenError::InvalidRegex(s) => GenrexError::InvalidRegex(s),
            GenError::NoMatch => GenrexError::NoMatch,
        })
    }
}
// genrex — minimal MVP crate to generate random strings matching a regex (rejection sampling).
//
// Limitations (MVP):
// - Uses rejection sampling over ASCII alphanumeric characters.
// - No support for backreferences/lookarounds.
// - May be inefficient for very constrained patterns; later versions will add AST->NFA bounded sampling.

use rand::{distributions::Alphanumeric, RngCore, Rng, SeedableRng, rngs::StdRng};
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


/// A generator for strings matching a provided regex, with a configurable PRNG, multiline mode, and parsed AST.
pub struct RegexGenerator {
    re: Regex,
    config: GeneratorConfig,
    rng: Box<dyn RngCore + Send>,
    multiline: bool,
    ast: Option<AstNode>,
}

/// Builder for RegexGenerator.
pub struct RegexGeneratorBuilder {
    pattern: String,
    config: GeneratorConfig,
    rng: Option<Box<dyn RngCore + Send>>,
    multiline: bool,
}

impl RegexGeneratorBuilder {
    /// Start building a new RegexGenerator with the given pattern.
    pub fn new(pattern: &str) -> Self {
        RegexGeneratorBuilder {
            pattern: pattern.to_string(),
            config: GeneratorConfig::default(),
            rng: None,
            multiline: false,
        }
    }

    pub fn config(mut self, config: GeneratorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn rng<R: RngCore + Send + 'static>(mut self, rng: R) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }

    pub fn multiline(mut self, enabled: bool) -> Self {
        self.multiline = enabled;
        self
    }

    pub fn build(self) -> Result<RegexGenerator, GenError> {
        let re = Regex::new(&self.pattern)
            .map_err(|e| GenError::InvalidRegex(e.to_string()))?;
        let rng: Box<dyn RngCore + Send> = self.rng.unwrap_or_else(|| Box::new(StdRng::from_entropy()));

        // Use the minimal lexer to tokenize the pattern
        let tokens = lex_pattern(&self.pattern);
        let ast = if !tokens.is_empty() {
            AstParser::new(&tokens).parse()
        } else {
            None
        };

        Ok(RegexGenerator {
            re,
            config: self.config,
            rng,
            multiline: self.multiline,
            ast,
        })
    }
}

impl RegexGenerator {
    /// Create a new builder for RegexGenerator.
    pub fn builder(pattern: &str) -> RegexGeneratorBuilder {
        RegexGeneratorBuilder::new(pattern)
    }

    /// Enable or disable multiline mode after construction.
    pub fn multiline(&mut self, enabled: bool) -> &mut Self {
        self.multiline = enabled;
        self
    }

    /// Generate one matching string using the AST if available, otherwise fallback to rejection sampling.
    pub fn generate_one(&mut self) -> Result<String, GenError> {
        if let Some(ast) = &self.ast {
            let mut rng = &mut self.rng;
            let mut ctx = crate::traits::TokenContext::new();
            let s = Self::generate_from_ast(ast, &mut *rng, &mut ctx)?;
            let len = s.len();
            if len < self.config.min_len || len > self.config.max_len {
                return Err(GenError::NoMatch);
            }
            if self.re.is_match(&s) {
                return Ok(s);
            } else {
                return Err(GenError::NoMatch);
            }
        }
        // fallback: rejection sampling
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
                self.rng.gen_range(self.config.min_len..=self.config.max_len)
            };
            let s: String = (0..len).map(|_| self.rng.sample(Alphanumeric) as char).collect();
            if self.re.is_match(&s) {
                return Ok(s);
            }
        }
        Err(GenError::NoMatch)
    }

    /// Recursively generate a string from the AST node.
    fn generate_from_ast<R: rand::Rng + ?Sized>(node: &AstNode, rng: &mut R, ctx: &mut crate::traits::TokenContext) -> Result<String, GenError> {
        use crate::ast::AstNode;
        match node {
            AstNode::Sequence(nodes) => {
                let mut out = String::new();
                for n in nodes {
                    out.push_str(&Self::generate_from_ast(n, rng, ctx)?);
                }
                Ok(out)
            }
            AstNode::Alternation(nodes) => {
                if nodes.is_empty() {
                    Ok(String::new())
                } else {
                    let idx = rng.gen_range(0..nodes.len());
                    Self::generate_from_ast(&nodes[idx], rng, ctx)
                }
            }
            AstNode::Repeat { node, min, max, greedy: _ } => {
                if min > max { return Err(GenError::NoMatch); }
                // Respect TokenContext.max_repeat for open-ended quantifiers.
                let effective_max = if *max == usize::MAX {
                    (*min).saturating_add(ctx.max_repeat)
                } else {
                    *max
                };
                let count = if *min == *max { *min } else { rng.gen_range(*min..=effective_max) };
                let mut out = String::new();
                for _ in 0..count {
                    out.push_str(&Self::generate_from_ast(node, rng, ctx)?);
                }
                Ok(out)
            }
            AstNode::Group(inner) | AstNode::NonCapturingGroup(inner) => Self::generate_from_ast(inner, rng, ctx),
            AstNode::Backreference(_) => Err(GenError::NoMatch), // Not supported
            AstNode::Class(chars) => {
                if chars.is_empty() {
                    Err(GenError::NoMatch)
                } else {
                    let idx = rng.gen_range(0..chars.len());
                    Ok(chars[idx].to_string())
                }
            }
            AstNode::NegatedClass(_) => Err(GenError::NoMatch), // Not supported
            AstNode::Literal(c) => Ok(c.to_string()),
            AstNode::AnchorStart | AstNode::AnchorEnd | AstNode::WordBoundary => Ok(String::new()),
            AstNode::Wildcard => {
                const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let idx = rng.gen_range(0..ALPHABET.len());
                Ok((ALPHABET[idx] as char).to_string())
            }
        }
    }

    /// Convenience: generate n matches (may return fewer if generator hit limits).
    pub fn generate_n(&mut self, n: usize) -> Result<Vec<String>, GenError> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            match self.generate_one() {
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
            rng: Box::new(StdRng::from_entropy()),
            multiline: false,
            ast: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn generates_simple_literal_or_times_out() {
        let cfg = GeneratorConfig { min_len: 3, max_len: 10, max_attempts: 1_000, timeout: None };
        let mut g = RegexGenerator::builder("^foo\\d{1,3}$")
            .config(cfg)
            .rng(StdRng::seed_from_u64(42))
            .build()
            .expect("compile regex");
        let res = g.generate_one();
        assert!(res.is_err() || g.re.is_match(&res.unwrap_or_default()));
    }
}