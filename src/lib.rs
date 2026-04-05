/// Minimal lexer: converts a regex pattern string into a vector of Tokens.
/// Only supports literals and character classes for now.
fn lex_pattern(pattern: &str, next_group: &mut usize) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '[' => {
                let mut class = Vec::new();
                let mut negated = false;
                if chars.peek() == Some(&'^') {
                    chars.next();
                    negated = true;
                }
                while let Some(&next) = chars.peek() {
                    if next == ']' {
                        chars.next();
                        break;
                    }
                    let c = chars.next().unwrap();
                    // Check for range syntax `c-X` where X is not `]`.
                    if chars.peek() == Some(&'-') {
                        let mut lookahead = chars.clone();
                        lookahead.next(); // skip '-'
                        match lookahead.peek() {
                            Some(&']') | None => {
                                // '-' is a literal at the end of the class.
                                class.push(c);
                            }
                            Some(_) => {
                                chars.next(); // consume '-'
                                let end = chars.next().unwrap();
                                for cp in (c as u32)..=(end as u32) {
                                    if let Some(ch) = char::from_u32(cp) {
                                        class.push(ch);
                                    }
                                }
                            }
                        }
                    } else {
                        class.push(c);
                    }
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
                // Detect (?:…) non-capturing group before assigning a group id.
                // We must peek two chars ahead: '?' then ':'.
                let is_non_capturing = chars.peek() == Some(&'?') && {
                    let mut lookahead = chars.clone();
                    lookahead.next(); // skip '?'
                    lookahead.peek() == Some(&':')
                };
                if is_non_capturing {
                    chars.next(); // consume '?'
                    chars.next(); // consume ':'
                }
                // Reserve the group id now, before parsing inner content, so that
                // nested groups receive higher ids than their enclosing group.
                let group_id = if !is_non_capturing {
                    let id = *next_group;
                    *next_group += 1;
                    id
                } else {
                    0 // unused for non-capturing groups
                };
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
                let inner_tokens = lex_pattern(&group, next_group);
                if is_non_capturing {
                    tokens.push(Token::NonCapturingGroup(Box::new(Token::Concatenation(inner_tokens))));
                } else {
                    tokens.push(Token::Group(Box::new(Token::Concatenation(inner_tokens)), group_id));
                }
            }
            '?' => {
                // Quantifier ? (zero or one). (?:…) non-capturing groups are handled
                // entirely by the '(' branch above, so '?' here is always a quantifier.
                if let Some(last) = tokens.pop() {
                    // Support lazy modifier "??" (non-greedy for the '?' quantifier).
                    let mut greedy = true;
                    if chars.peek() == Some(&'?') {
                        chars.next();
                        greedy = false;
                    }
                    tokens.push(Token::Quantifier { token: Box::new(last), min: 0, max: 1, greedy });
                }
            }
            '*' => {
                if let Some(last) = tokens.pop() {
                    // Detect lazy modifier "*?" -> non-greedy
                    let mut greedy = true;
                    if let Some(&'?') = chars.peek() {
                        chars.next();
                        greedy = false;
                    }
                    tokens.push(Token::Quantifier { token: Box::new(last), min: 0, max: usize::MAX, greedy });
                }
            }
            '+' => {
                if let Some(last) = tokens.pop() {
                    // Detect lazy modifier "+?" -> non-greedy
                    let mut greedy = true;
                    if let Some(&'?') = chars.peek() {
                        chars.next();
                        greedy = false;
                    }
                    tokens.push(Token::Quantifier { token: Box::new(last), min: 1, max: usize::MAX, greedy });
                }
            }
            '{' => {
                // Parse {min,max}
                let mut num = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ',' || ch == '}' { break; }
                    num.push(chars.next().unwrap());
                }
                let min = num.trim().parse::<usize>().unwrap_or(0);
                let mut max = min;
                if let Some(&',') = chars.peek() {
                    chars.next();
                    let mut num2 = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' { break; }
                        num2.push(chars.next().unwrap());
                    }
                    if !num2.trim().is_empty() {
                        max = num2.trim().parse::<usize>().unwrap_or(min);
                    } else {
                        max = usize::MAX;
                    }
                }
                if let Some('}') = chars.peek() { chars.next(); }
                if let Some(last) = tokens.pop() {
                    // Detect lazy modifier "{m,n}?" -> non-greedy
                    let mut greedy = true;
                    if let Some(&'?') = chars.peek() {
                        chars.next();
                        greedy = false;
                    }
                    tokens.push(Token::Quantifier { token: Box::new(last), min, max, greedy });
                }
            }
            '|' => {
                // Alternation: split tokens at this point
                let rest: String = chars.collect();
                let right = lex_pattern(&rest, next_group);
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
pub use crate::tokens::Token;
pub use crate::traits::{RegexToken, TokenContext};

/// Returns the minimum number of characters this token will generate.
fn token_min_len(token: &Token) -> usize {
    match token {
        Token::Literal(_) | Token::Class(_) | Token::NegatedClass(_) | Token::Wildcard => 1,
        Token::Concatenation(tokens) => tokens.iter().map(token_min_len).sum(),
        Token::Alternation(choices) => choices.iter().map(token_min_len).min().unwrap_or(0),
        Token::Quantifier { token, min, .. } => min * token_min_len(token),
        Token::Group(inner, _) | Token::NonCapturingGroup(inner) => token_min_len(inner),
        Token::AnchorStart | Token::AnchorEnd | Token::WordBoundary | Token::Backreference(_) => 0,
    }
}

/// Returns the maximum number of characters this token can generate, or `None` if unbounded.
fn token_max_len(token: &Token) -> Option<usize> {
    match token {
        Token::Literal(_) | Token::Class(_) | Token::NegatedClass(_) | Token::Wildcard => Some(1),
        Token::Concatenation(tokens) => {
            let mut total = 0usize;
            for t in tokens {
                total = total.saturating_add(token_max_len(t)?);
            }
            Some(total)
        }
        Token::Alternation(choices) => {
            let mut best = 0usize;
            for c in choices {
                best = best.max(token_max_len(c)?);
            }
            Some(best)
        }
        Token::Quantifier { token, max, .. } if *max == usize::MAX => None,
        Token::Quantifier { token, max, .. } => {
            Some(token_max_len(token)?.saturating_mul(*max))
        }
        Token::Group(inner, _) | Token::NonCapturingGroup(inner) => token_max_len(inner),
        Token::AnchorStart | Token::AnchorEnd | Token::WordBoundary | Token::Backreference(_) => Some(0),
    }
}
impl RegexStringGenerator for RegexGenerator {
    fn generate_one(&mut self) -> Result<String, GenrexError> {
        self.generate_one()
    }

    fn generate_n(&mut self, n: usize) -> Result<Vec<String>, GenrexError> {
        self.generate_n(n)
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
        self.generate_one()
    }
}

use rand::{distributions::Alphanumeric, RngCore, Rng, SeedableRng, rngs::StdRng};
use regex::Regex;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global verbose flag — when enabled the crate will print internal warnings and rejection diagnostics.
pub static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Convenience to set verbosity from binaries.
pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
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


/// A generator for strings matching a provided regex, with a configurable PRNG, multiline mode, and parsed tokens.
pub struct RegexGenerator {
    re: Regex,
    /// True when `allow_backrefs` caused `re` to be substituted with `.*`.
    /// In that case regex-based validation and rejection sampling are skipped;
    /// only token-based generation is used.
    re_is_fallback: bool,
    config: GeneratorConfig,
    rng: Box<dyn RngCore + Send>,
    multiline: bool,
    tokens: Option<Vec<Token>>,
    /// Number of capturing groups discovered by the lexer.
    group_count: usize,
}

/// Builder for RegexGenerator.
pub struct RegexGeneratorBuilder {
    pattern: String,
    config: GeneratorConfig,
    rng: Option<Box<dyn RngCore + Send>>,
    multiline: bool,
    /// When true, skip strict `regex::Regex` compilation errors (useful to allow backreferences);
    /// the generator will fall back to a permissive `.*` matcher and rely on token-generation instead.
    allow_backrefs: bool,
}

impl RegexGeneratorBuilder {
    /// Start building a new RegexGenerator with the given pattern.
    pub fn new(pattern: &str) -> Self {
        RegexGeneratorBuilder {
            pattern: pattern.to_string(),
            config: GeneratorConfig::default(),
            rng: None,
            multiline: false,
            allow_backrefs: false,
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

    /// Allow patterns that the `regex` crate cannot compile (e.g., backreferences).
    /// When enabled, the generator will skip failing `Regex::new` and use a permissive matcher.
    pub fn allow_backrefs(mut self) -> Self {
        self.allow_backrefs = true;
        self
    }

    pub fn build(self) -> Result<RegexGenerator, GenrexError> {
        // Try to compile the regex; if allow_backrefs is enabled, fall back to a permissive matcher on error.
        let (re, re_is_fallback) = if !self.allow_backrefs {
            (Regex::new(&self.pattern).map_err(|e| GenrexError::InvalidRegex(e.to_string()))?, false)
        } else {
            match Regex::new(&self.pattern) {
                Ok(r) => (r, false),
                Err(_) => {
                    if VERBOSE.load(Ordering::Relaxed) {
                        eprintln!("warning: pattern failed to compile with regex crate; proceeding with token-based generation (allow_backrefs enabled)");
                    }
                    (Regex::new(".*").unwrap(), true)
                }
            }
        };

        let rng: Box<dyn RngCore + Send> = self.rng.unwrap_or_else(|| Box::new(StdRng::from_entropy()));

        let mut next_group: usize = 1;
        let tokens = lex_pattern(&self.pattern, &mut next_group);
        let tokens = if tokens.is_empty() { None } else { Some(tokens) };

        // Auto-raise max_len so the pattern's required output length is never
        // unconditionally rejected by the length filter.
        let mut config = self.config;
        if let Some(toks) = &tokens {
            let pattern_min: usize = toks.iter().map(token_min_len).sum();
            if pattern_min > config.max_len {
                config.max_len = pattern_min;
            }
            // For bounded patterns, also raise max_len to the pattern's max so the
            // full quantifier range is reachable (e.g. \w{1,100} can produce 100 chars).
            let pattern_max = toks.iter().try_fold(0usize, |acc, t| {
                token_max_len(t).map(|n| acc.saturating_add(n))
            });
            if let Some(pat_max) = pattern_max {
                if pat_max > config.max_len {
                    config.max_len = pat_max;
                }
            }
        }

        Ok(RegexGenerator {
            re,
            re_is_fallback,
            config,
            rng,
            multiline: self.multiline,
            tokens,
            group_count: next_group.saturating_sub(1),
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

    /// Generate one matching string using lexer tokens if available, otherwise fallback to rejection sampling.
    pub fn generate_one(&mut self) -> Result<String, GenrexError> {
        // 1) Token-based generation (preferred)
        if let Some(tokens) = &self.tokens {
            let start = Instant::now();
            let mut attempts = 0usize;
            while attempts < self.config.max_attempts {
                if let Some(timeout) = self.config.timeout {
                    if start.elapsed() >= timeout { break; }
                }
                attempts += 1;
                let mut ctx = crate::traits::TokenContext::new();
                // Pre-size captures so backreferences referring to future groups are recorded
                // as unresolved placeholders instead of causing immediate errors.
                ctx.captures.resize(self.group_count, None);
                let rng = &mut self.rng;
                let mut out = String::new();
                let mut ok = true;
                for t in tokens {
                    // inform context of current output length so tokens (especially Backreference)
                    // can record unresolved placeholders relative to the current byte position.
                    ctx.set_output_len(out.len());
                    match t.generate(&mut *rng, &mut ctx) {
                        Ok(s) => out.push_str(&s),
                        Err(_) => { ok = false; break; }
                    }
                }
                if !ok { continue; }
                // If any unresolved backreferences were recorded, attempt to resolve them now.
                if !ctx.unresolved_refs.is_empty() {
                    let mut unresolved_missing = false;
                    // Sort by position to insert in-order (they should already be in order but ensure correctness).
                    ctx.unresolved_refs.sort_by_key(|(pos, _)| *pos);
                    let mut final_out = out.clone();
                    let mut offset = 0usize;
                    for (pos, gid) in &ctx.unresolved_refs {
                        if let Some(cap) = ctx.get_capture(*gid) {
                            let insert_pos = (*pos).saturating_add(offset);
                            if insert_pos <= final_out.len() {
                                final_out.insert_str(insert_pos, &cap);
                                offset += cap.len();
                            } else {
                                // Unexpected: recorded position out of bounds -> treat as unresolved.
                                unresolved_missing = true;
                                break;
                            }
                        } else {
                            unresolved_missing = true;
                            break;
                        }
                    }
                    if unresolved_missing {
                        // Unable to resolve forward refs for this candidate; try again.
                        if VERBOSE.load(Ordering::Relaxed) {
                            eprintln!("candidate rejected (unresolved backreference) during resolution: {}", out);
                        }
                        continue;
                    } else {
                        out = final_out;
                    }
                }
                let len = out.len();
                if len < self.config.min_len || len > self.config.max_len {
                    if VERBOSE.load(Ordering::Relaxed) {
                        eprintln!("candidate rejected (len {} not in {}..={}): {}", len, self.config.min_len, self.config.max_len, out);
                    }
                    continue;
                }
                if self.re_is_fallback || self.re.is_match(&out) {
                    return Ok(out);
                } else {
                    if VERBOSE.load(Ordering::Relaxed) {
                        eprintln!("candidate rejected (regex mismatch): {}", out);
                    }
                    continue;
                }
            }
        }

        // 2) Fallback: rejection sampling (only when a real regex is available for validation).
        // When re_is_fallback is true the compiled regex is `.*` and accepts anything, so
        // rejection sampling would return arbitrary strings — skip it entirely.
        if self.re_is_fallback {
            return Err(GenrexError::NoMatch);
        }
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
        Err(GenrexError::NoMatch)
    }

    /// Convenience: generate n matches (may return fewer if generator hit limits).
    pub fn generate_n(&mut self, n: usize) -> Result<Vec<String>, GenrexError> {
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
            re_is_fallback: false,
            config: GeneratorConfig::default(),
            rng: Box::new(StdRng::from_entropy()),
            multiline: false,
            tokens: None,
            group_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn fixed_quantifier_exceeding_default_max_len() {
        // \w{100} requires 100 chars; default max_len is 64 but should be auto-raised.
        let mut g = RegexGenerator::builder("\\w{100}")
            .rng(StdRng::seed_from_u64(1))
            .build()
            .expect("compile regex");
        let s = g.generate_one().expect("should generate 100 chars");
        assert_eq!(s.len(), 100, "\\w{{100}} must produce exactly 100 characters");
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    }

    #[test]
    fn range_quantifier_uses_full_range() {
        // \w{1,100} should produce varied lengths across the full [1,100] range.
        // Use a single generator (one regex compilation) and call generate_one many times.
        let mut g = RegexGenerator::builder("\\w{1,100}")
            .rng(StdRng::seed_from_u64(42))
            .build()
            .expect("compile regex");
        let mut saw_above_64 = false;
        let mut saw_above_1 = false;
        for _ in 0..100 {
            let s = g.generate_one().expect("should generate");
            let len = s.len();
            assert!((1..=100).contains(&len), "length {} out of [1,100]", len);
            if len > 64 { saw_above_64 = true; }
            if len > 1  { saw_above_1  = true; }
        }
        assert!(saw_above_1,  "\\w{{1,100}} must not always produce length 1");
        assert!(saw_above_64, "\\w{{1,100}} should sometimes produce strings longer than 64 chars");
    }

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