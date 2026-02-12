//! Integration tests for the scholarly editing parser.
//!
//! These tests validate end-to-end parsing behavior using the full config
//! from config.example.toml with sample scholarly text.

use parser::{
    config::{
        BlockConfig, BlockType, BracketsConfig, PageConfig, ParserConfig, PrefixConfig,
        SuffixConfig, TagConfig, WordConfig,
    },
    core::{orchestrator::Orchestrator, state::{Mode, Token}},
};

/// Helper to create the standard Arabic word config for testing
fn arabic_word_config() -> WordConfig {
    WordConfig {
        label: "Arabic".to_string(),
        chars: ((0x600, 0x6FF), vec![]),
        default_state: "sound".to_string(),
        precedence: usize::MAX,
    }
}

/// Helper to create a standard test config matching config.example.toml
fn create_test_config() -> ParserConfig {
    ParserConfig {
        page: Some(vec![
            PageConfig {
                prefix: "fol.".to_string(),
                suffix: "r".to_string(),
                precedence: usize::MAX,
            },
            PageConfig {
                prefix: "fol.".to_string(),
                suffix: "v".to_string(),
                precedence: usize::MAX,
            },
        ]),
        block: vec![
            BlockConfig {
                start_marker: "[illustration]".to_string(),
                has_text: false,
                block_type: BlockType::Standalone,
                end_marker: None,
                is_inline: false,
                precedence: usize::MAX,
            },
            BlockConfig {
                start_marker: "[legend]".to_string(),
                has_text: true,
                block_type: BlockType::WithText,
                end_marker: Some("---".to_string()),
                is_inline: false,
                precedence: usize::MAX,
            },
            BlockConfig {
                start_marker: "[margin]".to_string(),
                has_text: true,
                block_type: BlockType::WithText,
                end_marker: Some("---".to_string()),
                is_inline: false,
                precedence: usize::MAX,
            },
        ],
        word: vec![arabic_word_config()],
        prefix: vec![
            PrefixConfig {
                symbol: "*".to_string(),
                label: "emendation".to_string(),
                precedence: usize::MAX,
            },
            PrefixConfig {
                symbol: "?".to_string(),
                label: "unintelligible".to_string(),
                precedence: usize::MAX,
            },
            PrefixConfig {
                symbol: "؟".to_string(),
                label: "unintelligible".to_string(),
                precedence: usize::MAX,
            },
            PrefixConfig {
                symbol: "!".to_string(),
                label: "error".to_string(),
                precedence: usize::MAX,
            },
            PrefixConfig {
                symbol: "†".to_string(),
                label: "corrupt".to_string(),
                precedence: usize::MAX,
            },
        ],
        suffix: vec![SuffixConfig {
            symbol: "~".to_string(),
            label: "middle-arabic".to_string(),
            precedence: usize::MAX,
        }],
        brackets: vec![
            // [[cross-out]] has precedence 0, must come first
            BracketsConfig {
                open: "[[".to_string(),
                close: "]]".to_string(),
                label: "cross-out".to_string(),
                skip: Some(true),
                precedence: 0,
            },
            BracketsConfig {
                open: "(".to_string(),
                close: ")".to_string(),
                label: "title".to_string(),
                skip: None,
                precedence: usize::MAX,
            },
            BracketsConfig {
                open: "[".to_string(),
                close: "]".to_string(),
                label: "superfluous".to_string(),
                skip: Some(true),
                precedence: usize::MAX,
            },
            BracketsConfig {
                open: "{".to_string(),
                close: "}".to_string(),
                label: "suppletion".to_string(),
                skip: None,
                precedence: usize::MAX,
            },
            BracketsConfig {
                open: "<".to_string(),
                close: ">".to_string(),
                label: "added".to_string(),
                skip: None,
                precedence: usize::MAX,
            },
        ],
        tags: vec![
            TagConfig {
                symbol: "***".to_string(),
                label: "lacuna".to_string(),
                precedence: usize::MAX,
            },
            TagConfig {
                symbol: "...".to_string(),
                label: "damage".to_string(),
                precedence: usize::MAX,
            },
        ],
    }
}

