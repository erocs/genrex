# genrex Agent: Top-Level Design Document

## Overview
This document outlines the top-level design and development plan for the `genrex` library, which generates random strings matching a given regular expression. The goal is to evolve from a minimal MVP (rejection sampling) to a robust, efficient, and extensible solution.

---

## 1. Goals
- Generate random strings that match user-supplied regular expressions.
- Support configuration for string length, attempts, and timeouts.
- Provide clear error handling and diagnostics.
- Enable future extensibility for more efficient algorithms (e.g., NFA-based sampling).

---

## 2. Architecture
- **Core Generator**: Responsible for string generation and regex matching.
- **Configuration Layer**: Allows users to specify constraints (length, attempts, timeout).
- **Error Handling**: Uses structured error types for invalid regex, timeouts, and no matches.
- **Testing & Validation**: Comprehensive unit and property-based tests.
- **Agent Layer (Future)**: For advanced sampling strategies, AST/NFA-based generation, and pluggable backends.

---

## 3. Key Components
- `GeneratorConfig`: Holds user-specified constraints.
- `RegexGenerator`: Main struct for generation logic.
- `GenError`: Error enum for all failure modes.
- **(Planned)**: `Agent` trait for extensible generation strategies.

---

## 4. Development Phases
1. **MVP (Current)**: Rejection sampling over ASCII alphanumerics.
2. **Diagnostics**: Improve error messages and add generation statistics.
3. **Efficiency**: Implement AST/NFA-based bounded sampling for complex patterns.
4. **Extensibility**: Introduce `Agent` trait and pluggable strategies.
5. **API Polish**: Documentation, examples, and ergonomic improvements.

---

## 5. Future Directions
- Support for Unicode and custom alphabets.
- Efficient handling of complex regex features (lookarounds, backreferences).
- CLI and WebAssembly bindings.
- Integration with fuzzing and property-based testing tools.

## 6. Open Questions
- How to balance efficiency and generality for arbitrary regexes?


- How to expose diagnostics and statistics to users?


- Backreference handling?

- **Naive Approach (v1):**
	- Track the string generated for each capturing group during generation.
	- When a backreference is encountered, insert the previously generated group value.
	- This works for simple, non-nested, and non-repetitive cases.
	- If a pattern cannot be satisfied (e.g., due to conflicting constraints), generation may fail or require rejection sampling.

- **Limitations:**
	- Nested, repeated, or mutually dependent backreferences may not be handled correctly.
    - Result will be best effort and the result might be undefined with respect to the regular expression language.
	- Some patterns with backreferences are non-regular and may require backtracking or constraint solving.
	- Efficiency is not guaranteed for complex cases.

- **Future Work:**
	- Explore more robust algorithms for complex backreference handling.
	- Consider user warnings or errors for unsupported patterns.

This naive implementation will be included in v1, with clear documentation of its limitations.
---

## 7. References
- [regex crate documentation](https://docs.rs/regex)
- [rand crate documentation](https://docs.rs/rand)
- [thiserror crate documentation](https://docs.rs/thiserror)

---

*This document should be updated as the project evolves.*
