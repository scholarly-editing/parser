# `parser`

The `parser` crate is the Rust core of the Scholarly Editing Parser. It tokenizes text according to a `ParserConfig` and returns typed tokens plus indexes for any error tokens.

## Configure and tokenize

Build a configuration in Rust with `ParserConfig::builder()`. At least one word definition is required.

~~~rust
use parser::{
    config::{ParserConfig, WordConfig},
    core::{
        orchestrator::Orchestrator,
        state::{Mode, Token},
    },
};

let config = ParserConfig::builder()
    .add_word(
        WordConfig::builder()
            .label("Arabic")
            .char_range(vec![0x0600, 0x06FF])
            .default_state("sound")
            .build(),
    )
    .build()
    .expect("a word definition is required");

let mode = Mode::SinglePage;
let parser = Orchestrator::new(&config, &mode);
let (tokens, error_indices) = parser.tokenize("باب الأسد والثور");

let words: Vec<&str> = tokens
    .iter()
    .filter_map(|token| match token {
        Token::Word(word) => Some(word.word),
        _ => None,
    })
    .collect();
~~~

Add further definitions with `add_page`, `add_block`, `add_prefix`, `add_suffix`, `add_brackets`, and `add_tag`. For file-based configuration, see [`config-deserializer`](../config-deserializer/README.md).

## Modes and results

- `Mode::SinglePage` parses text without configured page breaks.
- `Mode::MultiplePages` recognizes configured page markers.
- `Mode::Passage` parses passage text.

`Orchestrator::tokenize` returns `(Vec<Token>, Vec<usize>)`. Token variants include `Word`, `Tag`, `PageBreak`, `Block`, `WordInBlock`, and `Error`. Each error index points to an `Error` token in the same token vector. Word and structural tokens carry metadata such as page, line, order, and source span.

## Tests

Run this crate's tests from the workspace root:

~~~sh
cargo test -p parser
~~~