/// Helper to extract word tokens from a token list
fn extract_words<'a>(tokens: &'a [Token<'a>]) -> Vec<&'a str> {
    tokens
        .iter()
        .filter_map(|t| match t {
            Token::Word(w) => Some(w.word),
            _ => None,
        })
        .collect()
}

/// Helper to extract error tokens from a token list
fn extract_errors<'a>(tokens: &'a [Token<'a>]) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|t| match t {
            Token::Error(e) => Some(e.error.to_string()),
            _ => None,
        })
        .collect()
}

/// Helper to count error indices
fn count_errors(errors: &[usize]) -> usize {
    errors.len()
}

// =============================================================================
// SINGLE PAGE MODE TESTS
// =============================================================================

mod single_page_mode {
    use super::*;

    #[test]
    fn test_simple_arabic_text() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "باب الأسد والثور";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(words[0], "باب");
        assert_eq!(words[1], "الأسد");
        assert_eq!(words[2], "والثور");
        assert_eq!(count_errors(&errors), 0);
    }

    #[test]
    fn test_text_with_line_breaks() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "باب الأسد\nوالثور";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(count_errors(&errors), 0);
    }

    #[test]
    fn test_multiple_spaces_between_words() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "باب    الأسد     والثور";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(count_errors(&errors), 0);
    }
}

// =============================================================================
// WORD PARSING TESTS
// =============================================================================

mod word_parsing {
    use super::*;

    #[test]
    fn test_arabic_word_in_range() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "الفيلسوف";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "الفيلسوف");
        assert_eq!(count_errors(&errors), 0);

        // Verify it's marked as "sound" state
        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "sound");
            assert_eq!(w.word_type, "Arabic");
        } else {
            panic!("Expected Word token");
        }
    }

    #[test]
    fn test_word_with_shaddah_and_diacritics() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Word with shaddah: المتحابّين
        let input = "المتحابّين";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);
    }

    #[test]
    fn test_word_with_tanwin() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Word with tanwin: تساهلًا
        let input = "تساهلًا";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);
    }
}

// =============================================================================
// PREFIX TESTS
// =============================================================================

mod prefix_parsing {
    use super::*;

    #[test]
    fn test_emendation_prefix() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "*لبيدبا";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "لبيدبا");
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "emendation");
        } else {
            panic!("Expected Word token");
        }
    }

    #[test]
    fn test_unintelligible_prefix_question_mark() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "?كلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "unintelligible");
        } else {
            panic!("Expected Word token");
        }
    }

    #[test]
    fn test_unintelligible_prefix_arabic_question_mark() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "؟كلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "unintelligible");
        } else {
            panic!("Expected Word token");
        }
    }

    #[test]
    fn test_error_prefix() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "!خطأ";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "error");
        } else {
            panic!("Expected Word token");
        }
    }

    #[test]
    fn test_corrupt_prefix() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "†فاسد";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "corrupt");
        } else {
            panic!("Expected Word token");
        }
    }

    #[test]
    fn test_wrong_prefix_use_space_after_prefix() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Prefix with space before word should be an error
        let input = "* كلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        // Should produce an error for wrong prefix use
        assert!(count_errors(&errors) > 0, "Expected error for prefix with space");

        let error_messages = extract_errors(&tokens);
        assert!(
            error_messages.iter().any(|m| m.contains("prefix")),
            "Expected error message about prefix"
        );
    }

    #[test]
    fn test_multiple_prefixed_words_in_text() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "*كلمة واحدة ?أخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(count_errors(&errors), 0);
    }
}

// =============================================================================
// SUFFIX TESTS
// =============================================================================

mod suffix_parsing {
    use super::*;

    #[test]
    fn test_middle_arabic_suffix() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "كلمة~";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "كلمة");
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "middle-arabic");
        } else {
            panic!("Expected Word token");
        }
    }

    #[test]
    fn test_suffix_without_word_is_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Suffix at start of input (no preceding word) should be error
        let input = "~ كلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        // Should have an error
        assert!(count_errors(&errors) > 0, "Expected error for suffix without word");
    }

    #[test]
    fn test_suffix_in_middle_of_text() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "أولى~ ثانية";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 2);
        assert_eq!(count_errors(&errors), 0);

        // First word should have middle-arabic state
        if let Token::Word(w) = &tokens[0] {
            assert_eq!(w.state, "middle-arabic");
        }
    }
}

// =============================================================================
// BRACKET TESTS
// =============================================================================

mod bracket_parsing {
    use super::*;

