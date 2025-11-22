//! Simple parser: converts a vector of tokens into a semantic AST.

use crate::tokens::Token;
use crate::ast::AstNode;

/// Parses a vector of tokens into an AST.
pub struct AstParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> AstParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        AstParser { tokens, pos: 0 }
    }

    /// Entry point: parse the full regex as a sequence or alternation.
    pub fn parse(&mut self) -> Option<AstNode> {
        self.parse_alternation()
    }

    fn parse_alternation(&mut self) -> Option<AstNode> {
        let mut branches = vec![self.parse_sequence()?];
        while self.peek_is_alternation() {
            self.pos += 1; // skip alternation token
            branches.push(self.parse_sequence()?);
        }
        if branches.len() == 1 {
            Some(branches.remove(0))
        } else {
            Some(AstNode::Alternation(branches))
        }
    }

    fn parse_sequence(&mut self) -> Option<AstNode> {
        let mut nodes = Vec::new();
        while let Some(node) = self.parse_atom() {
            nodes.push(node);
            if self.peek_is_alternation() || self.is_end() {
                break;
            }
        }
        if nodes.len() == 1 {
            Some(nodes.remove(0))
        } else if !nodes.is_empty() {
            Some(AstNode::Sequence(nodes))
        } else {
            None
        }
    }

    fn parse_atom(&mut self) -> Option<AstNode> {
        let token = self.tokens.get(self.pos)?;
        let node = match token {
            Token::Literal(c) => AstNode::Literal(*c),
            Token::Class(chars) => AstNode::Class(chars.clone()),
            Token::NegatedClass(_chars) => AstNode::NegatedClass,
            Token::AnchorStart => AstNode::AnchorStart,
            Token::AnchorEnd => AstNode::AnchorEnd,
            Token::WordBoundary => AstNode::WordBoundary,
            Token::Wildcard => AstNode::Wildcard,
            Token::Backreference(_idx) => AstNode::Backreference,
            Token::Group(inner, _idx) => AstNode::Group(Box::new(
                AstParser::new(&[(**inner).clone()]).parse().unwrap_or(AstNode::Literal(' '))
            )),
            Token::NonCapturingGroup(inner) => AstNode::NonCapturingGroup(Box::new(
                AstParser::new(&[(**inner).clone()]).parse().unwrap_or(AstNode::Literal(' '))
            )),
            Token::Quantifier { token, min, max, greedy } => AstNode::Repeat {
                node: Box::new(
                    AstParser::new(&[(**token).clone()]).parse().unwrap_or(AstNode::Literal(' '))
                ),
                min: *min,
                max: *max,
                greedy: *greedy,
            },
            Token::Concatenation(tokens) => AstParser::new(tokens).parse().unwrap_or(AstNode::Literal(' ')),
            Token::Alternation(tokens) => AstNode::Alternation(
                tokens.iter().map(|t| AstParser::new(&[t.clone()]).parse().unwrap_or(AstNode::Literal(' '))).collect()
            ),
        };
        self.pos += 1;
        Some(node)
    }

    fn peek_is_alternation(&self) -> bool {
        matches!(self.tokens.get(self.pos), Some(Token::Alternation(_)))
    }

    fn is_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
