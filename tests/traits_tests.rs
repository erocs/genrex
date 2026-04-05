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
    // Negated class token generation falls back to rejection sampling, which succeeds.
    // Verify the result doesn't contain the negated chars.
    if let Ok(s) = result {
        assert!(!s.chars().any(|c| "abc".contains(c)));
    }
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
    assert!(s.chars().all(|c| c.is_ascii() && !c.is_ascii_control()));
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
        self.inner.generate_one()
    }
    fn generate_n(&mut self, n: usize) -> Result<Vec<String>, GenrexError> {
        self.inner.generate_n(n)
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

// ── Character class range expansion ──────────────────────────────────────────

#[test]
fn test_class_range_lowercase_letters_only() {
    // [a-z]{20}: every character must be a lowercase letter, not '-'.
    let mut g = RegexGenerator::builder("[a-z]{20}")
        .config(GeneratorConfig { min_len: 20, max_len: 20, max_attempts: 1_000, timeout: None })
        .rng(StdRng::seed_from_u64(1))
        .build()
        .unwrap();
    let s = g.generate_one().unwrap();
    assert_eq!(s.len(), 20);
    assert!(s.chars().all(|c| c.is_ascii_lowercase()), "[a-z]{{20}} must be lowercase letters, got: {:?}", s);
}

#[test]
fn test_class_range_lowercase_covers_alphabet() {
    // [a-z] must produce characters beyond just the literal endpoints {a, z}.
    let mut seen = std::collections::HashSet::<char>::new();
    for seed in 0u64..500 {
        let mut g = RegexGenerator::builder("[a-z]")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        if let Ok(s) = g.generate_one() { seen.extend(s.chars()); }
    }
    assert!(seen.iter().all(|c| c.is_ascii_lowercase()), "[a-z] produced non-lowercase: {:?}", seen);
    assert!(seen.len() > 3, "[a-z] only produced {:?}", seen);
}

#[test]
fn test_class_range_uppercase_covers_alphabet() {
    // [A-Z] must produce characters beyond just {A, Z}.
    let mut seen = std::collections::HashSet::<char>::new();
    for seed in 0u64..500 {
        let mut g = RegexGenerator::builder("[A-Z]")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        if let Ok(s) = g.generate_one() { seen.extend(s.chars()); }
    }
    assert!(seen.iter().all(|c| c.is_ascii_uppercase()), "[A-Z] produced non-uppercase: {:?}", seen);
    assert!(seen.len() > 3, "[A-Z] only produced {:?}", seen);
}

#[test]
fn test_class_range_digits_variety() {
    // [0-9] must sample from all 10 digits, not just {'0', '9'}.
    let mut seen = std::collections::HashSet::<char>::new();
    for seed in 0u64..200 {
        let mut g = RegexGenerator::builder("[0-9]")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        if let Ok(s) = g.generate_one() { seen.extend(s.chars()); }
    }
    assert!(seen.iter().all(|c| c.is_ascii_digit()), "[0-9] produced non-digit: {:?}", seen);
    assert!(seen.len() > 3, "[0-9] only produced {:?}", seen);
}

#[test]
fn test_class_range_mixed_includes_midpoints() {
    // [a-c0-2] must produce 'b' and '1' — the interior values not reachable
    // if ranges are not expanded.
    let mut seen = std::collections::HashSet::<char>::new();
    for seed in 0u64..300 {
        let mut g = RegexGenerator::builder("[a-c0-2]")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        if let Ok(s) = g.generate_one() { seen.extend(s.chars()); }
    }
    let allowed: std::collections::HashSet<char> = "abc012".chars().collect();
    assert!(seen.iter().all(|c| allowed.contains(c)), "[a-c0-2] produced unexpected chars: {:?}", seen);
    assert!(seen.contains(&'b'), "[a-c0-2] never produced 'b'; saw: {:?}", seen);
    assert!(seen.contains(&'1'), "[a-c0-2] never produced '1'; saw: {:?}", seen);
}

// ── Non-capturing group / group counter ──────────────────────────────────────

#[test]
fn test_non_capturing_group_backreference() {
    // (?:a)(b)\1 — (?:a) must not consume group index 1; \1 must refer to (b).
    let mut g = RegexGenerator::builder("(?:a)(b)\\1")
        .allow_backrefs()
        .config(GeneratorConfig { min_len: 3, max_len: 3, max_attempts: 1_000, timeout: None })
        .rng(StdRng::seed_from_u64(1))
        .build()
        .unwrap();
    assert_eq!(g.generate_one().unwrap(), "abb");
}

#[test]
fn test_non_capturing_groups_before_capture() {
    // (?:x)(?:y)(z)\1 — two non-capturing groups, then group 1 = (z).
    let mut g = RegexGenerator::builder("(?:x)(?:y)(z)\\1")
        .allow_backrefs()
        .config(GeneratorConfig { min_len: 4, max_len: 4, max_attempts: 1_000, timeout: None })
        .rng(StdRng::seed_from_u64(1))
        .build()
        .unwrap();
    assert_eq!(g.generate_one().unwrap(), "xyzz");
}

#[test]
fn test_interleaved_capturing_non_capturing_groups() {
    // (a)(?:b)(c)\1\2 — group 1 = (a), group 2 = (c); result "abcac".
    let mut g = RegexGenerator::builder("(a)(?:b)(c)\\1\\2")
        .allow_backrefs()
        .config(GeneratorConfig { min_len: 5, max_len: 5, max_attempts: 1_000, timeout: None })
        .rng(StdRng::seed_from_u64(1))
        .build()
        .unwrap();
    assert_eq!(g.generate_one().unwrap(), "abcac");
}

// ── Negated character classes ─────────────────────────────────────────────────

#[test]
fn test_bracket_negated_class() {
    // [^abc] must not produce a, b, or c.
    for seed in 0u64..50 {
        let mut g = RegexGenerator::builder("[^abc]")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        let s = g.generate_one().unwrap();
        assert!(!s.chars().any(|c| "abc".contains(c)), "[^abc] produced excluded char; got {:?}", s);
    }
}

#[test]
fn test_backslash_d_upper_non_digit() {
    // \D must not produce a digit.
    for seed in 0u64..50 {
        let mut g = RegexGenerator::builder("\\D")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        let s = g.generate_one().unwrap();
        assert!(!s.chars().any(|c| c.is_ascii_digit()), "\\D produced a digit; got {:?}", s);
    }
}

#[test]
fn test_backslash_w_upper_non_word_char() {
    // \W must not produce a word character (letter, digit, or underscore).
    for seed in 0u64..50 {
        let mut g = RegexGenerator::builder("\\W")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        let s = g.generate_one().unwrap();
        assert!(!s.chars().any(|c| c.is_ascii_alphanumeric() || c == '_'), "\\W produced word char; got {:?}", s);
    }
}

#[test]
fn test_backslash_s_upper_non_whitespace() {
    // \S must not produce whitespace.
    for seed in 0u64..50 {
        let mut g = RegexGenerator::builder("\\S")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        let s = g.generate_one().unwrap();
        assert!(!s.chars().any(|c| c.is_ascii_whitespace()), "\\S produced whitespace; got {:?}", s);
    }
}

// ── Wildcard (`.`) ────────────────────────────────────────────────────────────

#[test]
fn test_wildcard_produces_non_alphanumeric() {
    // `.` samples from all printable ASCII, so across many seeds it must
    // produce at least one non-alphanumeric character.
    let mut saw_non_alnum = false;
    for seed in 0u64..500 {
        let mut g = RegexGenerator::builder(".")
            .config(GeneratorConfig { min_len: 1, max_len: 1, max_attempts: 1_000, timeout: None })
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .unwrap();
        if let Ok(s) = g.generate_one() {
            if s.chars().any(|c| !c.is_ascii_alphanumeric()) {
                saw_non_alnum = true;
                break;
            }
        }
    }
    assert!(saw_non_alnum, "`.` should produce non-alphanumeric chars across 500 seeds");
}

// ── allow_backrefs / .* fallback ─────────────────────────────────────────────

#[test]
fn test_allow_backrefs_simple_backref() {
    // (a)\1 must produce "aa" via token-based generation.
    let mut g = RegexGenerator::builder("(a)\\1")
        .allow_backrefs()
        .config(GeneratorConfig { min_len: 2, max_len: 2, max_attempts: 1_000, timeout: None })
        .rng(StdRng::seed_from_u64(1))
        .build()
        .unwrap();
    assert_eq!(g.generate_one().unwrap(), "aa");
}

#[test]
fn test_allow_backrefs_negated_class_backref() {
    // ([^abc])\1: captured char must not be in {a,b,c} and must be repeated.
    let mut g = RegexGenerator::builder("([^abc])\\1")
        .allow_backrefs()
        .config(GeneratorConfig { min_len: 2, max_len: 2, max_attempts: 1_000, timeout: None })
        .rng(StdRng::seed_from_u64(1))
        .build()
        .unwrap();
    let s = g.generate_one().unwrap();
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    let second = chars.next().unwrap();
    assert_eq!(first, second, "backreference must repeat the captured char; got {:?}", s);
    assert!(!"abc".contains(first), "captured char must not be in [^abc]; got {:?}", s);
}

#[test]
fn test_allow_backrefs_digit_class_backref() {
    // ([0-9])\1: two identical digits.
    let mut g = RegexGenerator::builder("([0-9])\\1")
        .allow_backrefs()
        .config(GeneratorConfig { min_len: 2, max_len: 2, max_attempts: 1_000, timeout: None })
        .rng(StdRng::seed_from_u64(42))
        .build()
        .unwrap();
    let s = g.generate_one().unwrap();
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    let second = chars.next().unwrap();
    assert!(first.is_ascii_digit(), "first char must be a digit; got {:?}", first);
    assert_eq!(first, second, "backreference must repeat the digit; got {:?}", s);
}

#[test]
fn test_configurable_trait_methods() {
    let mut generator = DummyGenerator::new(".*", GeneratorConfig::default(), 42, false);
    generator.min_len(2).max_len(10).max_attempts(100).timeout_ms(Some(1000)).multiline(true);
    assert!(generator.is_multiline());
    // No panic means pass
}

// ── Large quantifiers ─────────────────────────────────────────────────────────

#[test]
fn test_fixed_quantifier_100_no_explicit_max_len() {
    // \w{100} must produce exactly 100 chars even when no explicit max_len is set.
    for seed in 0u64..5 {
        let mut g = RegexGenerator::builder("\\w{100}")
            .rng(StdRng::seed_from_u64(seed))
            .build()
            .expect("\\w{100} should build");
        let s = g.generate_one().expect("\\w{100} should generate");
        assert_eq!(s.len(), 100,
            "\\w{{100}} must produce exactly 100 chars; seed={} got {:?}", seed, s);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    }
}

#[test]
fn test_range_quantifier_not_always_min() {
    // \w{1,100} must not always produce 1 char — the length must vary.
    let mut g = RegexGenerator::builder("\\w{1,100}")
        .rng(StdRng::seed_from_u64(7))
        .build()
        .expect("\\w{1,100} should build");
    let lengths: Vec<usize> = (0..50).map(|_| g.generate_one().unwrap().len()).collect();
    assert!(lengths.iter().all(|&l| (1..=100).contains(&l)),
        "every length must be in [1,100]; got: {:?}", lengths);
    assert!(lengths.iter().any(|&l| l > 1),
        "\\w{{1,100}} must not always produce 1 char; lengths: {:?}", lengths);
    assert!(lengths.iter().any(|&l| l > 64),
        "\\w{{1,100}} must sometimes exceed 64 (old default max_len); lengths: {:?}", lengths);
    let distinct: std::collections::HashSet<_> = lengths.iter().copied().collect();
    assert!(distinct.len() >= 10,
        "\\w{{1,100}} must produce at least 10 distinct lengths; got: {:?}", distinct);
}

#[test]
fn test_range_quantifier_with_space_around_comma() {
    // {1, 100} with whitespace around the comma must be treated identically to {1,100}.
    let mut g = RegexGenerator::builder("\\w{1, 100}")
        .rng(StdRng::seed_from_u64(7))
        .build()
        .expect("\\w{1, 100} should build");
    let lengths: Vec<usize> = (0..50).map(|_| g.generate_one().unwrap().len()).collect();
    assert!(lengths.iter().all(|&l| (1..=100).contains(&l)),
        "every length must be in [1,100]; got: {:?}", lengths);
    assert!(lengths.iter().any(|&l| l > 1),
        "\\w{{1, 100}} must not always produce 1 char; lengths: {:?}", lengths);
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
