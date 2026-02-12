use crate::{
    config::BracketsConfig,
    core::{
        errors::TextError,
        handlers::HandlerSequence,
        parsers::{ bracketed_text_parser, empty_brackets_error_parser },
        updates::{
            ErrorPayload,
            LineBreakPayload,
            PageBreakPayload,
            PassageLineBreakPayload,
            SpacePayload,
            TagPayload,
            TokenizerUpdate,
            WordPayload,
        },
    },
};

pub fn create_bracketed_text_updates<'a>(
    input: &'a str,
    brackets_def: &'a BracketsConfig,
    all_bracket_defs: &Vec<&'a BracketsConfig>,
    content_expression: &HandlerSequence<'a>
) -> Option<Vec<TokenizerUpdate<'a>>> {
    let empty_brackets_error = empty_brackets_error_parser(
        input,
        &brackets_def.open,
        &brackets_def.close
    )
        .ok()
        .map(|(rest, (open, possible_space, close))| {
            let combined_word = format!("{}{}{}", open, possible_space, close);
            let len = combined_word.chars().count();
            vec![
                TokenizerUpdate::AddError(ErrorPayload {
                    text: Box::leak(combined_word.into_boxed_str()),
                    error: TextError::EmptyBrackets,
                    len,
                    rest: Some(rest),
                })
            ]
        });

    if empty_brackets_error.is_some() {
        return empty_brackets_error;
    }

    let updates = bracketed_text_parser(input, &brackets_def.open, &brackets_def.close)
        .ok()
        .and_then(|(rest, (open, extracted_text, close))| {
            let bracket_errors = check_for_bracket_errors(
                extracted_text,
                open,
                close,
                all_bracket_defs,
                content_expression
            );

            if let Some(content_updates) = bracket_errors {
                transform_inner_commands(content_updates, brackets_def, rest)
            } else if let Some(content_updates) = content_expression.process_all(extracted_text) {
                transform_inner_commands(content_updates, brackets_def, rest)
            } else {
                None
            }
        });

    updates
}

fn transform_inner_commands<'a>(
    inner_commands: Vec<TokenizerUpdate<'a>>,
    brackets_def: &'a BracketsConfig,
    rest: &'a str
) -> Option<Vec<TokenizerUpdate<'a>>> {
    let mut actions = Vec::new();
    let number = inner_commands.len();

    for (i, command) in inner_commands.into_iter().enumerate() {
        let is_last_command = i == number - 1;
        let is_first_command = i == 0;
        let is_single_command = number == 1;
        let rest = if is_last_command || is_single_command { Some(rest) } else { None };

        actions.extend(
            transform_inner_command(
                command,
                is_first_command,
                is_single_command,
                is_last_command,
                brackets_def,
                rest
            )
        );
    }

    Some(actions)
}

fn transform_inner_command<'a>(
    command: TokenizerUpdate<'a>,
    is_first_command: bool,
    is_single_command: bool,
    is_last_command: bool,
    brackets_def: &'a BracketsConfig,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    match command {
        TokenizerUpdate::AddPageBreak(payload) =>
            transform_page_break_inner_update(
                payload,
                is_first_command,
                is_single_command,
                is_last_command,
                rest
            ),
        TokenizerUpdate::AddLineBreak(payload) =>
            transform_line_break_inner_update(
                payload,
                is_first_command,
                is_single_command,
                is_last_command,
                rest
            ),
        TokenizerUpdate::AddSpace(payload) =>
            transform_space_inner_update(
                payload,
                is_first_command,
                is_single_command,
                is_last_command,
                rest
            ),
        TokenizerUpdate::AddWord(payload) =>
            transform_add_word_inner_update(
                payload,
                is_single_command,
                is_first_command,
                is_last_command,
                brackets_def,
                rest
            ),
        TokenizerUpdate::AddTag(payload) => transform_add_tag_inner_update(payload, rest),
        TokenizerUpdate::AddError(payload) => transform_inner_add_error(payload, rest),
        TokenizerUpdate::AddPassageLineBreak(payload) =>
            transform_add_passage_line_break_inner_update(
                payload,
                is_first_command,
                is_single_command,
                is_last_command,
                rest
            ),
        // Block updates should not appear inside brackets - treat as errors
        TokenizerUpdate::AddBlockStart(payload) => {
            vec![TokenizerUpdate::AddError(ErrorPayload {
                text: payload.block_name,
                error: TextError::BlockStartInBrackets(payload.block_name.to_string()),
                len: payload.len,
                rest,
            })]
        },
        TokenizerUpdate::AddBlockEnd(payload) => {
            vec![TokenizerUpdate::AddError(ErrorPayload {
                text: "---",
                error: TextError::BlockEndInBrackets,
                len: payload.len,
                rest,
            })]
        },
    }
}

