use crate::{
    config::BracketsConfig,
    core::{
        handlers::HandlerSequence,
        parsers::{ bracketed_text_parser, empty_beackets_error_parser },
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
    all_bracket_defs: &'a Vec<&'a BracketsConfig>,
    content_expression: &'a HandlerSequence<'a>
) -> Option<Vec<TokenizerUpdate<'a>>> {
    let empty_brackets_error = empty_beackets_error_parser(
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
                    message: "Empty brackets".into(),
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
                message: "Invalid position for page break".into(),
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
                message: "Invalid position for line break".into(),
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
                message: "Space on the wrong side of a bracket".into(),
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
            message: "Invalid position for tag".into(),
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
            message: payload.message,
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
                message: "Invalid position for passage line break".into(),
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
    content_expression: &'a HandlerSequence<'a>
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
