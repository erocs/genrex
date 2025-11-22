//! Semantic AST for regex expressions.

// use crate::tokens::Token; // removed unused import

/// The semantic AST node for a regex expression.
/// DEPRECATED: AST is retained as a legacy fallback for AST-based generation.
/// Prefer token-based generation (lexer tokens + TokenContext) for primary behavior.
/// TODO: remove AST and parser or reintroduce only when AST-based sampling/analysis is implemented.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AstNode {
    /// A sequence of nodes (concatenation)
    Sequence(Vec<AstNode>),
    /// A choice between alternatives (alternation)
    Alternation(Vec<AstNode>),
    /// A repeated node (quantifier)
    /// Note: include a `greedy` flag to preserve parsing info used elsewhere.
    Repeat {
        node: Box<AstNode>,
        min: usize,
        max: usize,
        greedy: bool,
    },
    /// A capturing group
    Group(Box<AstNode>),
    /// A non-capturing group
    NonCapturingGroup(Box<AstNode>),
    /// A backreference to a group (unit variant — backreference handled at token level).
    Backreference,
    /// A character class
    Class(Vec<char>),
    /// A negated character class (unit variant — details handled by tokens).
    NegatedClass,
    /// A literal character
    Literal(char),
    /// Start anchor (^)
    AnchorStart,
    /// End anchor ($)
    AnchorEnd,
    /// Word boundary (\b)
    WordBoundary,
    /// Wildcard (.)
    Wildcard,
}

// No AST-level describe impl (unused) to avoid warnings; token-level describe remains in src/tokens.rs.
