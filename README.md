# genrex

Generate random strings matching a provided regular expression. Minimal MVP crate and CLI.

## Quick links
- Repository: https://github.com/erocs/genrex
- Package metadata: [`Cargo.toml`](Cargo.toml:1)
- CLI binary: [`genrex-cli`](src/main.rs:14)
- Library builder: [`RegexGeneratorBuilder::new`](src/lib.rs:388)

## Features
- Generate random strings that match a given regex.
- Configurable min/max length, attempts, timeout, multiline, RNG seed.
- Supports basic character classes, quantifiers, groups and limited backreference handling when enabled.
- Library API and a simple CLI.

## Limitations
- MVP uses token/AST-based generation and rejection sampling. May be inefficient for complex patterns.
- Limited support for lookarounds and advanced regex features.
- Backreferences are best-effort when enabled via --allow-backrefs.

## Installation

Build locally:

```bash
cargo build
```

Install CLI to cargo bin:

```bash
cargo install --path .
```

## CLI Usage

Basic usage:

```bash
genrex-cli "<pattern>"
```

Options (supported by [`src/main.rs`](src/main.rs:1)):
- --n N            : generate N outputs (default 1)
- --seed S         : seed the RNG with unsigned 64-bit value
- --min M          : minimum string length
- --max M          : maximum string length
- --attempts A     : maximum candidate attempts (rejection sampling)
- --timeout-ms T   : generation timeout in milliseconds
- --multiline      : enable multiline mode
- --allow-backrefs : allow patterns that fail regex::Regex compilation
- -v               : verbose diagnostics

Examples:

```bash
# Generate one string matching a simple pattern
genrex-cli "^foo\\d{1,3}$"

# Generate 5 strings with a fixed RNG seed
genrex-cli "user_[a-z]{3}\\d" --n 5 --seed 42

# Constrain length and increase attempts
genrex-cli "a.*b" --min 2 --max 20 --attempts 50000

# Allow backreferences (best-effort)
genrex-cli "(foo)\\1" --allow-backrefs --n 3

# Verbose mode for diagnostics
genrex-cli "[A-Z]{2}\\d+" -v --n 3
```

## Library usage

Minimal Rust example using [`RegexGeneratorBuilder::new`](src/lib.rs:388) and `GeneratorConfig`:

```rust
use rand::rngs::StdRng;
use rand::SeedableRng;
use genrex::{RegexGeneratorBuilder, GeneratorConfig};

// Build a seeded generator with length constraints
let cfg = GeneratorConfig { min_len: 1, max_len: 16, max_attempts: 20_000, timeout: None };
let mut g = RegexGeneratorBuilder::new("^foo\\d{1,3}$")
    .config(cfg)
    .rng(StdRng::seed_from_u64(42))
    .multiline(false)
    .build()
    .expect("build generator");

let s = g.generate_one().expect("generate");
println!("{}", s);
```

## Testing

Run the test suite with:

```bash
cargo test
```

Reference existing tests: [`tests/traits_tests.rs`](tests/traits_tests.rs:1), [`tests/tokens_tests.rs`](tests/tokens_tests.rs:1)

## Contributing
- Please follow the code style in the repo and add tests for new features.
- See top-level design notes: [`agent/TOP_LEVEL_DESIGN.md`](agent/TOP_LEVEL_DESIGN.md:1) and design doc: [`agent/EXHAUSTIVE_ENGINEERING_DESIGN.md`](agent/EXHAUSTIVE_ENGINEERING_DESIGN.md:1)

## License
MIT — see [`LICENSE`](LICENSE:1)

## Contact
Maintainer: erocs <github@erocs.org> (see [`Cargo.toml`](Cargo.toml:1))

-- end --