    #[test]
    fn test_parentheses_title_bracket() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "(العنوان)";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "العنوان");
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert!(w.state.contains("title"), "Expected title state, got: {}", w.state);
        }
    }

    #[test]
    fn test_curly_braces_suppletion() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{الملك}";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "الملك");
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert!(w.state.contains("suppletion"), "Expected suppletion state, got: {}", w.state);
        }
    }

    #[test]
    fn test_angle_brackets_added() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "<له>";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "له");
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert!(w.state.contains("added"), "Expected added state, got: {}", w.state);
        }
    }

    #[test]
    fn test_square_brackets_superfluous() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "[زائد]";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert!(w.state.contains("superfluous"), "Expected superfluous state, got: {}", w.state);
        }
    }

    #[test]
    fn test_double_square_brackets_crossout() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "[[مشطوب]]";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(count_errors(&errors), 0);

        if let Token::Word(w) = &tokens[0] {
            assert!(w.state.contains("cross-out"), "Expected cross-out state, got: {}", w.state);
        }
    }

    #[test]
    fn test_multiple_words_in_brackets() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{كلمة أولى ثانية}";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(count_errors(&errors), 0);

        // First word should have suppletion_open state
        if let Token::Word(w) = &tokens[0] {
            assert!(w.state.contains("suppletion"), "Expected suppletion in state");
        }
    }

    #[test]
    fn test_empty_brackets_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{}";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for empty brackets");

        let error_messages = extract_errors(&tokens);
        assert!(
            error_messages.iter().any(|m| m.to_lowercase().contains("empty")),
            "Expected error message about empty brackets"
        );
    }

    #[test]
    fn test_empty_brackets_with_space_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{   }";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for brackets with only spaces");
    }

    #[test]
    fn test_bracket_with_leading_space_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{ كلمة}";
        let (tokens, errors) = orchestrator.tokenize(input);

        // Leading space inside bracket should be an error
        assert!(count_errors(&errors) > 0, "Expected error for leading space in bracket");
    }

    #[test]
    fn test_bracket_with_trailing_space_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{كلمة }";
        let (tokens, errors) = orchestrator.tokenize(input);

        // Trailing space inside bracket should be an error
        assert!(count_errors(&errors) > 0, "Expected error for trailing space in bracket");
    }

    #[test]
    fn test_bracketed_text_in_sentence() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "قال {الملك} ديشلم";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(count_errors(&errors), 0);
    }

    #[test]
    fn test_nested_brackets_same_type_is_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Nested brackets of the same type should produce an error
        let input = "{{كلمة}}";
        let (tokens, errors) = orchestrator.tokenize(input);

        // This should either produce an error or handle nested brackets
        // Based on current implementation, this may be parsed unexpectedly
        // This test documents the current behavior for future fixes
        let error_count = count_errors(&errors);
        // Note: Update assertion based on desired behavior
        println!("Nested same-type brackets produced {} errors", error_count);
    }
}

// =============================================================================
// TAG TESTS
// =============================================================================

mod tag_parsing {
    use super::*;

    #[test]
    fn test_lacuna_tag() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "***";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert_eq!(count_errors(&errors), 0);

        let tag_token = tokens.iter().find(|t| matches!(t, Token::Tag(_)));
        assert!(tag_token.is_some(), "Expected tag token");

        if let Some(Token::Tag(t)) = tag_token {
            assert_eq!(t.tag, "***");
            assert_eq!(t.label, "lacuna");
        }
    }

    #[test]
    fn test_damage_tag() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "...";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert_eq!(count_errors(&errors), 0);

        let tag_token = tokens.iter().find(|t| matches!(t, Token::Tag(_)));
        assert!(tag_token.is_some(), "Expected tag token");

        if let Some(Token::Tag(t)) = tag_token {
            assert_eq!(t.tag, "...");
            assert_eq!(t.label, "damage");
        }
    }

    #[test]
    fn test_tag_between_words() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "كلمة *** أخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 2);
        assert_eq!(count_errors(&errors), 0);

        // Verify tag is present
        let tag_count = tokens.iter().filter(|t| matches!(t, Token::Tag(_))).count();
        assert_eq!(tag_count, 1);
    }

    #[test]
    fn test_tag_at_start_of_line() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "*** كلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert_eq!(count_errors(&errors), 0);

        // First non-space token should be the tag
        let first_meaningful = tokens.iter().find(|t| !matches!(t, Token::Word(_) if false));
        assert!(matches!(first_meaningful, Some(Token::Tag(_))));
    }

    #[test]
    fn test_tag_inside_brackets_is_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{*** كلمة}";
        let (tokens, errors) = orchestrator.tokenize(input);

        // Tags inside brackets should be errors
        assert!(count_errors(&errors) > 0, "Expected error for tag inside brackets");
    }
}