fn transform_page_break_inner_update<'a>(
    payload: PageBreakPayload<'a>,
    is_first_command: bool,
    is_single_command: bool,
    is_last_command: bool,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    if is_first_command || is_single_command || is_last_command {
        vec![
            TokenizerUpdate::AddError(ErrorPayload {
                text: payload.repr,
                error: TextError::InvalidPageBreak(payload.repr.to_string()),
                len: payload.len,
                rest,
            })
        ]
    } else {
        vec![
            TokenizerUpdate::AddPageBreak(PageBreakPayload {
                rest: None,
                pre_len: payload.pre_len,
                len: payload.len,
                post_len: payload.post_len,
                repr: payload.repr,
            })
        ]
    }
}

fn transform_line_break_inner_update<'a>(
    payload: LineBreakPayload<'a>,
    is_first_command: bool,
    is_single_command: bool,
    is_last_command: bool,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    if is_first_command || is_single_command || is_last_command {
        vec![
            TokenizerUpdate::AddError(ErrorPayload {
                text: "".into(),
                error: TextError::InvalidLineBreak,
                len: payload.len,
                rest,
            })
        ]
    } else {
        vec![
            TokenizerUpdate::AddLineBreak(LineBreakPayload {
                rest: None,
                len: payload.len,
            })
        ]
    }
}

fn transform_space_inner_update<'a>(
    payload: SpacePayload<'a>,
    is_first_command: bool,
    is_single_command: bool,
    is_last_command: bool,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    if is_first_command || is_single_command || is_last_command {
        vec![
            TokenizerUpdate::AddError(ErrorPayload {
                text: "".into(),
                error: TextError::SpaceInBrackets,
                len: payload.len,
                rest,
            })
        ]
    } else {
        vec![
            TokenizerUpdate::AddSpace(SpacePayload {
                rest: None,
                len: payload.len,
            })
        ]
    }
}

fn transform_add_word_inner_update<'a>(
    payload: WordPayload<'a>,
    is_single_command: bool,
    is_first_command: bool,
    is_last_command: bool,
    brackets_def: &'a BracketsConfig,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    let (word_state, pre_len, post_len) = if is_single_command {
        (
            Box::leak(format!("{}_single", brackets_def.label).into_boxed_str()),
            brackets_def.open.chars().count(),
            brackets_def.close.chars().count(),
        )
    } else if is_first_command {
        (
            Box::leak(format!("{}_open", brackets_def.label).into_boxed_str()),
            brackets_def.open.chars().count(),
            0,
        )
    } else if is_last_command {
        (
            Box::leak(format!("{}_close", brackets_def.label).into_boxed_str()),
            0,
            brackets_def.close.chars().count(),
        )
    } else {
        (Box::leak(format!("{}", brackets_def.label).into_boxed_str()), 0, 0)
    };

    vec![
        TokenizerUpdate::AddWord(WordPayload {
            word: payload.word,
            word_state,
            word_type: payload.word_type,
            rest,
            len: payload.len,
            pre_len,
            post_len,
        })
    ]
}

fn transform_add_tag_inner_update<'a>(
    payload: TagPayload<'a>,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    vec![
        TokenizerUpdate::AddError(ErrorPayload {
            text: payload.tag,
            error: TextError::Custom("Invalid position for tag".into()),
            len: payload.len,
            rest,
        })
    ]
}

fn transform_inner_add_error<'a>(
    payload: ErrorPayload<'a>,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    vec![
        TokenizerUpdate::AddError(ErrorPayload {
            text: payload.text,
            error: payload.error,
            len: payload.len,
            rest,
        })
    ]
}

