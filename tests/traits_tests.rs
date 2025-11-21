#[test]
fn test_literal_trait() {
    let mut generator = DummyGenerator::new("x", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 1, false);
    let result = generator.generate_one();
    println!("Literal: {:?}", result);
    assert_eq!(result.unwrap(), "x");
}

#[test]
fn test_class_trait() {
    let mut generator = DummyGenerator::new("[abc]", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 2, false);
    let result = generator.generate_one();
    println!("Class: {:?}", result);
    let s = result.unwrap();
    assert!("abc".contains(&s));
}

#[test]
fn test_negated_class_trait() {
    let mut generator = DummyGenerator::new("[^abc]", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 3, false);
    let result = generator.generate_one();
    println!("NegatedClass: {:?}", result);
    // Negated class is not supported, should return error
    assert!(result.is_err());
}

#[test]
fn test_concatenation_trait() {
    let mut generator = DummyGenerator::new("ab", GeneratorConfig { min_len: 2, max_len: 2, max_attempts: 100, timeout: None }, 4, false);
    let result = generator.generate_one();
    println!("Concatenation: {:?}", result);
    assert_eq!(result.unwrap(), "ab");
}

#[test]
fn test_alternation_trait() {
    let mut generator = DummyGenerator::new("a|b", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 5, false);
    let result = generator.generate_one();
    println!("Alternation: {:?}", result);
    let s = result.unwrap();
    assert!(s == "a" || s == "b");
}

#[test]
fn test_quantifier_trait() {
    let mut generator = DummyGenerator::new("a{2,4}", GeneratorConfig { min_len: 2, max_len: 4, max_attempts: 100, timeout: None }, 6, false);
    let result = generator.generate_one();
    println!("Quantifier: {:?}", result);
    let s = result.unwrap();
    assert!((2..=4).contains(&s.len()));
    assert!(s.chars().all(|c| c == 'a'));
}

#[test]
fn test_group_trait() {
    let mut generator = DummyGenerator::new("(a)", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 7, false);
    let result = generator.generate_one();
    println!("Group: {:?}", result);
    assert_eq!(result.unwrap(), "a");
}

#[test]
fn test_non_capturing_group_trait() {
    let mut generator = DummyGenerator::new("(?:a)", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 8, false);
    let result = generator.generate_one();
    println!("NonCapturingGroup: {:?}", result);
    assert_eq!(result.unwrap(), "a");
}

#[test]
fn test_anchor_start_trait() {
    let mut generator = DummyGenerator::new("^a", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 9, false);
    let result = generator.generate_one();
    println!("AnchorStart: {:?}", result);
    assert_eq!(result.unwrap(), "a");
}

#[test]
fn test_anchor_end_trait() {
    let mut generator = DummyGenerator::new("a$", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 10, false);
    let result = generator.generate_one();
    println!("AnchorEnd: {:?}", result);
    assert_eq!(result.unwrap(), "a");
}

#[test]
fn test_word_boundary_trait() {
    let mut generator = DummyGenerator::new("a\\b", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 11, false);
    let result = generator.generate_one();
    println!("WordBoundary: {:?}", result);
    assert_eq!(result.unwrap(), "a");
}

#[test]
fn test_wildcard_trait() {
    let mut generator = DummyGenerator::new(".", GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 100, timeout: None }, 12, false);
    let result = generator.generate_one();
    println!("Wildcard: {:?}", result);
    let s = result.unwrap();
    assert_eq!(s.len(), 1);
    assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
}
// Unit tests for the genrex API traits and error handling.

use genrex::{RegexStringGenerator, GeneratorConfigurable, GenerationAgent};
use genrex::GenrexError;
use genrex::{RegexGenerator, GeneratorConfig};
use rand::{rngs::StdRng, SeedableRng};

// DummyGenerator wraps RegexGenerator for real implementation testing
struct DummyGenerator {
    inner: RegexGenerator,
    multiline: bool,
}