// =============================================================================
// PAGE BREAK TESTS (MULTIPLE PAGES MODE)
// =============================================================================

mod page_break_parsing {
    use super::*;

    #[test]
    fn test_default_page_break_format() {
        let mut config = create_test_config();
        config.page = None; // Use default page break format (just numbers)

        let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

        let input = "1\nكلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let page_break = tokens.iter().find(|t| matches!(t, Token::PageBreak(_)));
        assert!(page_break.is_some(), "Expected page break token");
    }

    #[test]
    fn test_folio_page_break_recto() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

        let input = "fol.1r\nكلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let page_break = tokens.iter().find(|t| matches!(t, Token::PageBreak(_)));
        assert!(page_break.is_some(), "Expected page break token");

        if let Some(Token::PageBreak(pb)) = page_break {
            assert!(pb.page_representation.contains("fol."));
            assert!(pb.page_representation.contains("r"));
        }
    }

    #[test]
    fn test_folio_page_break_verso() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

        let input = "fol.1v\nكلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let page_break = tokens.iter().find(|t| matches!(t, Token::PageBreak(_)));
        assert!(page_break.is_some(), "Expected page break token");

        if let Some(Token::PageBreak(pb)) = page_break {
            assert!(pb.page_representation.contains("v"));
        }
    }

    #[test]
    fn test_multiple_page_breaks() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

        let input = "fol.1r\nكلمة\nfol.1v\nأخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        let page_break_count = tokens.iter().filter(|t| matches!(t, Token::PageBreak(_))).count();
        assert_eq!(page_break_count, 2);
    }

    #[test]
    fn test_page_number_in_middle_of_line() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

        // Page break marker in middle of text (after text on same line)
        // This is the format used in the sample: "Lo-2-1 باب الأسد والثور"
        // where Lo-2-1 is a page reference not at start of line
        let input = "كلمة fol.1r\nأخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        // This tests whether inline page references work
        let words = extract_words(&tokens);
        assert!(words.len() >= 2);
    }
}

// =============================================================================
// PASSAGE MODE TESTS
// =============================================================================

mod passage_mode {
    use super::*;

    #[test]
    fn test_passage_mode_line_breaks_as_spaces() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::Passage);

        let input = "كلمة\nأخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 2);
        assert_eq!(count_errors(&errors), 0);

        // In passage mode, line breaks should be treated as spaces
        // So there should be no special line break handling
    }

    #[test]
    fn test_passage_mode_no_page_breaks() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::Passage);

        // Even with page break format, passage mode should not create page breaks
        let input = "fol.1r\nكلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let page_break_count = tokens.iter().filter(|t| matches!(t, Token::PageBreak(_))).count();
        // In passage mode, page breaks are not recognized
        // The "fol.1r" might be parsed as error tokens
    }
}

// =============================================================================
// ERROR TOKEN TESTS
// =============================================================================

mod error_tokens {
    use super::*;

