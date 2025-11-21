//! Semantic AST for regex expressions.

// use crate::tokens::Token; // removed unused import

/// The semantic AST node for a regex expression.
#[derive(Debug, Clone)]
pub enum AstNode {
    /// A sequence of nodes (concatenation)
    Sequence(Vec<AstNode>),
    /// A choice between alternatives (alternation)
    Alternation(Vec<AstNode>),
    /// A repeated node (quantifier)
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
    /// A backreference to a group
    Backreference(usize),
    /// A character class
    Class(Vec<char>),
    /// A negated character class
    NegatedClass(Vec<char>),
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

impl AstNode {
    /// Returns a human-readable description of the node.
    pub fn describe(&self) -> String {
        match self {
            AstNode::Sequence(nodes) => format!("Sequence({})", nodes.len()),
            AstNode::Alternation(nodes) => format!("Alternation({})", nodes.len()),
            AstNode::Repeat { min, max, greedy, .. } => format!("Repeat{{{},{}}}{}", min, max, if *greedy {""} else {"?"}),
            AstNode::Group(_) => "Group".to_string(),
            AstNode::NonCapturingGroup(_) => "NonCapturingGroup".to_string(),
            AstNode::Backreference(idx) => format!("Backreference({})", idx),
            AstNode::Class(chars) => format!("Class[{}]", chars.iter().collect::<String>()),
            AstNode::NegatedClass(chars) => format!("NegatedClass[{}]", chars.iter().collect::<String>()),
            AstNode::Literal(c) => format!("Literal('{}')", c),
            AstNode::AnchorStart => "AnchorStart".to_string(),
            AstNode::AnchorEnd => "AnchorEnd".to_string(),
            AstNode::WordBoundary => "WordBoundary".to_string(),
            AstNode::Wildcard => "Wildcard".to_string(),
        }
    }
}
