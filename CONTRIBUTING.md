# Contributing to Scholarly Text Tools

Thank you for your interest in contributing to the Scholarly Text Tools project! This document outlines the coding standards and conventions we follow.

## Naming Conventions

### Rust Naming Standards

We strictly follow Rust's official naming conventions:

#### Types, Structs, and Enums
- Use **PascalCase** for type names, struct names, and enum variants:
  ```rust
  pub struct ParserConfig { ... }
  pub enum Handler { ... }
  pub struct TagConfigBuilder { ... }
  ```

#### Functions and Variables
- Use **snake_case** for function names, variable names, and module names:
  ```rust
  pub fn create_add_word_update(...) -> ... { ... }
  let passage_line_break_handler = Handler::PassageLineBreak;
  ```

#### Constants
- Use **SCREAMING_SNAKE_CASE** for constants:
  ```rust
  pub const DEFAULT_PRECEDENCE: usize = usize::MAX;
  ```

#### Modules and Files
- Use **snake_case** for module names and file names:
  ```rust
  pub mod page_breaks;
  // file: page_breaks.rs
  ```

### Specific Project Conventions

#### Builder Pattern
- All config builders should follow the pattern `{ConfigName}Builder`:
  - `TagConfigBuilder` for `TagConfig`
  - `PageConfigBuilder` for `PageConfig`
  - `WordConfigBuilder` for `WordConfig`

#### Handler Naming
- Handler enum variants should use descriptive PascalCase names:
  - `PassageLineBreak` (not `PassgeLineBreak`)
  - `PageBreak`
  - `PrefixedWord`

#### Parser Function Naming
- Parser functions should be descriptive and use snake_case:
  - `empty_brackets_error_parser` (not `empty_beackets_error_parser`)
  - `bracketed_text_parser`
  - `word_parser`

## Code Quality Standards

### Documentation
- All public structs, enums, and functions must have rustdoc comments
- Use `///` for documentation comments
- Include examples where appropriate

### Error Handling
- Use `Result<T, E>` for functions that can fail
- Prefer custom error types using `thiserror` or `anyhow`
- Handle all error cases explicitly

### Testing
- Write unit tests for all new functionality
- Use property-based testing for parsers when appropriate
- Test both success and error cases

### Performance
- Prefer zero-copy parsing where possible
- Use string slices (`&str`) over owned strings (`String`) when appropriate
- Profile performance-critical code paths

## Commit Guidelines

- Use clear, descriptive commit messages
- Follow the format: `[area]: brief description`
- Examples:
  - `fix: correct typo in Handler enum (PassgeLineBreak -> PassageLineBreak)`
  - `refactor: rename SequenceConfigBuilder to TagConfigBuilder`
  - `docs: add naming conventions to CONTRIBUTING.md`

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes following these conventions
3. Run `cargo fmt` to format your code
4. Run `cargo clippy` to check for lints
5. Run `cargo test` to ensure all tests pass
6. Submit a pull request with a clear description

## Questions?

If you have questions about these conventions or need clarification, please open an issue or reach out to the maintainers.
