# Project Guidelines

## Overview

This is a **scholarly text parser** for medieval/classical manuscripts, primarily Arabic texts. It handles special editorial markup (page breaks, brackets, annotations, blocks) with configurable, precedence-based parsing. The workspace provides Rust core, CLI, Python (PyO3), and WASM bindings.

## Architecture

**Core flow:** `Config → Orchestrator → Handlers → Parsers → Updates → State`

### Key Components

- **[parser/src/core/orchestrator.rs](../parser/src/core/orchestrator.rs)**: Central coordinator, builds handler sequences, manages `HandlerContext`
- **[parser/src/core/handlers.rs](../parser/src/core/handlers.rs)**: Strategy pattern enum (`Word`, `Tag`, `PrefixedWord`, `Bracketed`, `PageBreak`, etc.) - each handler module in [handlers/](../parser/src/core/handlers/)
- **[parser/src/core/parsers.rs](../parser/src/core/parsers.rs)**: Pure `nom` combinator functions returning `IResult<&str, T>`
- **[parser/src/core/updates.rs](../parser/src/core/updates.rs)**: `TokenizerUpdate` enum with payload structs for state mutations
- **[parser/src/core/update_manager.rs](../parser/src/core/update_manager.rs)**: Applies updates to state, tracks position/line/block context
- **[parser/src/core/state.rs](../parser/src/core/state.rs)**: `TokenizerState` holds tokens, errors, remaining input, block state
- **[parser/src/core/errors.rs](../parser/src/core/errors.rs)**: `TextError` enum - errors become tokens, parsing continues

### Cross-Language Bindings

- **Python** ([Python/py-parser/src/lib.rs](../python/py-parser/src/lib.rs)): Thin `#[pyclass]` wrapper around `PassageState`, accepts TOML config strings
- **WASM** ([WASM/parser-wasm/src/lib.rs](../wasm/parser-wasm/src/lib.rs)): Uses Unicode NFC normalization on input, returns `JsValue` via `serde-wasm-bindgen`

## Code Style

Follow strict Rust conventions per [CONTRIBUTING.md](../CONTRIBUTING.md):

- **Types/Enums**: `PascalCase` (`ParserConfig`, `Handler`, `TokenizerUpdate`)
- **Functions/Variables**: `snake_case` (`create_add_word_update`, `is_valid_char`)
- **Constants**: `SCREAMING_SNAKE_CASE` (`DEFAULT_PRECEDENCE`)
- **Builders**: Use pattern `{ConfigName}Builder` (e.g., `TagConfigBuilder`)
- **Lifetimes**: Pervasive `<'a>` for zero-copy parsing - tokens hold `&'a str` slices

### Critical Patterns

**Builder pattern (required for all configs):**
```rust
let config = ParserConfig::builder()
    .add_word(WordConfig { /* ... */ })
    .add_prefix(PrefixConfig { /* ... */ })
    .build()
    .unwrap();
```

**Handler creation pattern:**
```rust
pub fn create_add_X_update<'a>(
    input: &'a str,
    config: &'a Config
) -> Option<Vec<TokenizerUpdate<'a>>> {
    parser(input, config).ok().map(|(rest, data)| {
        vec![TokenizerUpdate::AddX(XPayload {
            rest: Some(rest),
            len: data.chars().count(),
            // ... semantic fields
        })]
    })
}
```

**Zero-copy principle**: Never clone strings. Use `&'a str` slices. Only allocate for error messages (via `Box::leak` for static lifetime).

**Precedence sorting**: All configs have `precedence: usize`. Lower number = higher priority (checked first). Handlers sorted once during orchestrator construction.

## Build and Test

```bash
# Build workspace
cargo build --release

# Run binaries
cargo run --bin single_run
cargo run --bin experiments

# Test core parser
cargo test -p parser

# Test specific suite
cargo test --test integration_tests
cargo test --test kitchen_sink_test

# Format and lint
cargo fmt
cargo clippy
```