fn transform_add_passage_line_break_inner_update<'a>(
    payload: PassageLineBreakPayload<'a>,
    is_first_command: bool,
    is_single_command: bool,
    is_last_command: bool,
    rest: Option<&'a str>
) -> Vec<TokenizerUpdate<'a>> {
    if is_first_command || is_single_command || is_last_command {
        vec![
            TokenizerUpdate::AddError(ErrorPayload {
                text: "".into(),
                error: TextError::InvalidLineBreak,
                len: payload.len,
                rest,
            })
        ]
    } else {
        vec![
            TokenizerUpdate::AddPassageLineBreak(PassageLineBreakPayload {
                rest: None,
                len: payload.len,
            })
        ]
    }
}

fn check_for_bracket_errors<'a>(
    content: &'a str,
    curr_open: &'a str,
    curr_close: &'a str,
    all_bracket_defs: &Vec<&'a BracketsConfig>,
    content_expression: &HandlerSequence<'a>
) -> Option<Vec<TokenizerUpdate<'a>>> {
    // Here only brackets are parsed, not other symbols
    let mut updates = vec![];
    let mut rest = content;
    while !rest.is_empty() {
        if let Some(update) = content_expression.process_first(rest) {
            let last_update = update.last().unwrap();
            rest = last_update.get_rest().unwrap_or("");
            updates.extend(update);
        } else {
            // check here if it is a bracket error, otherwise append an unknown error
            return None;
        }
    }

    if updates.iter().any(|update| matches!(update, TokenizerUpdate::AddError(_))) {
        Some(updates)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BracketsConfig, WordConfig};
    use crate::core::handlers::{Handler, HandlerSequence};
    use crate::core::updates::TokenizerUpdate;

    fn arabic_word_config() -> WordConfig {
        WordConfig {
            label: "Arabic".to_string(),
            chars: ((0x600, 0x6FF), vec![]),
            default_state: "sound".to_string(),
            precedence: usize::MAX,
        }
    }

    fn curly_brackets() -> BracketsConfig {
        BracketsConfig {
            open: "{".to_string(),
            close: "}".to_string(),
            label: "suppletion".to_string(),
            skip: None,
            precedence: usize::MAX,
        }
    }

    fn parentheses() -> BracketsConfig {
        BracketsConfig {
            open: "(".to_string(),
            close: ")".to_string(),
            label: "title".to_string(),
            skip: None,
            precedence: usize::MAX,
        }
    }

    fn square_brackets() -> BracketsConfig {
        BracketsConfig {
            open: "[".to_string(),
            close: "]".to_string(),
            label: "superfluous".to_string(),
            skip: Some(true),
            precedence: usize::MAX,
        }
    }

    fn double_square_brackets() -> BracketsConfig {
        BracketsConfig {
            open: "[[".to_string(),
            close: "]]".to_string(),
            label: "cross-out".to_string(),
            skip: Some(true),
            precedence: 0, // Higher priority
        }
    }

    fn angle_brackets() -> BracketsConfig {
        BracketsConfig {
            open: "<".to_string(),
            close: ">".to_string(),
            label: "added".to_string(),
            skip: None,
            precedence: usize::MAX,
        }
    }

    fn create_content_handler_sequence<'a>(word_config: &'a WordConfig) -> HandlerSequence<'a> {
        HandlerSequence(vec![
            Handler::Space,
            Handler::Word(word_config),
        ])
    }

    #[test]
    fn test_single_word_in_curly_brackets() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{الملك}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Should have one word update
        let word_update = updates.iter().find(|u| matches!(u, TokenizerUpdate::AddWord(_)));
        assert!(word_update.is_some());

        if let Some(TokenizerUpdate::AddWord(payload)) = word_update {
            assert_eq!(payload.word, "الملك");
            assert!(payload.word_state.contains("suppletion"));
        }
    }

    #[test]
    fn test_multiple_words_in_brackets() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة أولى}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Count word updates
        let word_count = updates.iter()
            .filter(|u| matches!(u, TokenizerUpdate::AddWord(_)))
            .count();
        assert_eq!(word_count, 2);
    }

    #[test]
    fn test_empty_brackets_produces_error() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Should have error for empty brackets
        let error_update = updates.iter().find(|u| matches!(u, TokenizerUpdate::AddError(_)));
        assert!(error_update.is_some());

        if let Some(TokenizerUpdate::AddError(payload)) = error_update {
            assert!(payload.error.to_string().to_lowercase().contains("empty"));
        }
    }

    #[test]
    fn test_brackets_with_only_spaces_produces_error() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{   }",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Should have error
        let has_error = updates.iter().any(|u| matches!(u, TokenizerUpdate::AddError(_)));
        assert!(has_error);
    }

    #[test]
    fn test_unmatched_opening_bracket_returns_none() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        // Should return None for unmatched bracket
        assert!(result.is_none());
    }

    #[test]
    fn test_bracket_not_at_start_returns_none() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "text{كلمة}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        // Should return None when bracket not at start
        assert!(result.is_none());
    }

    #[test]
    fn test_leading_space_in_brackets_produces_error() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{ كلمة}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Should produce error for space at wrong position
        let has_error = updates.iter().any(|u| matches!(u, TokenizerUpdate::AddError(_)));
        assert!(has_error);
    }

    #[test]
    fn test_trailing_space_in_brackets_produces_error() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة }",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Should produce error for trailing space
        let has_error = updates.iter().any(|u| matches!(u, TokenizerUpdate::AddError(_)));
        assert!(has_error);
    }

    #[test]
    fn test_rest_after_brackets() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة} أخرى",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Last update should have rest pointing to remaining text
        let last = updates.last().unwrap();
        let rest = last.get_rest();
        assert!(rest.is_some());
        assert!(rest.unwrap().contains("أخرى"));
    }

    #[test]
    fn test_double_square_brackets_precedence() {
        let word_config = arabic_word_config();
        let single = square_brackets();
        let double = double_square_brackets();
        let all_brackets = vec![&double, &single]; // Double first due to precedence
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "[[مشطوب]]",
            &double,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        // Should parse as double brackets (cross-out), not single
        let word_update = updates.iter().find(|u| matches!(u, TokenizerUpdate::AddWord(_)));
        if let Some(TokenizerUpdate::AddWord(payload)) = word_update {
            assert!(payload.word_state.contains("cross-out"));
        }
    }

    #[test]
    fn test_angle_brackets() {
        let word_config = arabic_word_config();
        let brackets = angle_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "<له>",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        assert!(result.is_some());
        let updates = result.unwrap();

        let word_update = updates.iter().find(|u| matches!(u, TokenizerUpdate::AddWord(_)));
        if let Some(TokenizerUpdate::AddWord(payload)) = word_update {
            assert!(payload.word_state.contains("added"));
        }
    }

    #[test]
    fn test_single_word_state_format() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        let updates = result.unwrap();
        let word_update = updates.iter().find(|u| matches!(u, TokenizerUpdate::AddWord(_)));

        if let Some(TokenizerUpdate::AddWord(payload)) = word_update {
            // Single word should have _single suffix
            assert!(payload.word_state.contains("single") || payload.word_state.contains("suppletion"));
        }
    }

    #[test]
    fn test_first_word_state_format() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة أخرى}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        let updates = result.unwrap();
        let word_updates: Vec<_> = updates.iter()
            .filter_map(|u| match u {
                TokenizerUpdate::AddWord(p) => Some(p),
                _ => None,
            })
            .collect();

        assert_eq!(word_updates.len(), 2);
        // First word should have _open suffix
        assert!(word_updates[0].word_state.contains("open") || word_updates[0].word_state.contains("suppletion"));
    }

    #[test]
    fn test_bracket_pre_len() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        let updates = result.unwrap();
        let word_update = updates.iter().find(|u| matches!(u, TokenizerUpdate::AddWord(_)));

        if let Some(TokenizerUpdate::AddWord(payload)) = word_update {
            // pre_len should include opening bracket
            assert!(payload.pre_len > 0);
        }
    }

    #[test]
    fn test_bracket_post_len() {
        let word_config = arabic_word_config();
        let brackets = curly_brackets();
        let all_brackets = vec![&brackets];
        let content_handlers = create_content_handler_sequence(&word_config);

        let result = create_bracketed_text_updates(
            "{كلمة}",
            &brackets,
            &all_brackets,
            &content_handlers
        );

        let updates = result.unwrap();
        let word_update = updates.iter().find(|u| matches!(u, TokenizerUpdate::AddWord(_)));

        if let Some(TokenizerUpdate::AddWord(payload)) = word_update {
            // post_len should include closing bracket for single word
            assert!(payload.post_len > 0);
        }
    }
}
