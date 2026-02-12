# Scholarly Editing Parser - AI Agent Guidelines

## Project Overview

Configurable lexical parser for scholarly critical editions, tokenizing Arabic manuscripts with editorial markup (cross-outs, emendations, marginal notes, lacunae). Zero-copy design using nom parser combinators.

## Architecture

### Core Components (parser/ package - heart of the repo)

- **[parser/src/core/orchestrator.rs](parser/src/core/orchestrator.rs)** - Builds handler sequences based on mode/config
- **[parser/src/core/state.rs](parser/src/core/state.rs)** - Token types and tokenizer state machine
- **[parser/src/core/handlers.rs](parser/src/core/handlers.rs)** - Handler enum and dispatch logic  
- **[parser/src/core/update_manager.rs](parser/src/core/update_manager.rs)** - Applies state transitions during parsing
- **[parser/src/core/updates.rs](parser/src/core/updates.rs)** - Update payload types
- **[parser/src/core/parsers.rs](parser/src/core/parsers.rs)** - Low-level nom-based parser functions
- **[parser/src/core/handlers/](parser/src/core/handlers/)** - Individual handler implementations (blocks, bracketed, tags, words, etc.)

### Data Flow Pattern

```
Input → Orchestrator (builds handler sequence) → Handlers (check patterns, emit updates) → UpdateManager (applies updates to state) → Tokens + Errors
```

### Key Abstractions

- **Token enum**: PageBreak, Word, WordInBlock, Tag, Block, Error - all include position metadata
- **Mode enum**: SinglePage, MultiplePages, Passage (affects parsing behavior)
- **Handler pattern**: Check if input matches → return `Option<Vec<TokenizerUpdate>>` → UpdateManager applies changes
- **ParserConfig**: Builder-based config system defining word chars, prefixes, brackets, tags, blocks, page breaks

## Code Style

### Naming Conventions
- **Types**: PascalCase (`WordToken`, `ParserConfig`)
- **Functions**: snake_case (`create_add_word_update`, `tokenize`)
- **Handlers**: `create_add_xxx_update` pattern for handler functions
- **Configs**: Builder pattern (`WordConfig::builder()`)

### Lifetime Management
- Extensive use of `'a` lifetime for zero-copy parsing - tokens borrow from input string
- Pattern: `pub fn tokenize<'a>(input: &'a str) -> (Vec<Token<'a>>, Vec<usize>)`

### Organization
- One file per handler type in [parser/src/core/handlers/](parser/src/core/handlers/)
- Separation: parsers (nom combinators) → handlers (business logic) → update_manager (state changes)
- Public API minimal - [parser/src/lib.rs](parser/src/lib.rs) exports only essential modules

## Error Handling

### Pattern: Permissive Parsing
- Errors become `Token::Error` entries, parsing continues
- Error indices returned separately: `(tokens, error_indices)`
- All errors have full position metadata (page, line, span)

### Error Detection Order Matters
Check error conditions BEFORE valid patterns. Example from [words.rs](parser/src/core/handlers/words.rs):
1. Check infixed errors (invalid chars mid-word)
2. Check prefixed errors (unwanted chars before word)
3. Check suffixed errors (unwanted chars after word)  
4. Finally check valid word patterns

### Error Types
See [parser/src/core/errors.rs](parser/src/core/errors.rs) for exhaustive list: `WordPrefixedWithUnwantedChars`, `EmptyBrackets`, `BlockStartInBrackets`, `InvalidPageBreak`, etc.

## Testing

### Test Organization
- **[tests/integration_tests.rs](parser/tests/integration_tests.rs)** - Comprehensive feature tests by category
- **[tests/kitchen_sink_test.rs](parser/tests/kitchen_sink_test.rs)** - All features combined
- **[tests/error_kitchen_sink_test.rs](parser/tests/error_kitchen_sink_test.rs)** - All error cases