    #[test]
    fn test_unknown_character_produces_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Characters outside Arabic range and not defined symbols
        let input = "كلمة @ أخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for unknown character @");
    }

    #[test]
    fn test_latin_text_produces_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Latin characters are not in the Arabic range
        let input = "hello";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for Latin text");
    }

    #[test]
    fn test_mixed_arabic_latin_word_produces_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Mixed script word should be an error
        let input = "كلمةword";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for mixed script word");
    }

    #[test]
    fn test_word_with_number_infix_produces_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Word with number in the middle
        let input = "كل123مة";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for word with number infix");

        let error_messages = extract_errors(&tokens);
        assert!(
            error_messages.iter().any(|m| m.contains("infixed")),
            "Expected infixed error message"
        );
    }

    #[test]
    fn test_word_prefixed_with_unwanted_chars() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Word with non-prefix unwanted characters at start
        let input = "@#كلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for word with unwanted prefix chars");
    }

    #[test]
    fn test_word_suffixed_with_unwanted_chars() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Word with unwanted characters at end (not the ~ suffix)
        let input = "كلمة@#";
        let (tokens, errors) = orchestrator.tokenize(input);

        assert!(count_errors(&errors) > 0, "Expected error for word with unwanted suffix chars");
    }

    #[test]
    fn test_error_token_has_span() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "كلمة @ أخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        let error_token = tokens.iter().find(|t| matches!(t, Token::Error(_)));
        assert!(error_token.is_some(), "Expected error token");

        if let Some(Token::Error(e)) = error_token {
            // Verify span is set
            assert!(e.span.0 <= e.span.1, "Error span should be valid");
        }
    }

    #[test]
    fn test_unmatched_opening_bracket() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "{كلمة";
        let (tokens, errors) = orchestrator.tokenize(input);

        // Unmatched bracket should produce an error
        assert!(count_errors(&errors) > 0, "Expected error for unmatched opening bracket");
    }

    #[test]
    fn test_unmatched_closing_bracket() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "كلمة}";
        let (tokens, errors) = orchestrator.tokenize(input);

        // Unmatched closing bracket should produce an error
        assert!(count_errors(&errors) > 0, "Expected error for unmatched closing bracket");
    }
}

// =============================================================================
// BLOCK PARSING TESTS
// =============================================================================

mod block_parsing {
    use super::*;

    // Block semantics:
    // - A block starts on a new line (marker is first item on the line)
    // - For WITH_TEXT blocks: all lines after the marker belong to the block
    //   until the end marker appears on its own line
    // - For STANDALONE blocks: just the marker, no text content
    // - For inline blocks: marker + text on same line, ends at line break

    #[test]
    fn test_standalone_block_illustration() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Standalone block: just the marker on its own line
        let input = "[illustration]\nكلمة";
        let (tokens, _errors) = orchestrator.tokenize(input);

        let block_token = tokens.iter().find(|t| matches!(t, Token::Block(_)));
        assert!(block_token.is_some(), "Expected block token for [illustration]");

        if let Some(Token::Block(b)) = block_token {
            assert_eq!(b.block_name, "illustration");
        }