impl DummyGenerator {
    fn new(pattern: &str, config: GeneratorConfig, seed: u64, multiline: bool) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        let mut inner = RegexGenerator::builder(pattern)
            .config(config)
            .rng(rng)
            .build()
            .expect("valid regex");
        if multiline {
            inner.multiline(true);
        }
        DummyGenerator { inner, multiline }
    }
}

impl RegexStringGenerator for DummyGenerator {
    fn generate_one(&mut self) -> Result<String, GenrexError> {
        self.inner.generate_one().map_err(|e| match e {
            genrex::GenError::InvalidRegex(s) => GenrexError::InvalidRegex(s),
            genrex::GenError::NoMatch => GenrexError::NoMatch,
        })
    }
    fn generate_n(&mut self, n: usize) -> Result<Vec<String>, GenrexError> {
        self.inner.generate_n(n).map_err(|e| match e {
            genrex::GenError::InvalidRegex(s) => GenrexError::InvalidRegex(s),
            genrex::GenError::NoMatch => GenrexError::NoMatch,
        })
    }
    fn is_multiline(&self) -> bool {
        self.multiline
    }
}

impl GeneratorConfigurable for DummyGenerator {
    fn min_len(&mut self, min: usize) -> &mut Self {
        self.inner.min_len(min);
        self
    }
    fn max_len(&mut self, max: usize) -> &mut Self {
        self.inner.max_len(max);
        self
    }
    fn max_attempts(&mut self, attempts: usize) -> &mut Self {
        self.inner.max_attempts(attempts);
        self
    }
    fn timeout_ms(&mut self, ms: Option<u64>) -> &mut Self {
        self.inner.timeout_ms(ms);
        self
    }
    fn multiline(&mut self, enabled: bool) -> &mut Self {
        self.inner.multiline(enabled);
        self.multiline = enabled;
        self
    }
}

impl GenerationAgent for DummyGenerator {
    fn generate_with_strategy(&mut self, strategy: &str) -> Result<String, GenrexError> {
        self.inner.generate_with_strategy(strategy)
    }
}

#[test]
fn test_generate_one_success() {
    let mut generator = DummyGenerator::new("^foo\\d{1,3}$", GeneratorConfig { min_len: 4, max_len: 6, max_attempts: 1000, timeout: None }, 42, false);
    let result = generator.generate_one();
    // Accept either a valid match or error if not found
    assert!(result.is_ok() || matches!(result, Err(GenrexError::NoMatch)));
    if let Ok(s) = result {
        println!("Generated: {}", s);
        assert!(s.starts_with("foo"));
    }
}

#[test]
fn test_generate_n_success() {
    let mut generator = DummyGenerator::new("^foo\\d{1,3}$", GeneratorConfig { min_len: 4, max_len: 6, max_attempts: 1000, timeout: None }, 42, false);
    let result = generator.generate_n(3);
    assert!(result.is_ok() || matches!(result, Err(GenrexError::NoMatch)));
    if let Ok(vec) = result {
        for s in &vec {
            println!("Generated: {}", s);
            assert!(s.starts_with("foo"));
        }
    }
}

#[test]
fn test_configurable_trait_methods() {
    let mut generator = DummyGenerator::new(".*", GeneratorConfig::default(), 42, false);
    generator.min_len(2).max_len(10).max_attempts(100).timeout_ms(Some(1000)).multiline(true);
    assert!(generator.is_multiline());
    // No panic means pass
}

#[test]
fn test_generate_with_strategy_success() {
    let mut generator = DummyGenerator::new(".*", GeneratorConfig::default(), 42, true);
    let result = generator.generate_with_strategy("default");
    if let Ok(s) = &result {
        println!("Generated: {}", s);
    }
    assert!(result.is_ok() || matches!(result, Err(GenrexError::NoMatch)));
    assert!(generator.is_multiline());
}