### Standard Test Pattern
```rust
#[test]
fn test_feature() {
    let config = WordConfig::builder().chars(((0x0600, 0x06FF), vec![])).build();
    let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);
    let (tokens, errors) = orchestrator.tokenize("input text");
    
    assert_eq!(count_errors(&errors), 0);
    let words = extract_words(&tokens);
    assert_eq!(words[0], "expected");
}
```

Helper functions: `extract_words()`, `count_errors()`, `assert_has_error()`

### Unit Tests
Each handler has inline `#[cfg(test)]` tests - see [words.rs](parser/src/core/handlers/words.rs), [prefix.rs](parser/src/core/handlers/prefix.rs)

## Adding New Handlers

7-step process:

1. **Config** ([config.rs](parser/src/config.rs)): Add config type with builder, include `precedence: usize`
2. **Parser** ([parsers.rs](parser/src/core/parsers.rs)): Create nom parser function  
3. **Update Payload** ([updates.rs](parser/src/core/updates.rs)): Define payload struct, add variant to `TokenizerUpdate` enum
4. **Handler** ([handlers/](parser/src/core/handlers/)): Create `create_add_xxx_update()` function in new file
5. **Register** ([handlers.rs](parser/src/core/handlers.rs)): Add variant to `Handler` enum, implement in `process_with_context()`
6. **Build** ([orchestrator.rs](parser/src/core/orchestrator.rs)): Add builder function, include in handler sequence
7. **Apply** ([update_manager.rs](parser/src/core/update_manager.rs)): Handle update in `apply()`, implement state changes

See "How to Add New Handlers" section in research notes for detailed example.

## Project Conventions

### Precedence System
- All configs have `precedence: usize` field  
- `DEFAULT_PRECEDENCE = usize::MAX` (lowest priority)
- Lower numbers = higher priority
- Handlers automatically sorted by precedence in orchestrator

### Char Range Config
```rust
pub type Chars = ((u32, u32), Vec<u32>);  // (unicode_range, additional_chars)
// Example: ((0x600, 0x6FF), vec![0x003A])  // Arabic block + colon
```

### Block Behavior
- Blocks MUST start at line beginning - enforced via `HandlerContext::is_at_line_start`
- `WITH_TEXT` blocks parse content; `STANDALONE` blocks ignored
- End markers tracked in state: `current_block_end_marker`

### Bracket Skip Flag
Brackets with `skip: Some(true)` are excluded from word extraction (e.g., cross-outs, deletions)

### Context-Aware Parsing
```rust
pub struct HandlerContext {
    pub is_at_line_start: bool,
    pub is_in_block: bool,
    pub current_block_end_marker: Option<String>,
}
```
Handlers receive context to make state-dependent decisions - see [handlers.rs](parser/src/core/handlers.rs)

## Build and Test

```bash
# Build and test parser package
cargo build -p parser
cargo test -p parser

# Run specific tests
cargo test integration_tests
cargo test test_kitchen_sink

# Full workspace build
cargo build --workspace
cargo test --workspace

# Interface packages
cargo build -p parser-wasm     # WASM bindings
cargo build -p py-parser       # Python bindings (PyO3)
cargo build -p cli             # CLI tools
cargo build -p config-deserializer  # Config loading

# WASM workflow
cd wasm && pnpm install && pnpm build && pnpm test

# Python bindings
cd python/py-parser && maturin develop
```

## Integration Points

- **CLI** ([cli/](cli/)): Binaries in `src/bin/` for single_run, slicing, stress_test
- **WASM** ([wasm/parser-wasm/](wasm/parser-wasm/)): wasm-bindgen exports for JavaScript
- **Python** ([python/py-parser/](python/py-parser/)): PyO3 bindings - see `lib.rs` for Python class definitions
- **Config** ([config-deserializer/](config-deserializer/)): Loads TOML/JSON/YAML/XML configs

All interface crates depend on core `parser` package - make breaking changes carefully.

## Security Notes

- **Memory leak intentional**: [errors.rs](parser/src/core/errors.rs) uses `Box::leak()` for error message lifetime extension
- **No unsafe code** except in FFI boundaries (PyO3, wasm-bindgen)
- Input validation happens via parser - malformed input produces Error tokens, never panics