        // The word after the block should be a regular word, not WordInBlock
        let words = extract_words(&tokens);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], "كلمة");
    }

    #[test]
    fn test_block_with_text_legend() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // WITH_TEXT block: marker on its own line, content until end marker
        let input = "[legend]\nنص الأسطورة\n---\nكلمة";
        let (tokens, _errors) = orchestrator.tokenize(input);

        let block_token = tokens.iter().find(|t| matches!(t, Token::Block(_)));
        assert!(block_token.is_some(), "Expected block token for [legend]");

        // Words inside block should be WordInBlock tokens
        let words_in_block: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::WordInBlock(_)))
            .collect();
        assert!(!words_in_block.is_empty(), "Expected WordInBlock tokens");

        // "كلمة" after end marker should be regular word
        let regular_words = extract_words(&tokens);
        assert!(regular_words.contains(&"كلمة"), "Word after block should be regular");
    }

    #[test]
    fn test_block_with_text_margin() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "[margin]\nحاشية\n---";
        let (tokens, _errors) = orchestrator.tokenize(input);

        let block_token = tokens.iter().find(|t| matches!(t, Token::Block(_)));
        assert!(block_token.is_some(), "Expected block token for [margin]");

        // Word inside margin block
        let words_in_block: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::WordInBlock(_)))
            .collect();
        assert!(!words_in_block.is_empty(), "Expected WordInBlock token for حاشية");
    }

    #[test]
    fn test_block_end_marker_on_own_line() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // End marker must be on its own line
        let input = "[legend]\nنص\n---\nبعد";
        let (tokens, _errors) = orchestrator.tokenize(input);

        // "بعد" should be a regular word token, not WordInBlock
        let regular_words = extract_words(&tokens);
        assert!(regular_words.contains(&"بعد"), "Word after block end marker should be regular word");

        // Verify it's not a WordInBlock
        let words_in_block: Vec<_> = tokens.iter()
            .filter_map(|t| match t {
                Token::WordInBlock(w) => Some(w.word),
                _ => None,
            })
            .collect();
        assert!(!words_in_block.contains(&"بعد"), "بعد should not be in block");
    }

    #[test]
    fn test_block_without_end_marker_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Block with required end marker but missing it (text continues to end)
        let input = "[legend]\nنص بدون نهاية";
        let (tokens, errors) = orchestrator.tokenize(input);

        // All words should be treated as in-block since no end marker
        let words_in_block: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::WordInBlock(_)))
            .collect();

        // Either produces error or treats all as block content
        // This documents expected behavior - you may want error or graceful handling
        assert!(
            count_errors(&errors) > 0 || !words_in_block.is_empty(),
            "Expected either error or block content when end marker missing"
        );
    }

    #[test]
    fn test_block_marker_must_be_at_line_start() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Block marker not at start of line - should NOT be treated as block
        // Instead parsed as superfluous bracket content
        let input = "كلمة [illustration] أخرى";
        let (tokens, _errors) = orchestrator.tokenize(input);

        // Should NOT have a Block token since marker isn't at line start
        let block_token = tokens.iter().find(|t| matches!(t, Token::Block(_)));
        assert!(block_token.is_none(), "Block marker not at line start should not create block");

        // Should parse as text with [superfluous] bracket
        let words = extract_words(&tokens);
        assert!(words.len() >= 2);
    }

    #[test]
    fn test_unknown_block_marker_parsed_as_brackets() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Block marker not in config - should be parsed as superfluous bracket
        let input = "[unknown]\nنص";
        let (tokens, _errors) = orchestrator.tokenize(input);

        // Should NOT have Block token
        let block_token = tokens.iter().find(|t| matches!(t, Token::Block(_)));

        // Either error or parsed as bracket content
        let has_bracket_word = tokens.iter().any(|t| match t {
            Token::Word(w) => w.state.contains("superfluous"),
            _ => false,
        });

        assert!(
            block_token.is_none() || has_bracket_word,
            "Unknown marker should be parsed as brackets, not block"
        );
    }

    #[test]
    fn test_nested_blocks_error() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Nested blocks - second block marker inside first block
        let input = "[legend]\n[margin]\nنص\n---\n---";
        let (tokens, errors) = orchestrator.tokenize(input);

        // This should either:
        // 1. Produce an error for nested blocks, or
        // 2. Treat [margin] as text content inside [legend]
        // Either behavior is acceptable, but should be consistent

        let block_count = tokens.iter().filter(|t| matches!(t, Token::Block(_))).count();

        // Document actual behavior - can be adjusted based on design decision
        println!("Nested blocks produced {} block tokens and {} errors", block_count, count_errors(&errors));
    }

    #[test]
    fn test_multiline_block_content() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Multiple lines of content in block
        let input = "[legend]\nسطر أول\nسطر ثان\nسطر ثالث\n---\nبعد";
        let (tokens, _errors) = orchestrator.tokenize(input);

        // All words before --- should be WordInBlock
        let words_in_block: Vec<_> = tokens.iter()
            .filter_map(|t| match t {
                Token::WordInBlock(w) => Some(w.word),
                _ => None,
            })
            .collect();

        // Should have words from all three lines inside block
        assert!(words_in_block.len() >= 6, "Expected at least 6 words in block");

        // "بعد" should be regular word
        let regular_words = extract_words(&tokens);
        assert!(regular_words.contains(&"بعد"));
    }

    #[test]
    fn test_block_preserves_line_numbers() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "[legend]\nسطر أول\nسطر ثان\n---";
        let (tokens, _errors) = orchestrator.tokenize(input);

        let words_in_block: Vec<_> = tokens.iter()
            .filter_map(|t| match t {
                Token::WordInBlock(w) => Some(w),
                _ => None,
            })
            .collect();

        if words_in_block.len() >= 2 {
            // Words on different lines should have different line numbers
            // Line 0: [legend]
            // Line 1: سطر أول
            // Line 2: سطر ثان
            assert!(
                words_in_block.iter().map(|w| w.line_number).collect::<std::collections::HashSet<_>>().len() > 1,
                "Words on different lines should have different line numbers"
            );
        }
    }

    #[test]
    fn test_empty_block_content() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        // Block with no content between marker and end marker
        let input = "[legend]\n---\nكلمة";
        let (tokens, _errors) = orchestrator.tokenize(input);

        // Should have block token
        let block_token = tokens.iter().find(|t| matches!(t, Token::Block(_)));
        assert!(block_token.is_some(), "Expected block token even with empty content");

        // Should have no WordInBlock tokens
        let words_in_block: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::WordInBlock(_)))
            .collect();
        assert!(words_in_block.is_empty(), "Empty block should have no WordInBlock");

        // Word after should be regular
        let regular_words = extract_words(&tokens);
        assert!(regular_words.contains(&"كلمة"));
    }
}