**Python bindings:**
```bash
cd python/py-parser
maturin build --release --target x86_64-unknown-linux-gnu
```

**WASM testing:**
```bash
cd wasm
pnpm install
pnpm test  # vitest
```

### Test Pattern

See [parser/tests/integration_tests.rs](../parser/tests/integration_tests.rs):
```rust
fn create_test_config() -> ParserConfig {
    ParserConfig::builder().add_word(/* ... */).build().unwrap()
}

#[test]
fn test_feature() {
    let parser = Orchestrator::new(&config, &Mode::SinglePage);
    let (tokens, errors) = parser.tokenize(input);
    assert_eq!(errors.len(), 0);
    assert!(matches!(tokens[0], Token::Word(_)));
}
```

## Project Conventions

### Error Recovery Architecture
Errors **do not stop parsing**. Invalid text becomes `Token::Error(TokenizerError)` with span info. The `errors: Vec<usize>` field tracks error token indices. This design enables batch validation.

### Context-Sensitive Parsing
Handlers receive `HandlerContext` with:
- `is_at_line_start: bool` - affects bracket/block parsing rules
- `current_block: Option<BlockState>` - enables nested block validation

See [parser/src/core/orchestrator.rs](../parser/src/core/orchestrator.rs#L15-L30) for context management.

### Handler Modules Organization
Each handler type lives in `parser/src/core/handlers/{name}.rs`. When adding handlers:
1. Create module in `handlers/` directory
2. Define update payload in [updates.rs](../parser/src/core/updates.rs)
3. Add variant to `Handler` enum in [handlers.rs](../parser/src/core/handlers.rs)
4. Add parser function to [parsers.rs](../parser/src/core/parsers.rs)
5. Implement `process_with_context()` for strategy pattern

### Unicode Handling
- Parser supports **Arabic** (U+0600 to U+06FF) via `is_valid_char()` in [parsers.rs](../parser/src/core/parsers.rs)
- WASM bindings **normalize input** (NFC) - Python bindings currently do not
- Use Unicode-aware `chars().count()` for `len` fields, not `.len()`

### Configuration Layer
Config deserialization is separate crate ([config-deserializer/](../config-deserializer/)). Supports TOML/JSON/YAML via `ParserConfigDeserializer`. See [config.example.toml](../config-deserializer/config.example.toml) for schema.

### Precedence System
When multiple handlers match same input, precedence determines order:
- Lower number = higher priority
- Default: `usize::MAX` (lowest priority)
- Critical for overlapping patterns (e.g., prefixed words vs. plain words)

## Integration Points

### Dependencies
- **nom** (7.1.3): Parser combinators - all parsing functions use this
- **serde**: Serialization for tokens, configs, errors
- **pyo3** (0.23.4): Python bindings via maturin
- **wasm-bindgen**: JavaScript interop for WASM target

### Workspace Structure
```
parser/              # Core library (no_std compatible)
cli/                 # Command-line tools
config-deserializer/ # Config parsing (TOML/JSON/YAML)
Python/py-parser/    # PyO3 bindings
WASM/parser-wasm/    # wasm-bindgen target
```

Use `path = "../parser"` dependencies for intra-workspace references.

## Security

### Input Validation
- Character validation via `is_valid_char()` restricts to defined Unicode ranges
- Config builders return `Result` - validate before use
- Page break parsing checks format patterns (e.g., "fol.2r") - see [page_breaks.rs](../parser/src/core/handlers/page_breaks.rs)

### Memory Safety
Zero-copy design means tokens borrow input lifetime `<'a>`. Invalid lifetimes cause compile errors. Never use `unsafe` to extend lifetimes.

### Error Leak Pattern
Error messages use `Box::leak()` to create `&'static str` from owned strings. This is intentional - errors are rare and memory is negligible. See [errors.rs](../parser/src/core/errors.rs).
