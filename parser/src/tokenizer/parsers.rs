use nom::{
    bytes::complete::{ tag, take_while1 },
    character::complete::space1,
    sequence::terminated,
};

use crate::config::ParserConfig;

use super::{
    state::{ Mode, Token, TokenizerAction, TokenizerState },
    util::{
        enclosed_word_parser,
        is_valid_char,
        line_break,
        parse_sqeuence,
        prefixed_word_parser,
        spaces,
        suffixed_word_parser,
        word_parser,
        Chars,
    },
};

pub type ParserFn = Box<dyn Fn(&TokenizerState) -> Option<TokenizerAction>>;

fn word_parser_fn(state: &TokenizerState, chars: &Chars) -> Option<TokenizerAction> {
    word_parser(&state.remaining, chars)
        .ok()
        .and_then(|(rest, word)| {
            if state.can_add_token() {
                Some(
                    TokenizerAction::AddWord(
                        word.to_string(),
                        "sound".to_string(),
                        rest.to_string(),
                        word.len()
                    )
                )
            } else {
                None
            }
        })
}
fn prefixed_word_parser_fn(
    state: &TokenizerState,
    chars: &Chars,
    prefix: String,
    label: String
) -> Option<TokenizerAction> {
    prefixed_word_parser(&state.remaining, &prefix, chars)
        .ok()
        .and_then(|(rest, word)| {
            if state.can_add_token() {
                Some(
                    TokenizerAction::AddWord(
                        word.to_string(),
                        label.clone(),
                        rest.to_string(),
                        word.len() + prefix.len()
                    )
                )
            } else {
                None
            }
        })
}

fn run_bracketed_words_parser(
    state: &TokenizerState,
    chars: &Chars,
    open: String,
    close: String,
    label: String
) -> Option<TokenizerAction> {
    prefixed_word_parser(&state.remaining, &open, chars)
        .ok()
        .and_then(|(rest, word)| {
            if bracket_is_closed(rest, &close, chars) {
                create_bracketed_token(state, word, format!("{}_open", &label), rest, open.len())
            } else {
                None
            }
        })
        .or_else(|| {
            suffixed_word_parser(&state.remaining, &close, chars)
                .ok()
                .and_then(|(rest, word)| {
                    if bracket_was_open(state, &label) {
                        create_bracketed_token(
                            state,
                            word,
                            format!("{}_close", &label),
                            rest,
                            close.len()
                        )
                    } else {
                        None
                    }
                })
        })
        .or_else(|| {
            enclosed_word_parser(&state.remaining, &open, &close, chars)
                .ok()
                .and_then(|(rest, word)|
                    create_bracketed_token(
                        state,
                        word,
                        label.clone(),
                        rest,
                        open.len() + close.len()
                    )
                )
        })
}

fn run_sequence_parser(
    state: &TokenizerState,
    seq: String,
    seq_type: String
) -> Option<TokenizerAction> {
    parse_sqeuence(&seq)(&state.remaining)
        .ok()
        .and_then(|(rest, _)| {
            if state.can_add_token() {
                Some(
                    TokenizerAction::AddTag(
                        seq.to_string(),
                        seq_type.to_string(),
                        rest.to_string(),
                        seq.len()
                    )
                )
            } else {
                None
            }
        })
}

pub fn create_parser(config: ParserConfig, mode: &Mode) -> ParserFn {
    let range = &config.main.char_range;
    let first = range.first().unwrap();
    let last = range.last().unwrap();
    let chars = ((*first, *last), config.main.additional_chars.clone());

    match mode {
        Mode::SinglePage => {
            Box::new(move |state| {
                if let Some(action) = parse_spaces(state) {
                    return Some(action);
                }
                if let Some(action) = parse_line_break(state) {
                    return Some(action);
                }
                for prefix_def in &config.prefix {
                    let result = prefixed_word_parser_fn(
                        state,
                        &chars,
                        prefix_def.symbol.clone(),
                        prefix_def.label.clone()
                    );
                    if result.is_some() {
                        return result;
                    }
                }

                for bracket_def in &config.brackets {
                    let result = run_bracketed_words_parser(
                        state,
                        &chars,
                        bracket_def.open.clone(),
                        bracket_def.close.clone(),
                        bracket_def.label.clone()
                    );
                    if result.is_some() {
                        return result;
                    }
                }

                for seq_def in &config.sequence {
                    let result = run_sequence_parser(
                        state,
                        seq_def.symbol.clone(),
                        seq_def.label.clone()
                    );
                    if result.is_some() {
                        return result;
                    }
                }

                if let Some(action) = word_parser_fn(state, &chars) {
                    return Some(action);
                }

                None
            })
        }
        Mode::Passage => { Box::new(move |state| { None }) }
        Mode::MultiplePages => unimplemented!(),
    }
}

pub fn parse_line_break(state: &TokenizerState) -> Option<TokenizerAction> {
    line_break(&state.remaining)
        .ok()
        .map(|(rest, chars)| { TokenizerAction::AddLineBreak(rest.to_string(), chars.len()) })
}

pub fn parse_passage_line_break(state: &TokenizerState) -> Option<TokenizerAction> {
    line_break(&state.remaining)
        .ok()
        .map(|(rest, chars)| {
            TokenizerAction::AddPassageLineBreak(rest.to_string(), chars.len())
        })
}

// pub fn parse_page_break(state: &mut TokenizerState) -> Option<TokenizerState> {
//     page_break(&state.remaining)
//         .ok()
//         .map(|(rest, repr)| {
//             let mut new_state = state.clone();
//             new_state.update_remaining(rest);
//             new_state.update_position(repr.len());
//             new_state.add_page_break(repr);
//             new_state
//         })
// }

pub fn parse_spaces(state: &TokenizerState) -> Option<TokenizerAction> {
    spaces(&state.remaining)
        .ok()
        .map(|(rest, repr)| { TokenizerAction::AddSpace(rest.to_string(), repr.len()) })
}

fn bracket_is_closed<'a>(input: &'a str, close: &'a str, chars: &Chars) -> bool {
    let result = terminated::<&'a str, &'a str, &'a str, nom::error::Error<&'a str>, _, _>(
        take_while1(
            |c: char|
                c != close.chars().next().unwrap() && (c.is_whitespace() || is_valid_char(c, chars))
        ),
        terminated::<&'a str, &'a str, &'a str, nom::error::Error<&'a str>, _, _>(
            tag(close),
            space1
        )
    )(input);
    result.is_ok()
}

fn bracket_was_open<'a>(state: &TokenizerState, bracket_type: &'a str) -> bool {
    let mut open_found = false;
    let openning_bracket_type = format!("{}_open", bracket_type);
    for token in state.tokens.iter().rev() {
        if let Token::Word(_, token_type, _, _, _, _) = token {
            if token_type.as_str() == &openning_bracket_type {
                open_found = true;
                break;
            } else if token_type.as_str() != "sound" {
                break;
            }
        }
    }

    open_found
}

pub fn create_bracketed_token(
    state: &TokenizerState,
    word: &str,
    token_type: String,
    rest: &str,
    additional_len: usize
) -> Option<TokenizerAction> {
    if state.can_add_token() {
        Some(
            TokenizerAction::AddWord(
                word.to_string(),
                token_type.to_string(),
                rest.to_string(),
                word.len() + additional_len
            )
        )
    } else {
        None
    }
}