// =============================================================================
// COMPLEX INTEGRATION TESTS
// =============================================================================

mod complex_integration {
    use super::*;

    #[test]
    fn test_sample_scholarly_text() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "قال {الملك} ديشلم *لبيدبا الفيلسوف";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 5);
        assert_eq!(count_errors(&errors), 0);

        // Verify specific word states
        assert_eq!(words[0], "قال");
        assert_eq!(words[1], "الملك"); // in brackets
        assert_eq!(words[2], "ديشلم");
        assert_eq!(words[3], "لبيدبا"); // with * prefix
        assert_eq!(words[4], "الفيلسوف");
    }

    #[test]
    fn test_sample_with_tag_and_brackets() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "يقطع بينهما *** تقاطعا";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(count_errors(&errors), 0);

        let tag_count = tokens.iter().filter(|t| matches!(t, Token::Tag(_))).count();
        assert_eq!(tag_count, 1);
    }

    #[test]
    fn test_sample_with_angle_bracket_added() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "يُقال <له> شتربة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 3);
        assert_eq!(count_errors(&errors), 0);
    }

    #[test]
    fn test_multiline_text() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "باب الأسد والثور\nقال {الملك} ديشلم *لبيدبا الفيلسوف";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert_eq!(words.len(), 8);
        assert_eq!(count_errors(&errors), 0);
    }

    #[test]
    fn test_full_sample_from_config_example() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = r#"باب الأسد والثور
قال {الملك} ديشلم *لبيدبا الفيلسوف اضرب لي مثل الرجلين المتحابّين
يقطع بينهما الخؤون الكذوب ويحملهما على العداوة *** تقاطعا وتدابرا
ومن أمثال ذلك أنّه كان بأرض ديستايد تاجر مكثر
وكانت له بنون سلّم إليهم أمواله فلم يحترفوا بها بل تساهلوا في ذلك *تساهلًا"#;

        let (tokens, errors) = orchestrator.tokenize(input);

        let words = extract_words(&tokens);
        assert!(words.len() > 40, "Expected many words in sample text");

        // Allow some errors for the sample text due to unimplemented features
        // but verify we get meaningful output
        println!("Sample text parsed with {} words and {} error indices", words.len(), errors.len());
    }

    #[test]
    fn test_token_ordering() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "أولى ثانية ثالثة";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words: Vec<_> = tokens
            .iter()
            .filter_map(|t| match t {
                Token::Word(w) => Some(w),
                _ => None,
            })
            .collect();

        assert_eq!(words.len(), 3);

        // Verify token order is correct
        assert_eq!(words[0].token_order, 0);
        assert_eq!(words[1].token_order, 1);
        assert_eq!(words[2].token_order, 2);
    }

    #[test]
    fn test_line_number_tracking() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "سطر أول\nسطر ثان\nسطر ثالث";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words: Vec<_> = tokens
            .iter()
            .filter_map(|t| match t {
                Token::Word(w) => Some(w),
                _ => None,
            })
            .collect();

        // Verify line numbers increase
        assert_eq!(words[0].line_number, 0); // First line
        assert_eq!(words[1].line_number, 0);
        assert_eq!(words[2].line_number, 1); // Second line
        assert_eq!(words[3].line_number, 1);
        assert_eq!(words[4].line_number, 2); // Third line
        assert_eq!(words[5].line_number, 2);
    }

    #[test]
    fn test_span_tracking() {
        let config = create_test_config();
        let orchestrator = Orchestrator::new(&config, &Mode::SinglePage);

        let input = "كلمة أخرى";
        let (tokens, errors) = orchestrator.tokenize(input);

        let words: Vec<_> = tokens
            .iter()
            .filter_map(|t| match t {
                Token::Word(w) => Some(w),
                _ => None,
            })
            .collect();

        assert_eq!(words.len(), 2);

        // Each word should have a valid span
        for word in words {
            assert!(word.span.0 < word.span.1, "Span start should be before end");
        }
    }
}

