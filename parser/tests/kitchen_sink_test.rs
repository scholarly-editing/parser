//! Kitchen sink integration test for the scholarly editing parser.
//!
//! This test combines all features of the parser into a single complex document
//! to verify they work together correctly.

use parser::{
    config::{
        BlockConfig, BlockType, BracketsConfig, PageConfig, ParserConfig, PrefixConfig,
        SuffixConfig, TagConfig, WordConfig,
    },
    core::{orchestrator::Orchestrator, state::{Mode, Token}},
};

// =============================================================================
// Helper Functions
// =============================================================================

fn create_full_config() -> ParserConfig {
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
        word: vec![
            WordConfig {
                label: "Arabic".to_string(),
                chars: ((0x0600, 0x06FF), vec![
                    0x003A, // :
                    0x002E, // .
                    0x060C, // ،
                ]),
                default_state: "sound".to_string(),
                precedence: usize::MAX,
            }
        ],
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
                symbol: "†".to_string(),
                label: "corrupt".to_string(),
                precedence: usize::MAX,
            },
        ],
        suffix: vec![
            SuffixConfig {
                symbol: "~".to_string(),
                label: "middle-arabic".to_string(),
                precedence: usize::MAX,
            }
        ],
        brackets: vec![
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

// =============================================================================
// Kitchen Sink Test
// =============================================================================

#[test]
fn test_kitchen_sink_document() {
    let config = create_full_config();
    let orchestrator = Orchestrator::new(&config, &Mode::MultiplePages);

    let input = r#"
fol.1r
(عنوان الكتاب)
[illustration]
[legend]
صورة الأسد والثور
في مرج أخضر
---
قال {الملك} دبشليم للفيلسوف: اضرب لي مثل الرجلين *المتحابين يقطع بينهما
الخؤون الكذوب. قال بيدبا: إذا ابتلي المتحابان بأن يدخل بينهما *** نمام.

[margin]
حاشية: النمام هو الذي ينقل الحديث
على وجه الإفساد
---

fol.1v
ومن أمثال ذلك ما يزعمون أنه كان بأرض دستبا كنز~ ومكان خصيب.
وكانت <به> وحوش كثيرة. ولم يكن ?لهم ملك.
فاجتمعوا وتشاوروا في تأمير أسد عليهم.
ففعلوا ذلك واستقام أمرهم.
ثم إن الأسد مرض †مرضا شديدا وعجز عن الصيد.
...
فجاع الجند واضطربوا.
"#;

    let (tokens, errors) = orchestrator.tokenize(input);

    // 1. Check for errors
    if !errors.is_empty() {
        for err_idx in &errors {
            if let Token::Error(e) = &tokens[*err_idx] {
                println!("Error at line {}: {} ({})", e.line, e.error, e.token);
            }
        }
        panic!("Found {} errors in parsing (see output above)", errors.len());
    }

    // 2. Verify structure
    let pages: std::collections::HashSet<_> = tokens.iter()
        .filter_map(|t| match t {
            Token::PageBreak(p) => Some(p.page_representation),
            _ => None
        })
        .collect();

    assert!(pages.contains("fol.1r"), "Missing first page break");
    assert!(pages.contains("fol.1v"), "Missing second page break");

    // 3. Verify Blocks
    let blocks: Vec<_> = tokens.iter()
        .filter_map(|t| match t {
            Token::Block(b) => Some(b.block_name),
            _ => None
        })
        .collect();

    assert!(blocks.contains(&"illustration"));
    assert!(blocks.contains(&"legend"));
    assert!(blocks.contains(&"margin"));

    // 4. Verify Content
    let words: Vec<&str> = tokens.iter()
        .filter_map(|t| match t {
            Token::Word(w) => Some(w.word),
            Token::WordInBlock(w) => Some(w.word),
            _ => None
        })
        .collect();

    // Check bracketed text
    assert!(words.contains(&"عنوان"), "Missing title word");
    assert!(words.contains(&"الملك"), "Missing suppletion word");

    // Check prefixed word
    let emendation = tokens.iter().find(|t| match t {
        Token::Word(w) => w.word == "المتحابين" && w.state == "emendation",
        _ => false
    });
    assert!(emendation.is_some(), "Emendation prefix * not parsed correctly");

    // Check suffixed word
    let middle_arabic = tokens.iter().find(|t| match t {
        Token::Word(w) => w.word == "كنز" && w.state == "middle-arabic",
        _ => false
    });
    assert!(middle_arabic.is_some(), "Suffix ~ not parsed correctly");

    // Check added text (angle brackets)
    let added = tokens.iter().find(|t| match t {
        Token::Word(w) => w.word == "به" && w.state.contains("added"),
        _ => false
    });
    assert!(added.is_some(), "Added text <...> not parsed correctly");

    // Check tags
    let lacuna = tokens.iter().find(|t| match t {
        Token::Tag(tag) => tag.tag == "***",
        _ => false
    });
    assert!(lacuna.is_some(), "Lacuna tag *** not found");

    let damage = tokens.iter().find(|t| match t {
        Token::Tag(tag) => tag.tag == "...",
        _ => false
    });
    assert!(damage.is_some(), "Damage tag ... not found");

    // 5. Check Block Content
    let legend_content = tokens.iter().any(|t| match t {
        Token::WordInBlock(w) => w.block_name == "legend" && w.word == "صورة",
        _ => false
    });
    assert!(legend_content, "Legend block content parsing failed");

    let margin_content = tokens.iter().any(|t| match t {
        Token::WordInBlock(w) => w.block_name == "margin" && w.word == "الحديث",
        _ => false
    });
    assert!(margin_content, "Margin block content parsing failed");

    // Check corrupted prefix
    let corrupt = tokens.iter().find(|t| match t {
        Token::Word(w) => w.word == "مرضا" && w.state == "corrupt",
        _ => false
    });
    assert!(corrupt.is_some(), "Corrupt prefix † not parsed correctly");

    // Check unintelligible prefix
    let unintelligible = tokens.iter().find(|t| match t {
        Token::Word(w) => w.word == "لهم" && w.state == "unintelligible",
        _ => false
    });
    assert!(unintelligible.is_some(), "Unintelligible prefix ? not parsed correctly");

    println!("Kitchen sink test passed successfully with {} tokens", tokens.len());
}

