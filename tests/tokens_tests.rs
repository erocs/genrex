//! Unit tests for regex token generation and description.

use genrex::Token;
use rand::{rngs::StdRng, SeedableRng};
use genrex::{RegexToken, TokenContext};

#[test]
fn test_literal_token() {
    let tok = Token::Literal('x');
    let mut rng = StdRng::seed_from_u64(1);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s, "x");
    assert_eq!(tok.describe(), "Literal('x')");
}

#[test]
fn test_class_token() {
    let tok = Token::Class(vec!['a', 'b', 'c']);
    let mut rng = StdRng::seed_from_u64(2);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert!("abc".contains(&s));
    assert_eq!(tok.describe(), "Class[abc]");
}

#[test]
fn test_concatenation_token() {
    let tok = Token::Concatenation(vec![Token::Literal('a'), Token::Literal('b'), Token::Literal('c')]);
    let mut rng = StdRng::seed_from_u64(3);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s, "abc");
    assert!(tok.describe().starts_with("Concat("));
}

#[test]
fn test_alternation_token() {
    let tok = Token::Alternation(vec![Token::Literal('x'), Token::Literal('y')]);
    let mut rng = StdRng::seed_from_u64(4);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert!(["x", "y"].contains(&s.as_str()));
    assert!(tok.describe().starts_with("Alt("));
}

#[test]
fn test_quantifier_token() {
    let tok = Token::Quantifier {
        token: Box::new(Token::Literal('z')),
        min: 2,
        max: 4,
        greedy: true,
    };
    let mut rng = StdRng::seed_from_u64(5);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert!((2..=4).contains(&s.len()));
    assert!(s.chars().all(|c| c == 'z'));
    assert!(tok.describe().starts_with("Quantifier{"));
}

#[test]
fn test_group_token() {
    let tok = Token::Group(Box::new(Token::Literal('g')), 1);
    let mut rng = StdRng::seed_from_u64(6);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s, "g");
    assert_eq!(tok.describe(), "Group(1)");
}

#[test]
fn test_non_capturing_group_token() {
    let tok = Token::NonCapturingGroup(Box::new(Token::Literal('h')));
    let mut rng = StdRng::seed_from_u64(7);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s, "h");
    assert_eq!(tok.describe(), "NonCapturingGroup");
}

#[test]
fn test_backreference_token_unsupported() {
    let tok = Token::Backreference(1);
    let mut rng = StdRng::seed_from_u64(8);
    let mut ctx = TokenContext::new();
    let res = tok.generate(&mut rng, &mut ctx);
    assert!(res.is_err());
    assert!(tok.describe().starts_with("Backreference("));
}

#[test]
fn test_backreference_token_simple() {
    // Pattern equivalent: (a)\1 -> should produce "aa"
    let tok = Token::Concatenation(vec![
        Token::Group(Box::new(Token::Literal('a')), 1),
        Token::Backreference(1),
    ]);
    let mut rng = StdRng::seed_from_u64(11);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s, "aa");
}

#[test]
fn test_backreference_token_repeated() {
    // Pattern equivalent: ([ab])\1\1 -> produces three identical chars from {a,b}
    let tok = Token::Concatenation(vec![
        Token::Group(Box::new(Token::Class(vec!['a', 'b'])), 1),
        Token::Backreference(1),
        Token::Backreference(1),
    ]);
    let mut rng = StdRng::seed_from_u64(12);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s.len(), 3);
    let first = s.chars().next().unwrap();
    assert!(first == 'a' || first == 'b');
    assert!(s.chars().all(|c| c == first));
}

#[test]
fn test_anchor_tokens() {
    let start = Token::AnchorStart;
    let end = Token::AnchorEnd;
    let word = Token::WordBoundary;
    let mut rng = StdRng::seed_from_u64(9);
    let mut ctx = TokenContext::new();
    assert_eq!(start.generate(&mut rng, &mut ctx).unwrap(), "");
    assert_eq!(end.generate(&mut rng, &mut ctx).unwrap(), "");
    assert_eq!(word.generate(&mut rng, &mut ctx).unwrap(), "");
    assert_eq!(start.describe(), "AnchorStart");
    assert_eq!(end.describe(), "AnchorEnd");
    assert_eq!(word.describe(), "WordBoundary");
}

#[test]
fn test_wildcard_token() {
    let tok = Token::Wildcard;
    let mut rng = StdRng::seed_from_u64(10);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s.len(), 1);
    assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(tok.describe(), "Wildcard");
}

#[test]
fn test_quantifier_greedy_vs_non_greedy() {
    // Verify greedy quantifiers tend to choose larger counts than non-greedy ones
    let greedy = Token::Quantifier {
        token: Box::new(Token::Literal('z')),
        min: 0,
        max: 5,
        greedy: true,
    };
    let lazy = Token::Quantifier {
        token: Box::new(Token::Literal('z')),
        min: 0,
        max: 5,
        greedy: false,
    };
    let mut ctx = TokenContext::new();
    // Use deterministic per-iteration seeding so both tokens see the same RNG stream for that iteration.
    for i in 0..200 {
        let mut rng_g = StdRng::seed_from_u64(0xDEADBEEF + i);
        let mut rng_l = StdRng::seed_from_u64(0xDEADBEEF + i);
        let s_g = greedy.generate(&mut rng_g, &mut ctx).unwrap();
        let s_l = lazy.generate(&mut rng_l, &mut ctx).unwrap();
        assert!(s_g.len() >= s_l.len(), "greedy len {} should be >= lazy len {}", s_g.len(), s_l.len());
    }
}
