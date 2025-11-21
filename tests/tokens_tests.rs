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
    let tok = Token::Group(Box::new(Token::Literal('g')));
    let mut rng = StdRng::seed_from_u64(6);
    let mut ctx = TokenContext::new();
    let s = tok.generate(&mut rng, &mut ctx).unwrap();
    assert_eq!(s, "g");
    assert_eq!(tok.describe(), "Group");
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
