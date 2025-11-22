use rand::Rng;
use crate::traits::{RegexToken, TokenContext};
use crate::error::GenrexError;

/// Enum representing all possible regex AST token types.
#[derive(Debug, Clone)]
pub enum Token {
	Literal(char),
	Class(Vec<char>),
	NegatedClass(Vec<char>),
	Concatenation(Vec<Token>),
	Alternation(Vec<Token>),
	Quantifier {
		token: Box<Token>,
		min: usize,
		max: usize,
		greedy: bool,
	},
	/// Capturing group with explicit group index (1-based). Index 0 may be used temporarily before assignment.
	Group(Box<Token>, usize),
	/// Non-capturing group (does not record captures).
	NonCapturingGroup(Box<Token>),
	Backreference(usize),
	AnchorStart,
	AnchorEnd,
	WordBoundary,
	Wildcard,
}

impl RegexToken for Token {
	fn generate<R: Rng + ?Sized>(&self, rng: &mut R, ctx: &mut TokenContext) -> Result<String, GenrexError> {
		match self {
			Token::Literal(c) => Ok(c.to_string()),
			Token::Class(chars) => {
				if chars.is_empty() {
					Err(GenrexError::Internal("Empty class".to_string()))
				} else {
					let idx = rng.gen_range(0..chars.len());
					Ok(chars[idx].to_string())
				}
			}
			Token::NegatedClass(_chars) => {
				// Negated class generation would require full alphabet context
				Err(GenrexError::UnsupportedFeature("Negated class generation".to_string()))
			}
			Token::Concatenation(tokens) => {
				let mut out = String::new();
				for t in tokens {
					out.push_str(&t.generate(rng, ctx)?);
				}
				Ok(out)
			}
			Token::Alternation(choices) => {
				if choices.is_empty() {
					Err(GenrexError::Internal("Empty alternation".to_string()))
				} else {
					let idx = rng.gen_range(0..choices.len());
					choices[idx].generate(rng, ctx)
				}
			}
			Token::Quantifier { token, min, max, greedy } => {
				// Avoid unbounded quantifiers producing enormous ranges (e.g., max == usize::MAX).
				const MAX_REPEAT: usize = 32;
				if min > max { return Err(GenrexError::Internal("Quantifier min > max".to_string())); }
				let effective_max = if *max == usize::MAX { (*min).saturating_add(MAX_REPEAT) } else { *max };
				let count = if *min == *max {
					*min
				} else {
					// Bias selection: greedy favors larger counts, non-greedy favors smaller counts.
					let a = rng.gen_range(*min..=effective_max);
					let b = rng.gen_range(*min..=effective_max);
					if *greedy { a.max(b) } else { a.min(b) }
				};
				let mut out = String::new();
				for _ in 0..count {
					out.push_str(&token.generate(rng, ctx)?);
				}
				Ok(out)
			}
			Token::Group(inner, idx) => {
				let s = inner.generate(rng, ctx)?;
				// Record capture into context at the specified index (naive: overwrite slot).
				if *idx == 0 {
					// If index is 0 (temporary), append to captures vector.
					ctx.captures.push(s.clone());
				} else {
					let slot = idx.saturating_sub(1);
					if ctx.captures.len() <= slot {
						ctx.captures.resize(slot + 1, String::new());
					}
					ctx.captures[slot] = s.clone();
				}
				Ok(s)
			}
			Token::NonCapturingGroup(inner) => inner.generate(rng, ctx),
			Token::Backreference(idx) => {
				// Naive backreference support: lookup previously recorded capture by group index (1-based).
				if *idx == 0 {
					return Err(GenrexError::BackreferenceError("backreference index 0 is invalid".to_string()));
				}
				let capture_index = idx.saturating_sub(1);
				if capture_index < ctx.captures.len() && !ctx.captures[capture_index].is_empty() {
					Ok(ctx.captures[capture_index].clone())
				} else {
					Err(GenrexError::BackreferenceError(format!("no capture available for backreference \\{}", idx)))
				}
			}
			Token::AnchorStart | Token::AnchorEnd | Token::WordBoundary => Ok(String::new()),
			Token::Wildcard => {
				// For MVP, use ASCII alphanumeric
				const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
				let idx = rng.gen_range(0..ALPHABET.len());
				Ok((ALPHABET[idx] as char).to_string())
			}
		}
	}

	fn describe(&self) -> String {
		match self {
			Token::Literal(c) => format!("Literal('{}')", c),
			Token::Class(chars) => format!("Class[{}]", chars.iter().collect::<String>()),
			Token::NegatedClass(chars) => format!("NegatedClass[{}]", chars.iter().collect::<String>()),
			Token::Concatenation(tokens) => format!("Concat({})", tokens.len()),
			Token::Alternation(choices) => format!("Alt({})", choices.len()),
			Token::Quantifier { min, max, .. } => format!("Quantifier{{{},{}}}", min, max),
			Token::Group(_, idx) => format!("Group({})", idx),
			Token::NonCapturingGroup(_) => "NonCapturingGroup".to_string(),
			Token::Backreference(idx) => format!("Backreference({})", idx),
			Token::AnchorStart => "AnchorStart".to_string(),
			Token::AnchorEnd => "AnchorEnd".to_string(),
			Token::WordBoundary => "WordBoundary".to_string(),
			Token::Wildcard => "Wildcard".to_string(),
		}
	}
}
