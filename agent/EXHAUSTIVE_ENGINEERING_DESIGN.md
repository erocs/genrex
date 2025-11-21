# genrex Agent: Exhaustive Engineering Design for Regex String Generation

## Purpose
This document provides a comprehensive, AI-processable engineering design for generating random strings that match regular expressions. It details the base theory and generation strategies for each semantic token in the regex language, serving as a blueprint for future agent-based or automated implementations.

---

## 1. Overview of Regex Token Semantics
Regular expressions are parsed into an Abstract Syntax Tree (AST) of tokens, each representing a semantic operation. String generation requires producing a string that, when parsed, matches the regex's AST. Each token type has a distinct generation strategy.

---

## 2. Token Types and Generation Theory

### 2.1. Literals
- **Description:** A literal character (e.g., `a`, `b`, `1`).
- **Generation:** Output the literal character directly.


### 2.2. Character Classes
- **Description:** A set of possible characters (e.g., `[a-z]`, `[0-9]`, `\w`, `\d`).
- **Generation:** Uniformly sample one character from the class set, using a customizable character set provider.
- **Custom Character Set API:**
  - A trait (e.g., `Charset`) is defined with methods to:
    - Check if a character is valid: `fn contains(&self, ch: char) -> bool`.
    - Optionally, check if a string is valid: `fn contains_str(&self, s: &str) -> bool`.
    - Provide a random valid character: `fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<char>`.
  - The builder accepts a boxed implementation of this trait, allowing users to provide custom alphabets, code pages, or validation logic.
  - If not provided, a default ASCII or Unicode set is used.
- **Special:** Negated classes (e.g., `[^a-z]`) use the full provided alphabet to determine valid exclusions.

**Example Trait:**
```rust
pub trait Charset: Send + Sync {
  fn contains(&self, ch: char) -> bool;
  fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Option<char>;
}
```

**Builder Usage:**
```rust
RegexGeneratorBuilder::new(pattern)
  .charset(Box::new(MyCustomCharset::new(...)))
  .build()
```

### 2.3. Concatenation
- **Description:** Sequence of tokens (e.g., `abc`, `[a-z][0-9]`).
- **Generation:** Recursively generate for each child token and concatenate results.


### 2.4. Alternation
- **Description:** Choice between alternatives (e.g., `a|b|c`, `cat|dog|mouse`, `hi|\w\d`).
- **Clarification:** Alternation applies to the largest possible subexpression to the left and right of the `|` operator, not just a single character. For example:
  - `abc|def` alternates between the entire sequence `abc` and the entire sequence `def`.
  - `a|b|c` alternates between the single characters `a`, `b`, and `c`.
  - `hi|\w\d` alternates between the literal string `hi` and the sequence "word character followed by digit".
  - Parentheses can be used to group subexpressions, e.g., `ab(c|d)ef` alternates between `c` and `d` within the group.
- **Example:**
  - The regex `hi|\w\d` will match either the string `hi` or any two-character string where the first is a word character and the second is a digit (e.g., `a1`, `Z9`, `g7`).
  - For the string `agent007`, the substring `t0` matches the `\w\d` alternative, so the regex matches.
- **Generation:** Uniformly select one branch (subexpression) and generate for that branch.


### 2.5. Quantifiers
- **Description:** Repeat a token (e.g., `a*`, `b{2,5}`, `c+`).
- **Types:**
  - **Greedy Quantifiers:** Match as many repetitions as possible (default in most regex engines, e.g., `a*`, `b+`, `c{2,5}`).
  - **Conservative (Lazy) Quantifiers:** Match as few repetitions as possible (denoted by `*?`, `+?`, `{m,n}?`).
- **Generation:**
  - Determine the allowed range (min, max).
  - Uniformly sample a count in the range, unless a specific strategy is required.
  - For greedy quantifiers, prefer generating the maximum allowed repetitions (if simulating greedy matching behavior).
  - For conservative quantifiers, prefer generating the minimum allowed repetitions (if simulating lazy matching behavior).
  - Generate the child token that many times and concatenate.
- **Special:** For `*` and `+`, max may be capped by config.
- **Note:**
  - For random generation, uniform sampling is often used, but the generator can be configured to bias toward greedy (max) or conservative (min) repetition counts to simulate regex engine behavior.

### 2.6. Groups (Capturing and Non-Capturing)
- **Description:** Parenthesized sub-expressions (e.g., `(abc)`, `(?:xyz)`).
- **Generation:**
  - Generate for the child token.
  - For capturing groups, store the generated string for possible backreference use.

### 2.7. Backreferences
- **Description:** Reference to a previous capturing group (e.g., `\1`).
- **Generation:**
  - Retrieve the string generated for the referenced group and insert it at this position.
  - If the group is inside a quantifier, ensure consistent handling.
  - If the group is not yet generated, backtrack or fail.


### 2.8. Anchors
- **Description:** Position constraints (e.g., `^`, `$`, `\b`).
- **Multiline Mode:**
  - In multiline mode (enabled by the `m` flag), `^` and `$` match the start and end of any line within the string, not just the start and end of the entire string.
  - This means the generator must be able to produce strings with embedded newlines and ensure anchors are respected at line boundaries.
- **Generation:**
  - Do not generate characters for anchors; instead, ensure the generated string starts/ends as required by the anchor semantics.
  - In multiline mode, generate strings that may contain multiple lines, and ensure that `^` and `$` are satisfied at the start/end of each line as needed.
  - For word boundaries (`\b`), ensure the context matches the boundary condition (e.g., transition between word and non-word characters).
- **Builder/Config:**
  - The generator should accept a configuration option to enable or disable multiline mode, affecting how anchors are interpreted and how strings are generated.

### 2.9. Lookahead/Lookbehind (Zero-width assertions)
- **Description:** Assert a pattern ahead/behind (e.g., `(?=abc)`, `(?<=xyz)`).
- **Generation:**
  - Not supported in MVP; requires constraint solving or backtracking.
  - For future: generate a string such that the assertion holds at the current position.

### 2.10. Wildcard
- **Description:** Any character (e.g., `.`).
- **Generation:** Uniformly sample a character from the allowed alphabet (configurable, e.g., ASCII).

---

## 3. Generation Algorithm (High-Level)
1. Parse regex into AST of tokens.
2. Recursively generate string for each token using the above strategies.
3. For tokens with dependencies (groups, backreferences), maintain a context stack.
4. For unsupported features, fallback to rejection sampling or return an error.

---

## 4. Extensibility and AI Considerations
- Each token handler should be modular and composable.
- The agent can reason about constraints, dependencies, and context.
- For ambiguous or complex cases, the agent may use backtracking, constraint solving, or fallback strategies.
- The design supports future extension to Unicode, custom alphabets, and advanced regex features.

---

## 5. References
- [regex crate documentation](https://docs.rs/regex)
- [Regular Expressions: Theory and Practice](https://en.wikipedia.org/wiki/Regular_expression)

---

*This document is intended for AI and automated agents to guide the implementation of robust, extensible regex string generation.*
