use parser::{
    config::{
        BlockConfig, BlockType, BracketsConfig, PageConfig, ParserConfig, PrefixConfig,
        SuffixConfig, TagConfig, WordConfig,
    },
    core::{
        orchestrator::Orchestrator,
        state::{Mode, Token},
        errors::TextError,
    },
};

fn create_error_test_config() -> ParserConfig {
    ParserConfig {
        page: Some(vec![
            PageConfig {
                prefix: "fol.".to_string(),
                suffix: "r".to_string(),
                precedence: usize::MAX,
            },
        ]),
        block: vec![
            BlockConfig {
                start_marker: "[block]".to_string(),
                has_text: true,
                block_type: BlockType::WithText,
                end_marker: Some("---".to_string()),
                is_inline: false,
                precedence: usize::MAX,
            },
        ],
        brackets: vec![
            BracketsConfig {
                open: "{".to_string(),
                close: "}".to_string(),
                label: "curly".to_string(),
                skip: None,
                precedence: usize::MAX,
            },
        ],
        tags: vec![
            TagConfig {
                symbol: "<tag>".to_string(),
                label: "tag".to_string(),
                precedence: usize::MAX,
            },
        ],
        word: vec![
            WordConfig {
                label: "Arabic".to_string(),
                chars: ((0x0600, 0x06FF), vec![]),
                default_state: "sound".to_string(),
                precedence: usize::MAX,
            },
        ],
        prefix: vec![
            PrefixConfig {
                symbol: "*".to_string(),
                label: "prefix".to_string(),
                precedence: usize::MAX,
            },
        ],
        suffix: vec![
            SuffixConfig {
                symbol: "~".to_string(),
                label: "suffix".to_string(),
                precedence: usize::MAX,
            },
        ],
    }
}

fn assert_has_error(tokens: &[Token], error_indices: &[usize], expected_error: &TextError) {
    let found = error_indices.iter().any(|idx| {
        if let Token::Error(e) = &tokens[*idx] {
            &e.error == expected_error
        } else {
            false
        }
    });

    if !found {
        let actual_errors: Vec<_> = error_indices.iter().map(|idx| {
            if let Token::Error(e) = &tokens[*idx] {
                format!("{:?}", e.error)
            } else {
                "Not an error token".to_string()
            }
        }).collect();

        panic!("Expected error {:?} not found. Found: {:?}", expected_error, actual_errors);
    }
}

#[test]
fn test_word_errors() {
    let config = create_error_test_config();
    let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

    // 1. Prefixed with unwanted chars
    let (tokens, errors) = orchestrator.tokenize("@كلمة");
    assert_has_error(&tokens, &errors, &TextError::WordPrefixedWithUnwantedChars);

    // 2. Suffixed with unwanted chars
    let (tokens, errors) = orchestrator.tokenize("كلمة@");
    assert_has_error(&tokens, &errors, &TextError::WordSuffixedWithUnwantedChars);

    // 3. Infixed with unwanted chars
    let (tokens, errors) = orchestrator.tokenize("كل@مة");
    assert_has_error(&tokens, &errors, &TextError::WordInfixedWithUnwantedChars);
}

#[test]
fn test_bracket_errors() {
    let config = create_error_test_config();
    let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

    // 1. Empty brackets
    let (tokens, errors) = orchestrator.tokenize("{}");
    assert_has_error(&tokens, &errors, &TextError::EmptyBrackets);

    // 2. Space in brackets (start)
    let (tokens, errors) = orchestrator.tokenize("{ كلمة}");
    assert_has_error(&tokens, &errors, &TextError::SpaceInBrackets);

    // 3. Space in brackets (end)
    let (tokens, errors) = orchestrator.tokenize("{كلمة }");
    assert_has_error(&tokens, &errors, &TextError::SpaceInBrackets);

    // 4. Block start in brackets
    let (tokens, errors) = orchestrator.tokenize("{[block]}");
    assert_has_error(&tokens, &errors, &TextError::BlockStartInBrackets("block".to_string()));

    // 5. Block end in brackets
    let (tokens, errors) = orchestrator.tokenize("{---}");
    assert_has_error(&tokens, &errors, &TextError::BlockEndInBrackets);

    // 6. Page break in brackets
    let (tokens, errors) = orchestrator.tokenize("{fol.1r}");
    assert_has_error(&tokens, &errors, &TextError::InvalidPageBreak("fol.1r".to_string()));

    // 7. Tag in brackets
    let (tokens, errors) = orchestrator.tokenize("{<tag>}");
    assert_has_error(&tokens, &errors, &TextError::Custom("Invalid position for tag".to_string()));
}

#[test]
fn test_affix_errors() {
    let config = create_error_test_config();
    let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

    // 1. Invalid suffix position (start of word/standalone)
    let (tokens, errors) = orchestrator.tokenize("~");
    assert_has_error(&tokens, &errors, &TextError::InvalidSuffixPosition);

    // 2. Invalid prefix use (space after prefix)
    let (tokens, errors) = orchestrator.tokenize("* كلمة");
    assert_has_error(&tokens, &errors, &TextError::InvalidPrefixUse);
}

#[test]
fn test_unknown_error() {
    let config = create_error_test_config();
    let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

    // Unknown characters
    let (tokens, errors) = orchestrator.tokenize("abc"); // Latin chars not configured
    // This should produce a Custom error with "Unwanted characters..."

    let found = errors.iter().any(|idx| {
        if let Token::Error(e) = &tokens[*idx] {
            if let TextError::Custom(msg) = &e.error {
                return msg.contains("Unwanted characters");
            }
        }
        false
    });

    assert!(found, "Expected unknown error for unconfigured characters");
}

